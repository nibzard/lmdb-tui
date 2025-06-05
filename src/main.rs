use std::path::PathBuf;

use clap::{command, Parser};

use lmdb_tui::app;

/// Simple LMDB TUI explorer
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the LMDB environment directory
    path: PathBuf,

    /// Open environment read-only
    #[arg(long)]
    read_only: bool,
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = app::run(&cli.path, cli.read_only) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
