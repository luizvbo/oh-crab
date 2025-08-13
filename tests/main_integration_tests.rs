// FILE: ./tests/main_integration_tests.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_alias_generation() {
    let mut cmd = Command::cargo_bin("ohcrab").unwrap();
    cmd.arg("--alias=crabalias")
        .assert()
        .success()
        .stdout(predicate::str::contains("crabalias"));
}

#[test]
fn test_command_correction_suggestion() {
    let mut cmd = Command::cargo_bin("ohcrab").unwrap();
    cmd.arg("--select-first")
        .arg("--")
        // Use a command with a single, correctable error
        .arg("git")
        .arg("brnch")
        .assert()
        .success()
        // Assert the correct, single-step suggestion
        .stdout(predicate::str::contains("git branch"));
}

#[test]
fn test_debug_output() {
    let mut cmd = Command::cargo_bin("ohcrab").unwrap();
    cmd.arg("--select-first")
        .arg("--debug")
        .arg("--")
        // Use the same robust command here
        .arg("git")
        .arg("brnch")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Retrieved command(s):")
                .and(predicate::str::contains("git branch")),
        );
}
