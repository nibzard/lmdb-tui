use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use anyhow::{anyhow, Result};
use csv::Writer;
use heed::{
    types::{Bytes, Str},
    Database, Env,
};
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};

use crate::db::txn::Txn;

#[derive(Serialize, Deserialize)]
struct Record {
    key: String,
    value: Value,
}

pub fn export_json(env: &Env, db_name: &str, path: &Path) -> Result<()> {
    let rtxn = env.read_txn()?;
    let db: Database<Str, Bytes> = env
        .open_database(&rtxn, Some(db_name))?
        .ok_or_else(|| anyhow!("database not found"))?;
    let stats = db.stat(&rtxn)?;
    let pb = ProgressBar::new(stats.entries as u64);
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    write!(writer, "[")?;
    let mut first = true;
    for result in db.iter(&rtxn)? {
        let (key, value) = result?;
        let json: Value = serde_json::from_slice(value)?;
        let rec = Record {
            key: key.to_string(),
            value: json,
        };
        if !first {
            write!(writer, ",")?;
        }
        serde_json::to_writer(&mut writer, &rec)?;
        first = false;
        pb.inc(1);
    }
    write!(writer, "]")?;
    writer.flush()?;
    pb.finish();
    rtxn.commit()?;
    Ok(())
}

pub fn export_csv(env: &Env, db_name: &str, path: &Path) -> Result<()> {
    let rtxn = env.read_txn()?;
    let db: Database<Str, Bytes> = env
        .open_database(&rtxn, Some(db_name))?
        .ok_or_else(|| anyhow!("database not found"))?;
    let stats = db.stat(&rtxn)?;
    let pb = ProgressBar::new(stats.entries as u64);
    let file = File::create(path)?;
    let mut wtr = Writer::from_writer(file);
    for result in db.iter(&rtxn)? {
        let (key, value) = result?;
        wtr.write_record([key, &String::from_utf8_lossy(value)])?;
        pb.inc(1);
    }
    wtr.flush()?;
    pb.finish();
    rtxn.commit()?;
    Ok(())
}

pub fn import_json(env: &Env, db_name: &str, path: &Path) -> Result<usize> {
    let mut txn = Txn::begin(env)?;
    let db: Database<Str, Bytes> = env
        .open_database(txn.inner_mut(), Some(db_name))?
        .ok_or_else(|| anyhow!("database not found"))?;
    let file = File::open(path)?;
    let records: Vec<Record> = serde_json::from_reader(file)?;
    let pb = ProgressBar::new(records.len() as u64);
    for rec in &records {
        let bytes = serde_json::to_vec(&rec.value)?;
        db.put(txn.inner_mut(), &rec.key, &bytes)?;
        pb.inc(1);
    }
    pb.finish();
    txn.commit()?;
    Ok(records.len())
}

pub fn import_csv(env: &Env, db_name: &str, path: &Path) -> Result<usize> {
    let mut txn = Txn::begin(env)?;
    let db: Database<Str, Bytes> = env
        .open_database(txn.inner_mut(), Some(db_name))?
        .ok_or_else(|| anyhow!("database not found"))?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)?;
    let pb = ProgressBar::new_spinner();
    let mut count = 0usize;
    for result in rdr.records() {
        let rec = result?;
        let key = rec.get(0).ok_or_else(|| anyhow!("missing key"))?;
        let value = rec.get(1).unwrap_or("");
        db.put(txn.inner_mut(), key, value.as_bytes())?;
        count += 1;
        pb.inc(1);
    }
    pb.finish();
    txn.commit()?;
    Ok(count)
}
