use std::cell::Cell;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

use secure_delete::{methods, platform, signals, wiper};

#[derive(Parser)]
#[command(name = "secure-delete", version)]
#[command(about = "Securely delete files or storage devices using data overwriting", long_about = None)]
struct Cli {
    /// Target path (file or device)
    #[arg(value_name = "PATH")]
    target: Option<PathBuf>,

    /// Wipe method (use --list-methods to see all; supports random:N and fill:XX)
    #[arg(short, long, default_value = "dod")]
    method: String,

    /// Wipe a device/block device instead of a single file
    #[arg(short, long)]
    device: bool,

    /// Force (skip confirmation prompt; required for wiping block devices)
    #[arg(short, long)]
    force: bool,

    /// Number of passes for the "random" method (default: 1)
    #[arg(long, value_name = "N")]
    passes: Option<usize>,

    /// List available wipe methods
    #[arg(long)]
    list_methods: bool,

    /// List the storage devices that can be wiped (Linux)
    #[arg(long)]
    list_devices: bool,

    /// Use the device's native firmware erase (ATA Security Erase for SATA,
    /// Format NVM with Secure Erase for NVMe) instead of overwriting. Only for
    /// block devices; ideal for SSDs where overwrite is unreliable. Linux only.
    #[arg(long, requires = "device")]
    secure_erase: bool,

    /// Use the "enhanced" / cryptographic variant of the native erase when
    /// supported (ATA enhanced security erase; NVMe crypto erase).
    #[arg(long, requires = "secure_erase")]
    enhanced: bool,

    /// Read back and verify the final pass after wiping
    #[arg(long)]
    verify: bool,

    /// Show what would be done without wiping anything
    #[arg(long)]
    dry_run: bool,

    /// Suppress the progress bar and informational notes
    #[arg(short, long)]
    quiet: bool,

    /// Internal, for the graphical front-end running this tool through
    /// pkexec: print machine-readable progress ("@sd ..." lines) on stdout
    /// and cancel the wipe when stdin is closed.
    #[arg(long, hide = true, requires = "force")]
    gui_helper: bool,
}

fn is_random_method(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower == "random" || lower.starts_with("random:")
}

fn validate_target(cli: &Cli, target: &Path) -> Result<(), String> {
    if cli.device {
        // Cheap checks first so the --force error is not hidden behind them.
        if !platform::is_block_device(target) {
            return secure_delete::validate_device_target(target);
        }
        if !cli.force {
            return Err("wiping a block device is dangerous and requires --force".into());
        }
        return secure_delete::validate_device_target(target);
    }
    secure_delete::validate_file_target(target)
}

/// Terminal progress bar for the wiper.
struct CliProgress(ProgressBar);

impl CliProgress {
    fn new(quiet: bool) -> Self {
        if quiet {
            return CliProgress(ProgressBar::hidden());
        }
        let pb = ProgressBar::new(0);
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}",
            )
            .unwrap()
            .progress_chars("#>-"),
        );
        CliProgress(pb)
    }
}

impl wiper::Progress for CliProgress {
    fn start(&self, total: u64) {
        self.0.set_length(total);
    }
    fn set_message(&self, msg: &str) {
        self.0.set_message(msg.to_string());
    }
    fn set_position(&self, pos: u64) {
        self.0.set_position(pos);
    }
    fn finish(&self) {
        self.0.finish_and_clear();
    }
}

/// Machine-readable progress for `--gui-helper`:
/// `@sd start <bytes>`, `@sd phase <text>`, `@sd pos <permille>`.
struct HelperProgress {
    total: Cell<u64>,
    last: Cell<Option<u64>>,
}

impl HelperProgress {
    fn new() -> Self {
        HelperProgress {
            total: Cell::new(0),
            last: Cell::new(None),
        }
    }
}

impl wiper::Progress for HelperProgress {
    fn start(&self, total: u64) {
        self.total.set(total);
        println!("@sd start {total}");
    }
    fn set_message(&self, msg: &str) {
        self.last.set(None);
        println!("@sd phase {msg}");
    }
    fn set_position(&self, pos: u64) {
        let total = self.total.get().max(1);
        let permille = (pos.min(total) as u128 * 1000 / total as u128) as u64;
        if self.last.get() != Some(permille) {
            self.last.set(Some(permille));
            println!("@sd pos {permille}");
        }
    }
}

/// The GUI cannot signal a process running as root, so it closes our stdin
/// instead; treat EOF (or a read error) as a cancellation request.
fn cancel_on_stdin_eof() {
    std::thread::spawn(|| {
        let mut buf = [0u8; 64];
        let mut stdin = io::stdin();
        while let Ok(n) = stdin.read(&mut buf) {
            if n == 0 {
                break;
            }
        }
        signals::request_cancel();
    });
}

fn list_devices() {
    use secure_delete::devices;

    let found = devices::list_devices();
    if found.is_empty() {
        println!("No storage devices found (device discovery is only supported on Linux).");
        return;
    }
    for d in found {
        let mut tags = Vec::new();
        if d.usb {
            tags.push("USB");
        } else if d.removable {
            tags.push("removable");
        }
        if d.loop_backing.is_some() {
            tags.push("loop");
        } else {
            tags.push(if d.rotational { "HDD" } else { "SSD/flash" });
        }
        let model = if d.model.is_empty() { "-" } else { &d.model };
        let status = match secure_delete::validate_device_target(&d.path) {
            Ok(()) => "ok".to_string(),
            Err(e) => format!("in use: {e}"),
        };
        println!(
            "{:<14} {:>9}  {:<28} [{}]  {}",
            d.path.display(),
            devices::format_size(d.size),
            model,
            tags.join(", "),
            status
        );
    }
}

fn confirm(message: &str) -> bool {
    eprintln!("{}", message);
    print!("Are you sure? Type 'yes' to confirm: ");
    io::stdout().flush().expect("failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("failed to read input");
    input.trim() == "yes"
}

fn main() {
    signals::install();
    let cli = Cli::parse();
    if cli.gui_helper {
        cancel_on_stdin_eof();
    }

    if cli.list_methods {
        println!("Available wipe methods:");
        for entry in methods::method_entries() {
            let m = (entry.make)();
            let mut line = format!("  - {}", m.name());
            if !entry.aliases.is_empty() {
                line.push_str(&format!(" [aliases: {}]", entry.aliases.join(", ")));
            }
            println!("{line}");
        }
        println!("  - Random data (N passes) [random, random:N, or --passes N]");
        println!("  - Fill with a fixed byte (1 pass) [fill:XX / pattern:XX, e.g. fill:ff]");
        return;
    }

    if cli.list_devices {
        list_devices();
        return;
    }

    let target = cli.target.as_ref().unwrap_or_else(|| {
        eprintln!("Error: a target <PATH> is required. Use --help for usage.");
        std::process::exit(2);
    });

    if cli.secure_erase {
        run_secure_erase(&cli, target);
        return;
    }

    let method = methods::get_method(&cli.method, cli.passes).unwrap_or_else(|| {
        eprintln!(
            "Unknown method '{}'. Use --list-methods to see available methods.",
            cli.method
        );
        std::process::exit(2);
    });

    if let Some(p) = cli.passes {
        if !is_random_method(&cli.method) {
            eprintln!(
                "Warning: --passes {} ignored for method '{}' (it only applies to the 'random' method).",
                p, cli.method
            );
        }
    }

    if let Err(e) = validate_target(&cli, target) {
        eprintln!("Refusing to wipe {}: {}", target.display(), e);
        std::process::exit(2);
    }

    let target_type = if cli.device { "DEVICE" } else { "FILE" };
    if cli.dry_run {
        println!(
            "[dry-run] Would wipe {} ({}) using method \"{}\" ({} pass{}).",
            target.display(),
            target_type,
            method,
            method.passes(),
            if method.passes() == 1 { "" } else { "es" }
        );
        return;
    }

    if !cli.force {
        let ok = confirm(&format!(
            "\nWARNING: This will IRREVERSIBLY wipe {} ({}) using method \"{}\".",
            target.display(),
            target_type,
            method
        ));
        if !ok {
            println!("Aborted.");
            return;
        }
    }

    if !cli.quiet && !cli.gui_helper {
        let ssd = if cli.device {
            platform::device_is_ssd(target)
        } else {
            platform::file_on_solid_state(target)
        };
        if ssd {
            eprintln!(
                "Note: the target is on a solid-state device; overwriting may not physically \
                 erase all blocks (wear leveling). On Linux, prefer --secure-erase for SSDs."
            );
        }
    }

    println!("Wiping {} using {}...", target.display(), method);

    let opts = wiper::WipeOptions { verify: cli.verify };
    let progress: Box<dyn wiper::Progress> = if cli.gui_helper {
        Box::new(HelperProgress::new())
    } else {
        Box::new(CliProgress::new(cli.quiet))
    };
    let result = if cli.device {
        wiper::wipe_device(target, method.as_ref(), &opts, progress.as_ref())
    } else {
        wiper::wipe_file(target, method.as_ref(), &opts, progress.as_ref())
    };

    match result {
        Ok(()) => println!(" Done."),
        Err(e) if e.kind() == io::ErrorKind::Interrupted => {
            eprintln!(
                "Interrupted. The target was partially wiped and was NOT {}.",
                if cli.device {
                    "fully erased"
                } else {
                    "deleted"
                }
            );
            std::process::exit(130);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(target_os = "linux")]
fn run_secure_erase(cli: &Cli, target: &Path) {
    use platform::secure_erase;

    if platform::is_system_root(target) {
        eprintln!("Refusing to erase {} (system root).", target.display());
        std::process::exit(2);
    }
    if !platform::is_block_device(target) {
        eprintln!(
            "Error: --secure-erase needs a block device (e.g. /dev/sdX or /dev/nvme0nX); {} is not one.",
            target.display()
        );
        std::process::exit(2);
    }
    if let Err(e) = platform::check_not_mounted(target) {
        eprintln!("Refusing to erase {}: {}", target.display(), e);
        std::process::exit(2);
    }

    let kind = match secure_erase::media_kind(target) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("Cannot probe device {}: {}", target.display(), e);
            std::process::exit(2);
        }
    };

    if cli.dry_run {
        println!(
            "[dry-run] Would issue a native firmware erase on {} ({}).",
            target.display(),
            device_kind_label(&kind)
        );
        return;
    }

    if !cli.force {
        let ok = confirm(&format!(
            "\nWARNING: This will IRREVERSIBLY erase {} ({}) using the device's native firmware erase.",
            target.display(),
            device_kind_label(&kind)
        ));
        if !ok {
            println!("Aborted.");
            return;
        }
    }

    println!(
        "Secure-erasing {} ({}) via native firmware command...",
        target.display(),
        device_kind_label(&kind)
    );

    match secure_erase::secure_erase_device(target, cli.enhanced) {
        Ok(report) => {
            println!(" Done. (backend: {})", report.backend);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!(
                "Hint: ATA Security Erase often fails if the disk is FROZEN. Unfreeze it \
                 (suspend/resume cycle or check BIOS) and retry. For NVMe, ensure the namespace \
                 is not in use."
            );
            std::process::exit(1);
        }
    }
}

#[cfg(target_os = "linux")]
fn device_kind_label(kind: &platform::secure_erase::MediaKind) -> &'static str {
    use platform::secure_erase::MediaKind;
    match kind {
        MediaKind::Rotational => "rotational",
        MediaKind::SolidState => "solid-state",
        MediaKind::Unknown => "unknown",
    }
}

#[cfg(not(target_os = "linux"))]
fn run_secure_erase(_cli: &Cli, target: &Path) {
    eprintln!(
        "Error: --secure-erase is only supported on Linux; on {} overwrite the device with -d -m <method>.",
        std::env::consts::OS
    );
    let _ = target;
    std::process::exit(2);
}
