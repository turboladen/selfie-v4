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
    fn list_packages(&self) -> Result<ListPackagesOutput, PackageRepoError>;

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

    #[error("Parse error `{source}` from package in path {}", packages_path.display())]
    ParseError {
        #[source]
        source: PackageParseError,
        packages_path: PathBuf,
    },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Directory does not exist: {0}")]
    DirectoryNotFound(String),
}

#[derive(Debug)]
pub struct ListPackagesOutput(pub(crate) Vec<Result<Package, PackageParseError>>);

impl ListPackagesOutput {
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn valid_packages(&self) -> impl Iterator<Item = &Package> {
        self.0.iter().filter_map(|maybe_p| match maybe_p {
            Ok(p) => Some(p),
            Err(_) => None,
        })
    }

    #[must_use]
    pub fn get(&self, package_name: &str) -> Option<&Package> {
        self.0.iter().find_map(|maybe_p| match maybe_p {
            Ok(p) => {
                if p.name == package_name {
                    Some(p)
                } else {
                    None
                }
            }
            Err(_) => None,
        })
    }

    pub fn invalid_packages(&self) -> impl Iterator<Item = &PackageParseError> {
        self.0.iter().filter_map(|maybe_p| match maybe_p {
            Ok(_) => None,
            Err(e) => Some(e),
        })
    }
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
