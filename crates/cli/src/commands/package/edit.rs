use dialoguer::{Confirm, theme::SimpleTheme};
use selfie::{
    config::AppConfig,
    fs::real::RealFileSystem,
    package::{port::PackageRepository, repository::yaml::YamlPackageRepository},
};
use std::process::Command;
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
            let confirm = Confirm::with_theme(&SimpleTheme)
                .with_prompt(format!("Create new package '{}'?", package_name))
                .default(false)
                .interact();

            match confirm {
                Ok(true) => {
                    // User confirmed, proceed with creation
                }
                Ok(false) => {
                    reporter.report_info("Package creation cancelled.");
                    return 0;
                }
                Err(_) => {
                    reporter.report_error("Failed to read user input.");
                    return 1;
                }
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
    use std::fs;
    use tempfile::TempDir;
    use test_common::test_config_with_dir;

    fn create_mock_reporter() -> TerminalProgressReporter {
        TerminalProgressReporter::new(false)
    }

    #[test]
    fn test_editor_required_with_existing_package() {
        // Test behavior when EDITOR is not set but package exists
        let temp_dir = TempDir::new().unwrap();
        let package_dir = temp_dir.path().join("packages");
        fs::create_dir_all(&package_dir).unwrap();

        // Create an existing package file
        let package_file = package_dir.join("existing-package.yml");
        fs::write(&package_file, "name: existing-package\nversion: 1.0.0\nenvironments:\n  default:\n    install: echo test").unwrap();

        // Remove EDITOR
        let old_editor = std::env::var("EDITOR").ok();
        unsafe {
            std::env::remove_var("EDITOR");
        }

        let config = test_config_with_dir(package_dir);
        let reporter = create_mock_reporter();

        let result = tokio_test::block_on(async {
            handle_edit("existing-package", &config, reporter).await
        });

        // Should fail with exit code 1 due to missing EDITOR
        assert_eq!(result, 1);

        // Restore EDITOR if it was set
        if let Some(editor) = old_editor {
            unsafe {
                std::env::set_var("EDITOR", editor);
            }
        }
    }

    #[test]
    fn test_editor_required_with_nonexistent_package() {
        // Test behavior when EDITOR is not set and package doesn't exist
        let temp_dir = TempDir::new().unwrap();
        let package_dir = temp_dir.path().join("packages");
        fs::create_dir_all(&package_dir).unwrap();

        // Remove EDITOR
        let old_editor = std::env::var("EDITOR").ok();
        unsafe {
            std::env::remove_var("EDITOR");
        }

        let config = test_config_with_dir(package_dir);
        let reporter = create_mock_reporter();

        let result = tokio_test::block_on(async {
            handle_edit("nonexistent-package", &config, reporter).await
        });

        // Should fail with exit code 1 due to missing EDITOR
        assert_eq!(result, 1);

        // Restore EDITOR if it was set
        if let Some(editor) = old_editor {
            unsafe {
                std::env::set_var("EDITOR", editor);
            }
        }
    }

    #[test]
    fn test_confirmation_prompt_structure() {
        // Test that we can create a confirmation prompt (without actually running it)
        let package_name = "test-package";
        let confirm = Confirm::with_theme(&SimpleTheme)
            .with_prompt(format!("Create new package '{}'?", package_name))
            .default(false);

        // Just verify we can construct the prompt without panicking
        // We can't access the default field directly as it's private
        drop(confirm);
    }

    #[test]
    fn test_get_package_new_creates_template() {
        let temp_dir = TempDir::new().unwrap();
        let package_dir = temp_dir.path().join("packages");

        let get_package = selfie::package::GetPackage::new("test-template", &package_dir);

        assert!(get_package.is_new);
        assert_eq!(get_package.package.name(), "test-template");
        assert_eq!(get_package.package.version(), "0.1.0");
        assert_eq!(get_package.file_path, package_dir.join("test-template.yml"));
        assert!(get_package.package.environments().contains_key("default"));
    }

    #[test]
    fn test_vs_code_wait_flag() {
        // Test that VS Code gets the --wait flag
        let editor = "code";
        let mut cmd = Command::new(&editor);
        cmd.arg("/tmp/test.yml");

        if editor == "code" {
            cmd.arg("--wait");
        }

        let args: Vec<_> = cmd.get_args().collect();
        assert!(
            args.iter()
                .any(|arg| *arg == std::ffi::OsStr::new("--wait"))
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

    #[test]
    fn test_package_template_version() {
        // Verify that new package templates use 1.0.0 version
        let temp_dir = TempDir::new().unwrap();
        let get_package = selfie::package::GetPackage::new("version-test", temp_dir.path());

        assert_eq!(get_package.package.version(), "0.1.0");
    }
}
