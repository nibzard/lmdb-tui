use heed::types::{Bytes, Str};
use lmdb_tui::{
    app::{Action, App},
    config::Config,
    db::env::{list_databases, open_env},
};
use tempfile::tempdir;

#[test]
fn reducer_switches_databases() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    let mut wtxn = env.write_txn()?;
    let db1 = env.create_database::<Str, Str>(&mut wtxn, Some("first"))?;
    db1.put(&mut wtxn, "k1", "v1")?;
    let db2 = env.create_database::<Str, Bytes>(&mut wtxn, Some("second"))?;
    db2.put(&mut wtxn, "k2", b"v2")?;
    wtxn.commit()?;

    let names = list_databases(&env)?;
    let config = Config::default();
    let mut app = App::new(env, names, config)?;
    assert_eq!(app.selected, 0);
    assert_eq!(app.entries.len(), 1);
    assert_eq!(app.entries[0].0, "k1");

    app.reduce(Action::NextDb)?;
    assert_eq!(app.selected, 1);
    assert_eq!(app.entries.len(), 1);
    assert_eq!(app.entries[0].0, "k2");
    Ok(())
}

#[test]
fn reducer_toggles_help_and_clears_query() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    let names = list_databases(&env)?;
    let config = Config::default();
    let mut app = App::new(env, names, config)?;

    assert!(!app.show_help);
    app.reduce(Action::ToggleHelp)?;
    assert!(app.show_help);
    app.help_query.push('a');
    app.reduce(Action::ToggleHelp)?;
    assert!(!app.show_help);
    assert!(app.help_query.is_empty());
    Ok(())
}
