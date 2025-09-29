use std::path::Path;

use crate::constants::MAX_DATABASES;
use anyhow::{anyhow, Result};
use heed::EnvFlags;
use heed::{
    types::{Bytes, DecodeIgnore, Str},
    Database, Env, EnvOpenOptions,
};

pub fn open_env(path: &Path, read_only: bool) -> Result<Env> {
    let mut builder = EnvOpenOptions::new();
    builder.max_dbs(MAX_DATABASES);
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
    list_entries_paginated(env, db_name, 0, limit)
}

/// Lazy loading with offset/limit pagination for efficient browsing of large datasets
pub fn list_entries_paginated(
    env: &Env, 
    db_name: &str, 
    offset: usize, 
    limit: usize
) -> Result<Vec<(String, Vec<u8>)>> {
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
    let mut items = Vec::with_capacity(limit.min(1000));
    let mut skipped = 0;
    
    for result in iter {
        if skipped < offset {
            skipped += 1;
            continue;
        }
        
        if items.len() >= limit {
            break;
        }
        
        let (key, value) = result?;
        items.push((key.to_string(), value.to_vec()));
    }
    
    // Read transaction will be automatically dropped/aborted here
    Ok(items)
}

/// Count total entries in a database without loading them (for pagination)
pub fn count_entries(env: &Env, db_name: &str) -> Result<usize> {
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
    let count = iter.count();
    
    Ok(count)
}
