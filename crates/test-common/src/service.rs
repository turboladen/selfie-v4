//! Service creation helpers to eliminate service setup duplication in tests.

use crate::config::{
    service_test_config_with_dir, test_config_with_dir, test_config_with_dir_and_env,
};
use selfie::{
    commands::shell::ShellCommandRunner,
    config::AppConfig,
    fs::real::RealFileSystem,
    package::{repository::YamlPackageRepository, service::PackageServiceImpl},
};
use std::time::Duration;
use tempfile::TempDir;

/// Creates a test service with real filesystem and default settings.
/// This is the most commonly used service setup for integration tests.
pub fn create_test_service(
    temp_dir: &TempDir,
) -> PackageServiceImpl<YamlPackageRepository<RealFileSystem>, ShellCommandRunner> {
    let config = test_config_with_dir(temp_dir.path());
    create_test_service_with_config(config)
}

/// Creates a test service with a specific configuration.
/// Useful when you need custom config settings like different environments.
pub fn create_test_service_with_config(
    config: AppConfig,
) -> PackageServiceImpl<YamlPackageRepository<RealFileSystem>, ShellCommandRunner> {
    let fs = RealFileSystem;
    let repo = YamlPackageRepository::new(fs, config.package_directory().clone());
    let runner = ShellCommandRunner::new("/bin/sh", Duration::from_secs(30));
    PackageServiceImpl::new(repo, runner, config)
}

/// Creates a test service with custom command timeout.
/// Useful for testing timeout scenarios or when you need longer-running commands.
pub fn create_test_service_with_timeout(
    temp_dir: &TempDir,
    timeout: Duration,
) -> PackageServiceImpl<YamlPackageRepository<RealFileSystem>, ShellCommandRunner> {
    let config = test_config_with_dir(temp_dir.path());
    let fs = RealFileSystem;
    let repo = YamlPackageRepository::new(fs, config.package_directory().clone());
    let runner = ShellCommandRunner::new("/bin/sh", timeout);
    PackageServiceImpl::new(repo, runner, config)
}

/// Creates a test service for a specific environment.
/// Useful for testing environment-specific behavior.
pub fn create_test_service_for_env(
    temp_dir: &TempDir,
    environment: &str,
) -> PackageServiceImpl<YamlPackageRepository<RealFileSystem>, ShellCommandRunner> {
    let config = test_config_with_dir_and_env(temp_dir.path(), environment);
    create_test_service_with_config(config)
}

/// Creates the standard CLI service setup used in command handlers.
/// This matches the exact pattern used in CLI commands for consistency.
pub fn create_cli_service(
    config: &AppConfig,
) -> PackageServiceImpl<YamlPackageRepository<RealFileSystem>, ShellCommandRunner> {
    let repo = YamlPackageRepository::new(RealFileSystem, config.package_directory().to_path_buf());
    let command_runner = ShellCommandRunner::new("/bin/sh", config.command_timeout());
    PackageServiceImpl::new(repo, command_runner, config.clone())
}

/// Creates a test service specifically for service layer integration tests.
/// Uses the correct "test" environment expected by service tests.
pub fn create_service_test_service(
    temp_dir: &TempDir,
) -> PackageServiceImpl<YamlPackageRepository<RealFileSystem>, ShellCommandRunner> {
    let config = service_test_config_with_dir(temp_dir.path());
    create_test_service_with_config(config)
}
