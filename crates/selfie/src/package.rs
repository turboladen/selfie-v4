// src/package.rs

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use thiserror::Error;

use crate::filesystem::FileSystem;

#[derive(Debug, Clone, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub maintainer: Option<String>,
    pub homepage: Option<String>,
    // Other package metadata fields can be added here
}

#[derive(Debug, Error)]
pub enum PackageError {
    #[error("Failed to list packages in directory {0}: {1}")]
    ListError(String, String),

    #[error("Failed to read package file {0}: {1}")]
    ReadError(String, String),

    #[error("Failed to parse package file {0}: {1}")]
    ParseError(String, String),
}

/// Lists available packages in the configured package directory
pub fn list_packages<F: FileSystem>(
    fs: &F,
    package_dir: &Path,
) -> Result<Vec<Package>, PackageError> {
    if !fs.path_exists(package_dir) {
        return Err(PackageError::ListError(
            package_dir.display().to_string(),
            "Directory does not exist".to_string(),
        ));
    }

    let files = fs
        .list_directory(package_dir)
        .map_err(|e| PackageError::ListError(package_dir.display().to_string(), e.to_string()))?;

    let mut packages = Vec::new();
    
    // Only consider YAML files as package definitions
    for file in files.iter().filter(|p| {
        let extension = p.extension().and_then(|ext| ext.to_str());
        matches!(extension, Some("yaml") | Some("yml"))
    }) {
        match load_package_from_file(fs, file) {
            Ok(package) => packages.push(package),
            Err(e) => {
                // Log error but continue with other packages
                eprintln!("Error loading package from {}: {}", file.display(), e);
            }
        }
    }

    Ok(packages)
}

/// Loads a package from a YAML file
pub fn load_package_from_file<F: FileSystem>(
    fs: &F,
    file_path: &Path,
) -> Result<Package, PackageError> {
    let content = fs
        .read_file(file_path)
        .map_err(|e| PackageError::ReadError(file_path.display().to_string(), e.to_string()))?;

    serde_yaml::from_str(&content).map_err(|e| {
        PackageError::ParseError(file_path.display().to_string(), e.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filesystem::MockFileSystem;
    use std::path::PathBuf;

    fn create_test_package() -> Package {
        Package {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A test package".to_string()),
            maintainer: Some("Test User".to_string()),
            homepage: Some("https://example.com".to_string()),
        }
    }

    #[test]
    fn test_list_packages_empty_dir() {
        let mut fs = MockFileSystem::default();
        let package_dir = PathBuf::from("/test/packages");

        // Mock an empty directory
        fs.expect_path_exists()
            .with(mockall::predicate::eq(package_dir.clone()))
            .return_const(true);
        fs.expect_list_directory()
            .with(mockall::predicate::eq(package_dir.clone()))
            .return_once(|_| Ok(vec![]));

        let packages = list_packages(&fs, &package_dir).unwrap();
        assert!(packages.is_empty());
    }

    #[test]
    fn test_list_packages_with_packages() {
        let mut fs = MockFileSystem::default();
        let package_dir = PathBuf::from("/test/packages");
        let yaml_file = package_dir.join("test-package.yaml");
        let non_yaml_file = package_dir.join("README.md");

        // Mock directory with one YAML file and one non-YAML file
        fs.expect_path_exists()
            .with(mockall::predicate::eq(package_dir.clone()))
            .return_const(true);
        fs.expect_list_directory()
            .with(mockall::predicate::eq(package_dir.clone()))
            .return_once(move |_| Ok(vec![yaml_file.clone(), non_yaml_file]));

        // Mock reading the YAML file
        let package_yaml = r#"
            name: test-package
            version: 1.0.0
            description: A test package
            maintainer: Test User
            homepage: https://example.com
        "#;
        fs.expect_read_file()
            .with(mockall::predicate::eq(yaml_file.clone()))
            .return_once(move |_| Ok(package_yaml.to_string()));

        let packages = list_packages(&fs, &package_dir).unwrap();
        
        assert_eq!(packages.len(), 1);
        let package = &packages[0];
        assert_eq!(package.name, "test-package");
        assert_eq!(package.version, "1.0.0");
        assert_eq!(package.description, Some("A test package".to_string()));
        assert_eq!(package.maintainer, Some("Test User".to_string()));
        assert_eq!(package.homepage, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_list_packages_nonexistent_dir() {
        let mut fs = MockFileSystem::default();
        let package_dir = PathBuf::from("/nonexistent/dir");

        // Mock a nonexistent directory
        fs.expect_path_exists()
            .with(mockall::predicate::eq(package_dir.clone()))
            .return_const(false);

        let result = list_packages(&fs, &package_dir);
        assert!(result.is_err());
        if let Err(PackageError::ListError(dir, _)) = result {
            assert_eq!(dir, package_dir.display().to_string());
        } else {
            panic!("Expected ListError");
        }
    }

    #[test]
    fn test_load_package_from_file_success() {
        let mut fs = MockFileSystem::default();
        let file_path = PathBuf::from("/test/packages/test-package.yaml");

        // Mock reading the file
        let package_yaml = r#"
            name: test-package
            version: 1.0.0
            description: A test package
            maintainer: Test User
            homepage: https://example.com
        "#;
        fs.expect_read_file()
            .with(mockall::predicate::eq(file_path.clone()))
            .return_once(move |_| Ok(package_yaml.to_string()));

        let package = load_package_from_file(&fs, &file_path).unwrap();
        
        assert_eq!(package.name, "test-package");
        assert_eq!(package.version, "1.0.0");
        assert_eq!(package.description, Some("A test package".to_string()));
        assert_eq!(package.maintainer, Some("Test User".to_string()));
        assert_eq!(package.homepage, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_load_package_from_file_read_error() {
        let mut fs = MockFileSystem::default();
        let file_path = PathBuf::from("/test/packages/test-package.yaml");

        // Mock file reading error
        fs.expect_read_file()
            .with(mockall::predicate::eq(file_path.clone()))
            .return_once(move |_| Err(crate::filesystem::FileSystemError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            ))));

        let result = load_package_from_file(&fs, &file_path);
        assert!(result.is_err());
        if let Err(PackageError::ReadError(file, _)) = result {
            assert_eq!(file, file_path.display().to_string());
        } else {
            panic!("Expected ReadError");
        }
    }

    #[test]
    fn test_load_package_from_file_parse_error() {
        let mut fs = MockFileSystem::default();
        let file_path = PathBuf::from("/test/packages/test-package.yaml");

        // Mock reading the file with invalid YAML content
        let invalid_yaml = "name: test-package\nversion: invalid: yaml: content";
        fs.expect_read_file()
            .with(mockall::predicate::eq(file_path.clone()))
            .return_once(move |_| Ok(invalid_yaml.to_string()));

        let result = load_package_from_file(&fs, &file_path);
        assert!(result.is_err());
        if let Err(PackageError::ParseError(file, _)) = result {
            assert_eq!(file, file_path.display().to_string());
        } else {
            panic!("Expected ParseError");
        }
    }
}
