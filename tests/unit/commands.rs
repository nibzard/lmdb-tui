use heed::types::{Bytes, Str};
use lmdb_tui::{
    commands,
    db::{env::open_env, txn::Txn, undo::UndoStack},
};
use tempfile::tempdir;

#[test]
fn crud_with_undo() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    let mut tx = env.write_txn()?;
    env.create_database::<Str, Bytes>(&mut tx, Some("data"))?;
    tx.commit()?;

    let mut undo = UndoStack::new();
    let mut txn = Txn::begin(&env)?;

    commands::put(&env, &mut txn, &mut undo, "data", "foo", b"bar")?;
    assert_eq!(
        commands::get(&env, &txn, "data", "foo")?,
        Some(b"bar".to_vec())
    );

    commands::put(&env, &mut txn, &mut undo, "data", "foo", b"baz")?;
    assert_eq!(
        commands::get(&env, &txn, "data", "foo")?,
        Some(b"baz".to_vec())
    );

    undo.undo(&env, &mut txn)?;
    assert_eq!(
        commands::get(&env, &txn, "data", "foo")?,
        Some(b"bar".to_vec())
    );
    undo.redo(&env, &mut txn)?;
    assert_eq!(
        commands::get(&env, &txn, "data", "foo")?,
        Some(b"baz".to_vec())
    );

    commands::delete(&env, &mut txn, &mut undo, "data", "foo")?;
    assert_eq!(commands::get(&env, &txn, "data", "foo")?, None);
    undo.undo(&env, &mut txn)?;
    assert_eq!(
        commands::get(&env, &txn, "data", "foo")?,
        Some(b"baz".to_vec())
    );

    txn.commit()?;
    Ok(())
}

#[test]
fn export_import_json() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    let mut tx = env.write_txn()?;
    env.create_database::<Str, Str>(&mut tx, Some("data"))?;
    tx.commit()?;

    let mut undo = UndoStack::new();
    let mut txn = Txn::begin(&env)?;
    commands::put(&env, &mut txn, &mut undo, "data", "a", b"1")?;
    commands::put(&env, &mut txn, &mut undo, "data", "b", b"2")?;
    txn.commit()?;

    let mut buf = Vec::new();
    commands::export_json(&env, "data", &mut buf)?;

    let mut txn2 = Txn::begin(&env)?;
    let mut undo2 = UndoStack::new();
    commands::delete(&env, &mut txn2, &mut undo2, "data", "a")?;
    commands::delete(&env, &mut txn2, &mut undo2, "data", "b")?;
    commands::import_json(&env, &mut txn2, &mut undo2, "data", buf.as_slice())?;
    txn2.commit()?;

    let rtxn = env.read_txn()?;
    let db = env.open_database::<Str, Str>(&rtxn, Some("data"))?.unwrap();
    assert_eq!(db.get(&rtxn, "a")?, Some("1"));
    assert_eq!(db.get(&rtxn, "b")?, Some("2"));
    Ok(())
}
