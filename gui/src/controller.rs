//! `WipeController`: the QObject exposed to QML. It runs wipes on a worker
//! thread and reports progress back to the Qt event loop.

use std::cell::Cell;
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::{ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cxx_qt::{CxxQtThread, CxxQtType, Threading};
use cxx_qt_lib::{QString, QStringList};

use secure_delete::{methods, platform, signals, wiper};

use crate::i18n::tr;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, busy)]
        #[qproperty(f64, progress)]
        #[qproperty(QString, phase)]
        type WipeController = super::WipeControllerRust;

        /// Display names of the selectable wipe methods, in index order.
        #[qinvokable]
        #[cxx_name = "methodNames"]
        fn method_names(self: &WipeController) -> QStringList;

        /// Index of the default method (DoD 5220.22-M).
        #[qinvokable]
        #[cxx_name = "defaultMethod"]
        fn default_method(self: &WipeController) -> i32;

        /// Path passed on the command line (e.g. from a file manager), or "".
        #[qinvokable]
        #[cxx_name = "initialTarget"]
        fn initial_target(self: &WipeController) -> QString;

        /// Empty if `path` may be wiped, otherwise the reason it may not.
        #[qinvokable]
        #[cxx_name = "checkTarget"]
        fn check_target(self: &WipeController, path: &QString, device: bool) -> QString;

        /// Detected storage devices as a JSON array of
        /// {path, title, size, detail, icon, usable, reason}.
        #[qinvokable]
        #[cxx_name = "devicesJson"]
        fn devices_json(self: &WipeController) -> QString;

        /// Whether the GUI can list and wipe whole devices (Linux only: device
        /// discovery reads /sys/block and elevation goes through pkexec).
        #[qinvokable]
        #[cxx_name = "supportsDevices"]
        fn supports_devices(self: &WipeController) -> bool;

        /// Whether wiping a device will go through pkexec (we are not root).
        #[qinvokable]
        #[cxx_name = "needsElevation"]
        fn needs_elevation(self: &WipeController) -> bool;

        /// Whether `path` is on (or is) a solid-state device.
        #[qinvokable]
        #[cxx_name = "isSolidState"]
        fn is_solid_state(self: &WipeController, path: &QString, device: bool) -> bool;

        #[qinvokable]
        fn start(
            self: Pin<&mut WipeController>,
            path: &QString,
            method: i32,
            verify: bool,
            device: bool,
        );

        #[qinvokable]
        fn cancel(self: &WipeController);

        /// Emitted once per `start`, when the wipe ends.
        #[qsignal]
        fn finished(self: Pin<&mut WipeController>, ok: bool, message: QString);
    }

    impl cxx_qt::Threading for WipeController {}
}

#[derive(Default)]
pub struct WipeControllerRust {
    busy: bool,
    progress: f64,
    phase: QString,
    /// Stdin of the privileged helper while it runs; dropping it cancels.
    helper_stdin: Arc<Mutex<Option<ChildStdin>>>,
}

/// Method keys offered in the UI: the static registry plus a 1-pass random.
fn method_keys() -> Vec<&'static str> {
    let mut keys: Vec<&'static str> = methods::method_entries()
        .iter()
        .map(|e| e.canonical)
        .collect();
    keys.insert(2.min(keys.len()), "random");
    keys
}

impl qobject::WipeController {
    fn method_names(&self) -> QStringList {
        method_keys()
            .into_iter()
            .filter_map(|k| methods::get_method(k, None))
            .map(|m| {
                let name = m.name();
                if name.contains("pass") {
                    QString::from(&localize_method_name(name))
                } else {
                    QString::from(&format!("{name} ({})", passes_label(m.passes())))
                }
            })
            .collect()
    }

    fn default_method(&self) -> i32 {
        method_keys().iter().position(|k| *k == "dod").unwrap_or(0) as i32
    }

    fn initial_target(&self) -> QString {
        std::env::args_os()
            .nth(1)
            .map(|a| QString::from(&a.to_string_lossy().into_owned()))
            .unwrap_or_default()
    }

    fn check_target(&self, path: &QString, device: bool) -> QString {
        let path = PathBuf::from(path.to_string());
        let result = if device {
            secure_delete::validate_device_target(&path)
        } else {
            secure_delete::validate_file_target(&path)
        };
        match result {
            Ok(()) => QString::default(),
            Err(e) => QString::from(&localize_error(&e)),
        }
    }

    fn devices_json(&self) -> QString {
        use secure_delete::devices;

        let items: Vec<String> = devices::list_devices()
            .iter()
            .map(|d| {
                let mut tags = Vec::new();
                if d.usb {
                    tags.push(tr("USB"));
                } else if d.removable {
                    tags.push(tr("Removable"));
                }
                if d.loop_backing.is_some() {
                    tags.push(tr("Loop device"));
                } else if d.rotational {
                    tags.push(tr("Hard disk"));
                } else {
                    tags.push(tr("SSD / flash"));
                }
                let mut detail = d.path.display().to_string();
                for t in &tags {
                    detail.push_str(" · ");
                    detail.push_str(t);
                }
                let title = match (&d.loop_backing, d.model.is_empty()) {
                    (Some(file), _) => file.display().to_string(),
                    (None, false) => d.model.clone(),
                    (None, true) => d.name.clone(),
                };
                let icon = if d.usb || d.removable {
                    "drive-removable-media"
                } else {
                    "drive-harddisk"
                };
                let (usable, reason) = match secure_delete::validate_device_target(&d.path) {
                    Ok(()) => (true, String::new()),
                    Err(e) => (false, localize_error(&e)),
                };
                format!(
                    r#"{{"path":{},"title":{},"size":{},"detail":{},"icon":{},"usable":{},"reason":{}}}"#,
                    json_str(&d.path.display().to_string()),
                    json_str(&title),
                    json_str(&devices::format_size(d.size)),
                    json_str(&detail),
                    json_str(icon),
                    usable,
                    json_str(&reason),
                )
            })
            .collect();
        QString::from(&format!("[{}]", items.join(",")))
    }

    fn supports_devices(&self) -> bool {
        cfg!(target_os = "linux")
    }

    fn needs_elevation(&self) -> bool {
        !is_root()
    }

    fn is_solid_state(&self, path: &QString, device: bool) -> bool {
        let path = PathBuf::from(path.to_string());
        if device {
            platform::device_is_ssd(&path)
        } else {
            platform::file_on_solid_state(&path)
        }
    }

    fn start(mut self: Pin<&mut Self>, path: &QString, method: i32, verify: bool, device: bool) {
        if *self.busy() {
            return;
        }
        let path = PathBuf::from(path.to_string());
        let key = usize::try_from(method)
            .ok()
            .and_then(|i| method_keys().get(i).copied());
        let Some(method) = key.and_then(|k| methods::get_method(k, None)) else {
            self.as_mut()
                .finished(false, QString::from(&tr("Unknown wipe method.")));
            return;
        };
        if device && !self.supports_devices() {
            self.as_mut().finished(
                false,
                QString::from(&tr(
                    "Wiping whole devices is only available from the command line on this system.",
                )),
            );
            return;
        }
        // Re-validate: the target may have changed since the user picked it.
        let check = self.check_target(&QString::from(&path.to_string_lossy().into_owned()), device);
        if !check.is_empty() {
            self.as_mut().finished(false, check);
            return;
        }

        signals::reset();
        self.as_mut().set_busy(true);
        self.as_mut().set_progress(0.0);
        self.as_mut().set_phase(QString::default());

        let qt_thread = self.qt_thread();
        let helper_stdin = self.rust().helper_stdin.clone();
        let key = key.unwrap_or_default();
        std::thread::spawn(move || {
            let progress = GuiProgress::new(qt_thread.clone());
            if device && !is_root() {
                let (ok, message) = run_helper(&path, key, verify, &helper_stdin, &progress);
                let _ = qt_thread.queue(move |mut obj| {
                    obj.as_mut().set_busy(false);
                    if ok {
                        obj.as_mut().set_progress(1.0);
                    }
                    obj.as_mut().finished(ok, QString::from(&message));
                });
                return;
            }
            let opts = wiper::WipeOptions { verify };
            let result = if device {
                wiper::wipe_device(&path, method.as_ref(), &opts, &progress)
            } else {
                wiper::wipe_file(&path, method.as_ref(), &opts, &progress)
            };
            let (ok, message) = match result {
                Ok(()) => (true, String::new()),
                Err(e) if e.kind() == io::ErrorKind::Interrupted => (
                    false,
                    if device {
                        tr("Cancelled. The device was only partially wiped.")
                    } else {
                        tr("Cancelled. The file was partially wiped and was NOT deleted.")
                    },
                ),
                Err(e) => (false, localize_error(&e.to_string())),
            };
            let _ = qt_thread.queue(move |mut obj| {
                obj.as_mut().set_busy(false);
                if ok {
                    obj.as_mut().set_progress(1.0);
                }
                obj.as_mut().finished(ok, QString::from(&message));
            });
        });
    }

    fn cancel(&self) {
        signals::request_cancel();
        // Closing the helper's stdin is how a root helper gets cancelled.
        if let Ok(mut stdin) = self.rust().helper_stdin.lock() {
            stdin.take();
        }
    }
}

/// Minimal JSON string encoder (quotes, backslashes, control characters).
fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(unix)]
fn is_root() -> bool {
    // SAFETY: geteuid has no preconditions and cannot fail.
    unsafe { libc::geteuid() == 0 }
}

/// Device wiping (the only privileged operation) is Linux-only in the GUI.
#[cfg(not(unix))]
fn is_root() -> bool {
    false
}

/// The CLI binary to run as the privileged helper: `$SECURE_DELETE_CLI`, the
/// `secure_delete` next to this executable, or the installed one.
fn helper_binary() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("SECURE_DELETE_CLI") {
        return Some(PathBuf::from(p));
    }
    let sibling = std::env::current_exe()
        .ok()?
        .parent()?
        .join("secure_delete");
    if sibling.is_file() {
        return Some(sibling);
    }
    ["/usr/bin/secure_delete", "/usr/local/bin/secure_delete"]
        .iter()
        .map(PathBuf::from)
        .find(|p| p.is_file())
}

/// Wipe a device through `pkexec secure_delete --gui-helper ...`, relaying its
/// progress. Returns `(ok, message)`.
fn run_helper(
    path: &Path,
    method: &str,
    verify: bool,
    stdin_slot: &Mutex<Option<ChildStdin>>,
    progress: &GuiProgress,
) -> (bool, String) {
    use wiper::Progress;

    let Some(cli) = helper_binary() else {
        return (
            false,
            tr("Cannot find the secure_delete command-line tool needed to wipe devices."),
        );
    };
    let mut cmd = Command::new("pkexec");
    cmd.arg(&cli)
        .args(["--gui-helper", "--device", "--force", "--method", method]);
    if verify {
        cmd.arg("--verify");
    }
    cmd.arg("--")
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return (
                false,
                tr("Cannot run pkexec: %1").replace("%1", &e.to_string()),
            )
        }
    };
    if let Ok(mut slot) = stdin_slot.lock() {
        *slot = child.stdin.take();
    }
    // A cancel that arrived before the slot was filled.
    if signals::interrupted() {
        stdin_slot.lock().ok().and_then(|mut s| s.take());
    }

    let mut stderr = child.stderr.take();
    let stderr_reader = std::thread::spawn(move || {
        let mut text = String::new();
        if let Some(err) = stderr.as_mut() {
            let _ = err.read_to_string(&mut text);
        }
        text
    });

    let mut started = false;
    if let Some(out) = child.stdout.take() {
        for line in BufReader::new(out).lines().map_while(Result::ok) {
            let Some(rest) = line.strip_prefix("@sd ") else {
                continue;
            };
            started = true;
            match rest.split_once(' ') {
                Some(("start", _)) => progress.start(1000),
                Some(("phase", msg)) => progress.set_message(msg),
                Some(("pos", n)) => progress.set_position(n.parse().unwrap_or(0)),
                _ => {}
            }
        }
    }

    let status = child.wait();
    stdin_slot.lock().ok().and_then(|mut s| s.take());
    let stderr = stderr_reader.join().unwrap_or_default();
    let last_error = stderr
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .map(|l| {
            let l = l.trim();
            let refusing = format!("Refusing to wipe {}: ", path.display());
            let l = l
                .strip_prefix("Error: ")
                .or_else(|| l.strip_prefix(refusing.as_str()))
                .unwrap_or(l);
            localize_error(l)
        });

    match status.ok().and_then(|s| s.code()) {
        Some(0) => (true, String::new()),
        Some(130) => (false, tr("Cancelled. The device was only partially wiped.")),
        // pkexec: 126 = authentication dialog dismissed, 127 = not authorized.
        Some(126) if !started => (false, tr("Authentication was cancelled.")),
        Some(127) if !started => (
            false,
            last_error.unwrap_or_else(|| tr("Not authorized to wipe devices.")),
        ),
        code => (
            false,
            last_error.unwrap_or_else(|| {
                let code = code.map_or_else(|| "?".to_string(), |c| c.to_string());
                tr("The helper failed (exit status %1).").replace("%1", &code)
            }),
        ),
    }
}

fn passes_label(n: usize) -> String {
    if n == 1 {
        tr("1 pass")
    } else {
        tr("%1 passes").replace("%1", &n.to_string())
    }
}

/// "DoD 5220.22-M (3 passes)" -> "DoD 5220.22-M (3 pasadas)".
fn localize_method_name(name: &str) -> String {
    if let Some((base, inner)) = name.rsplit_once(" (") {
        let inner = inner.trim_end_matches(')');
        let count = inner
            .strip_suffix(" passes")
            .or_else(|| inner.strip_suffix(" pass"))
            .and_then(|n| n.parse::<usize>().ok());
        if let Some(n) = count {
            return format!("{base} ({})", passes_label(n));
        }
    }
    name.to_string()
}

/// Wiper phase text: "pass 2/3 - <method>" or "verifying".
fn localize_phase(msg: &str) -> String {
    if msg == "verifying" {
        return tr("Verifying the last pass…");
    }
    if let Some((frac, name)) = msg.strip_prefix("pass ").and_then(|r| r.split_once(" - ")) {
        if let Some((current, total)) = frac.split_once('/') {
            return tr("Pass %1 of %2 · %3")
                .replace("%1", current)
                .replace("%2", total)
                .replace("%3", &localize_method_name(name));
        }
    }
    msg.to_string()
}

/// Translate the library's validation messages; others pass through (OS
/// errors are already localized by the C library).
fn localize_error(msg: &str) -> String {
    const MOUNTED: &str = "device (or a partition of it) is currently mounted at ";
    if let Some(mountpoint) = msg.strip_prefix(MOUNTED) {
        return tr("The device (or one of its partitions) is mounted at %1.")
            .replace("%1", mountpoint);
    }
    tr(msg)
}

/// Forwards wiper progress to the QObject, throttled to ~20 updates/s.
struct GuiProgress {
    thread: CxxQtThread<qobject::WipeController>,
    total: Cell<u64>,
    last: Cell<Option<Instant>>,
}

impl GuiProgress {
    const INTERVAL: Duration = Duration::from_millis(50);

    fn new(thread: CxxQtThread<qobject::WipeController>) -> Self {
        GuiProgress {
            thread,
            total: Cell::new(0),
            last: Cell::new(None),
        }
    }

    fn send(&self, f: impl FnOnce(Pin<&mut qobject::WipeController>) + Send + 'static) {
        let _ = self.thread.queue(f);
    }
}

impl wiper::Progress for GuiProgress {
    fn start(&self, total: u64) {
        self.total.set(total);
    }

    fn set_message(&self, msg: &str) {
        let msg = localize_phase(msg);
        self.last.set(None);
        self.send(move |mut obj| {
            obj.as_mut().set_phase(QString::from(&msg));
            obj.as_mut().set_progress(0.0);
        });
    }

    fn set_position(&self, pos: u64) {
        let now = Instant::now();
        let total = self.total.get();
        let due = self
            .last
            .get()
            .is_none_or(|t| now.duration_since(t) >= Self::INTERVAL);
        if !due && pos < total {
            return;
        }
        self.last.set(Some(now));
        let fraction = if total == 0 {
            1.0
        } else {
            pos as f64 / total as f64
        };
        self.send(move |mut obj| obj.as_mut().set_progress(fraction));
    }
}
