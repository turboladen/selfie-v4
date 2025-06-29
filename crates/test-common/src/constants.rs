//! Common test constants used across multiple test files.

/// Default test environment name used in tests
pub const TEST_ENV: &str = "test-env";

/// Alternative test environment name for multi-environment tests
pub const ALT_TEST_ENV: &str = "prod-env";

/// Default test package directory path
pub const TEST_PACKAGE_DIR: &str = "/tmp/test-packages";

/// Default test package version
pub const TEST_VERSION: &str = "1.0.0";

/// Test package name prefix for generated packages
pub const TEST_PACKAGE_PREFIX: &str = "test-package";

/// The environment name used in CLI integration tests
pub const SELFIE_ENV: &str = "test-env";

/// Default timeout for test commands (in seconds)
pub const TEST_COMMAND_TIMEOUT_SECS: u64 = 30;
