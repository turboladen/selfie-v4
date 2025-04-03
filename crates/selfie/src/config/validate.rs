use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::validation::ValidationIssues;

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
    // TODO: Convert to use `crate::validation`.
    //
    pub fn validate<F>(&self, report_fn: F) -> Result<(), ConfigValidationError>
    where
        F: Fn(&'static str),
    {
        validate_environment(&self.environment)?;
        report_fn("`environment` is valid");
        validate_package_directory(&self.package_directory)?;
        report_fn("`package_directory` is valid");

        Ok(())
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ConfigValidationError {
    #[error("Empty field: {0}")]
    EmptyField(String),

    #[error("Invalid package directory: {0}")]
    InvalidPackageDirectory(String),
}

fn validate_environment(environment: &str) -> Result<(), ConfigValidationError> {
    if environment.is_empty() {
        Err(ConfigValidationError::EmptyField("environment".to_string()))
    } else {
        Ok(())
    }
}

fn validate_package_directory(package_directory: &Path) -> Result<(), ConfigValidationError> {
    if package_directory.as_os_str().is_empty() {
        return Err(ConfigValidationError::EmptyField(
            "package_directory".to_string(),
        ));
    }

    // Validate the package directory path
    let package_dir = package_directory.to_string_lossy();
    let expanded_path = shellexpand::tilde(&package_dir);
    let expanded_path = Path::new(expanded_path.as_ref());

    if !expanded_path.is_absolute() {
        return Err(ConfigValidationError::InvalidPackageDirectory(
            "Package directory must be an absolute path".to_string(),
        ));
    }

    Ok(())
}
