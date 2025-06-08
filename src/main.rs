use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use clap::{command, CommandFactory, Parser, Subcommand};
use heed::Error as HeedError;
use lmdb_tui::{app, export};
use log::LevelFilter;

/// Simple LMDB TUI explorer
#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Simple LMDB TUI explorer",
    arg_required_else_help = true,
    after_help = "Examples:\n  lmdb-tui path/to/env\n  lmdb-tui --plain path/to/env\n\nFull docs: https://lmdb.nibzard.com"
)]
#[command(subcommand_negates_reqs = true)]
struct Cli {
    /// Path to the LMDB environment directory
    path: Option<PathBuf>,

    /// Open environment read-only
    #[arg(long)]
    read_only: bool,

    /// Output plain text instead of TUI
    #[arg(long, conflicts_with = "json")]
    plain: bool,

    /// Output JSON instead of TUI
    #[arg(long, conflicts_with = "plain")]
    json: bool,

    /// Suppress non-error output
    #[arg(short = 'q', long)]
    quiet: bool,

    /// Increase logging verbosity
    #[arg(long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Export a database to a file
    Export(export::ExportArgs),
    /// Import records from a file
    Import(export::ImportArgs),
}

fn main() {
    handle_help_pager();
    let cli = Cli::parse();

    init_logger(&cli);

    let result = match cli.command {
        Some(Commands::Export(args)) => export::export(args),
        Some(Commands::Import(args)) => export::import(args),
        None => {
            let path = cli.path.expect("path required");
            if cli.plain || cli.json {
                app::run_plain(&path, cli.read_only, cli.json)
            } else {
                app::run(&path, cli.read_only)
            }
        }
    };

    if let Err(e) = result {
        log::error!("{e}");
        std::process::exit(exit_code(&e));
    }
}

fn handle_help_pager() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1
        || args.iter().any(|a| a == "--help" || a == "-h")
        || args.iter().any(|a| a == "--version" || a == "-V")
    {
        let mut cmd = Cli::command();
        let mut buf = Vec::new();
        if args.iter().any(|a| a == "--version" || a == "-V") {
            println!("{}", cmd.render_version());
        } else {
            cmd.write_long_help(&mut buf).unwrap();
            let help = String::from_utf8(buf).unwrap();
            if let Ok(pager) = std::env::var("PAGER") {
                if let Ok(mut child) = Command::new(pager).stdin(Stdio::piped()).spawn() {
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(help.as_bytes());
                    }
                    let _ = child.wait();
                } else {
                    println!("{help}");
                }
            } else {
                println!("{help}");
            }
        }
        std::process::exit(0);
    }
}

fn init_logger(cli: &Cli) {
    let verbosity = cli.verbose + if std::env::var("DEBUG").is_ok() { 1 } else { 0 };
    let level = if cli.quiet {
        LevelFilter::Error
    } else {
        match verbosity {
            0 => LevelFilter::Warn,
            1 => LevelFilter::Info,
            _ => LevelFilter::Debug,
        }
    };
    env_logger::Builder::from_env(env_logger::Env::default())
        .filter_level(level)
        .init();
}

fn exit_code(e: &anyhow::Error) -> i32 {
    for cause in e.chain() {
        if let Some(io) = cause.downcast_ref::<std::io::Error>() {
            use std::io::ErrorKind::*;
            return match io.kind() {
                NotFound => 1,
                PermissionDenied => 3,
                _ => 1,
            };
        }
        if let Some(HeedError::Io(io)) = cause.downcast_ref::<HeedError>() {
            use std::io::ErrorKind::*;
            return match io.kind() {
                NotFound => 1,
                PermissionDenied => 3,
                _ => 1,
            };
        }
    }
    1
}
