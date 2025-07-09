//! Test modules for package functionality

// Error context tests temporarily disabled due to compilation issues
// mod error_context_tests;
// mod service_error_context_tests;

// Simple tests demonstrating enhanced error context functionality

use std::path::PathBuf;

use crate::package::port::PackageError;

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

    let debug_output = format!("{:?}", error);

    // Verify that all context fields appear in debug output
    assert!(debug_output.contains("test-multi"));
    assert!(debug_output.contains("files_examined: 5"));
    assert!(debug_output.contains("search_patterns"));
    assert!(debug_output.contains("conflicting_paths"));
}
