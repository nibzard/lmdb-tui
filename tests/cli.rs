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
        .stdout(contains("Simple LMDB TUI explorer"));
}
