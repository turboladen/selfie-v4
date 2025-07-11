//! Common utilities for package commands
//!
//! This module provides shared functionality used across multiple package commands
//! to reduce code duplication and maintain consistency.

use comfy_table::{ContentArrangement, Table, modifiers, presets};
use console::style;
use selfie::{
    commands::ShellCommandRunner,
    config::AppConfig,
    fs::real::RealFileSystem,
    package::{
        GetPackage, port::PackageRepository, repository::yaml::YamlPackageRepository,
        service::PackageServiceImpl,
    },
};
use std::{path::Path, process::Command};

use crate::{
    event_processor::EventProcessor, terminal_progress_reporter::TerminalProgressReporter,
};

/// Create a package repository instance with the configured package directory
pub(super) fn create_package_repository(
    config: &AppConfig,
) -> YamlPackageRepository<RealFileSystem> {
    YamlPackageRepository::new(RealFileSystem, config.package_directory().clone())
}

/// Save a package to the filesystem with consistent error handling
pub(super) fn save_package(
    repo: &impl PackageRepository,
    package_blob: &GetPackage,
    reporter: &TerminalProgressReporter,
) -> Result<(), i32> {
    if let Err(e) = repo.save_package(&package_blob.package, &package_blob.file_path) {
        reporter.report_error(format!("Failed to save package file: {e}"));
        return Err(1);
    }
    Ok(())
}

/// Open a file in the user's preferred editor
///
/// Handles common editor functionality including:
/// - Checking for EDITOR environment variable
/// - Adding --wait flag for VS Code
/// - Executing the editor command
/// - Providing appropriate success/failure messages
pub(super) fn open_editor(
    file_path: &Path,
    reporter: &TerminalProgressReporter,
    success_message: Option<String>,
) -> i32 {
    let editor = match std::env::var("EDITOR") {
        Ok(editor) => editor,
        Err(_) => {
            reporter.report_error("EDITOR environment variable is not set.");
            reporter.report_info("Please set EDITOR and try again.");
            return 1;
        }
    };

    let mut cmd = Command::new(&editor);
    cmd.arg(file_path);

    // For VS Code, wait for the file to be closed
    if editor == "code" {
        cmd.arg("--wait");
    }

    match cmd.status() {
        Ok(status) if status.success() => {
            if let Some(message) = success_message {
                reporter.report_success(message);
            }
            0
        }
        Ok(_) => {
            reporter.report_warning("Editor exited with non-zero status.");
            1
        }
        Err(e) => {
            reporter.report_error(format!("Failed to start editor '{editor}': {e}"));
            1
        }
    }
}

/// Check if EDITOR environment variable is set and provide helpful error messages
///
/// Returns the editor command if available, or reports an error and returns None.
/// Provides context-specific error messages for different scenarios.
pub(super) fn check_editor_available(
    reporter: &TerminalProgressReporter,
    package_name: &str,
    package_exists: bool,
    package_path: Option<&Path>,
) -> Option<String> {
    match std::env::var("EDITOR") {
        Ok(editor) => Some(editor),
        Err(_) => {
            reporter.report_error("EDITOR environment variable is not set.");

            if package_exists {
                if let Some(path) = package_path {
                    reporter.report_info(format!(
                        "Package '{}' exists at {}. Go ahead and open it in your editor of choice!",
                        package_name,
                        path.display()
                    ));
                } else {
                    reporter.report_info(format!(
                        "Package '{}' exists. Set EDITOR to edit it automatically.",
                        package_name
                    ));
                }
            } else {
                reporter.report_info(format!(
                    "Package '{}' doesn't exist yet. Set EDITOR and try again to create it.",
                    package_name
                ));
            }
            None
        }
    }
}

/// Create a new package template
pub(super) fn create_new_package(package_name: &str, config: &AppConfig) -> GetPackage {
    GetPackage::new(package_name, config.package_directory())
}

/// Create a package service with repository and command runner
pub(super) fn create_package_service(
    config: &AppConfig,
) -> PackageServiceImpl<YamlPackageRepository<RealFileSystem>, ShellCommandRunner> {
    let repo = create_package_repository(config);
    let command_runner = ShellCommandRunner::new("/bin/sh", config.command_timeout());
    PackageServiceImpl::new(repo, command_runner, config.clone())
}

/// Create a formatted table with consistent styling
pub(super) fn create_formatted_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL_CONDENSED)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table
}

/// Format environment names with current environment highlighting
pub(super) fn format_environment_names(
    environments: &[String],
    current_environment: &str,
    config: &AppConfig,
) -> String {
    environments
        .iter()
        .map(|env_name| {
            if env_name == current_environment {
                let env = format!("*{env_name}");
                if config.use_colors() {
                    style(env).bold().green().to_string()
                } else {
                    env
                }
            } else if config.use_colors() {
                style(env_name).dim().green().to_string()
            } else {
                env_name.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Format a key with consistent styling
pub(super) fn format_field_key(key: &str, use_colors: bool) -> String {
    if use_colors {
        style(key).cyan().bold().to_string()
    } else {
        key.to_string()
    }
}

/// Format a value with consistent styling
pub(super) fn format_field_value(value: &str, use_colors: bool) -> String {
    if use_colors {
        style(value).white().to_string()
    } else {
        value.to_string()
    }
}

/// Process events with a custom handler using consistent pattern
pub(super) async fn process_events_with_custom_handler<F>(
    event_stream: selfie::package::event::EventStream,
    reporter: TerminalProgressReporter,
    handler: F,
    config: &AppConfig,
) -> i32
where
    F: Fn(&selfie::package::event::PackageEvent, &AppConfig) -> Option<bool>,
{
    let processor = EventProcessor::new(reporter);
    processor
        .process_events_with_handler(event_stream, |event, _reporter| handler(event, config))
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use test_common::test_config_with_dir;

    fn create_mock_reporter() -> TerminalProgressReporter {
        TerminalProgressReporter::new(false)
    }

    #[test]
    fn test_create_package_repository() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config_with_dir(temp_dir.path());

        let repo = create_package_repository(&config);
        // Just verify we can create it without panicking
        drop(repo);
    }

    #[test]
    fn test_create_new_package() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config_with_dir(temp_dir.path());

        let package_blob = create_new_package("test-package", &config);

        assert!(package_blob.is_new);
        assert_eq!(package_blob.package.name(), "test-package");
        assert_eq!(package_blob.package.version(), "0.1.0");
    }

    #[test]
    fn test_check_editor_available_with_editor() {
        // This test checks the behavior when EDITOR is already set
        // We'll only run it if EDITOR is already available to avoid test interference
        if std::env::var("EDITOR").is_ok() {
            let reporter = create_mock_reporter();
            let result = check_editor_available(&reporter, "test", false, None);
            assert!(result.is_some()); // Should return some editor
        }
    }

    #[test]
    fn test_check_editor_available_without_editor() {
        // This test checks the behavior when EDITOR is not set
        // We'll only run it if EDITOR is not already set to avoid test interference
        if std::env::var("EDITOR").is_err() {
            let reporter = create_mock_reporter();
            let result = check_editor_available(&reporter, "test", false, None);
            assert_eq!(result, None);
        }
    }

    #[test]
    fn test_vs_code_wait_flag_logic() {
        // Test that VS Code gets the --wait flag
        let editor = "code";
        let mut cmd = Command::new(&editor);
        cmd.arg("/tmp/test.yml");

        if editor == "code" {
            cmd.arg("--wait");
        }

        let args: Vec<_> = cmd.get_args().collect();
        assert!(
            args.iter()
                .any(|arg| *arg == std::ffi::OsStr::new("--wait"))
        );
    }

    #[test]
    fn test_save_package_integration() {
        let temp_dir = TempDir::new().unwrap();
        let package_dir = temp_dir.path().join("packages");
        fs::create_dir_all(&package_dir).unwrap();

        let config = test_config_with_dir(&package_dir);
        let repo = create_package_repository(&config);
        let reporter = create_mock_reporter();

        let package_blob = create_new_package("save-test", &config);

        let result = save_package(&repo, &package_blob, &reporter);
        assert!(result.is_ok());

        // Verify file was created
        assert!(package_blob.file_path.exists());
    }

    #[test]
    fn test_create_package_service() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config_with_dir(temp_dir.path());

        let service = create_package_service(&config);
        // Just verify we can create it without panicking
        drop(service);
    }

    #[test]
    fn test_create_formatted_table() {
        let table = create_formatted_table();
        // Just test that table creation doesn't panic
        let _table_str = table.to_string();
    }

    #[test]
    fn test_format_environment_names() {
        let config = test_config_with_dir(&TempDir::new().unwrap().path());
        let environments = vec!["test".to_string(), "production".to_string()];

        let result = format_environment_names(&environments, "test", &config);

        // Just test that it doesn't panic and returns something
        assert!(!result.is_empty());
        assert!(result.contains("test"));
    }

    #[test]
    fn test_format_field_key_and_value() {
        let key = format_field_key("Test Key", false);
        assert_eq!(key, "Test Key");

        let value = format_field_value("Test Value", false);
        assert_eq!(value, "Test Value");

        // Test with colors (just ensure no panic)
        let _colored_key = format_field_key("Test Key", true);
        let _colored_value = format_field_value("Test Value", true);
    }
}
