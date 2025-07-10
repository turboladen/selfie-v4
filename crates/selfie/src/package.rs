mod builder;
pub mod event;
pub mod port;
pub mod repository;
pub mod service;
pub mod validate;

pub use self::builder::{EnvironmentConfigBuilder, PackageBuilder};

// Core package entity and related types
use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

/// Package data for editing operations
///
/// Contains a package and its file metadata for editing workflows.
/// Can represent either an existing package loaded from the repository
/// or a new package template ready for creation.
#[derive(Debug, Clone)]
pub struct GetPackage {
    /// The package content (either loaded or template)
    pub package: Package,
    /// The file path where the package is/should be stored
    pub file_path: PathBuf,
    /// Whether this is a new package (true) or existing (false)
    pub is_new: bool,
}

impl GetPackage {
    /// Create a new package template for the given name and directory
    ///
    /// This creates a basic package template with minimal configuration
    /// that can be used as a starting point for new packages.
    ///
    /// # Arguments
    ///
    /// * `name` - The package name
    /// * `packages_directory` - The directory where package files are stored
    pub fn new(name: &str, packages_directory: &std::path::Path) -> Self {
        let file_path = packages_directory.join(format!("{}.yml", name));
        let package = Package::new_template(name);

        Self {
            package,
            file_path,
            is_new: true,
        }
    }

    /// Create a GetPackage from an existing package and file path
    ///
    /// This is used when loading existing packages from the repository.
    ///
    /// # Arguments
    ///
    /// * `package` - The loaded package
    /// * `file_path` - The path where the package file is stored
    pub fn from_existing(package: Package, file_path: PathBuf) -> Self {
        Self {
            package,
            file_path,
            is_new: false,
        }
    }
}

/// Core package entity representing a package definition
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Package {
    /// Package name
    pub(crate) name: String,

    /// Package version (for the selfie package file, not the underlying package)
    pub(crate) version: String,

    /// Optional homepage URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) homepage: Option<String>,

    /// Optional package description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,

    /// Map of environment configurations
    #[serde(default)]
    pub(crate) environments: HashMap<String, EnvironmentConfig>,

    /// Path to the package file (not serialized/deserialized)
    #[serde(skip)]
    pub(crate) path: PathBuf,
}

/// Configuration for a specific environment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Command to install the package
    pub(crate) install: String,

    /// Optional command to check if the package is already installed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) check: Option<String>,

    /// Dependencies that must be installed before this package
    #[serde(default)]
    pub(crate) dependencies: Vec<String>,
}

impl EnvironmentConfig {
    /// Create a new environment configuration
    #[must_use]
    pub fn new(install: String, check: Option<String>, dependencies: Vec<String>) -> Self {
        Self {
            install,
            check,
            dependencies,
        }
    }

    #[must_use]
    pub fn install(&self) -> &str {
        &self.install
    }

    #[must_use]
    pub fn check(&self) -> Option<&str> {
        self.check.as_deref()
    }

    #[must_use]
    pub fn dependencies(&self) -> &[String] {
        &self.dependencies
    }
}

impl Package {
    /// Create a new package with the specified attributes. See `PackageBuilder`.
    #[must_use]
    pub fn new(
        name: String,
        version: String,
        homepage: Option<String>,
        description: Option<String>,
        environments: HashMap<String, EnvironmentConfig>,
        path: PathBuf,
    ) -> Self {
        Self {
            name,
            version,
            homepage,
            description,
            environments,
            path,
        }
    }

    /// Create a basic package template
    ///
    /// Creates a minimal package template suitable for new packages.
    /// The template includes basic metadata and a placeholder environment.
    ///
    /// # Arguments
    ///
    /// * `name` - The package name
    #[must_use]
    pub fn new_template(name: &str) -> Self {
        let mut environments = HashMap::new();
        environments.insert(
            "default".to_string(),
            EnvironmentConfig {
                install: format!("# TODO: Add install command for {}", name),
                check: Some(format!("# TODO: Add check command for {}", name)),
                dependencies: Vec::new(),
            },
        );

        Self {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            homepage: None,
            description: None,
            environments,
            path: PathBuf::new(), // Will be set by GetPackage::new
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    #[must_use]
    pub fn homepage(&self) -> Option<&str> {
        self.homepage.as_deref()
    }

    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    #[must_use]
    pub fn environments(&self) -> &HashMap<String, EnvironmentConfig> {
        &self.environments
    }

    #[must_use]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

#[cfg(test)]
mod package_tests {
    use std::path::PathBuf;

    use builder::PackageBuilder;

    use crate::package::port::PackageError;

    use super::*;

    #[test]
    fn test_create_package_node() {
        let package = PackageBuilder::default()
            .name("test-package")
            .version("1.0.0")
            .environment("test-env", |b| b.install("test install"))
            .build();

        assert_eq!(package.name, "test-package");
        assert_eq!(package.version, "1.0.0");
        assert_eq!(package.environments.len(), 1);
        assert_eq!(
            package.environments.get("test-env").unwrap().install,
            "test install"
        );
    }

    #[test]
    fn test_create_package_with_metadata() {
        let package = PackageBuilder::default()
            .name("test-package")
            .version("1.0.0")
            .homepage("https://example.com")
            .description("Test package description")
            .environment("test-env", |b| b.install("test install"))
            .build();

        assert_eq!(package.name, "test-package");
        assert_eq!(package.version, "1.0.0");
        assert_eq!(package.homepage, Some("https://example.com".to_string()));
        assert_eq!(
            package.description,
            Some("Test package description".to_string())
        );
        assert_eq!(package.environments.len(), 1);
    }

    #[test]
    fn test_package_not_found_error_contains_context() {
        let error = PackageError::PackageNotFound {
            name: "test-package".to_string(),
            packages_path: PathBuf::from("/home/user/.config/selfie/packages"),
            files_examined: 15,
            search_patterns: vec![
                "test-package.yml".to_string(),
                "test-package.yaml".to_string(),
            ],
        };

        // Test that the error message contains the package name and path
        let error_message = error.to_string();
        assert!(error_message.contains("test-package"));
        assert!(error_message.contains("/home/user/.config/selfie/packages"));

        // Test that context fields are accessible for debugging
        match error {
            PackageError::PackageNotFound {
                name,
                packages_path,
                files_examined,
                search_patterns,
            } => {
                assert_eq!(name, "test-package");
                assert_eq!(
                    packages_path,
                    PathBuf::from("/home/user/.config/selfie/packages")
                );
                assert_eq!(files_examined, 15);
                assert_eq!(
                    search_patterns,
                    vec!["test-package.yml", "test-package.yaml"]
                );
            }
            _ => panic!("Expected PackageNotFound error"),
        }
    }

    #[test]
    fn test_multiple_packages_found_error_contains_conflicting_paths() {
        let conflicting_paths = vec![
            PathBuf::from("/packages/test-package.yml"),
            PathBuf::from("/packages/test-package.yaml"),
        ];

        let error = PackageError::MultiplePackagesFound {
            name: "test-package".to_string(),
            packages_path: PathBuf::from("/packages"),
            conflicting_paths: conflicting_paths.clone(),
            files_examined: 10,
            search_patterns: vec![
                "test-package.yml".to_string(),
                "test-package.yaml".to_string(),
            ],
        };

        // Test that context information is preserved
        match error {
            PackageError::MultiplePackagesFound {
                name,
                conflicting_paths: paths,
                files_examined,
                search_patterns,
                ..
            } => {
                assert_eq!(name, "test-package");
                assert_eq!(paths, conflicting_paths);
                assert_eq!(paths.len(), 2);
                assert_eq!(files_examined, 10);
                assert_eq!(search_patterns.len(), 2);
            }
            _ => panic!("Expected MultiplePackagesFound error"),
        }
    }

    #[test]
    fn test_environment_not_found_error_provides_suggestions() {
        let available_environments = vec![
            "macos".to_string(),
            "linux".to_string(),
            "windows".to_string(),
        ];

        let error = PackageError::EnvironmentNotFound {
            package_name: "test-package".to_string(),
            environment: "freebsd".to_string(),
            available_environments: available_environments.clone(),
            package_file: PathBuf::from("/packages/test-package.yml"),
        };

        // Test error message content
        let error_message = error.to_string();
        assert!(error_message.contains("freebsd"));
        assert!(error_message.contains("test-package"));

        // Test that available environments are accessible for user suggestions
        match error {
            PackageError::EnvironmentNotFound {
                package_name,
                environment,
                available_environments: envs,
                package_file,
            } => {
                assert_eq!(package_name, "test-package");
                assert_eq!(environment, "freebsd");
                assert_eq!(envs, available_environments);
                assert!(envs.contains(&"macos".to_string()));
                assert!(envs.contains(&"linux".to_string()));
                assert!(envs.contains(&"windows".to_string()));
                assert_eq!(package_file, PathBuf::from("/packages/test-package.yml"));
            }
            _ => panic!("Expected EnvironmentNotFound error"),
        }
    }

    #[test]
    fn test_no_check_command_error_shows_alternatives() {
        let other_envs_with_check = vec!["linux".to_string(), "windows".to_string()];

        let error = PackageError::NoCheckCommand {
            package_name: "test-package".to_string(),
            environment: "macos".to_string(),
            package_file: PathBuf::from("/packages/test-package.yml"),
            other_envs_with_check: other_envs_with_check.clone(),
        };

        // Test error message
        let error_message = error.to_string();
        assert!(error_message.contains("macos"));
        assert!(error_message.contains("test-package"));

        // Test that alternative environments are available for suggestions
        match error {
            PackageError::NoCheckCommand {
                package_name,
                environment,
                package_file,
                other_envs_with_check: envs,
            } => {
                assert_eq!(package_name, "test-package");
                assert_eq!(environment, "macos");
                assert_eq!(package_file, PathBuf::from("/packages/test-package.yml"));
                assert_eq!(envs, other_envs_with_check);
                assert!(envs.contains(&"linux".to_string()));
                assert!(envs.contains(&"windows".to_string()));
            }
            _ => panic!("Expected NoCheckCommand error"),
        }
    }

    #[test]
    fn test_parse_error_contains_file_metadata() {
        use crate::package::port::PackageParseError;
        use std::sync::Arc;

        // Create a realistic parse error
        let parse_error = PackageParseError::YamlParse {
            package_path: PathBuf::from("/packages/broken.yml"),
            source: Arc::new(
                serde_yaml::from_str::<serde_yaml::Value>("invalid: yaml: [unclosed").unwrap_err(),
            ),
        };

        let error = PackageError::ParseError {
            name: "broken-package".to_string(),
            packages_path: PathBuf::from("/packages"),
            failed_file: PathBuf::from("/packages/broken.yml"),
            file_size_bytes: 1024,
            source: parse_error,
        };

        // Test that file context is available
        match error {
            PackageError::ParseError {
                name,
                packages_path,
                failed_file,
                file_size_bytes,
                source,
            } => {
                assert_eq!(name, "broken-package");
                assert_eq!(packages_path, PathBuf::from("/packages"));
                assert_eq!(failed_file, PathBuf::from("/packages/broken.yml"));
                assert_eq!(file_size_bytes, 1024);

                // Verify the parse error is preserved
                match source {
                    PackageParseError::YamlParse { package_path, .. } => {
                        assert_eq!(package_path, PathBuf::from("/packages/broken.yml"));
                    }
                    _ => panic!("Expected YamlParse error"),
                }
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_error_context_can_be_extracted_for_debugging() {
        let error = PackageError::PackageNotFound {
            name: "debug-package".to_string(),
            packages_path: PathBuf::from("/debug/packages"),
            files_examined: 42,
            search_patterns: vec!["debug-package.yml".to_string()],
        };

        // Demonstrate how to extract context for debugging/logging
        let debug_info = match &error {
            PackageError::PackageNotFound {
                name,
                packages_path,
                files_examined,
                search_patterns,
            } => {
                format!(
                    "Package '{}' not found in '{}' after examining {} files with patterns: {:?}",
                    name,
                    packages_path.display(),
                    files_examined,
                    search_patterns
                )
            }
            _ => panic!("Expected PackageNotFound error"),
        };

        assert!(debug_info.contains("debug-package"));
        assert!(debug_info.contains("42 files"));
        assert!(debug_info.contains("debug-package.yml"));
    }

    #[test]
    fn test_error_debug_output_includes_all_context() {
        let error = PackageError::MultiplePackagesFound {
            name: "test-multi".to_string(),
            packages_path: PathBuf::from("/test"),
            conflicting_paths: vec![
                PathBuf::from("/test/test-multi.yml"),
                PathBuf::from("/test/test-multi.yaml"),
            ],
            files_examined: 5,
            search_patterns: vec!["test-multi.yml".to_string(), "test-multi.yaml".to_string()],
        };

        let debug_output = format!("{error:?}");

        // Verify that all context fields appear in debug output
        assert!(debug_output.contains("test-multi"));
        assert!(debug_output.contains("files_examined: 5"));
        assert!(debug_output.contains("search_patterns"));
        assert!(debug_output.contains("conflicting_paths"));
    }
}
