use anyhow::{anyhow, Result};
use heed::{
    types::{Bytes, Str},
    Database, Env,
};
use jsonpath_lib as jsonpath;
use regex::Regex;
use serde_json::Value;
use std::str;

/// Attempt to decode raw bytes into a `serde_json::Value`.
///
/// The function first tries JSON and then MessagePack. If both fail, an error
/// is returned so callers can decide how to handle undecodable values.
pub fn decode_value(bytes: &[u8]) -> Result<Value> {
    if let Ok(v) = serde_json::from_slice::<Value>(bytes) {
        return Ok(v);
    }
    if let Ok(v) = rmp_serde::from_slice::<Value>(bytes) {
        return Ok(v);
    }
    Err(anyhow!("unable to decode value"))
}

pub enum Mode<'a> {
    Prefix(&'a str),
    Range(&'a str, &'a str),
    Regex(Regex),
    JsonPath(&'a str),
}

/// Parse a user provided query string into a [`Mode`].
pub fn parse_query<'a>(input: &'a str) -> Result<Mode<'a>> {
    let mut parts = input.split_whitespace();
    let kind = parts
        .next()
        .ok_or_else(|| anyhow!("empty query"))?
        .to_lowercase();
    match kind.as_str() {
        "prefix" => {
            let pre = parts.next().ok_or_else(|| anyhow!("missing prefix"))?;
            Ok(Mode::Prefix(pre))
        }
        "range" => {
            let start = parts.next().ok_or_else(|| anyhow!("missing start"))?;
            let end = parts.next().ok_or_else(|| anyhow!("missing end"))?;
            Ok(Mode::Range(start, end))
        }
        "regex" => {
            let pat = parts.next().ok_or_else(|| anyhow!("missing pattern"))?;
            Ok(Mode::Regex(Regex::new(pat)?))
        }
        "jsonpath" => {
            let path = parts.next().ok_or_else(|| anyhow!("missing path"))?;
            Ok(Mode::JsonPath(path))
        }
        _ => Err(anyhow!("invalid query")),
    }
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
                if let Ok(v) = decode_value(value) {
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
