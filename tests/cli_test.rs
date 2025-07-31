use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_without_args() {
    let mut cmd = Command::cargo_bin("sc-tui").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No default workspace configured").or(predicate::str::contains("No configuration file found")));
}

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("sc-tui").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("TUI client for Shortcut stories"))
        .stdout(predicate::str::contains("--token"))
        .stdout(predicate::str::contains("--workspace"))
        .stdout(predicate::str::contains("--limit"))
        .stdout(predicate::str::contains("--debug"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("sc-tui").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("sc-tui"));
}

#[test]
fn test_cli_missing_token() {
    let mut cmd = Command::cargo_bin("sc-tui").unwrap();
    cmd.arg("testuser")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Either --token or --workspace must be provided"));
}

#[test]
fn test_cli_debug_flag() {
    // This test verifies the debug flag is accepted
    // We can't test the full TUI interaction easily in integration tests
    let mut cmd = Command::cargo_bin("sc-tui").unwrap();
    cmd.arg("testuser")
        .arg("--token")
        .arg("fake-token")
        .arg("--debug")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_cli_limit_validation() {
    let mut cmd = Command::cargo_bin("sc-tui").unwrap();
    cmd.arg("testuser")
        .arg("--token")
        .arg("fake-token")
        .arg("--limit")
        .arg("not-a-number")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));
}