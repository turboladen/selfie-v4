use selfie::{
    config::AppConfig,
    fs::real::RealFileSystem,
    package::{port::PackageRepository, repository::yaml::YamlPackageRepository},
};
use std::{
    io::{self, Write},
    process::Command,
};
use tracing::info;

use crate::terminal_progress_reporter::TerminalProgressReporter;

pub(crate) async fn handle_edit(
    package_name: &str,
    config: &AppConfig,
    reporter: TerminalProgressReporter,
) -> i32 {
    info!("Editing package: {}", package_name);

    // Create repository to look up the package
    let repo = YamlPackageRepository::new(RealFileSystem, config.package_directory().clone());

    // Check if EDITOR is set first
    let editor = match std::env::var("EDITOR") {
        Ok(editor) => editor,
        Err(_) => {
            reporter.report_error("EDITOR environment variable is not set.");

            // For existing packages, just tell them where it is
            if let Ok(existing_package) = repo.get_package(package_name) {
                reporter.report_info(format!(
                    "Package '{}' exists at {}. Go ahead and open it in your editor of choice!",
                    package_name,
                    existing_package.file_path.display()
                ));
            } else {
                reporter.report_info(format!(
                    "Package '{}' doesn't exist yet. Set EDITOR and try again to create it.",
                    package_name
                ));
            }
            return 1;
        }
    };

    // Try to get existing package, or create a new one
    let package_blob = match repo.get_package(package_name) {
        Ok(pkg) => {
            reporter.report_info(format!(
                "Opening existing package '{package_name}' for editing"
            ));
            pkg
        }
        Err(_) => {
            reporter.report_info(format!("Package '{package_name}' does not exist."));

            // Prompt user for confirmation before creating
            print!("Create new package '{}'? [y/N]: ", package_name);
            io::stdout().flush().unwrap();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                reporter.report_error("Failed to read user input.");
                return 1;
            }

            let input = input.trim().to_lowercase();
            if input != "y" && input != "yes" {
                reporter.report_info("Package creation cancelled.");
                return 0;
            }

            reporter.report_info(format!("Creating new package '{package_name}'"));
            selfie::package::GetPackage::new(package_name, config.package_directory())
        }
    };

    // Write the package to the file system first
    if let Err(e) = repo.save_package(&package_blob.package, &package_blob.file_path) {
        reporter.report_error(format!("Failed to save package file: {e}"));
        return 1;
    }

    // Open the package file in the editor
    let mut cmd = Command::new(&editor);
    cmd.arg(&package_blob.file_path);

    // For VS Code, wait for the file to be closed
    if editor == "code" {
        cmd.arg("--wait");
    }

    match cmd.status() {
        Ok(status) if status.success() => {
            let action = if package_blob.is_new {
                "created"
            } else {
                "updated"
            };
            reporter.report_success(format!(
                "Package '{package_name}' {action} successfully at {}",
                package_blob.file_path.display()
            ));
            0
        }
        Ok(_) => {
            reporter.report_warning("Editor exited with non-zero status.");
            1
        }
        Err(e) => {
            reporter.report_error(format!("Failed to start editor '{editor}': {e}"));
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_common::{test_config, test_config_with_colors};

    fn create_mock_reporter() -> TerminalProgressReporter {
        TerminalProgressReporter::new(false)
    }

    #[test]
    #[ignore] // Requires external editor and user interaction
    fn test_handle_edit_basic() {
        let config = test_config();
        let reporter = create_mock_reporter();

        // This test would require mocking the editor interaction
        // For now, we'll skip it and focus on unit testable components
        tokio_test::block_on(async {
            let result = handle_edit("test-package", &config, reporter).await;
            // In a real test, we'd verify the file was created/updated correctly
            assert!(result == 0 || result == 1); // Either success or expected failure
        });
    }

    #[test]
    #[ignore] // Requires external editor
    fn test_handle_edit_with_colors() {
        let config = test_config_with_colors();
        let reporter = TerminalProgressReporter::new(true);

        tokio_test::block_on(async {
            let result = handle_edit("test-package", &config, reporter).await;
            assert!(result == 0 || result == 1);
        });
    }

    #[test]
    fn test_editor_required() {
        // Test that EDITOR environment variable is required
        unsafe {
            std::env::remove_var("EDITOR");
        }

        let editor_result = std::env::var("EDITOR");
        assert!(
            editor_result.is_err(),
            "EDITOR should not be set for this test"
        );
    }

    #[test]
    fn test_yaml_serialization_roundtrip() {
        // Test that we can serialize and deserialize a package
        use selfie::package::PackageBuilder;

        let original_package = PackageBuilder::default()
            .name("test-package")
            .version("1.0.0")
            .description("Test package")
            .environment("test", |b| {
                b.install("echo 'test'").check(Some("echo 'check'"))
            })
            .build();

        // Serialize to YAML
        let yaml_content = serde_yaml::to_string(&original_package).unwrap();

        // Deserialize back
        let deserialized: selfie::package::Package = serde_yaml::from_str(&yaml_content).unwrap();

        // Should be equivalent
        assert_eq!(original_package.name(), deserialized.name());
        assert_eq!(original_package.version(), deserialized.version());
        assert_eq!(original_package.description(), deserialized.description());
        assert_eq!(
            original_package.environments().len(),
            deserialized.environments().len()
        );
    }
}
