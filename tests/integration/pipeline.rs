use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn plain_output_line_count() -> anyhow::Result<()> {
    let output = Command::cargo_bin("lmdb-tui")?
        .args(["--plain", "test_env"])
        .output()?;
    assert!(output.status.success());
    let count = String::from_utf8_lossy(&output.stdout).lines().count();
    assert!(count <= 1);
    Ok(())
}

#[test]
fn json_output_empty_array() -> anyhow::Result<()> {
    Command::cargo_bin("lmdb-tui")?
        .args(["--json", "test_env"])
        .assert()
        .success()
        .stdout(contains("[]"));
    Ok(())
}
