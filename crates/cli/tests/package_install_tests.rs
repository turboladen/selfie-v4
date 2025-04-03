// crates/cli/tests/package_install_tests.rs
use assert_cmd::Command;
use predicates::prelude::*;
use std::{fs, io::Write};
use tempfile::TempDir;

fn setup_test_env() -> TempDir {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create config
    let config_dir = temp_dir.path().join(".config").join("selfie");
    fs::create_dir_all(&config_dir).unwrap();

    let config_path = config_dir.join("config.yaml");
    let mut config_file = fs::File::create(&config_path).unwrap();

    writeln!(config_file, "environment: test-env").unwrap();
    writeln!(
        config_file,
        "package_directory: {}",
        temp_dir.path().join("packages").display()
    )
    .unwrap();

    // Create package directory
    let packages_dir = temp_dir.path().join("packages");
    fs::create_dir_all(&packages_dir).unwrap();

    // Create test package
    let package_yaml = r#"
name: test-package
version: 1.0.0
environments:
  test-env:
    install: echo "Installing test package"
    check: echo "Checking test package"
"#;

    let package_path = packages_dir.join("test-package.yaml");
    fs::write(&package_path, package_yaml).unwrap();

    temp_dir
}

#[test]
fn test_package_install() {
    let temp_dir = setup_test_env();

    let mut cmd = Command::cargo_bin("selfie-cli").unwrap();
    cmd.env(
        "SELFIE_CONFIG_DIR",
        temp_dir.path().join(".config").join("selfie"),
    );

    cmd.args(["package", "install", "test-package"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Installing package"));
}
