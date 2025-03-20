// crates/cli/tests/cli_tests.rs

use assert_cmd::Command;
use predicates::prelude::*;
use std::{fs, io::Write};
use tempfile::TempDir;

// Helper to create a temporary config environment
fn setup_test_config() -> TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("selfie");
    fs::create_dir_all(&config_dir).unwrap();

    let config_path = config_dir.join("config.yaml");
    let mut config_file = fs::File::create(&config_path).unwrap();

    // Write minimal valid config
    writeln!(config_file, "environment: test-env").unwrap();
    writeln!(
        config_file,
        "package_directory: {}",
        temp_dir.path().join("packages").display()
    )
    .unwrap();

    temp_dir
}

// Helper function to get a command instance with environment variables pointing to our test config
fn get_command_with_test_config(temp_dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("selfie-cli").unwrap();

    // Override the config directory location
    // This assumes we can add a CLI flag or env var to override the config directory
    cmd.env(
        "SELFIE_CONFIG_DIR",
        temp_dir.path().join(".config").join("selfie"),
    );

    cmd
}

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
    let temp_dir = setup_test_config();
    let mut cmd = get_command_with_test_config(&temp_dir);
    cmd.args(["--no-color", "config", "validate"]);
    cmd.assert().success();
}

// The following tests just check that the CLI accepts these commands,
// but they don't verify actual functionality since that's not implemented yet

#[test]
fn test_cli_config_validate() {
    let temp_dir = setup_test_config();
    let mut cmd = get_command_with_test_config(&temp_dir);
    cmd.args(["config", "validate"]);
    cmd.assert().success();
}

#[test]
fn test_cli_package_list() {
    let temp_dir = setup_test_config();
    let mut cmd = get_command_with_test_config(&temp_dir);
    cmd.args(["package", "list"]);
    cmd.assert().success();
}

#[test]
fn test_cli_package_info() {
    let temp_dir = setup_test_config();
    let mut cmd = get_command_with_test_config(&temp_dir);
    cmd.args(["package", "info", "test-package"]);
    cmd.assert().success();
}

#[test]
fn test_cli_package_install() {
    let temp_dir = setup_test_config();
    let mut cmd = get_command_with_test_config(&temp_dir);
    cmd.args(["package", "install", "test-package"]);
    cmd.assert().success();
}

#[test]
fn test_cli_package_create() {
    let temp_dir = setup_test_config();
    let mut cmd = get_command_with_test_config(&temp_dir);
    cmd.args(["package", "create", "test-package"]);
    cmd.assert().success();
}

#[test]
fn test_cli_package_validate() {
    let temp_dir = setup_test_config();
    let mut cmd = get_command_with_test_config(&temp_dir);
    cmd.args(["package", "validate", "test-package"]);
    cmd.assert().success();
}

#[test]
fn test_cli_package_validate_with_path() {
    let temp_dir = setup_test_config();
    let mut cmd = get_command_with_test_config(&temp_dir);
    cmd.args([
        "package",
        "validate",
        "test-package",
        "--package-path",
        "/test/path/test-package.yaml",
    ]);
    cmd.assert().success();
}
