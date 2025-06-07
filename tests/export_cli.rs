use assert_cmd::Command;
use heed::types::{Bytes, Str};
use lmdb_tui::db::env::open_env;
use std::fs;
use tempfile::tempdir;

#[test]
fn export_import_via_cli() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env_dir = dir.path().join("env");
    fs::create_dir(&env_dir)?;
    let env = open_env(&env_dir, false)?;
    let mut tx = env.write_txn()?;
    env.create_database::<Str, Bytes>(&mut tx, Some("data"))?;
    tx.commit()?;
    let mut tx = env.write_txn()?;
    let db = env.open_database::<Str, Bytes>(&tx, Some("data"))?.unwrap();
    db.put(&mut tx, "a", br#"{"n":1}"#)?;
    tx.commit()?;

    let out = dir.path().join("out.json");
    Command::cargo_bin("lmdb-tui")?
        .args([
            "export",
            "--db",
            "data",
            "--out",
            out.to_str().unwrap(),
            env_dir.to_str().unwrap(),
        ])
        .assert()
        .success();

    Command::cargo_bin("lmdb-tui")?
        .args([
            "import",
            "--db",
            "data",
            "--input",
            out.to_str().unwrap(),
            env_dir.to_str().unwrap(),
        ])
        .assert()
        .success();
    Ok(())
}
