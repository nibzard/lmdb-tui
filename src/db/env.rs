use std::path::Path;

use anyhow::{anyhow, Result};
use heed::EnvFlags;
use heed::{
    types::{Bytes, DecodeIgnore, Str},
    Database, Env, EnvOpenOptions,
};

/// Maximum number of databases allowed in an environment.
pub const MAX_DBS: u32 = 128;

/// Default number of entries to retrieve when listing database contents.
pub const DEFAULT_ENTRY_LIMIT: usize = 100;

pub fn open_env(path: &Path, read_only: bool) -> Result<Env> {
    let mut builder = EnvOpenOptions::new();
    builder.max_dbs(MAX_DBS);
    if read_only {
        unsafe {
            builder.flags(EnvFlags::READ_ONLY);
        }
    }
    let env = unsafe { builder.open(path)? };
    Ok(env)
}

pub fn list_databases(env: &Env) -> Result<Vec<String>> {
    let rtxn = env.read_txn()?;
    let unnamed: Database<Str, DecodeIgnore> = env
        .open_database(&rtxn, None)?
        .ok_or_else(|| anyhow!("unnamed database not found"))?;
    let mut names = Vec::new();
    for entry in unnamed.iter(&rtxn)? {
        let (name, ()) = entry?;
        if env.open_database::<Str, Bytes>(&rtxn, Some(name)).is_ok() {
            names.push(name.to_string());
        }
    }

    // If no named databases exist, check if the unnamed database has data
    if names.is_empty() {
        if let Ok(Some(db)) = env.open_database::<Str, Bytes>(&rtxn, None) {
            // Check if unnamed database has any entries
            if db.iter(&rtxn)?.next().is_some() {
                names.push("(unnamed)".to_string());
            }
        }
    }

    // Read transactions are automatically aborted when dropped
    Ok(names)
}

pub fn list_entries(env: &Env, db_name: &str, limit: usize) -> Result<Vec<(String, Vec<u8>)>> {
    let rtxn = env.read_txn()?;

    let db: Database<Str, Bytes> = if db_name == "(unnamed)" {
        // Open the unnamed database
        env.open_database(&rtxn, None)?
            .ok_or_else(|| anyhow!("unnamed database not found"))?
    } else {
        // Open a named database
        env.open_database(&rtxn, Some(db_name))?
            .ok_or_else(|| anyhow!("database '{}' not found", db_name))?
    };

    let iter = db.iter(&rtxn)?;
    let mut items = Vec::new();
    for (count, result) in iter.enumerate() {
        if count >= limit {
            break;
        }
        let (key, value) = result?;
        items.push((key.to_string(), value.to_vec()));
    }
    // Read transaction will be automatically dropped/aborted here
    Ok(items)
}
