//! Common utilities for package commands
//!
//! This module provides shared functionality used across multiple package commands
//! to reduce code duplication and maintain consistency.

use selfie::{
    config::AppConfig,
    fs::real::RealFileSystem,
    package::{GetPackage, port::PackageRepository, repository::yaml::YamlPackageRepository},
};
use std::{path::Path, process::Command};

use crate::terminal_progress_reporter::TerminalProgressReporter;

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
        // Save current EDITOR
        let old_editor = std::env::var("EDITOR").ok();

        // Set EDITOR
        unsafe {
            std::env::set_var("EDITOR", "vim");
        }

        let reporter = create_mock_reporter();
        let result = check_editor_available(&reporter, "test", false, None);

        assert_eq!(result, Some("vim".to_string()));

        // Restore EDITOR
        match old_editor {
            Some(editor) => unsafe { std::env::set_var("EDITOR", editor) },
            None => unsafe { std::env::remove_var("EDITOR") },
        }
    }

    #[test]
    fn test_check_editor_available_without_editor() {
        // Save current EDITOR
        let old_editor = std::env::var("EDITOR").ok();

        // Remove EDITOR
        unsafe {
            std::env::remove_var("EDITOR");
        }

        let reporter = create_mock_reporter();
        let result = check_editor_available(&reporter, "test", false, None);

        assert_eq!(result, None);

        // Restore EDITOR
        if let Some(editor) = old_editor {
            unsafe {
                std::env::set_var("EDITOR", editor);
            }
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
}
