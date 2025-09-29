use crate::plugins;
use anyhow::{anyhow, Result};
use heed::{
    types::{Bytes, Str},
    Database, Env, RoTxn,
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
    if let Some(v) = plugins::decode_with_plugins(bytes) {
        return Ok(v);
    }
    Err(anyhow!("unable to decode value"))
}

#[derive(Clone)]
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
    if trimmed.is_empty() {
        return Err(anyhow!("empty query"));
    }
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

/// Count total matching entries without materializing all results
pub fn count_matches(env: &Env, db_name: &str, mode: Mode<'_>) -> Result<usize> {
    let rtxn = env.read_txn()?;
    let db = open_database(&rtxn, env, db_name)?;

    match &mode {
        Mode::Prefix(prefix) => count_prefix_matches(&rtxn, &db, prefix),
        Mode::Range(start, end) => count_range_matches(&rtxn, &db, start, end),
        Mode::Regex(re) => count_full_scan_matches(&rtxn, &db, |key, _| re.is_match(key)),
        Mode::JsonPath(path) => count_full_scan_matches(&rtxn, &db, |_, value| {
            decode_value(value)
                .ok()
                .map(|v| {
                    jsonpath::select(&v, path)
                        .map(|r| !r.is_empty())
                        .unwrap_or(false)
                })
                .unwrap_or(false)
        }),
    }
}

/// Optimized scan with pagination support that avoids double iteration
pub fn scan_paginated(
    env: &Env,
    db_name: &str,
    mode: Mode<'_>,
    offset: usize,
    limit: usize,
) -> Result<Vec<(String, Vec<u8>)>> {
    let rtxn = env.read_txn()?;
    let db = open_database(&rtxn, env, db_name)?;

    match &mode {
        Mode::Prefix(prefix) => scan_prefix_paginated(&rtxn, &db, prefix, offset, limit),
        Mode::Range(start, end) => scan_range_paginated(&rtxn, &db, start, end, offset, limit),
        Mode::Regex(re) => {
            scan_full_paginated(&rtxn, &db, offset, limit, |key, _| re.is_match(key))
        }
        Mode::JsonPath(path) => scan_full_paginated(&rtxn, &db, offset, limit, |_, value| {
            decode_value(value)
                .ok()
                .map(|v| {
                    jsonpath::select(&v, path)
                        .map(|r| !r.is_empty())
                        .unwrap_or(false)
                })
                .unwrap_or(false)
        }),
    }
}

/// Legacy scan function - now implemented using optimized functions
pub fn scan(
    env: &Env,
    db_name: &str,
    mode: Mode<'_>,
    limit: usize,
) -> Result<Vec<(String, Vec<u8>)>> {
    scan_paginated(env, db_name, mode, 0, limit)
}

// Helper functions for optimized operations

fn open_database(rtxn: &RoTxn, env: &Env, db_name: &str) -> Result<Database<Str, Bytes>> {
    if db_name == "(unnamed)" {
        env.open_database(rtxn, None)?
            .ok_or_else(|| anyhow!("unnamed database not found"))
    } else {
        env.open_database(rtxn, Some(db_name))?
            .ok_or_else(|| anyhow!("database '{}' not found", db_name))
    }
}

fn count_prefix_matches(rtxn: &RoTxn, db: &Database<Str, Bytes>, prefix: &str) -> Result<usize> {
    let mut count = 0;
    let iter = db.iter(rtxn)?;

    for result in iter {
        let (key, _) = result?;
        if key.starts_with(prefix) {
            count += 1;
        } else if key > prefix {
            // Since keys are sorted, we can break early if we've passed the prefix
            break;
        }
    }
    Ok(count)
}

fn count_range_matches(
    rtxn: &RoTxn,
    db: &Database<Str, Bytes>,
    start: &str,
    end: &str,
) -> Result<usize> {
    let mut count = 0;
    let iter = db.iter(rtxn)?;

    for result in iter {
        let (key, _) = result?;
        if key >= start && key < end {
            count += 1;
        } else if key >= end {
            // Since keys are sorted, we can break early
            break;
        }
    }
    Ok(count)
}

fn count_full_scan_matches<F>(
    rtxn: &RoTxn,
    db: &Database<Str, Bytes>,
    predicate: F,
) -> Result<usize>
where
    F: Fn(&str, &[u8]) -> bool,
{
    let mut count = 0;
    let iter = db.iter(rtxn)?;
    for result in iter {
        let (key, value) = result?;
        if predicate(key, value) {
            count += 1;
        }
    }
    Ok(count)
}

fn scan_prefix_paginated(
    rtxn: &RoTxn,
    db: &Database<Str, Bytes>,
    prefix: &str,
    offset: usize,
    limit: usize,
) -> Result<Vec<(String, Vec<u8>)>> {
    let mut items = Vec::with_capacity(limit.min(10000));
    let mut skipped = 0;
    let iter = db.iter(rtxn)?;

    for result in iter {
        let (key, value) = result?;
        if key.starts_with(prefix) {
            if skipped >= offset {
                items.push((key.to_string(), value.to_vec()));
                if items.len() >= limit {
                    break;
                }
            } else {
                skipped += 1;
            }
        } else if key > prefix {
            // Since keys are sorted, we can break early if we've passed the prefix
            break;
        }
    }
    Ok(items)
}

fn scan_range_paginated(
    rtxn: &RoTxn,
    db: &Database<Str, Bytes>,
    start: &str,
    end: &str,
    offset: usize,
    limit: usize,
) -> Result<Vec<(String, Vec<u8>)>> {
    let mut items = Vec::with_capacity(limit.min(10000));
    let mut skipped = 0;
    let iter = db.iter(rtxn)?;

    for result in iter {
        let (key, value) = result?;
        if key >= start && key < end {
            if skipped >= offset {
                items.push((key.to_string(), value.to_vec()));
                if items.len() >= limit {
                    break;
                }
            } else {
                skipped += 1;
            }
        } else if key >= end {
            // Since keys are sorted, we can break early
            break;
        }
    }
    Ok(items)
}

fn scan_full_paginated<F>(
    rtxn: &RoTxn,
    db: &Database<Str, Bytes>,
    offset: usize,
    limit: usize,
    predicate: F,
) -> Result<Vec<(String, Vec<u8>)>>
where
    F: Fn(&str, &[u8]) -> bool,
{
    let mut items = Vec::with_capacity(limit.min(10000));
    let mut skipped = 0;
    let iter = db.iter(rtxn)?;

    for result in iter {
        let (key, value) = result?;
        if predicate(key, value) {
            if skipped >= offset {
                items.push((key.to_string(), value.to_vec()));
                if items.len() >= limit {
                    break;
                }
            } else {
                skipped += 1;
            }
        }
    }
    Ok(items)
}
