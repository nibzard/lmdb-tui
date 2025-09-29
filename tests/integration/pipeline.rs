use assert_cmd::Command;
use predicates::str::contains;
use tempfile::tempdir;

#[test]
fn plain_output_line_count() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env_path = dir.path();

    let output = Command::cargo_bin("lmdb-tui")?
        .args(["--plain", env_path.to_str().unwrap()])
        .output()?;
    assert!(output.status.success());
    let count = String::from_utf8_lossy(&output.stdout).lines().count();
    assert!(count <= 1);
    Ok(())
}

#[test]
fn json_output_empty_array() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env_path = dir.path();

    Command::cargo_bin("lmdb-tui")?
        .args(["--json", env_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("[]"));
    Ok(())
}
