use std::path::Path;

use anyhow::{anyhow, Result};
use heed::EnvFlags;
use heed::{
    types::{Bytes, DecodeIgnore, Str},
    Database, Env, EnvOpenOptions,
};

pub fn open_env(path: &Path, read_only: bool) -> Result<Env> {
    let mut builder = EnvOpenOptions::new();
    builder.max_dbs(128);
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
        .expect("the unnamed database always exists");
    let mut names = Vec::new();
    for entry in unnamed.iter(&rtxn)? {
        let (name, ()) = entry?;
        if env.open_database::<Str, Bytes>(&rtxn, Some(name)).is_ok() {
            names.push(name.to_string());
        }
    }
    rtxn.commit()?;
    Ok(names)
}

pub fn list_entries(env: &Env, db_name: &str, limit: usize) -> Result<Vec<(String, Vec<u8>)>> {
    let rtxn = env.read_txn()?;
    let db: Database<Str, Bytes> = env
        .open_database(&rtxn, Some(db_name))?
        .ok_or_else(|| anyhow!("database not found"))?;
    let iter = db.iter(&rtxn)?;
    let mut items = Vec::new();
    for (count, result) in iter.enumerate() {
        if count >= limit {
            break;
        }
        let (key, value) = result?;
        items.push((key.to_string(), value.to_vec()));
    }
    Ok(items)
}
