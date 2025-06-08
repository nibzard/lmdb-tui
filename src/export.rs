use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, ValueEnum};

use crate::db::{env::open_env, io};

#[derive(Debug, Clone, ValueEnum)]
pub enum Format {
    Json,
    Csv,
}

#[derive(Debug, Args)]
pub struct ExportArgs {
    /// Path to the LMDB environment directory
    pub path: PathBuf,
    /// Database name to export
    #[arg(long)]
    pub db: String,
    /// Output file
    #[arg(long)]
    pub out: PathBuf,
    /// Export format
    #[arg(long, value_enum, default_value_t = Format::Json)]
    pub format: Format,
}

#[derive(Debug, Args)]
pub struct ImportArgs {
    /// Path to the LMDB environment directory
    pub path: PathBuf,
    /// Database name to import into
    #[arg(long)]
    pub db: String,
    /// Input file
    #[arg(long)]
    pub input: PathBuf,
    /// Input format
    #[arg(long, value_enum, default_value_t = Format::Json)]
    pub format: Format,
}

pub fn export(args: ExportArgs) -> Result<()> {
    let env = open_env(&args.path, true)?;
    match args.format {
        Format::Json => io::export_json(&env, &args.db, &args.out)?,
        Format::Csv => io::export_csv(&env, &args.db, &args.out)?,
    }
    Ok(())
}

pub fn import(args: ImportArgs) -> Result<()> {
    let env = open_env(&args.path, false)?;
    match args.format {
        Format::Json => {
            io::import_json(&env, &args.db, &args.input)?;
        }
        Format::Csv => {
            io::import_csv(&env, &args.db, &args.input)?;
        }
    }
    Ok(())
}
