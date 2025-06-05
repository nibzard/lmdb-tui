use heed::types::{Str, Unit};
use lmdb_tui::db::env::{list_databases, list_entries, open_env};
use tempfile::tempdir;

#[test]
fn lists_created_databases() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    let mut wtxn = env.write_txn()?;
    env.create_database::<Str, Unit>(&mut wtxn, Some("first"))?;
    env.create_database::<Str, Unit>(&mut wtxn, Some("second"))?;
    wtxn.commit()?;
    let mut names = list_databases(&env)?;
    names.sort();
    assert_eq!(names, vec!["first".to_string(), "second".to_string()]);
    Ok(())
}

#[test]
fn lists_entries() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    let mut wtxn = env.write_txn()?;
    let db = env.create_database::<Str, Str>(&mut wtxn, Some("entries"))?;
    db.put(&mut wtxn, "a", "1")?;
    db.put(&mut wtxn, "b", "2")?;
    db.put(&mut wtxn, "c", "3")?;
    wtxn.commit()?;

    let entries = list_entries(&env, "entries", 2)?;
    assert_eq!(
        entries,
        vec![
            ("a".to_string(), b"1".to_vec()),
            ("b".to_string(), b"2".to_vec()),
        ]
    );
    Ok(())
}

#[test]
fn opens_env_read_only() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env_rw = open_env(dir.path(), false)?;
    env_rw.prepare_for_closing().wait();
    let env = open_env(dir.path(), true)?;
    let _rtxn = env.read_txn()?;
    Ok(())
}
