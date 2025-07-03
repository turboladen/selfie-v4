//! `AppConfig` test helpers to eliminate duplication across CLI command tests.

use crate::constants::{SERVICE_TEST_ENV, TEST_ENV, TEST_PACKAGE_DIR};
use selfie::config::{AppConfig, AppConfigBuilder};
use std::path::Path;

/// Creates a standard test configuration with colors disabled.
/// This is the most commonly used config in CLI command tests.
#[must_use]
pub fn test_config() -> AppConfig {
    AppConfigBuilder::default()
        .environment(TEST_ENV)
        .package_directory(TEST_PACKAGE_DIR)
        .use_colors(false)
        .build()
}

/// Creates a test configuration with colors enabled.
/// Used for testing colored output formatting.
#[must_use]
pub fn test_config_with_colors() -> AppConfig {
    AppConfigBuilder::default()
        .environment(TEST_ENV)
        .package_directory(TEST_PACKAGE_DIR)
        .use_colors(true)
        .build()
}

/// Creates a verbose test configuration.
/// Used for testing verbose output modes.
#[must_use]
pub fn test_config_verbose() -> AppConfig {
    AppConfigBuilder::default()
        .environment(TEST_ENV)
        .package_directory(TEST_PACKAGE_DIR)
        .use_colors(false)
        .verbose(true)
        .build()
}

/// Creates a test configuration for a specific environment.
/// Useful for testing environment-specific behavior.
#[must_use]
pub fn test_config_for_env(environment: &str) -> AppConfig {
    AppConfigBuilder::default()
        .environment(environment)
        .package_directory(TEST_PACKAGE_DIR)
        .use_colors(false)
        .build()
}

/// Creates a test configuration with a specific package directory.
/// Used primarily in integration tests with temporary directories.
pub fn test_config_with_dir<P: AsRef<Path>>(package_dir: P) -> AppConfig {
    AppConfigBuilder::default()
        .environment(TEST_ENV)
        .package_directory(package_dir.as_ref())
        .use_colors(false)
        .build()
}

/// Creates a test configuration with both custom directory and environment.
/// Most flexible config creator for complex test scenarios.
pub fn test_config_with_dir_and_env<P: AsRef<Path>>(
    package_dir: P,
    environment: &str,
) -> AppConfig {
    AppConfigBuilder::default()
        .environment(environment)
        .package_directory(package_dir.as_ref())
        .use_colors(false)
        .build()
}

/// Creates a test configuration for service layer tests with the correct "test" environment.
/// Used primarily in service integration tests with temporary directories.
pub fn service_test_config_with_dir<P: AsRef<Path>>(package_dir: P) -> AppConfig {
    AppConfigBuilder::default()
        .environment(SERVICE_TEST_ENV)
        .package_directory(package_dir.as_ref())
        .use_colors(false)
        .build()
}
