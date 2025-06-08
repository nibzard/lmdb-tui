use anyhow::{anyhow, Result};
use heed::{
    types::{Bytes, Str},
    Database, Env,
};
use jsonpath_lib as jsonpath;
use regex::Regex;
use serde_json::Value;
use crate::plugins;
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
    if let Some(v) = plugins::decode_with_plugins(bytes) {
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

/// Parse a user supplied query string into a [`Mode`].
///
/// Supported formats:
/// - `"prefix <value>"` performs a prefix match on keys
/// - `"range <start> <end>"` or `"range <start>..<end>"`
/// - `"regex <expr>"` interprets `<expr>` as a regular expression
/// - `"jsonpath <expr>"` filters decoded JSON values with a JSONPath
///
/// Any string without a recognised prefix defaults to `Mode::Prefix` using the
/// entire input as the prefix.
pub fn parse_query(input: &str) -> Result<Mode<'_>> {
    let trimmed = input.trim();
    if let Some(rest) = trimmed.strip_prefix("prefix ") {
        return Ok(Mode::Prefix(rest));
    }
    if let Some(rest) = trimmed.strip_prefix("range ") {
        if let Some((start, end)) = rest.split_once("..") {
            return Ok(Mode::Range(start.trim(), end.trim()));
        }
        let mut parts = rest.split_whitespace();
        if let (Some(start), Some(end)) = (parts.next(), parts.next()) {
            return Ok(Mode::Range(start, end));
        }
        return Err(anyhow!("invalid range query"));
    }
    if let Some(rest) = trimmed.strip_prefix("regex ") {
        let re = Regex::new(rest)?;
        return Ok(Mode::Regex(re));
    }
    if let Some(rest) = trimmed.strip_prefix("jsonpath ") {
        return Ok(Mode::JsonPath(rest));
    }
    Ok(Mode::Prefix(trimmed))
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
