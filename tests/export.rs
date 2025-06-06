use heed::types::{Bytes, Str};
use lmdb_tui::db::{env::list_entries, env::open_env, io};
use tempfile::tempdir;

#[test]
fn export_import_json_roundtrip() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    let mut tx = env.write_txn()?;
    let db = env.create_database::<Str, Bytes>(&mut tx, Some("data"))?;
    db.put(&mut tx, "a", br#"{"n":1}"#)?;
    db.put(&mut tx, "b", br#"{"n":2}"#)?;
    tx.commit()?;
    let path = dir.path().join("out.json");
    io::export_json(&env, "data", &path)?;
    let mut tx = env.write_txn()?;
    db.clear(&mut tx)?;
    tx.commit()?;
    io::import_json(&env, "data", &path)?;
    let entries = list_entries(&env, "data", 10)?;
    let keys: Vec<String> = entries.iter().map(|(k, _)| k.clone()).collect();
    assert_eq!(keys.len(), 2);
    Ok(())
}

#[test]
fn export_json_validates() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    let mut tx = env.write_txn()?;
    let db = env.create_database::<Str, Bytes>(&mut tx, Some("data"))?;
    db.put(&mut tx, "a", b"not json")?;
    tx.commit()?;
    let path = dir.path().join("bad.json");
    assert!(io::export_json(&env, "data", &path).is_err());
    Ok(())
}

#[test]
fn export_import_csv_roundtrip() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    let mut tx = env.write_txn()?;
    let db = env.create_database::<Str, Str>(&mut tx, Some("data"))?;
    db.put(&mut tx, "x", "1")?;
    db.put(&mut tx, "y", "2")?;
    tx.commit()?;
    let path = dir.path().join("out.csv");
    io::export_csv(&env, "data", &path)?;
    let mut tx = env.write_txn()?;
    db.clear(&mut tx)?;
    tx.commit()?;
    io::import_csv(&env, "data", &path)?;
    let entries = list_entries(&env, "data", 10)?;
    assert_eq!(entries.len(), 2);
    Ok(())
}
