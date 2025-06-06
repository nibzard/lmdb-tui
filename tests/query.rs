use heed::types::{Bytes, Str};
use lmdb_tui::db::{
    env::open_env,
    query::{self, decode_value, Mode},
};
use regex::Regex;
use tempfile::tempdir;

#[test]
fn prefix_scan_returns_matching_keys() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    let mut tx = env.write_txn()?;
    let db = env.create_database::<Str, Str>(&mut tx, Some("data"))?;
    db.put(&mut tx, "apple", "1")?;
    db.put(&mut tx, "banana", "2")?;
    db.put(&mut tx, "apricot", "3")?;
    tx.commit()?;

    let items = query::scan(&env, "data", Mode::Prefix("ap"), 10)?;
    let keys: Vec<String> = items.iter().map(|(k, _)| k.clone()).collect();
    assert_eq!(keys, vec!["apple", "apricot"]);
    Ok(())
}

#[test]
fn range_scan_filters_keys() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    let mut tx = env.write_txn()?;
    let db = env.create_database::<Str, Str>(&mut tx, Some("data"))?;
    db.put(&mut tx, "a", "1")?;
    db.put(&mut tx, "b", "2")?;
    db.put(&mut tx, "c", "3")?;
    tx.commit()?;

    let items = query::scan(&env, "data", Mode::Range("a", "c"), 10)?;
    let keys: Vec<String> = items.iter().map(|(k, _)| k.clone()).collect();
    assert_eq!(keys, vec!["a", "b"]);
    Ok(())
}

#[test]
fn regex_scan_filters_keys() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    let mut tx = env.write_txn()?;
    let db = env.create_database::<Str, Str>(&mut tx, Some("data"))?;
    db.put(&mut tx, "foo1", "1")?;
    db.put(&mut tx, "bar", "2")?;
    db.put(&mut tx, "foo2", "3")?;
    tx.commit()?;

    let re = Regex::new(r"^foo[0-9]")?;
    let items = query::scan(&env, "data", Mode::Regex(re), 10)?;
    let keys: Vec<String> = items.iter().map(|(k, _)| k.clone()).collect();
    assert_eq!(keys, vec!["foo1", "foo2"]);
    Ok(())
}

#[test]
fn jsonpath_filters_values() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    let mut tx = env.write_txn()?;
    let db = env.create_database::<Str, Bytes>(&mut tx, Some("data"))?;
    db.put(&mut tx, "1", br#"{"name":"foo"}"#)?;
    db.put(&mut tx, "2", br#"{"name":"bar"}"#)?;
    tx.commit()?;

    let items = query::scan(&env, "data", Mode::JsonPath("$[?(@.name=='foo')]"), 10)?;
    let keys: Vec<String> = items.iter().map(|(k, _)| k.clone()).collect();
    assert_eq!(keys, vec!["1"]);
    Ok(())
}

#[test]
fn jsonpath_handles_msgpack() -> anyhow::Result<()> {
    use serde_json::json;

    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    let mut tx = env.write_txn()?;
    let db = env.create_database::<Str, Bytes>(&mut tx, Some("data"))?;
    let mp_foo = rmp_serde::to_vec(&json!({"name": "foo"}))?;
    let mp_bar = rmp_serde::to_vec(&json!({"name": "bar"}))?;
    db.put(&mut tx, "1", &mp_foo)?;
    db.put(&mut tx, "2", &mp_bar)?;
    tx.commit()?;

    let items = query::scan(&env, "data", Mode::JsonPath("$[?(@.name=='foo')]"), 10)?;
    let keys: Vec<String> = items.iter().map(|(k, _)| k.clone()).collect();
    assert_eq!(keys, vec!["1"]);
    Ok(())
}

#[test]
fn decode_value_parses_formats() -> anyhow::Result<()> {
    use serde_json::json;

    let json_bytes = br#"{"a":1}"#;
    let mp_bytes = rmp_serde::to_vec(&json!({"a": 1}))?;

    let v1 = decode_value(json_bytes)?;
    let v2 = decode_value(&mp_bytes)?;

    assert_eq!(v1, v2);
    Ok(())
}
