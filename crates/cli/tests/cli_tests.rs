// crates/cli/tests/cli_tests.rs

use assert_cmd::Command;
use predicates::prelude::*;

// Helper function to get a command instance
fn get_command() -> Command {
    Command::cargo_bin("selfie-cli").unwrap()
}

#[test]
fn test_cli_help() {
    let mut cmd = get_command();
    cmd.arg("--help");
    cmd.assert().success().stdout(predicate::str::contains(
        "Selfie - A personal package manager",
    ));
}

#[test]
fn test_cli_version() {
    let mut cmd = get_command();
    cmd.arg("--version");
    cmd.assert().success();
}

#[test]
fn test_cli_invalid_command() {
    let mut cmd = get_command();
    cmd.arg("invalid-command");
    cmd.assert().failure();
}

#[test]
fn test_cli_invalid_subcommand() {
    let mut cmd = get_command();
    cmd.args(["package", "invalid-subcommand"]);
    cmd.assert().failure();
}

#[test]
fn test_cli_missing_required_arg() {
    let mut cmd = get_command();
    cmd.args(["package", "install"]); // Missing package_name
    cmd.assert().failure();
}

#[test]
fn test_cli_with_environment() {
    let mut cmd = get_command();
    // Just test that the arg is accepted, not that it does anything yet
    cmd.args(["-e", "test-env", "help"]);
    cmd.assert().success();
}

#[test]
fn test_cli_with_package_directory() {
    let mut cmd = get_command();
    // Just test that the arg is accepted, not that it does anything yet
    cmd.args(["-p", "/test/path", "help"]);
    cmd.assert().success();
}

#[test]
fn test_cli_verbose_flag() {
    let mut cmd = get_command();
    cmd.args(["-v", "help"]);
    cmd.assert().success();
}

#[test]
fn test_cli_no_color() {
    let mut cmd = get_command();
    cmd.args(["--no-color", "config", "validate"]);
    cmd.assert().success();
}

// The following tests just check that the CLI accepts these commands,
// but they don't verify actual functionality since that's not implemented yet

#[test]
fn test_cli_config_validate() {
    let mut cmd = get_command();
    cmd.args(["config", "validate"]);
    cmd.assert().success();
}

#[test]
fn test_cli_package_list() {
    let mut cmd = get_command();
    cmd.args(["package", "list"]);
    cmd.assert().success();
}

#[test]
fn test_cli_package_info() {
    let mut cmd = get_command();
    cmd.args(["package", "info", "test-package"]);
    cmd.assert().success();
}

#[test]
fn test_cli_package_install() {
    let mut cmd = get_command();
    cmd.args(["package", "install", "test-package"]);
    cmd.assert().success();
}

#[test]
fn test_cli_package_create() {
    let mut cmd = get_command();
    cmd.args(["package", "create", "test-package"]);
    cmd.assert().success();
}

#[test]
fn test_cli_package_validate() {
    let mut cmd = get_command();
    cmd.args(["package", "validate", "test-package"]);
    cmd.assert().success();
}

#[test]
fn test_cli_package_validate_with_path() {
    let mut cmd = get_command();
    cmd.args([
        "package",
        "validate",
        "test-package",
        "--package-path",
        "/test/path/test-package.yaml",
    ]);
    cmd.assert().success();
}
