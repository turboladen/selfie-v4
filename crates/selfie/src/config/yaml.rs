use std::path::PathBuf;

use config::FileFormat;

use crate::{config::AppConfig, fs::FileSystem};

use super::loader::{ConfigLoadError, ConfigLoader};

pub struct YamlLoader<'a, F: FileSystem> {
    fs: &'a F,
}

impl<'a, F: FileSystem> YamlLoader<'a, F> {
    pub fn new(fs: &'a F) -> Self {
        Self { fs }
    }
}

impl<F: FileSystem> ConfigLoader for YamlLoader<'_, F> {
    fn load_config(&self) -> Result<AppConfig, ConfigLoadError> {
        let config_paths = match self.find_config_file_paths() {
            Ok(paths) => paths,
            Err(searched) => {
                return Err(ConfigLoadError::NotFound { searched });
            }
        };

        if config_paths.len() > 1 {
            return Err(ConfigLoadError::MultipleFound(
                config_paths
                    .into_iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>(),
            ));
        }

        // This should never happen now since find_config_file_paths
        // either returns a non-empty vector or an error
        if config_paths.is_empty() {
            return Err(ConfigLoadError::NotFound {
                searched: PathBuf::from("~/.config/selfie"),
            });
        }

        // Start with default configuration
        let mut builder = config::Config::builder();

        let config_path = &config_paths[0];

        let file_contents = self.fs.read_file(config_path)?;

        builder = builder.add_source(config::File::from_str(&file_contents, FileFormat::Yaml));

        // Build the config
        let config = builder.build()?;

        // Convert to our type
        let mut app_config: AppConfig = config.try_deserialize()?;

        // Special handling for package_directory ~ expansion
        if let Ok(expanded) = self.fs.expand_path(app_config.package_directory()) {
            app_config.package_directory = expanded;
        }

        Ok(app_config)
    }

    fn find_config_file_paths(&self) -> Result<Vec<PathBuf>, PathBuf> {
        let mut paths = Vec::new();

        let config_dir = self.fs.config_dir().map_err(|_| {
            // When config_dir fails, we can't determine a search path
            // Return a generic path to indicate the search location
            PathBuf::from("~/.config/selfie")
        })?;

        let config_yaml = config_dir.join("config.yaml");
        let config_yml = config_dir.join("config.yml");

        if self.fs.path_exists(&config_yaml) {
            paths.push(config_yaml);
        }
        if self.fs.path_exists(&config_yml) {
            paths.push(config_yml);
        }

        if paths.is_empty() {
            return Err(config_dir);
        }

        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::filesystem::{FileSystemError, MockFileSystem};
    use std::path::Path;

    fn setup_test_fs() -> (MockFileSystem, PathBuf) {
        let mut fs = MockFileSystem::default();

        // Set up mock HOME environment for test
        let home_dir = Path::new("/home/test");

        // Create .config/selfie/config.yaml
        let config_yaml = r#"
            environment: "test-env"
            package_directory: "/test/packages"
        "#;

        let config_dir = home_dir.join(".config").join("selfie");
        let config_path = config_dir.join("config.yaml");

        fs.mock_path_exists(&config_path, true);
        fs.mock_path_exists(&config_dir.join("config.yml"), false);
        fs.mock_read_file(config_path, config_yaml);

        (fs, home_dir.into())
    }

    mod find_config_file_paths {
        use super::*;

        #[test]
        fn test_find_config_paths() {
            let (mut fs, home_dir) = setup_test_fs();
            let config_dir = home_dir.join(".config").join("selfie");
            fs.mock_config_dir_ok(&config_dir);
            fs.mock_path_exists(config_dir.join("selfie").join("config.yaml"), true);

            let loader = YamlLoader::new(&fs);

            let paths = loader.find_config_file_paths().unwrap();

            // Should find at least the one we set up
            assert!(!paths.is_empty());
            assert!(paths.iter().any(|p| p.ends_with("config.yaml")));
        }

        #[test]
        fn test_find_config_paths_multiple_formats() {
            let mut fs = MockFileSystem::default();
            let config_dir = Path::new("/home/test/.config/selfie");

            // Mock both .yaml and .yml existing
            let yaml_path = config_dir.join("config.yaml");
            let yml_path = config_dir.join("config.yml");

            fs.mock_config_dir_ok(&config_dir);
            fs.mock_path_exists(&yaml_path, true);
            fs.mock_path_exists(&yml_path, true);

            let loader = YamlLoader::new(&fs);
            let paths = loader.find_config_file_paths().unwrap();

            // Should find both files
            assert_eq!(paths.len(), 2);
            assert!(paths.contains(&yaml_path));
            assert!(paths.contains(&yml_path));
        }

        #[test]
        fn test_find_config_paths_no_config_dir() {
            let mut fs = MockFileSystem::default();

            // Mock config_dir failing
            fs.expect_config_dir()
                .return_once(|| Err(FileSystemError::HomeDirNotFound));

            let loader = YamlLoader::new(&fs);
            let result = loader.find_config_file_paths();

            // Should return error when config dir can't be found
            assert!(result.is_err());
            let searched_path = result.unwrap_err();
            assert_eq!(searched_path, PathBuf::from("~/.config/selfie"));
        }
    }

    mod load_config {
        use super::*;

        #[test]
        fn test_load_config() {
            let (mut fs, home_dir) = setup_test_fs();
            let config_dir = home_dir.join(".config").join("selfie");
            fs.mock_config_dir_ok(&config_dir);
            fs.mock_path_exists(config_dir.join("config.yaml"), true);

            let package_dir = Path::new("/test/packages");
            fs.mock_path_exists(&package_dir, true);
            fs.mock_expand_path(&package_dir, &package_dir);

            let loader = YamlLoader::new(&fs);
            let config = loader.load_config().unwrap();

            // Check the loaded values
            assert_eq!(config.environment, "test-env");
            assert_eq!(config.package_directory, package_dir);
        }

        #[test]
        fn test_load_config_not_found() {
            let mut fs = MockFileSystem::default(); // Empty file system
            let config_dir = Path::new("/home/test/.config/selfie");
            fs.mock_config_dir_ok(&config_dir);
            fs.mock_path_exists(config_dir, true);
            fs.mock_path_exists(config_dir.join("config.yaml"), false);
            fs.mock_path_exists(config_dir.join("config.yml"), false);

            let loader = YamlLoader::new(&fs);

            // Should return error
            let result = loader.load_config();
            assert!(matches!(
                result,
                Err(ConfigLoadError::NotFound { searched: _ })
            ));
        }

        #[test]
        fn test_load_config_with_extended_settings() {
            let mut fs = MockFileSystem::default();
            let config_dir = Path::new("/home/test/.config/selfie");

            // Config with extended settings
            let config_yaml = r#"
            environment: "test-env"
            package_directory: "/test/packages"
            command_timeout: 120
            stop_on_error: false
            max_parallel_installations: 8
        "#;

            fs.mock_config_file(config_dir, config_yaml);
            fs.mock_expand_path("/test/packages", "/test/packages");

            let loader = YamlLoader::new(&fs);
            let config = loader.load_config().unwrap();

            // Check basic settings
            assert_eq!(config.environment, "test-env");
            assert_eq!(config.package_directory, Path::new("/test/packages"));

            // Check extended settings
            assert_eq!(config.command_timeout, 120.try_into().unwrap());
            assert!(!config.stop_on_error);
            assert_eq!(config.max_parallel_installations, 8.try_into().unwrap());
        }

        #[test]
        fn test_load_config_invalid_yaml() {
            let mut fs = MockFileSystem::default();
            let config_dir = Path::new("/home/test/.config/selfie");

            // Set up an invalid YAML file
            let invalid_yaml = r#"
        environment: "test-env"
        package_directory: "/test/packages"
        invalid:yaml:format
    "#;

            fs.mock_config_file(config_dir, invalid_yaml);

            let loader = YamlLoader::new(&fs);
            let result = loader.load_config();

            assert!(result.is_err());
            if let Err(err) = result {
                match err {
                    ConfigLoadError::ConfigError(_) => {
                        // Expected error type
                    }
                    _ => panic!("Expected ConfigError, got: {err:?}"),
                }
            }
        }

        #[test]
        fn test_load_config_missing_required_fields() {
            let mut fs = MockFileSystem::default();
            let config_dir = Path::new("/home/test/.config/selfie");

            // Config missing required fields
            let incomplete_yaml = r#"
        # Missing environment field
        package_directory: "/test/packages"
    "#;

            fs.mock_config_file(config_dir, incomplete_yaml);

            let loader = YamlLoader::new(&fs);
            let result = loader.load_config();

            assert!(result.is_err());
            if let Err(err) = result {
                match err {
                    ConfigLoadError::ConfigError(_) => {
                        // Expected error type for missing fields
                    }
                    _ => panic!("Expected ConfigError, got: {err:?}"),
                }
            }
        }

        #[test]
        fn test_load_config_invalid_field_types() {
            let mut fs = MockFileSystem::default();
            let config_dir = Path::new("/home/test/.config/selfie");

            // Config with invalid types
            let invalid_types_yaml = r#"
        environment: "test-env"
        package_directory: "/test/packages"
        command_timeout: "not-a-number"  # Should be a number
    "#;

            fs.mock_config_file(config_dir, invalid_types_yaml);

            let loader = YamlLoader::new(&fs);
            let result = loader.load_config();

            assert!(result.is_err());
        }

        #[test]
        fn test_load_config_with_tilde_expansion() {
            let mut fs = MockFileSystem::default();
            let config_dir = Path::new("/home/test/.config/selfie");
            let home_dir = Path::new("/home/test");

            // Config with tilde in path
            let tilde_yaml = r#"
        environment: "test-env"
        package_directory: "~/packages"
    "#;

            let expanded_path = home_dir.join("packages");

            fs.mock_config_file(config_dir, tilde_yaml);
            fs.mock_expand_path(Path::new("~/packages"), &expanded_path);

            let loader = YamlLoader::new(&fs);
            let config = loader.load_config().unwrap();

            assert_eq!(config.package_directory, expanded_path);
        }

        #[test]
        fn test_load_config_defaults() {
            let mut fs = MockFileSystem::default();
            let config_dir = Path::new("/home/test/.config/selfie");

            // Minimal valid config
            let minimal_yaml = r#"
        environment: "test-env"
        package_directory: "/test/packages"
        # All other fields will use defaults
    "#;

            let config_path = config_dir.join("config.yaml");
            fs.mock_config_dir_ok(&config_dir);
            fs.mock_path_exists(&config_path, true);
            fs.mock_path_exists(&config_dir.join("config.yml"), false);
            fs.mock_read_file(&config_path, minimal_yaml);
            fs.mock_expand_path(Path::new("/test/packages"), Path::new("/test/packages"));

            let loader = YamlLoader::new(&fs);
            let config = loader.load_config().unwrap();

            // Check defaults were properly applied
            assert_eq!(config.environment, "test-env");
            assert_eq!(config.package_directory, Path::new("/test/packages"));
            assert!(!config.verbose); // Default
            assert!(config.use_colors); // Default
            assert!(config.stop_on_error); // Default

            // Check command_timeout has default value (60)
            assert_eq!(config.command_timeout.get(), 60);

            // Check max_parallel_installations has sensible default value
            assert!(config.max_parallel_installations.get() > 0);
        }

        #[test]
        fn test_multiple_files() {
            let mut fs = MockFileSystem::default();
            let config_dir = Path::new("/home/test/.config/selfie");

            // Mock both .yaml and .yml existing
            let yaml_path = config_dir.join("config.yaml");
            let yml_path = config_dir.join("config.yml");

            fs.mock_config_dir_ok(&config_dir);
            fs.mock_path_exists(&yaml_path, true);
            fs.mock_path_exists(&yml_path, true);

            let loader = YamlLoader::new(&fs);
            let err = loader.load_config();

            // Should find both files
            assert!(matches!(err, Err(ConfigLoadError::MultipleFound(_))));
        }

        #[test]
        fn test_config_load_invalid_yaml_error() {
            let mut fs = MockFileSystem::default();
            let config_dir = Path::new("/home/test/.config/selfie");
            let config_path = config_dir.join("config.yaml");

            fs.mock_config_dir_ok(&config_dir);
            fs.mock_path_exists(&config_path, true);
            fs.mock_path_exists(&config_dir.join("config.yml"), false);

            // Mock invalid YAML content
            let invalid_yaml = "invalid: yaml: content: [";
            fs.mock_read_file(&config_path, invalid_yaml);

            let loader = YamlLoader::new(&fs);
            let result = loader.load_config();

            assert!(result.is_err());
            match result.unwrap_err() {
                ConfigLoadError::ConfigError(_) => {
                    // Expected config error for invalid YAML
                }
                _ => panic!("Expected ConfigError for invalid YAML"),
            }
        }

        #[test]
        fn test_config_load_filesystem_error() {
            let mut fs = MockFileSystem::default();
            let config_dir = Path::new("/home/test/.config/selfie");
            let config_path = config_dir.join("config.yaml");

            fs.mock_config_dir_ok(&config_dir);
            fs.mock_path_exists(&config_path, true);
            fs.mock_path_exists(&config_dir.join("config.yml"), false);

            // Mock filesystem error when reading file
            fs.expect_read_file()
                .with(mockall::predicate::eq(config_path.clone()))
                .return_once(|_| {
                    Err(FileSystemError::IoError(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        "Permission denied",
                    )))
                });

            let loader = YamlLoader::new(&fs);
            let result = loader.load_config();

            assert!(result.is_err());
            match result.unwrap_err() {
                ConfigLoadError::FileSystemError(FileSystemError::IoError(e)) => {
                    assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied);
                }
                other => panic!("Expected FileSystemError::IoError, got: {other:?}"),
            }
        }

        #[test]
        fn test_config_load_config_dir_error() {
            let mut fs = MockFileSystem::default();

            // Mock config_dir failing
            fs.expect_config_dir()
                .return_once(|| Err(FileSystemError::HomeDirNotFound));

            let loader = YamlLoader::new(&fs);
            let result = loader.load_config();

            assert!(result.is_err());
            match result.unwrap_err() {
                ConfigLoadError::NotFound { searched } => {
                    assert_eq!(searched, PathBuf::from("~/.config/selfie"));
                }
                other => panic!("Expected ConfigLoadError::NotFound, got: {other:?}"),
            }
        }

        #[test]
        fn test_config_error_display_formatting() {
            // Test NotFound error
            let searched_path = PathBuf::from("/searched/paths");
            let not_found_error = ConfigLoadError::NotFound {
                searched: searched_path,
            };
            assert!(
                not_found_error
                    .to_string()
                    .contains("No configuration file found")
            );
            assert!(not_found_error.to_string().contains("/searched/paths"));

            // Test MultipleFound error
            let multiple_error = ConfigLoadError::MultipleFound(vec![
                "config1.yaml".to_string(),
                "config2.yml".to_string(),
            ]);
            assert!(
                multiple_error
                    .to_string()
                    .contains("Multiple configuration files found")
            );
            assert!(multiple_error.to_string().contains("config1.yaml"));
        }
    }
}
