use std::path::PathBuf;
use thiserror::Error;

use crate::package::Package;

/// Port for package repository operations
#[cfg_attr(test, mockall::automock)]
pub trait PackageRepository: Send + Sync {
    /// Get a package by name.
    ///
    fn get_package(&self, name: &str) -> Result<Package, PackageRepoError>;

    /// List all available packages in the package directory.
    ///
    fn list_packages(&self) -> Result<Vec<Result<Package, PackageParseError>>, PackageRepoError>;

    /// Find package files that match the given name.
    ///
    fn find_package_files(&self, name: &str) -> Result<Vec<PathBuf>, PackageRepoError>;
}

#[derive(Error, Debug)]
pub enum PackageRepoError {
    #[error("Package `{name}` not found in path {}", packages_path.display())]
    PackageNotFound {
        name: String,
        packages_path: PathBuf,
    },

    #[error("Multiple packages found with name: {0}")]
    MultiplePackagesFound(String),

    #[error("Parse error: {0}")]
    ParseError(#[from] PackageParseError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Directory does not exist: {0}")]
    DirectoryNotFound(String),
}

/// Errors related to package parsing
#[derive(Error, Debug)]
pub enum PackageParseError {
    #[error("YAML parsing error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("File system error: {0}")]
    FileSystemError(String),
}
