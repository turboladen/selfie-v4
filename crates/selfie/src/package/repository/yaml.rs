use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    fs::FileSystem,
    package::{
        Package,
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
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

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
    fn get_package(&self, name: &str) -> Result<Package, PackageRepoError> {
        let package_files = self.find_package_files(name)?;

        if package_files.is_empty() {
            return Err(PackageError::PackageNotFound {
                name: name.to_string(),
                packages_path: self.package_dir.to_path_buf(),
            }
            .into());
        }

        if package_files.len() > 1 {
            return Err(PackageError::MultiplePackagesFound {
                name: name.to_string(),
                packages_path: self.package_dir.to_path_buf(),
            }
            .into());
        }

        let package_file = &package_files[0];
        let package = self
            .load_package_from_file(package_file)
            .map_err(|source| PackageError::ParseError {
                name: name.to_string(),
                packages_path: self.package_dir.to_path_buf(),
                source,
            })?;

        Ok(package)
    }

    fn list_packages(&self) -> Result<ListPackagesOutput, PackageListError> {
        if !self.fs.path_exists(&self.package_dir) {
            return Err(PackageListError::PackageDirectoryNotFound(
                self.package_dir.to_path_buf(),
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
                self.package_dir.to_path_buf(),
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

        fs.expect_path_exists()
            .with(predicate::eq(package_dir.clone()))
            .returning(|_| true);

        // Create mock package file
        let package_path = package_dir.join("ripgrep.yaml");
        let yaml = r"
            name: ripgrep
            version: 0.1.0
            environments:
              mac:
                install: brew install ripgrep
        ";

        fs.expect_path_exists()
            .with(predicate::eq(package_path.clone()))
            .returning(|_| true);
        fs.expect_path_exists()
            .with(predicate::eq(package_dir.join("ripgrep.yml")))
            .returning(|_| false);
        fs.mock_read_file(package_path, yaml);

        let repo = YamlPackageRepository::new(fs, &package_dir);
        let package = repo.get_package("ripgrep").unwrap();

        assert_eq!(package.name, "ripgrep");
        assert_eq!(package.version, "0.1.0");
        assert_eq!(package.environments.len(), 1);
    }

    #[test]
    fn test_get_package_not_found() {
        let mut fs = MockFileSystem::default();
        let package_dir = PathBuf::from("/test/packages");

        fs.expect_path_exists()
            .with(predicate::eq(package_dir.clone()))
            .returning(|_| true);
        fs.expect_path_exists()
            .with(predicate::eq(package_dir.join("nonexistent.yaml")))
            .returning(|_| false);
        fs.expect_path_exists()
            .with(predicate::eq(package_dir.join("nonexistent.yml")))
            .returning(|_| false);

        let repo = YamlPackageRepository::new(fs, &package_dir);
        let result = repo.get_package("nonexistent");

        assert!(matches!(
            result,
            Err(PackageRepoError::PackageError(
                PackageError::PackageNotFound { .. }
            ))
        ));
    }

    #[test]
    fn test_get_package_directory_not_found() {
        let mut fs = MockFileSystem::default();
        let package_dir = PathBuf::from("/test/nonexistent");

        fs.expect_path_exists()
            .with(predicate::eq(package_dir.clone()))
            .returning(|_| false);

        let repo = YamlPackageRepository::new(fs, &package_dir);
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

        fs.mock_path_exists(&package_dir, true);
        fs.mock_path_exists(&yaml_path, true);
        fs.mock_path_exists(&yml_path, true);
        let repo = YamlPackageRepository::new(fs, &package_dir);
        let result = repo.get_package("ripgrep");

        assert!(matches!(
            result,
            Err(PackageRepoError::PackageError(
                PackageError::MultiplePackagesFound { .. }
            ))
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

        let repo = YamlPackageRepository::new(fs, &package_dir);

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

        let repo = YamlPackageRepository::new(fs, &package_dir);
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

        let repo = YamlPackageRepository::new(fs, Path::new("/dummy")); // Path doesn't matter here
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

        let repo = YamlPackageRepository::new(fs, &package_dir);
        let packages = repo.available_packages().unwrap();

        // Should find only valid packages
        assert_eq!(packages.len(), 2);

        // Check package details
        assert!(packages.iter().any(|p| *p == "ripgrep"));
        assert!(packages.iter().any(|p| *p == "fzf"));
    }
}
