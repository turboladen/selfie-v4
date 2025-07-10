//! Configuration validation functionality
//!
//! This module provides validation capabilities for application configuration,
//! ensuring that configuration values are valid and complete before use.

use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::validation::{ValidationErrorCategory, ValidationIssue, ValidationIssues};

use super::AppConfig;

/// Result of configuration validation
///
/// Contains the path to the configuration file that was validated
/// and any validation issues that were found during the process.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ValidationResult {
    /// The config file path that was validated
    pub(crate) config_file_path: Option<PathBuf>,

    /// List of validation issues found during validation
    pub(crate) issues: ValidationIssues,
}

impl ValidationResult {
    /// Get the path to the configuration file that was validated
    #[must_use]
    pub fn config_file_path(&self) -> Option<&PathBuf> {
        self.config_file_path.as_ref()
    }

    /// Get the validation issues found during validation
    #[must_use]
    pub fn issues(&self) -> &ValidationIssues {
        &self.issues
    }
}

impl AppConfig {
    /// Perform comprehensive validation of the application configuration
    ///
    /// Validates all configuration fields including environment name,
    /// package directory path, and other settings to ensure they are
    /// valid and usable.
    ///
    /// # Returns
    ///
    /// A [`ValidationResult`] containing any issues found during validation.
    /// The result includes both errors (which prevent the configuration from
    /// being used) and warnings (which indicate potential problems).
    #[must_use]
    pub fn validate(&self) -> ValidationResult {
        let mut issues = Vec::new();

        if let Some(issue) = validate_environment(&self.environment) {
            issues.push(issue);
        }

        let path_issues = validate_package_directory(&self.package_directory);
        issues.extend_from_slice(&path_issues);

        ValidationResult {
            config_file_path: Some(self.package_directory().clone()),
            issues: issues.into(),
        }
    }
}

/// Errors that can occur during configuration validation
///
/// These errors represent specific validation failures that can be
/// programmatically handled or displayed to users.
#[derive(Error, Debug, PartialEq)]
pub enum ConfigValidationError {
    /// A required configuration field is empty or missing
    #[error("Empty field: {0}")]
    EmptyField(String),

    /// The package directory path is invalid or cannot be used
    #[error("Invalid package directory: {0}")]
    InvalidPackageDirectory(String),
}

/// Validate the environment field
///
/// Ensures the environment name is not empty, as it's required for
/// determining which package installation commands to use.
fn validate_environment(environment: &str) -> Option<ValidationIssue> {
    environment.is_empty().then(|| {
        ValidationIssue::error(
            ValidationErrorCategory::RequiredField,
            "environment",
            "The `environment` field exists, but has no value",
            Some("Set a value for `environment`. Ex. `environment: macos`"),
        )
    })
}

/// Validate the package directory path
///
/// Ensures the package directory path is not empty and can be expanded
/// to an absolute path. This is critical because the package directory
/// is where selfie looks for package definition files.
fn validate_package_directory(package_directory: &Path) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if package_directory.as_os_str().is_empty() {
        issues.push(ValidationIssue::error(
            ValidationErrorCategory::RequiredField,
            "package_directory",
            "The `package_directory` field exists, but has no value",
            Some("Set a value for `package_directory`. Ex. `package_directory: ~/dev/selfie-packages`")
        ));
    }

    // Validate the package directory path
    let package_dir = package_directory.to_string_lossy();
    let expanded_path = shellexpand::tilde(&package_dir);
    let expanded_path = Path::new(expanded_path.as_ref());

    if !expanded_path.is_absolute() {
        issues.push(ValidationIssue::error(
            ValidationErrorCategory::PathFormat,
            "package_directory",
            "The path at `package_directory` exists, but cannot be expanded",
            Some("If the path is relative, simplify it, otherwise provide an absolute path"),
        ));
    }

    issues
}
