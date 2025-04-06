use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::validation::{ValidationErrorCategory, ValidationIssue, ValidationIssues};

use super::AppConfig;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ValidationResult {
    /// The config file path
    ///
    pub(crate) config_file_path: Option<PathBuf>,

    /// List of validation issues found
    ///
    pub(crate) issues: ValidationIssues,
}

impl ValidationResult {
    #[must_use]
    pub fn config_file_path(&self) -> Option<&PathBuf> {
        self.config_file_path.as_ref()
    }

    #[must_use]
    pub fn issues(&self) -> &ValidationIssues {
        &self.issues
    }
}

impl AppConfig {
    /// Full validation for the `AppConfig`
    ///
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

#[derive(Error, Debug, PartialEq)]
pub enum ConfigValidationError {
    #[error("Empty field: {0}")]
    EmptyField(String),

    #[error("Invalid package directory: {0}")]
    InvalidPackageDirectory(String),
}

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
