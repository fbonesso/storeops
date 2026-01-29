use assert_cmd::Command;
use predicates::prelude::*;

fn storeops() -> Command {
    Command::cargo_bin("storeops").unwrap()
}

#[test]
fn help_exits_zero() {
    storeops()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Manage App Store Connect & Google Play Store",
        ));
}

#[test]
fn apple_help_shows_subcommands() {
    storeops()
        .args(["apple", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("apps"))
        .stdout(predicate::str::contains("builds"))
        .stdout(predicate::str::contains("versions"))
        .stdout(predicate::str::contains("testflight"))
        .stdout(predicate::str::contains("submit"))
        .stdout(predicate::str::contains("reviews"));
}

#[test]
fn google_help_shows_subcommands() {
    storeops()
        .args(["google", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("apps"))
        .stdout(predicate::str::contains("tracks"))
        .stdout(predicate::str::contains("builds"))
        .stdout(predicate::str::contains("testers"))
        .stdout(predicate::str::contains("submit"))
        .stdout(predicate::str::contains("reviews"));
}

#[test]
fn auth_help_shows_subcommands() {
    storeops()
        .args(["auth", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("login"))
        .stdout(predicate::str::contains("switch"))
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("init"));
}

#[test]
fn invalid_command_returns_nonzero() {
    storeops().arg("nonexistent").assert().failure();
}

#[test]
fn help_shows_global_flags() {
    storeops()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--output"))
        .stdout(predicate::str::contains("--pretty"))
        .stdout(predicate::str::contains("--profile"))
        .stdout(predicate::str::contains("--verbose"));
}
