//! Test package file creation helpers to eliminate duplication in service tests.

use crate::constants::{SERVICE_TEST_ENV, TEST_ENV, TEST_VERSION};
use std::{fs, path::PathBuf};
use tempfile::TempDir;

/// Creates a standard test package file with install and check commands.
/// This is the most commonly used package file in service tests.
///
/// # Example
/// ```rust
/// let temp_dir = TempDir::new().unwrap();
/// let package_path = create_test_package_file(&temp_dir, "my-package");
/// ```
pub fn create_test_package_file(dir: &TempDir, name: &str) -> PathBuf {
    create_package_file_with_check(dir, name, true)
}

/// Creates a test package file with optional check command.
/// Gives you control over whether the package has a check command defined.
///
/// # Arguments
/// * `dir` - Temporary directory to create the package file in
/// * `name` - Name of the package
/// * `has_check` - Whether to include a check command
///
/// # Example
/// ```rust
/// // Package with check command
/// let with_check = create_package_file_with_check(&temp_dir, "pkg1", true);
///
/// // Package without check command
/// let no_check = create_package_file_with_check(&temp_dir, "pkg2", false);
/// ```
pub fn create_package_file_with_check(dir: &TempDir, name: &str, has_check: bool) -> PathBuf {
    let check_command = if has_check {
        format!("\n    check: \"echo 'checking {}'\"", name)
    } else {
        String::new()
    };

    let content = format!(
        r#"name: "{}"
version: "{}"
description: "Test package for service layer testing"
homepage: "https://example.com/{}"

environments:
  {}:
    install: "echo 'installing {}'"{}
    dependencies: []
"#,
        name, TEST_VERSION, name, TEST_ENV, name, check_command
    );

    let file_path = dir.path().join(format!("{}.yml", name));
    fs::write(&file_path, content).unwrap();
    file_path
}

/// Creates a test package file for multiple environments.
/// Useful for testing cross-environment behavior and environment selection.
///
/// # Example
/// ```rust
/// let package_path = create_multi_env_package_file(&temp_dir, "cross-platform-tool");
/// ```
pub fn create_multi_env_package_file(dir: &TempDir, name: &str) -> PathBuf {
    let content = format!(
        r#"name: "{}"
version: "{}"
description: "Multi-environment test package"
homepage: "https://example.com/{}"

environments:
  {}:
    install: "echo 'installing {} in test'"
    check: "echo 'checking {} in test'"
    dependencies: []
  prod:
    install: "apt-get install {}"
    check: "which {}"
    dependencies: ["build-essential"]
  macos:
    install: "brew install {}"
    check: "which {}"
    dependencies: []
"#,
        name, TEST_VERSION, name, TEST_ENV, name, name, name, name, name, name
    );

    let file_path = dir.path().join(format!("{}.yml", name));
    fs::write(&file_path, content).unwrap();
    file_path
}

/// Creates an invalid package file for error testing.
/// Contains malformed YAML that should cause parsing errors.
///
/// # Example
/// ```rust
/// let invalid_path = create_invalid_package_file(&temp_dir, "broken-package");
/// // This file will cause YAML parsing errors when loaded
/// ```
pub fn create_invalid_package_file(dir: &TempDir, name: &str) -> PathBuf {
    let content = r#"# Invalid YAML - syntax error
name: "invalid-package"
version: "1.0.0"
environments:
  test:
    install: "echo 'test'
    # Missing closing quote above - this will cause YAML parse error
"#;

    let file_path = dir.path().join(format!("{}.yml", name));
    fs::write(&file_path, content).unwrap();
    file_path
}

/// Creates a package file missing required fields for validation testing.
/// Contains valid YAML but missing fields required by the package schema.
///
/// # Example
/// ```rust
/// let incomplete_path = create_incomplete_package_file(&temp_dir, "incomplete-pkg");
/// // This file will cause validation errors when processed
/// ```
pub fn create_incomplete_package_file(dir: &TempDir, name: &str) -> PathBuf {
    let content = format!(
        r#"name: "{}"
# Missing version field - this should cause validation errors
description: "Package missing required fields"
environments:
  {}:
    install: "echo 'installing'"
    # Missing other potentially required fields
"#,
        name, TEST_ENV
    );

    let file_path = dir.path().join(format!("{}.yml", name));
    fs::write(&file_path, content).unwrap();
    file_path
}

/// Creates a package file with custom fields for advanced testing.
/// Allows you to specify custom YAML content while still using standard structure.
///
/// # Arguments
/// * `dir` - Temporary directory to create the package file in
/// * `name` - Name of the package
/// * `version` - Version string
/// * `environment` - Environment name
/// * `install_cmd` - Install command to use
/// * `check_cmd` - Optional check command
///
/// # Example
/// ```rust
/// let custom_path = create_custom_package_file(
///     &temp_dir,
///     "custom-tool",
///     "2.1.0",
///     "development",
///     "make install",
///     Some("make test")
/// );
/// ```
pub fn create_custom_package_file(
    dir: &TempDir,
    name: &str,
    version: &str,
    environment: &str,
    install_cmd: &str,
    check_cmd: Option<&str>,
) -> PathBuf {
    let check_section = if let Some(cmd) = check_cmd {
        format!("\n    check: \"{}\"", cmd)
    } else {
        String::new()
    };

    let content = format!(
        r#"name: "{}"
version: "{}"
description: "Custom test package"

environments:
  {}:
    install: "{}"{}
    dependencies: []
"#,
        name, version, environment, install_cmd, check_section
    );

    let file_path = dir.path().join(format!("{}.yml", name));
    fs::write(&file_path, content).unwrap();
    file_path
}

/// Creates a package file with failing commands for error scenario testing.
/// Commands are designed to fail when executed, useful for testing error handling.
///
/// # Example
/// ```rust
/// let failing_path = create_failing_package_file(&temp_dir, "broken-tool");
/// // This package's commands will fail when executed
/// ```
pub fn create_failing_package_file(dir: &TempDir, name: &str) -> PathBuf {
    create_custom_package_file(
        dir,
        name,
        TEST_VERSION,
        TEST_ENV,
        "exit 1",       // Install command that fails
        Some("exit 1"), // Check command that fails
    )
}

/// Creates a package file with slow commands for timeout testing.
/// Commands include sleep statements to test timeout handling.
///
/// # Arguments
/// * `dir` - Temporary directory to create the package file in
/// * `name` - Name of the package
/// * `sleep_seconds` - Number of seconds for commands to sleep
///
/// # Example
/// ```rust
/// let slow_path = create_slow_package_file(&temp_dir, "slow-tool", 10);
/// // This package's commands will sleep for 10 seconds
/// ```
pub fn create_slow_package_file(dir: &TempDir, name: &str, sleep_seconds: u32) -> PathBuf {
    create_custom_package_file(
        dir,
        name,
        TEST_VERSION,
        TEST_ENV,
        &format!("sleep {} && echo 'installed'", sleep_seconds),
        Some(&format!("sleep {} && echo 'checked'", sleep_seconds)),
    )
}

/// Creates a test package file for service tests using the correct "test" environment.
/// This is specifically for service layer integration tests.
pub fn create_service_test_package_file(dir: &TempDir, name: &str, has_check: bool) -> PathBuf {
    let check_command = if has_check {
        format!("\n    check: \"echo 'checking {}'\"", name)
    } else {
        String::new()
    };

    let content = format!(
        r#"name: "{}"
version: "{}"
description: "Test package for service layer testing"
homepage: "https://example.com/{}"

environments:
  {}:
    install: "echo 'installing {}'"{}
    dependencies: []
"#,
        name, TEST_VERSION, name, SERVICE_TEST_ENV, name, check_command
    );

    let file_path = dir.path().join(format!("{}.yml", name));
    fs::write(&file_path, content).unwrap();
    file_path
}

/// Creates an invalid package file for service tests using the correct "test" environment.
pub fn create_service_invalid_package_file(dir: &TempDir, name: &str) -> PathBuf {
    let content = r#"# Invalid YAML - syntax error
name: "invalid-package"
version: "1.0.0"
environments:
  test:
    install: "echo 'test'
    # Missing closing quote above - this will cause YAML parse error
"#;

    let file_path = dir.path().join(format!("{}.yml", name));
    fs::write(&file_path, content).unwrap();
    file_path
}
