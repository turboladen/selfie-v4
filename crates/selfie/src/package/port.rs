//! Package repository port and error types
//!
//! This module defines the core port for package repository operations in the
//! hexagonal architecture. The `PackageRepository` trait abstracts package
//! storage and retrieval, allowing different implementations (YAML files,
//! databases, remote repositories, etc.) while maintaining a consistent interface.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use thiserror::Error;

use crate::package::Package;

/// Port for package repository operations (Hexagonal Architecture)
///
/// This trait abstracts package storage and retrieval operations to allow
/// different implementations and enable comprehensive testing through mocking.
/// It provides the core operations needed for package discovery, loading,
/// and validation.
///
/// Implementations might include:
/// - YAML file-based repositories
/// - Database-backed repositories
/// - Remote repository clients
/// - In-memory repositories for testing
#[cfg_attr(test, mockall::automock)]
pub trait PackageRepository: Send + Sync {
    /// Get a package by name from the repository
    ///
    /// Loads a complete package definition including all environment configurations,
    /// dependencies, and metadata. The package is identified by its name, which
    /// should correspond to the package file name (without extension).
    ///
    /// # Arguments
    ///
    /// * `name` - The package name to load
    ///
    /// # Returns
    ///
    /// The complete package definition with all configurations
    ///
    /// # Errors
    ///
    /// Returns [`PackageRepoError`] if:
    /// - No package with the given name exists
    /// - Multiple packages with the same name are found
    /// - The package definition file is malformed
    /// - File system access fails
    fn get_package(&self, name: &str) -> Result<Package, PackageRepoError>;

    /// List all available packages in the package directory
    ///
    /// Discovers and attempts to load all package definition files in the
    /// configured package directory. Returns both successfully loaded packages
    /// and any parse errors encountered, allowing the caller to handle
    /// partial failures gracefully.
    ///
    /// # Returns
    ///
    /// A collection containing both valid packages and parse errors
    ///
    /// # Errors
    ///
    /// Returns [`PackageListError`] if:
    /// - The package directory cannot be accessed
    /// - Directory listing fails
    /// - Critical file system errors occur
    fn list_packages(&self) -> Result<ListPackagesOutput, PackageListError>;

    /// Get a list of all valid package names in the repository
    ///
    /// Extracts just the names of successfully parseable packages from the
    /// repository. This is useful for operations that only need package names
    /// rather than full package definitions.
    ///
    /// # Returns
    ///
    /// A vector of valid package names
    ///
    /// # Errors
    ///
    /// Returns [`PackageListError`] if the underlying list operation fails
    fn available_packages(&self) -> Result<Vec<String>, PackageListError> {
        let list_packages_output = self.list_packages()?;

        Ok(list_packages_output
            .valid_packages()
            .map(|package| package.name().to_string())
            .collect())
    }

    /// Find package files that match the given name
    ///
    /// Searches for package definition files (with various extensions like .yml, .yaml)
    /// that correspond to the given package name. This is useful for package
    /// discovery and resolving ambiguities when multiple files might match.
    ///
    /// # Arguments
    ///
    /// * `name` - The package name to search for
    ///
    /// # Returns
    ///
    /// A vector of file paths that match the package name
    ///
    /// # Errors
    ///
    /// Returns [`PackageListError`] if:
    /// - The package directory cannot be accessed
    /// - File system operations fail
    fn find_package_files(&self, name: &str) -> Result<Vec<PathBuf>, PackageListError>;
}

/// Errors that can occur during package repository operations
///
/// This enum represents all possible failures when interacting with the
/// package repository, providing detailed context for debugging and
/// error handling.
#[derive(Error, Debug, Clone)]
pub enum PackageRepoError {
    /// Package-specific error (not found, parse error, etc.)
    #[error(transparent)]
    PackageError(#[from] Box<PackageError>),

    /// Package listing operation failed
    #[error(transparent)]
    PackageListError(#[from] PackageListError),

    /// IO error during repository operation
    #[error("IO error: {0}")]
    IoError(#[from] Arc<std::io::Error>),
}

/// Errors that can occur when listing packages
///
/// Represents failures specific to package discovery and directory
/// operations during package listing.
#[derive(Error, Debug, Clone)]
pub enum PackageListError {
    /// IO error occurred while reading the package directory
    #[error("IO error reading package list: {0}")]
    IoError(#[from] Arc<std::io::Error>),

    /// The configured package directory does not exist
    #[error("Directory does not exist: {}", _0.display())]
    PackageDirectoryNotFound(PathBuf),
}

/// Errors that can occur with specific package operations
///
/// Represents detailed failures when working with individual packages,
/// providing rich context for debugging and user-friendly error messages.
#[derive(Error, Debug, Clone)]
pub enum PackageError {
    /// No package with the specified name could be found
    #[error("Package `{name}` not found in path {}", packages_path.display())]
    PackageNotFound {
        name: String,
        packages_path: PathBuf,
        /// Number of files examined during search
        files_examined: usize,
        /// Search patterns used (e.g., ["package.yml", "package.yaml"])
        search_patterns: Vec<String>,
    },

    /// Multiple package files found with the same name, creating ambiguity
    #[error("Multiple packages found with name `{name}` in path {}", packages_path.display())]
    MultiplePackagesFound {
        name: String,
        packages_path: PathBuf,
        /// The conflicting file paths found
        conflicting_paths: Vec<PathBuf>,
        files_examined: usize,
        search_patterns: Vec<String>,
    },

    /// Package definition file exists but could not be parsed
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

    /// The requested environment is not configured for this package
    #[error("Environment `{environment}` not found in package `{package_name}`")]
    EnvironmentNotFound {
        package_name: String,
        environment: String,
        /// Available environments for suggestions
        available_environments: Vec<String>,
        package_file: PathBuf,
    },

    /// Package environment exists but has no check command configured
    #[error("No check command defined for package `{package_name}` in environment `{environment}`")]
    NoCheckCommand {
        package_name: String,
        environment: String,
        package_file: PathBuf,
        /// Whether other environments have check commands (for suggestions)
        other_envs_with_check: Vec<String>,
    },
}

/// Output from listing packages in the repository
///
/// Contains the results of attempting to load all packages from the repository.
/// This includes both successfully loaded packages and any parse errors that
/// occurred, allowing callers to handle partial failures gracefully.
#[derive(Debug)]
pub struct ListPackagesOutput(pub(crate) Vec<Result<Package, PackageParseError>>);

impl ListPackagesOutput {
    /// Get the total number of packages found (both valid and invalid)
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if no packages were found
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get all package loading results (both successes and failures)
    ///
    /// Returns a slice containing the result of attempting to load each
    /// package file found in the repository.
    pub fn all_results(&self) -> &[Result<Package, PackageParseError>] {
        &self.0
    }

    /// Get an iterator over successfully loaded packages
    ///
    /// Filters out any packages that failed to parse and returns only
    /// the valid package definitions.
    pub fn valid_packages(&self) -> impl Iterator<Item = &Package> {
        self.0.iter().filter_map(|maybe_p| maybe_p.as_ref().ok())
    }

    /// Find a specific package by name
    ///
    /// Searches through the valid packages to find one with the specified name.
    /// Returns `None` if no package with that name was found or if the package
    /// failed to parse.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to find
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

    /// Get an iterator over packages that failed to parse
    ///
    /// Returns parse errors for packages that could not be loaded successfully.
    /// This is useful for debugging configuration issues.
    pub fn invalid_packages(&self) -> impl Iterator<Item = &PackageParseError> {
        self.0.iter().filter_map(|maybe_p| maybe_p.as_ref().err())
    }
}

/// Errors related to package parsing
///
/// Represents specific failures that can occur when attempting to parse
/// package definition files. These errors provide detailed context about
/// what went wrong during the parsing process.
#[derive(Error, Debug, Clone)]
pub enum PackageParseError {
    /// YAML syntax or structure error in the package file
    #[error("YAML parsing error reading package file `{}`: {source}", package_path.display())]
    YamlParse {
        package_path: PathBuf,
        #[source]
        source: Arc<serde_yaml::Error>,
    },

    /// IO error occurred while reading the package file
    #[error("I/O error reading package file `{}`: {source}", package_path.display())]
    IoError {
        package_path: PathBuf,
        #[source]
        source: Arc<std::io::Error>,
    },

    /// File system abstraction error during package file access
    #[error("File system error reading package file `{}`: {source_message}", package_path.display())]
    FileSystemError {
        package_path: PathBuf,
        source_message: String,
    },
}

impl PackageParseError {
    /// Get the path to the package file that failed to parse
    ///
    /// Returns the file path regardless of the specific parse error type.
    /// This is useful for error reporting and debugging.
    #[must_use]
    pub fn package_path(&self) -> &Path {
        match self {
            PackageParseError::YamlParse { package_path, .. } => package_path,
            PackageParseError::IoError { package_path, .. } => package_path,
            PackageParseError::FileSystemError { package_path, .. } => package_path,
        }
    }
}
