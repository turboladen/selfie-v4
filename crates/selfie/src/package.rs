mod builder;
pub mod port;
pub mod repository;
pub mod validate;

pub use self::builder::{EnvironmentConfigBuilder, PackageBuilder};

// Core package entity and related types
use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

/// Core package entity representing a package definition
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Package {
    /// Package name
    pub(crate) name: String,

    /// Package version (for the selfie package file, not the underlying package)
    pub(crate) version: String,

    /// Optional homepage URL
    #[serde(default)]
    pub(crate) homepage: Option<String>,

    /// Optional package description
    #[serde(default)]
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
    #[serde(default)]
    pub(crate) check: Option<String>,

    /// Dependencies that must be installed before this package
    #[serde(default)]
    pub(crate) dependencies: Vec<String>,
}

impl EnvironmentConfig {
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
mod tests {
    use builder::PackageBuilder;

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
}
