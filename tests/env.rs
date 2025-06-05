use heed::types::{Str, Unit};
use lmdb_tui::db::env::{list_databases, open_env};
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
