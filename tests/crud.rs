use heed::types::{Bytes, Str};
use lmdb_tui::db::{
    env::open_env,
    kv,
    txn::Txn,
    undo::{Op, UndoStack},
};
use tempfile::tempdir;

#[test]
fn put_get_delete_commit() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;

    // create db "data"
    let mut tx = env.write_txn()?;
    env.create_database::<Str, Bytes>(&mut tx, Some("data"))?;
    tx.commit()?;

    let mut txn = Txn::begin(&env)?;
    kv::put(&env, &mut txn, "data", "foo", b"bar")?;
    assert_eq!(kv::get(&env, &txn, "data", "foo")?, Some(b"bar".to_vec()));
    kv::delete(&env, &mut txn, "data", "foo")?;
    assert_eq!(kv::get(&env, &txn, "data", "foo")?, None);
    kv::put(&env, &mut txn, "data", "foo", b"baz")?;
    txn.commit()?;

    // verify persisted
    let rtxn = env.read_txn()?;
    let db = env
        .open_database::<Str, Bytes>(&rtxn, Some("data"))?
        .unwrap();
    assert_eq!(
        db.get(&rtxn, "foo")?.map(|v| v.to_vec()),
        Some(b"baz".to_vec())
    );
    Ok(())
}

#[test]
fn txn_abort_discards_changes() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;

    let mut tx = env.write_txn()?;
    env.create_database::<Str, Bytes>(&mut tx, Some("data"))?;
    tx.commit()?;

    let mut txn = Txn::begin(&env)?;
    kv::put(&env, &mut txn, "data", "foo", b"bar")?;
    txn.abort();

    let rtxn = env.read_txn()?;
    let db = env
        .open_database::<Str, Bytes>(&rtxn, Some("data"))?
        .unwrap();
    assert_eq!(db.get(&rtxn, "foo")?, None);
    Ok(())
}

#[test]
fn undo_redo_works() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;

    let mut tx = env.write_txn()?;
    env.create_database::<Str, Bytes>(&mut tx, Some("data"))?;
    tx.commit()?;

    let mut undo = UndoStack::new();
    let mut txn = Txn::begin(&env)?;
    kv::put(&env, &mut txn, "data", "foo", b"bar")?;
    undo.push(Op::Put {
        db: "data".into(),
        key: "foo".into(),
        prev: None,
        new: b"bar".to_vec(),
    });
    kv::put(&env, &mut txn, "data", "foo", b"baz")?;
    undo.push(Op::Put {
        db: "data".into(),
        key: "foo".into(),
        prev: Some(b"bar".to_vec()),
        new: b"baz".to_vec(),
    });

    undo.undo(&env, &mut txn)?;
    assert_eq!(kv::get(&env, &txn, "data", "foo")?, Some(b"bar".to_vec()));
    undo.redo(&env, &mut txn)?;
    assert_eq!(kv::get(&env, &txn, "data", "foo")?, Some(b"baz".to_vec()));

    txn.commit()?;
    let rtxn = env.read_txn()?;
    let db = env
        .open_database::<Str, Bytes>(&rtxn, Some("data"))?
        .unwrap();
    assert_eq!(
        db.get(&rtxn, "foo")?.map(|v| v.to_vec()),
        Some(b"baz".to_vec())
    );
    Ok(())
}
