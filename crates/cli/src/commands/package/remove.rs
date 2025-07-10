use dialoguer::{Confirm, theme::SimpleTheme};
use selfie::{
    config::AppConfig,
    fs::real::RealFileSystem,
    package::{port::PackageRepository, repository::yaml::YamlPackageRepository},
};
use tracing::info;

use crate::terminal_progress_reporter::TerminalProgressReporter;

pub(crate) async fn handle_remove(
    package_name: &str,
    config: &AppConfig,
    reporter: TerminalProgressReporter,
) -> i32 {
    info!("Removing package: {}", package_name);

    // Create repository to interact with packages
    let repo = YamlPackageRepository::new(RealFileSystem, config.package_directory().clone());

    // First, verify the package exists and get its details
    let package_blob = match repo.get_package(package_name) {
        Ok(blob) => blob,
        Err(_) => {
            reporter.report_error(format!("Package '{}' not found.", package_name));
            return 1;
        }
    };

    // Show package location
    reporter.report_info(format!("Package '{}' found at:", package_name));
    reporter.report_info(format!("  {}", package_blob.file_path.display()));

    // Check if this package is a dependency of others
    let dependent_packages = match repo.find_dependent_packages(package_name) {
        Ok(deps) => deps,
        Err(e) => {
            reporter.report_warning(format!("Could not check for dependent packages: {}", e));
            Vec::new()
        }
    };

    // Build confirmation prompt based on dependencies
    let (prompt, default_answer) = if !dependent_packages.is_empty() {
        reporter.report_warning(format!(
            "Package '{}' is a dependency of the following packages:",
            package_name
        ));
        for dep in &dependent_packages {
            reporter.report_warning(format!("  - {}", dep.name()));
        }
        (
            "Are you sure you want to remove this package?".to_string(),
            false,
        )
    } else {
        reporter.report_info(format!(
            "✓ Package '{}' is not a dependency of any other packages.",
            package_name
        ));
        (format!("Remove package '{}'?", package_name), false)
    };

    // Single confirmation prompt
    let confirm_removal = Confirm::with_theme(&SimpleTheme)
        .with_prompt(prompt)
        .default(default_answer)
        .interact();

    let proceed = match confirm_removal {
        Ok(true) => true,
        Ok(false) => {
            reporter.report_info("Package removal cancelled.");
            return 0;
        }
        Err(_) => {
            reporter.report_error("Failed to read user input.");
            return 1;
        }
    };

    if !proceed {
        return 0;
    }

    // Perform the actual removal
    if let Err(e) = repo.remove_package(package_name) {
        reporter.report_error(format!(
            "Failed to remove package '{}': {}",
            package_name, e
        ));
        return 1;
    }

    reporter.report_success(format!(
        "Package '{}' removed successfully from {}",
        package_name,
        package_blob.file_path.display()
    ));

    // Warn about broken dependencies if any exist
    if !dependent_packages.is_empty() {
        reporter.report_warning("Note: The following packages may have broken dependencies:");
        for dep in &dependent_packages {
            reporter.report_warning(format!("  - {}", dep.name()));
        }
        reporter.report_info("You may need to update these packages to remove the dependency.");
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use test_common::test_config_with_dir;

    fn create_mock_reporter() -> TerminalProgressReporter {
        TerminalProgressReporter::new(false)
    }

    #[test]
    fn test_handle_remove_package_not_found() {
        // Test behavior when trying to remove a non-existent package
        let temp_dir = TempDir::new().unwrap();
        let package_dir = temp_dir.path().join("packages");
        fs::create_dir_all(&package_dir).unwrap();

        let config = test_config_with_dir(&package_dir);
        let reporter = create_mock_reporter();

        let result = tokio_test::block_on(async {
            handle_remove("nonexistent-package", &config, reporter).await
        });

        // Should return error exit code
        assert_eq!(result, 1);
    }

    #[test]
    fn test_dependency_check_integration() {
        // Test that the dependency check integration works with the repository
        let temp_dir = TempDir::new().unwrap();
        let package_dir = temp_dir.path().join("packages");
        fs::create_dir_all(&package_dir).unwrap();

        // Create target package
        let target_content = r#"
name: target-package
version: 1.0.0
environments:
  test:
    install: echo 'install target'
    dependencies: []
"#;
        fs::write(
            package_dir.join("target-package.yml"),
            target_content.trim(),
        )
        .unwrap();

        // Create dependent package
        let dependent_content = r#"
name: dependent-package
version: 1.0.0
environments:
  test:
    install: echo 'install dependent'
    dependencies:
      - target-package
"#;
        fs::write(
            package_dir.join("dependent-package.yml"),
            dependent_content.trim(),
        )
        .unwrap();

        let repo = YamlPackageRepository::new(RealFileSystem, package_dir);

        // Test the repository method directly
        let dependents = repo.find_dependent_packages("target-package").unwrap();
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0].name(), "dependent-package");
    }
}
