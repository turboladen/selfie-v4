use assert_cmd::Command;
use std::{fs, io::Write};
use tempfile::TempDir;

fn setup_test_config(yaml_content: &str) -> TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("selfie");
    fs::create_dir_all(&config_dir).unwrap();

    let config_path = config_dir.join("config.yaml");
    let mut config_file = fs::File::create(&config_path).unwrap();
    writeln!(config_file, "{}", yaml_content).unwrap();

    temp_dir
}

fn get_command_with_test_config(temp_dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("selfie-cli").unwrap();
    cmd.env(
        "SELFIE_CONFIG_DIR",
        temp_dir.path().join(".config").join("selfie"),
    );
    cmd
}

#[test]
fn test_validate_valid_config() {
    // Valid config with all required fields
    let yaml = r#"
environment: "test-env"
package_directory: "/test/packages"
"#;

    let temp_dir = setup_test_config(yaml);
    let mut cmd = get_command_with_test_config(&temp_dir);
    cmd.args(["config", "validate"]);

    cmd.assert().success().stdout(predicates::str::contains(
        "Configuration validation successful",
    ));
}

#[test]
fn test_validate_invalid_config() {
    // Invalid config with missing required fields
    let yaml = r#"
# Missing environment
package_directory: "/test/packages"
"#;

    let temp_dir = setup_test_config(yaml);
    let mut cmd = get_command_with_test_config(&temp_dir);
    cmd.args(["config", "validate"]);

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("missing field `environment`"));
}

#[test]
fn test_validate_config_with_invalid_path() {
    // Config with invalid package directory (not absolute)
    let yaml = r#"
environment: "test-env"
package_directory: "relative/path"
"#;

    let temp_dir = setup_test_config(yaml);
    let mut cmd = get_command_with_test_config(&temp_dir);
    cmd.args(["config", "validate"]);

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("must be an absolute path"));
}
