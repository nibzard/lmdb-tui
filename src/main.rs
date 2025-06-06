use std::path::PathBuf;

use clap::{command, Parser};

use heed::Error as HeedError;
use lmdb_tui::app;

/// Simple LMDB TUI explorer
#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Simple LMDB TUI explorer",
    arg_required_else_help = true,
    after_help = "Examples:\n  lmdb-tui path/to/env\n  lmdb-tui --plain path/to/env\n\nFull docs: https://lmdb.nibzard.com"
)]
struct Cli {
    /// Path to the LMDB environment directory
    path: PathBuf,

    /// Open environment read-only
    #[arg(long)]
    read_only: bool,

    /// Output plain text instead of TUI
    #[arg(long, conflicts_with = "json")]
    plain: bool,

    /// Output JSON instead of TUI
    #[arg(long, conflicts_with = "plain")]
    json: bool,

    /// Reduce output to errors only
    #[arg(short, long)]
    quiet: bool,

    /// Show verbose debug messages
    #[arg(long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.verbose || std::env::var_os("DEBUG").is_some() {
        eprintln!("debug: opening {}", cli.path.display());
    }

    let res = if cli.plain || cli.json {
        app::run_plain(&cli.path, cli.read_only, cli.json)
    } else {
        app::run(&cli.path, cli.read_only)
    };

    if let Err(e) = res {
        if !cli.quiet {
            eprintln!("error: {e}");
        }
        let code = if e
            .downcast_ref::<std::io::Error>()
            .map(|io| io.kind() == std::io::ErrorKind::NotFound)
            .unwrap_or_else(|| {
                e.downcast_ref::<HeedError>()
                    .and_then(|heed_err| {
                        if let HeedError::Io(io) = heed_err {
                            Some(io.kind() == std::io::ErrorKind::NotFound)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(false)
            }) {
            2
        } else {
            1
        };
        std::process::exit(code);
    }
}
