use assert_cmd::cargo::cargo_bin;
use lmdb_tui::remote::RemoteClient;
use tempfile::tempdir;

#[test]
fn remote_lists_databases() -> anyhow::Result<()> {
    use heed::types::Unit;
    use lmdb_tui::db::env::open_env;

    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    let mut tx = env.write_txn()?;
    env.create_database::<Unit, Unit>(&mut tx, Some("foo"))?;
    tx.commit()?;

    std::env::set_var("LMDB_TUI_AGENT_PATH", cargo_bin("lmdb-tui-agent"));
    let mut client = RemoteClient::connect("local")?;
    let mut names = client.list_databases(dir.path(), true)?;
    names.sort();
    assert_eq!(names, vec!["foo".to_string()]);
    Ok(())
}
