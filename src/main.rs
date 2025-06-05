use clap::{command, Parser};

/// Simple LMDB TUI explorer
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {}

fn main() {
    let _cli = Cli::parse();
}
