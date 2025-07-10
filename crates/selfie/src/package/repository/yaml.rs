use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    fs::FileSystem,
    package::{
        GetPackage, Package,
        port::{
            ListPackagesOutput, PackageError, PackageListError, PackageParseError,
            PackageRepoError, PackageRepository,
        },
    },
};

#[derive(Debug, Clone)]
pub struct YamlPackageRepository<F: FileSystem> {
    fs: F,
    package_dir: PathBuf,
}

impl<F: FileSystem> YamlPackageRepository<F> {
    pub fn new(fs: F, package_dir: PathBuf) -> Self {
        Self { fs, package_dir }
    }

    /// List all YAML files in a directory.
    ///
    fn list_yaml_files(&self, dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
        let entries = self
            .fs
            .list_directory(dir)
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        let yaml_files: Vec<PathBuf> = entries
            .into_iter()
            .filter(|path| {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    ext_str == "yaml" || ext_str == "yml"
                } else {
                    false
                }
            })
            .collect();

        Ok(yaml_files)
    }

    /// Enhanced version that tracks search context
    fn find_package_files_with_context(
        &self,
        name: &str,
        files_examined: &mut usize,
    ) -> Result<Vec<PathBuf>, std::io::Error> {
        let entries = self
            .fs
            .list_directory(&self.package_dir)
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        let mut matching_files = Vec::new();

        for path in entries {
            *files_examined += 1;

            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name == format!("{name}.yml") || file_name == format!("{name}.yaml") {
                    matching_files.push(path);
                }
            }
        }

        Ok(matching_files)
    }

    fn get_file_size(&self, path: &Path) -> u64 {
        self.fs
            .read_file(path)
            .map(|content| content.len() as u64)
            .unwrap_or(0)
    }

    // Load a Package from a file using the FileSystem trait
    fn load_package_from_file(&self, path: &Path) -> Result<Package, PackageParseError> {
        let content = self
            .fs
            .read_file(path)
            .map_err(|e| PackageParseError::FileSystemError {
                package_path: path.to_path_buf(),
                source_message: e.to_string(),
            })?;

        let mut package: Package =
            serde_yaml::from_str(&content).map_err(|e| PackageParseError::YamlParse {
                package_path: path.to_path_buf(),
                source: Arc::new(e),
            })?;
        package.path = path.to_path_buf();

        Ok(package)
    }
}

impl<F: FileSystem> PackageRepository for YamlPackageRepository<F> {
    fn get_package(&self, name: &str) -> Result<GetPackage, PackageRepoError> {
        // Check if package directory exists first
        if !self.fs.path_exists(&self.package_dir) {
            return Err(PackageRepoError::PackageListError(
                PackageListError::PackageDirectoryNotFound(self.package_dir.clone()),
            ));
        }

        let search_patterns = vec![format!("{}.yml", name), format!("{}.yaml", name)];
        let mut files_examined = 0;

        let package_files = self
            .find_package_files_with_context(name, &mut files_examined)
            .map_err(|e| PackageRepoError::IoError(Arc::new(e)))?;

        if package_files.is_empty() {
            return Err(Box::new(PackageError::PackageNotFound {
                name: name.to_string(),
                packages_path: self.package_dir.clone(),
                files_examined,
                search_patterns,
            })
            .into());
        }

        if package_files.len() > 1 {
            return Err(Box::new(PackageError::MultiplePackagesFound {
                name: name.to_string(),
                packages_path: self.package_dir.clone(),
                conflicting_paths: package_files,
                files_examined,
                search_patterns,
            })
            .into());
        }

        let package_file = &package_files[0];
        let file_size = self.get_file_size(package_file);

        let package = self
            .load_package_from_file(package_file)
            .map_err(|source| {
                Box::new(PackageError::ParseError {
                    name: name.to_string(),
                    packages_path: self.package_dir.clone(),
                    failed_file: package_file.clone(),
                    file_size_bytes: file_size,
                    source,
                })
            })?;

        Ok(GetPackage::from_existing(package, package_file.clone()))
    }

    fn list_packages(&self) -> Result<ListPackagesOutput, PackageListError> {
        if !self.fs.path_exists(&self.package_dir) {
            return Err(PackageListError::PackageDirectoryNotFound(
                self.package_dir.clone(),
            ));
        }

        // Get all YAML files in the directory
        let yaml_files = self.list_yaml_files(&self.package_dir).map_err(Arc::new)?;

        // Parse each file into a Package
        let mut packages: Vec<Result<Package, PackageParseError>> = Vec::new();

        for path in yaml_files {
            packages.push(self.load_package_from_file(&path));
        }

        Ok(ListPackagesOutput(packages))
    }

    fn find_package_files(&self, name: &str) -> Result<Vec<PathBuf>, PackageListError> {
        if !self.fs.path_exists(&self.package_dir) {
            return Err(PackageListError::PackageDirectoryNotFound(
                self.package_dir.clone(),
            ));
        }

        // Look for both name.yaml and name.yml
        let yaml_path = self.package_dir.join(format!("{name}.yaml"));
        let yml_path = self.package_dir.join(format!("{name}.yml"));

        let mut result = Vec::new();
        if self.fs.path_exists(&yaml_path) {
            result.push(yaml_path);
        }
        if self.fs.path_exists(&yml_path) {
            result.push(yml_path);
        }

        Ok(result)
    }

    fn save_package(&self, package: &Package, path: &Path) -> Result<(), PackageRepoError> {
        // Serialize the package to YAML
        let yaml_content = serde_yaml::to_string(package).map_err(|e| {
            PackageRepoError::IoError(Arc::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to serialize package to YAML: {}", e),
            )))
        })?;

        // Write the YAML content to the specified path
        self.fs.write_file(path, yaml_content.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mockall::*;

    use super::*;
    use crate::fs::filesystem::MockFileSystem;
    use crate::package::port::PackageRepoError;

    #[test]
    fn test_get_package_success() {
        let mut fs = MockFileSystem::default();
        let package_dir = PathBuf::from("/test/packages");

        // Mock path_exists for directory
        fs.expect_path_exists()
            .with(predicate::eq(package_dir.clone()))
            .returning(|_| true);

        // Mock list_directory to return the package file
        let package_path = package_dir.join("ripgrep.yaml");
        let package_path_for_list = package_path.clone();
        fs.expect_list_directory()
            .with(predicate::eq(package_dir.clone()))
            .returning(move |_| Ok(vec![package_path_for_list.clone()]));

        let yaml = r"
            name: ripgrep
            version: 0.1.0
            environments:
              mac:
                install: brew install ripgrep
        ";

        fs.mock_read_file(package_path, yaml);

        let repo = YamlPackageRepository::new(fs, package_dir.clone());
        let package = repo.get_package("ripgrep").unwrap();

        assert_eq!(package.package.name, "ripgrep");
        assert_eq!(package.package.version, "0.1.0");
        assert_eq!(package.package.environments.len(), 1);
    }

    #[test]
    fn test_get_package_not_found() {
        let mut fs = MockFileSystem::default();
        // Mock filesystem to simulate package not found
        let package_dir = PathBuf::from("/test/packages");

        // Mock path_exists for directory
        fs.expect_path_exists()
            .with(predicate::eq(package_dir.clone()))
            .returning(|_| true);

        fs.expect_list_directory()
            .with(predicate::eq(package_dir.clone()))
            .returning(|_| Ok(vec![PathBuf::from("/test/packages/other.yaml")]));

        let repo = YamlPackageRepository::new(fs, package_dir.clone());
        let result = repo.get_package("nonexistent");

        assert!(matches!(
            result,
            Err(PackageRepoError::PackageError(ref box_error))
            if matches!(**box_error, PackageError::PackageNotFound { .. })
        ));
    }

    #[test]
    fn test_get_package_directory_not_found() {
        let mut fs = MockFileSystem::default();
        // Mock filesystem error
        let package_dir = PathBuf::from("/test/nonexistent");

        // Mock path_exists to return false for the directory
        fs.expect_path_exists()
            .with(predicate::eq(package_dir.clone()))
            .returning(|_| false);

        let repo = YamlPackageRepository::new(fs, package_dir.clone());
        let result = repo.get_package("ripgrep");

        assert!(matches!(
            result,
            Err(PackageRepoError::PackageListError(
                PackageListError::PackageDirectoryNotFound(_)
            ))
        ));
    }

    #[test]
    fn test_get_package_multiple_found() {
        let mut fs = MockFileSystem::default();
        let package_dir = PathBuf::from("/test/packages");

        // Create multiple mock package files with the same name
        let yaml_path = package_dir.join("ripgrep.yaml");
        let yml_path = package_dir.join("ripgrep.yml");

        // Mock path_exists for directory
        fs.expect_path_exists()
            .with(predicate::eq(package_dir.clone()))
            .returning(|_| true);

        // Mock list_directory to return both files
        let yaml_path_for_list = yaml_path.clone();
        let yml_path_for_list = yml_path.clone();
        fs.expect_list_directory()
            .with(predicate::eq(package_dir.clone()))
            .returning(move |_| Ok(vec![yaml_path_for_list.clone(), yml_path_for_list.clone()]));

        let repo = YamlPackageRepository::new(fs, package_dir.clone());
        let result = repo.get_package("ripgrep");

        assert!(matches!(
            result,
            Err(PackageRepoError::PackageError(ref box_error))
            if matches!(**box_error, PackageError::MultiplePackagesFound { .. })
        ));
    }

    #[test]
    fn test_find_package_files() {
        let mut fs = MockFileSystem::default();
        let package_dir = PathBuf::from("/test/packages");

        // Create mock package files
        let yaml_path = package_dir.join("ripgrep.yaml");
        let yml_path = package_dir.join("other.yml");

        fs.expect_path_exists()
            .with(predicate::eq(package_dir.clone()))
            .returning(|_| true);
        fs.expect_path_exists()
            .with(predicate::eq(yaml_path.clone()))
            .returning(|_| true);
        fs.expect_path_exists()
            .with(predicate::eq(package_dir.join("ripgrep.yml")))
            .returning(|_| false);
        fs.expect_path_exists()
            .with(predicate::eq(yml_path.clone()))
            .returning(|_| true);
        fs.expect_path_exists()
            .with(predicate::eq(package_dir.join("other.yaml")))
            .returning(|_| false);
        fs.expect_path_exists()
            .with(predicate::eq(package_dir.join("nonexistent.yaml")))
            .returning(|_| false);
        fs.expect_path_exists()
            .with(predicate::eq(package_dir.join("nonexistent.yml")))
            .returning(|_| false);

        let repo = YamlPackageRepository::new(fs, package_dir.clone());

        // Should find ripgrep.yaml
        let files = repo.find_package_files("ripgrep").unwrap();
        assert_eq!(files.len(), 1, "{:#?}", &files);
        assert_eq!(files[0], yaml_path);

        // Should find other.yml
        let files = repo.find_package_files("other").unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], yml_path);

        // Should not find nonexistent
        let files = repo.find_package_files("nonexistent").unwrap();
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_list_packages() {
        let mut fs = MockFileSystem::default();
        let package_dir = PathBuf::from("/test/packages");

        fs.expect_path_exists()
            .with(predicate::eq(package_dir.clone()))
            .returning(|_| true);

        // Add valid package files
        let package1 = r"
            name: ripgrep
            version: 1.0.0
            environments:
              test-env:
                install: brew install ripgrep
        ";

        let package2 = r"
            name: fzf
            version: 0.2.0
            environments:
              other-env:
                install: brew install fzf
        ";

        fs.mock_list_directory(
            package_dir.clone(),
            &[
                package_dir.join("ripgrep.yaml"),
                package_dir.join("fzf.yml"),
                package_dir.join("invalid.yaml"),
            ],
        );

        fs.mock_read_file(package_dir.join("ripgrep.yaml"), package1);
        fs.mock_read_file(package_dir.join("fzf.yml"), package2);
        fs.mock_read_file(package_dir.join("invalid.yaml"), "not valid yaml: :");

        let repo = YamlPackageRepository::new(fs, package_dir.clone());
        let package_output = repo.list_packages().unwrap();

        // Should find both valid packages
        assert_eq!(
            package_output.valid_packages().collect::<Vec<_>>().len(),
            2,
            "{:#?}",
            &package_output
        );
        assert_eq!(
            package_output.invalid_packages().collect::<Vec<_>>().len(),
            1,
            "{:#?}",
            &package_output
        );
        assert_eq!(package_output.len(), 3, "{:#?}", &package_output);

        // Check package details
        let ripgrep = package_output.get("ripgrep").unwrap();
        let fzf = package_output.get("fzf").unwrap();

        assert_eq!(ripgrep.version, "1.0.0");
        assert!(ripgrep.environments.contains_key("test-env"));

        assert_eq!(fzf.version, "0.2.0");
        assert!(fzf.environments.contains_key("other-env"));
    }

    #[test]
    fn test_list_yaml_files() {
        let mut fs = MockFileSystem::default();
        let dir = PathBuf::from("/test/dir");
        let cloned = dir.clone();

        fs.expect_list_directory()
            .with(predicate::eq(dir.clone()))
            .returning(move |_| {
                Ok(vec![
                    cloned.join("file1.yaml"),
                    cloned.join("file2.yml"),
                    cloned.join("file3.txt"),
                    cloned.join("file4.YAML"),
                    cloned.join("file5.YML"),
                ])
            });

        let repo = YamlPackageRepository::new(fs, Path::new("/dummy").to_path_buf()); // Path doesn't matter here
        let yaml_files = repo.list_yaml_files(&dir).unwrap();

        // Should find all yaml/yml files regardless of case
        assert_eq!(yaml_files.len(), 4);

        // Check each expected file is found
        assert!(yaml_files.contains(&dir.join("file1.yaml")));
        assert!(yaml_files.contains(&dir.join("file2.yml")));
        assert!(yaml_files.contains(&dir.join("file4.YAML")));
        assert!(yaml_files.contains(&dir.join("file5.YML")));

        // Check that non-yaml file is not included
        assert!(!yaml_files.contains(&dir.join("file3.txt")));
    }

    #[test]
    fn test_available_packages() {
        let mut fs = MockFileSystem::default();
        let package_dir = PathBuf::from("/test/packages");

        fs.expect_path_exists()
            .with(predicate::eq(package_dir.clone()))
            .returning(|_| true);

        // Add valid and invalid package files
        let package1 = r"
            name: ripgrep
            version: 1.0.0
            environments:
              test-env:
                install: brew install ripgrep
        ";

        let package2 = r"
            name: fzf
            version: 0.2.0
            environments:
              other-env:
                install: brew install fzf
        ";

        fs.mock_list_directory(
            package_dir.clone(),
            &[
                package_dir.join("ripgrep.yaml"),
                package_dir.join("fzf.yml"),
                package_dir.join("invalid.yaml"),
            ],
        );

        fs.mock_read_file(package_dir.join("ripgrep.yaml"), package1);
        fs.mock_read_file(package_dir.join("fzf.yml"), package2);
        fs.mock_read_file(package_dir.join("invalid.yaml"), "not valid yaml: :");

        let repo = YamlPackageRepository::new(fs, package_dir.clone());
        let packages = repo.available_packages().unwrap();

        // Should find only valid packages
        assert_eq!(packages.len(), 2);

        // Check package details
        assert!(packages.iter().any(|p| *p == "ripgrep"));
        assert!(packages.iter().any(|p| *p == "fzf"));
    }

    #[test]
    fn test_package_parse_error_handling() {
        let mut fs = MockFileSystem::default();
        let package_dir = PathBuf::from("/test/packages");
        let package_path = package_dir.join("invalid.yaml");

        fs.expect_path_exists()
            .with(predicate::eq(package_dir.clone()))
            .returning(|_| true);

        fs.expect_path_exists()
            .with(predicate::eq(package_path.clone()))
            .returning(|_| true);

        fs.expect_path_exists()
            .with(predicate::eq(package_dir.join("invalid.yml")))
            .returning(|_| false);

        // Mock invalid YAML content
        let invalid_yaml = "invalid: yaml: content: [";

        fs.mock_list_directory(package_dir.clone(), &[package_path.clone()]);
        fs.mock_read_file(package_path.clone(), invalid_yaml);

        let repo = YamlPackageRepository::new(fs, package_dir.clone());
        let result = repo.get_package("invalid");

        assert!(result.is_err());
        match result.unwrap_err() {
            PackageRepoError::PackageError(box_error) => match *box_error {
                PackageError::ParseError {
                    name,
                    packages_path,
                    source,
                    ..
                } => {
                    assert_eq!(name, "invalid");
                    assert_eq!(packages_path, package_dir);
                    match source {
                        PackageParseError::YamlParse {
                            package_path: error_path,
                            ..
                        } => {
                            assert_eq!(error_path, package_path);
                        }
                        _ => panic!("Expected YamlParse error"),
                    }
                }
                _ => panic!("Expected ParseError"),
            },
            _ => panic!("Expected PackageError"),
        }
    }

    #[test]
    fn test_directory_not_found_error() {
        let mut fs = MockFileSystem::default();
        let nonexistent_dir = PathBuf::from("/nonexistent");

        fs.expect_path_exists()
            .with(predicate::eq(nonexistent_dir.clone()))
            .returning(|_| false);

        let repo = YamlPackageRepository::new(fs, nonexistent_dir.clone());
        let result = repo.list_packages();

        assert!(result.is_err());
        match result.unwrap_err() {
            PackageListError::PackageDirectoryNotFound(path) => {
                assert_eq!(path, nonexistent_dir);
            }
            _ => panic!("Expected PackageDirectoryNotFound error"),
        }
    }

    #[test]
    fn test_multiple_packages_found_error() {
        let mut fs = MockFileSystem::default();
        let package_dir = PathBuf::from("/test/packages");

        fs.expect_path_exists()
            .with(predicate::eq(package_dir.clone()))
            .returning(|_| true);

        let file1 = package_dir.join("duplicate.yaml");
        let file2 = package_dir.join("duplicate.yml");

        fs.expect_path_exists()
            .with(predicate::eq(file1.clone()))
            .returning(|_| true);
        fs.expect_path_exists()
            .with(predicate::eq(file2.clone()))
            .returning(|_| true);

        // Create multiple files with the same package name
        let package_yaml = r"
            name: duplicate
            version: 1.0.0
            environments:
              test-env:
                install: echo test
        ";

        fs.mock_list_directory(package_dir.clone(), &[file1.clone(), file2.clone()]);
        fs.mock_read_file(file1, package_yaml);
        fs.mock_read_file(file2, package_yaml);

        let repo = YamlPackageRepository::new(fs, package_dir.clone());
        let result = repo.get_package("duplicate");

        assert!(result.is_err());
        match result.unwrap_err() {
            PackageRepoError::PackageError(box_error) => match *box_error {
                PackageError::MultiplePackagesFound {
                    name,
                    packages_path,
                    ..
                } => {
                    assert_eq!(name, "duplicate");
                    assert_eq!(packages_path, package_dir);
                }
                _ => panic!("Expected MultiplePackagesFound error"),
            },
            _ => panic!("Expected PackageError"),
        }
    }

    #[test]
    fn test_error_display_formatting() {
        let package_dir = PathBuf::from("/packages");

        // Test PackageNotFound error
        let not_found_error = PackageError::PackageNotFound {
            name: "missing".to_string(),
            packages_path: package_dir.clone(),
            files_examined: 0,
            search_patterns: vec!["missing.yml".to_string()],
        };
        assert!(not_found_error.to_string().contains("missing"));
        assert!(not_found_error.to_string().contains("/packages"));

        // Test MultiplePackagesFound error
        let multiple_error = PackageError::MultiplePackagesFound {
            name: "duplicate".to_string(),
            packages_path: package_dir.clone(),
            conflicting_paths: vec![
                PathBuf::from("/packages/duplicate.yml"),
                PathBuf::from("/packages/duplicate.yaml"),
            ],
            files_examined: 2,
            search_patterns: vec!["duplicate.yml".to_string(), "duplicate.yaml".to_string()],
        };
        assert!(multiple_error.to_string().contains("duplicate"));
        assert!(
            multiple_error
                .to_string()
                .contains("Multiple packages found")
        );

        // Test PackageDirectoryNotFound error
        let dir_error = PackageListError::PackageDirectoryNotFound(package_dir.clone());
        assert!(dir_error.to_string().contains("/packages"));
        assert!(dir_error.to_string().contains("does not exist"));
    }
}
