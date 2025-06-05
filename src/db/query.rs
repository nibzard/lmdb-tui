use anyhow::{anyhow, Result};
use heed::{
    types::{Bytes, Str},
    Database, Env,
};
use jsonpath_lib as jsonpath;
use regex::Regex;
use serde_json::Value;

pub enum Mode<'a> {
    Prefix(&'a str),
    Range(&'a str, &'a str),
    Regex(Regex),
    JsonPath(&'a str),
}

pub fn scan(
    env: &Env,
    db_name: &str,
    mode: Mode<'_>,
    limit: usize,
) -> Result<Vec<(String, Vec<u8>)>> {
    let rtxn = env.read_txn()?;
    let db: Database<Str, Bytes> = env
        .open_database(&rtxn, Some(db_name))?
        .ok_or_else(|| anyhow!("database not found"))?;
    let iter = db.iter(&rtxn)?;
    let mut items = Vec::new();
    for result in iter {
        let (key, value) = result?;
        let mut matched = false;
        match &mode {
            Mode::Prefix(pre) => matched = key.starts_with(*pre),
            Mode::Range(start, end) => matched = key >= *start && key < *end,
            Mode::Regex(re) => matched = re.is_match(key),
            Mode::JsonPath(path) => {
                if let Ok(v) = serde_json::from_slice::<Value>(value) {
                    matched = jsonpath::select(&v, path)
                        .map(|r| !r.is_empty())
                        .unwrap_or(false);
                }
            }
        }
        if matched {
            items.push((key.to_string(), value.to_vec()));
            if items.len() >= limit {
                break;
            }
        }
    }
    rtxn.commit()?;
    Ok(items)
}
