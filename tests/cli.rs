use assert_cmd::Command;

#[test]
fn shows_version() {
    let mut cmd = Command::cargo_bin("lmdb-tui").unwrap();
    cmd.arg("--version");
    cmd.assert().success();
}
