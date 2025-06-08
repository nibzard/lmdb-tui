use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn shows_version() {
    let mut cmd = Command::cargo_bin("lmdb-tui").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn shows_help() {
    let mut cmd = Command::cargo_bin("lmdb-tui").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("Simple LMDB TUI explorer"))
        .stdout(contains("https://lmdb.nibzard.com"));
}

#[test]
fn missing_env_returns_code_2() {
    let mut cmd = Command::cargo_bin("lmdb-tui").unwrap();
    cmd.arg("/no/such/path");
    cmd.assert().code(2);
}

#[test]
fn no_args_shows_help() {
    let mut cmd = Command::cargo_bin("lmdb-tui").unwrap();
    cmd.assert().success().stdout(contains("Usage:"));
}

#[test]
fn help_contains_readme_link() {
    let mut cmd = Command::cargo_bin("lmdb-tui").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("https://lmdb.nibzard.com"));
}

#[test]
fn plain_lists_databases() -> anyhow::Result<()> {
    use heed::types::Unit;
    use lmdb_tui::db::env::open_env;
    use tempfile::tempdir;

    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    let mut tx = env.write_txn()?;
    env.create_database::<Unit, Unit>(&mut tx, Some("foo"))?;
    tx.commit()?;

    let mut cmd = Command::cargo_bin("lmdb-tui").unwrap();
    cmd.args(["--plain", dir.path().to_str().unwrap()]);
    cmd.assert().success().stdout(contains("foo"));
    Ok(())
}

