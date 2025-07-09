use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
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
    fn list_packages(&self) -> Result<ListPackagesOutput, PackageListError>;

    /// List all valid package names in the repo.
    ///
    fn available_packages(&self) -> Result<Vec<String>, PackageListError> {
        let list_packages_output = self.list_packages()?;

        Ok(list_packages_output
            .valid_packages()
            .map(|package| package.name().to_string())
            .collect())
    }

    /// Find package files that match the given name.
    ///
    fn find_package_files(&self, name: &str) -> Result<Vec<PathBuf>, PackageListError>;
}

#[derive(Error, Debug, Clone)]
pub enum PackageRepoError {
    #[error(transparent)]
    PackageError(#[from] PackageError),

    #[error(transparent)]
    PackageListError(#[from] PackageListError),

    #[error("IO error: {0}")]
    IoError(#[from] Arc<std::io::Error>),
}

#[derive(Error, Debug, Clone)]
pub enum PackageListError {
    #[error("IO error reading package list: {0}")]
    IoError(#[from] Arc<std::io::Error>),

    #[error("Directory does not exist: {}", _0.display())]
    PackageDirectoryNotFound(PathBuf),
}

#[derive(Error, Debug, Clone)]
pub enum PackageError {
    #[error("Package `{name}` not found in path {}", packages_path.display())]
    PackageNotFound {
        name: String,
        packages_path: PathBuf,
        /// Number of files examined during search
        files_examined: usize,
        /// Search patterns used (e.g., ["package.yml", "package.yaml"])
        search_patterns: Vec<String>,
    },

    #[error("Multiple packages found with name `{name}` in path {}", packages_path.display())]
    MultiplePackagesFound {
        name: String,
        packages_path: PathBuf,
        /// The conflicting file paths found
        conflicting_paths: Vec<PathBuf>,
        files_examined: usize,
        search_patterns: Vec<String>,
    },

    #[error("Parse error in package `{name}` from {}", packages_path.display())]
    ParseError {
        name: String,
        packages_path: PathBuf,
        /// The specific file that failed to parse
        failed_file: PathBuf,
        /// File size for debugging
        file_size_bytes: u64,
        #[source]
        source: PackageParseError,
    },

    #[error("Environment `{environment}` not found in package `{package_name}`")]
    EnvironmentNotFound {
        package_name: String,
        environment: String,
        /// Available environments for suggestions
        available_environments: Vec<String>,
        package_file: PathBuf,
    },

    #[error("No check command defined for package `{package_name}` in environment `{environment}`")]
    NoCheckCommand {
        package_name: String,
        environment: String,
        package_file: PathBuf,
        /// Whether other environments have check commands (for suggestions)
        other_envs_with_check: Vec<String>,
    },
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

    pub fn all_results(&self) -> &[Result<Package, PackageParseError>] {
        &self.0
    }

    pub fn valid_packages(&self) -> impl Iterator<Item = &Package> {
        self.0.iter().filter_map(|maybe_p| maybe_p.as_ref().ok())
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
        self.0.iter().filter_map(|maybe_p| maybe_p.as_ref().err())
    }
}

/// Errors related to package parsing
#[derive(Error, Debug, Clone)]
pub enum PackageParseError {
    #[error("YAML parsing error reading package file `{}`: {source}", package_path.display())]
    YamlParse {
        package_path: PathBuf,
        #[source]
        source: Arc<serde_yaml::Error>,
    },

    #[error("I/O error reading package file `{}`: {source}", package_path.display())]
    IoError {
        package_path: PathBuf,
        #[source]
        source: Arc<std::io::Error>,
    },

    #[error("File system error reading package file `{}`: {source_message}", package_path.display())]
    FileSystemError {
        package_path: PathBuf,
        source_message: String,
    },
}

impl PackageParseError {
    #[must_use]
    pub fn package_path(&self) -> &Path {
        match self {
            PackageParseError::YamlParse { package_path, .. } => package_path,
            PackageParseError::IoError { package_path, .. } => package_path,
            PackageParseError::FileSystemError { package_path, .. } => package_path,
        }
    }
}
