mod methods;
mod platform;
mod wiper;

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(name = "secure-delete")]
#[command(about = "Securely delete files using data overwriting")]
struct Cli {
    /// Target path
    target: PathBuf,

    /// Wipe method (supports random:N)
    #[arg(short, long, default_value = "dod")]
    method: String,

    /// Number of passes for the "random" method (default: 1)
    #[arg(long, value_name = "N")]
    passes: Option<usize>,
}

fn main() {
    let cli = Cli::parse();

    match methods::get_method(&cli.method, cli.passes) {
        Some(method) => {
            println!("Wiping {} using {}...", cli.target.display(), method);
            if let Err(e) = wiper::wipe_file(&cli.target, method.as_ref()) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            println!(" Done.");
        }
        None => {
            eprintln!("Unknown method '{}'.", cli.method);
            std::process::exit(2);
        }
    }
}
