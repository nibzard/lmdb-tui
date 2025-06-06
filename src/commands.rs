use std::io::{Read, Write};

use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use heed::Env;
use serde::{Deserialize, Serialize};

use crate::db::{
    env as dbenv, kv,
    txn::Txn,
    undo::{Op, UndoStack},
};

/// Put a key/value pair into the database and record undo information.
pub fn put(
    env: &Env,
    txn: &mut Txn<'_>,
    undo: &mut UndoStack,
    db: &str,
    key: &str,
    value: &[u8],
) -> Result<()> {
    let prev = kv::get(env, txn, db, key)?;
    kv::put(env, txn, db, key, value)?;
    undo.push(Op::Put {
        db: db.to_string(),
        key: key.to_string(),
        prev,
        new: value.to_vec(),
    });
    Ok(())
}

/// Get a key from the database.
pub fn get(env: &Env, txn: &Txn<'_>, db: &str, key: &str) -> Result<Option<Vec<u8>>> {
    kv::get(env, txn, db, key)
}

/// Delete a key from the database and record undo information.
pub fn delete(
    env: &Env,
    txn: &mut Txn<'_>,
    undo: &mut UndoStack,
    db: &str,
    key: &str,
) -> Result<()> {
    let prev = kv::get(env, txn, db, key)?;
    kv::delete(env, txn, db, key)?;
    undo.push(Op::Delete {
        db: db.to_string(),
        key: key.to_string(),
        prev,
    });
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct ExportItem {
    key: String,
    value: String,
}

/// Export the entire database to JSON.
pub fn export_json<W: Write>(env: &Env, db_name: &str, mut writer: W) -> Result<()> {
    let entries = dbenv::list_entries(env, db_name, usize::MAX)?;
    let items: Vec<ExportItem> = entries
        .into_iter()
        .map(|(k, v)| ExportItem {
            key: k,
            value: general_purpose::STANDARD.encode(v),
        })
        .collect();
    serde_json::to_writer(&mut writer, &items)?;
    Ok(())
}

/// Import records from JSON and record undo information.
pub fn import_json<R: Read>(
    env: &Env,
    txn: &mut Txn<'_>,
    undo: &mut UndoStack,
    db_name: &str,
    mut reader: R,
) -> Result<()> {
    let items: Vec<ExportItem> = serde_json::from_reader(&mut reader)?;
    for item in items {
        let value = general_purpose::STANDARD.decode(item.value)?;
        let prev = kv::get(env, txn, db_name, &item.key)?;
        kv::put(env, txn, db_name, &item.key, &value)?;
        undo.push(Op::Put {
            db: db_name.to_string(),
            key: item.key,
            prev,
            new: value,
        });
    }
    Ok(())
}

/// Export the database to CSV (columns: key,value with base64 value).
pub fn export_csv<W: Write>(env: &Env, db_name: &str, writer: W) -> Result<()> {
    let entries = dbenv::list_entries(env, db_name, usize::MAX)?;
    let mut wtr = csv::Writer::from_writer(writer);
    wtr.write_record(["key", "value"])?;
    for (k, v) in entries {
        wtr.write_record([k, general_purpose::STANDARD.encode(v)])?;
    }
    wtr.flush()?;
    Ok(())
}

/// Import records from CSV and record undo information.
pub fn import_csv<R: Read>(
    env: &Env,
    txn: &mut Txn<'_>,
    undo: &mut UndoStack,
    db_name: &str,
    reader: R,
) -> Result<()> {
    let mut rdr = csv::Reader::from_reader(reader);
    for result in rdr.records() {
        let record = result?;
        let key = record
            .get(0)
            .ok_or_else(|| anyhow!("missing key"))?
            .to_string();
        let value_str = record.get(1).ok_or_else(|| anyhow!("missing value"))?;
        let value = general_purpose::STANDARD.decode(value_str)?;
        let prev = kv::get(env, txn, db_name, &key)?;
        kv::put(env, txn, db_name, &key, &value)?;
        undo.push(Op::Put {
            db: db_name.to_string(),
            key,
            prev,
            new: value,
        });
    }
    Ok(())
}
