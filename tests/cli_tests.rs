use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

fn storeops() -> assert_cmd::Command {
    cargo_bin_cmd!("storeops")
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
        .stdout(predicate::str::contains("--json"))
        .stdout(predicate::str::contains("--pretty"))
        .stdout(predicate::str::contains("--profile"))
        .stdout(predicate::str::contains("--verbose"));
}

// Apple subcommand tests
#[test]
fn apple_apps_help_shows_subsubcommands() {
    storeops()
        .args(["apple", "apps", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("info"));
}

#[test]
fn apple_versions_help_shows_subsubcommands() {
    storeops()
        .args(["apple", "versions", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("create"));
}

#[test]
fn apple_builds_help_shows_subsubcommands() {
    storeops()
        .args(["apple", "builds", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("info"));
}

#[test]
fn apple_testflight_help_shows_subsubcommands() {
    storeops()
        .args(["apple", "testflight", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("groups"))
        .stdout(predicate::str::contains("testers"));
}

#[test]
fn apple_reviews_help_shows_subsubcommands() {
    storeops()
        .args(["apple", "reviews", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("respond"));
}

#[test]
fn apple_metadata_help_shows_subsubcommands() {
    storeops()
        .args(["apple", "metadata", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("localizations"))
        .stdout(predicate::str::contains("categories"));
}

#[test]
fn apple_screenshots_help_shows_subsubcommands() {
    storeops()
        .args(["apple", "screenshots", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("sets"))
        .stdout(predicate::str::contains("images"));
}

#[test]
fn apple_analytics_help_shows_subsubcommands() {
    storeops()
        .args(["apple", "analytics", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("sales"));
}

#[test]
fn apple_submit_shows_required_args() {
    storeops()
        .args(["apple", "submit", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("APP_ID"))
        .stdout(predicate::str::contains("--version"));
}

// Google subcommand tests
#[test]
fn google_apps_help_shows_subsubcommands() {
    storeops()
        .args(["google", "apps", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("info"));
}

#[test]
fn google_tracks_help_shows_subsubcommands() {
    storeops()
        .args(["google", "tracks", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("update"));
}

#[test]
fn google_builds_help_shows_subsubcommands() {
    storeops()
        .args(["google", "builds", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("upload"));
}

#[test]
fn google_testers_help_shows_subsubcommands() {
    storeops()
        .args(["google", "testers", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("add"));
}

#[test]
fn google_reviews_help_shows_subsubcommands() {
    storeops()
        .args(["google", "reviews", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("reply"));
}

#[test]
fn google_listings_help_shows_subsubcommands() {
    storeops()
        .args(["google", "listings", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("update"));
}

#[test]
fn google_images_help_shows_subsubcommands() {
    storeops()
        .args(["google", "images", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("upload"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn google_submit_shows_required_args() {
    storeops()
        .args(["google", "submit", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("PACKAGE_NAME"))
        .stdout(predicate::str::contains("--track"));
}
