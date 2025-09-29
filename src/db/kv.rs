use anyhow::{anyhow, Result};
use heed::{
    types::{Bytes, Str},
    Database, Env,
};

use super::txn::Txn;

/// Put a key/value pair into the database.
pub fn put(env: &Env, txn: &mut Txn<'_>, db_name: &str, key: &str, value: &[u8]) -> Result<()> {
    let db: Database<Str, Bytes> = env.create_database(txn.inner_mut(), Some(db_name))?;
    db.put(txn.inner_mut(), key, value)?;
    Ok(())
}

/// Get a key from the database.
pub fn get(env: &Env, txn: &Txn<'_>, db_name: &str, key: &str) -> Result<Option<Vec<u8>>> {
    let db: Database<Str, Bytes> = match env.open_database(txn.inner(), Some(db_name))? {
        Some(db) => db,
        None => return Ok(None), // Database doesn't exist, so key doesn't exist
    };
    Ok(db.get(txn.inner(), key)?.map(|v| v.to_vec()))
}

/// Delete a key from the database.
pub fn delete(env: &Env, txn: &mut Txn<'_>, db_name: &str, key: &str) -> Result<()> {
    let db: Database<Str, Bytes> = env
        .open_database(txn.inner_mut(), Some(db_name))?
        .ok_or_else(|| anyhow!("database '{}' not found", db_name))?;
    db.delete(txn.inner_mut(), key)?;
    Ok(())
}
