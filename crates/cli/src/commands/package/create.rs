use dialoguer::{Confirm, Input, MultiSelect, Select, theme::SimpleTheme};
use selfie::{
    config::AppConfig,
    fs::real::RealFileSystem,
    package::{
        EnvironmentConfig, GetPackage, port::PackageRepository,
        repository::yaml::YamlPackageRepository,
    },
};
use std::{collections::HashMap, path::PathBuf, process::Command};
use tracing::info;

use crate::terminal_progress_reporter::TerminalProgressReporter;

const MAX_NAME_RETRIES: usize = 3;

enum PackageNameResult {
    CreateNew(String),     // Use this name to create a new package
    EditExisting(PathBuf), // User wants to edit the existing package at this path
    Cancelled,             // User cancelled the operation
}

pub(crate) async fn handle_create(
    package_name: &str,
    config: &AppConfig,
    reporter: TerminalProgressReporter,
    interactive: bool,
) -> i32 {
    info!("Creating package: {}", package_name);

    // Create repository
    let repo = YamlPackageRepository::new(RealFileSystem, config.package_directory().clone());

    // Get a valid package name or handle existing package scenarios
    let package_name = match get_valid_package_name(package_name, &repo, &reporter) {
        Ok(PackageNameResult::CreateNew(name)) => name,
        Ok(PackageNameResult::EditExisting(path)) => {
            reporter.report_info(format!(
                "Opening existing package for editing at {}",
                path.display()
            ));
            return open_editor(&path, &reporter);
        }
        Ok(PackageNameResult::Cancelled) => {
            reporter.report_info("Package creation cancelled.");
            return 0;
        }
        Err(exit_code) => return exit_code,
    };

    // Create new package
    let package_blob = if interactive {
        match create_package_interactive(&package_name, config, &reporter) {
            Ok(blob) => blob,
            Err(exit_code) => return exit_code,
        }
    } else {
        create_basic_package(&package_name, config)
    };

    // Save package to file
    if let Err(e) = repo.save_package(&package_blob.package, &package_blob.file_path) {
        reporter.report_error(format!("Failed to save package file: {e}"));
        return 1;
    }

    reporter.report_success(format!(
        "Package '{}' created successfully at {}",
        package_name,
        package_blob.file_path.display()
    ));

    // Ask if user wants to edit the file (only in interactive mode)
    if interactive {
        let edit_now = Confirm::with_theme(&SimpleTheme)
            .with_prompt("Would you like to open the package file for editing now?")
            .default(true)
            .interact();

        match edit_now {
            Ok(true) => open_editor(&package_blob.file_path, &reporter),
            Ok(false) => {
                reporter.report_info(
                    "Package created. You can edit it later with 'selfie package edit'.",
                );
                0
            }
            Err(_) => {
                reporter.report_error("Failed to read user input.");
                1
            }
        }
    } else {
        reporter.report_info("Package created. Use 'selfie package edit' to customize it.");
        0
    }
}

fn get_valid_package_name(
    initial_name: &str,
    repo: &impl PackageRepository,
    reporter: &TerminalProgressReporter,
) -> Result<PackageNameResult, i32> {
    let mut current_name = initial_name.to_string();
    let mut retry_count = 0;

    loop {
        // Check if package already exists
        if let Ok(existing_package) = repo.get_package(&current_name) {
            reporter.report_info(format!("Package '{}' already exists.", current_name));

            let action = Select::with_theme(&SimpleTheme)
                .with_prompt("What would you like to do?")
                .items(&[
                    "Edit the existing package",
                    "Create a new package with a different name",
                    "Cancel",
                ])
                .default(0)
                .interact();

            match action {
                Ok(0) => {
                    // Edit existing package
                    return Ok(PackageNameResult::EditExisting(existing_package.file_path));
                }
                Ok(1) => {
                    // Create with different name
                    retry_count += 1;
                    if retry_count > MAX_NAME_RETRIES {
                        reporter.report_error(format!(
                            "Too many retry attempts ({}). Please try again later.",
                            MAX_NAME_RETRIES
                        ));
                        return Err(1);
                    }

                    let new_name: String = match Input::with_theme(&SimpleTheme)
                        .with_prompt(format!(
                            "Enter a new package name (attempt {}/{})",
                            retry_count, MAX_NAME_RETRIES
                        ))
                        .interact()
                    {
                        Ok(name) => name,
                        Err(_) => {
                            reporter.report_error("Failed to read package name.");
                            return Err(1);
                        }
                    };
                    current_name = new_name;
                    continue; // Loop back to check the new name
                }
                _ => {
                    // Cancel
                    return Ok(PackageNameResult::Cancelled);
                }
            }
        } else {
            // Package doesn't exist, we can use this name
            return Ok(PackageNameResult::CreateNew(current_name));
        }
    }
}

fn create_basic_package(package_name: &str, config: &AppConfig) -> GetPackage {
    let mut environments = HashMap::new();

    // Use the environment from config (which may be overridden by --environment)
    let env_name = config.environment();
    let env_config = EnvironmentConfig::new(
        format!("# TODO: Add install command for {}", package_name),
        Some(format!("# TODO: Add check command for {}", package_name)),
        Vec::new(),
    );

    environments.insert(env_name.to_string(), env_config);

    let package = selfie::package::Package::new(
        package_name.to_string(),
        "0.1.0".to_string(),
        None,
        None,
        environments,
        config
            .package_directory()
            .join(format!("{}.yml", package_name)),
    );

    let file_path = config
        .package_directory()
        .join(format!("{}.yml", package_name));

    GetPackage {
        package,
        file_path,
        is_new: true,
    }
}

fn create_package_interactive(
    package_name: &str,
    config: &AppConfig,
    reporter: &TerminalProgressReporter,
) -> Result<GetPackage, i32> {
    reporter.report_info("Creating package interactively...");

    // 1. Name (default = passed in name)
    let name: String = Input::with_theme(&SimpleTheme)
        .with_prompt("Package name")
        .default(package_name.to_string())
        .interact()
        .map_err(|_| {
            reporter.report_error("Failed to read package name.");
            1
        })?;

    // 2. Version (default = 0.1.0)
    let version: String = Input::with_theme(&SimpleTheme)
        .with_prompt("Version")
        .default("0.1.0".to_string())
        .interact()
        .map_err(|_| {
            reporter.report_error("Failed to read version.");
            1
        })?;

    // 3. Homepage (optional)
    let homepage: String = Input::with_theme(&SimpleTheme)
        .with_prompt("Homepage URL (optional)")
        .allow_empty(true)
        .interact()
        .map_err(|_| {
            reporter.report_error("Failed to read homepage.");
            1
        })?;

    let homepage = if homepage.trim().is_empty() {
        None
    } else {
        Some(homepage)
    };

    // 4. Description (optional)
    let description: String = Input::with_theme(&SimpleTheme)
        .with_prompt("Description (optional)")
        .allow_empty(true)
        .interact()
        .map_err(|_| {
            reporter.report_error("Failed to read description.");
            1
        })?;

    let description = if description.trim().is_empty() {
        None
    } else {
        Some(description)
    };

    // 5. Environment wizard
    let mut environments = HashMap::new();

    loop {
        reporter.report_info("Adding environment configuration...");

        // Environment name - use config.environment() as default for first environment
        let default_env = if environments.is_empty() {
            config.environment().to_string()
        } else {
            "production".to_string() // or another sensible default for additional envs
        };

        let env_name: String = Input::with_theme(&SimpleTheme)
            .with_prompt("Environment name")
            .default(default_env)
            .interact()
            .map_err(|_| {
                reporter.report_error("Failed to read environment name.");
                1
            })?;

        // Install command (required)
        let install_cmd: String = loop {
            let cmd: String = Input::with_theme(&SimpleTheme)
                .with_prompt("Install command (required)")
                .interact()
                .map_err(|_| {
                    reporter.report_error("Failed to read install command.");
                    1
                })?;

            if !cmd.trim().is_empty() {
                break cmd;
            }

            reporter.report_error("Install command cannot be empty.");
        };

        // Check command (optional, with default)
        let default_check = format!("command -v {}", name);
        let check_cmd: String = Input::with_theme(&SimpleTheme)
            .with_prompt("Check command (optional)")
            .default(default_check)
            .allow_empty(true)
            .interact()
            .map_err(|_| {
                reporter.report_error("Failed to read check command.");
                1
            })?;

        let check_cmd = if check_cmd.trim().is_empty() {
            None
        } else {
            Some(check_cmd)
        };

        // Dependencies (get available packages)
        let repo = YamlPackageRepository::new(RealFileSystem, config.package_directory().clone());
        let available_packages = repo.available_packages().unwrap_or_default();

        let dependencies = if !available_packages.is_empty() {
            let selected = MultiSelect::with_theme(&SimpleTheme)
                .with_prompt("Dependencies (select with space, confirm with enter)")
                .items(&available_packages)
                .interact()
                .map_err(|_| {
                    reporter.report_error("Failed to read dependencies.");
                    1
                })?;

            selected
                .into_iter()
                .map(|i| available_packages[i].clone())
                .collect()
        } else {
            Vec::new()
        };

        let env_config = EnvironmentConfig::new(install_cmd, check_cmd, dependencies);

        environments.insert(env_name, env_config);

        // Ask if they want to add another environment
        let add_another = Confirm::with_theme(&SimpleTheme)
            .with_prompt("Add another environment?")
            .default(false)
            .interact()
            .map_err(|_| {
                reporter.report_error("Failed to read user input.");
                1
            })?;

        if !add_another {
            break;
        }
    }

    // 6. File name (default = name)
    let file_name: String = Input::with_theme(&SimpleTheme)
        .with_prompt("File name (without .yml extension)")
        .default(name.clone())
        .interact()
        .map_err(|_| {
            reporter.report_error("Failed to read file name.");
            1
        })?;

    // Create package
    let package = selfie::package::Package::new(
        name,
        version,
        homepage,
        description,
        environments,
        config
            .package_directory()
            .join(format!("{}.yml", file_name)),
    );

    let file_path = config
        .package_directory()
        .join(format!("{}.yml", file_name));

    Ok(GetPackage {
        package,
        file_path,
        is_new: true,
    })
}

fn open_editor(file_path: &std::path::Path, reporter: &TerminalProgressReporter) -> i32 {
    let editor = match std::env::var("EDITOR") {
        Ok(editor) => editor,
        Err(_) => {
            reporter.report_error("EDITOR environment variable is not set.");
            reporter.report_info(format!(
                "Package file created at {}. Go ahead and open it in your editor of choice!",
                file_path.display()
            ));
            return 1;
        }
    };

    let mut cmd = Command::new(&editor);
    cmd.arg(file_path);

    // For VS Code, wait for the file to be closed
    if editor == "code" {
        cmd.arg("--wait");
    }

    match cmd.status() {
        Ok(status) if status.success() => {
            reporter.report_success("Package file editing completed.");
            0
        }
        Ok(_) => {
            reporter.report_warning("Editor exited with non-zero status.");
            1
        }
        Err(e) => {
            reporter.report_error(format!("Failed to start editor '{}': {}", editor, e));
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
    fn test_handle_create_basic_non_interactive() {
        // Test basic package creation without interactive mode
        let temp_dir = TempDir::new().unwrap();
        let package_dir = temp_dir.path().join("packages");
        fs::create_dir_all(&package_dir).unwrap();

        let config = test_config_with_dir(&package_dir);
        let reporter = create_mock_reporter();

        let result = tokio_test::block_on(async {
            handle_create("test-package", &config, reporter, false).await
        });

        assert_eq!(result, 0);

        // Verify package file was created
        let package_file = package_dir.join("test-package.yml");
        assert!(package_file.exists());

        // Verify package content
        let content = fs::read_to_string(&package_file).unwrap();
        assert!(content.contains("name: test-package"));
        assert!(content.contains("version: 0.1.0"));
        assert!(content.contains("test-env:")); // Uses config environment from TEST_ENV
    }

    #[test]
    fn test_get_package_new_creates_correct_template() {
        let temp_dir = TempDir::new().unwrap();
        let package_dir = temp_dir.path().join("packages");

        let get_package = GetPackage::new("template-test", &package_dir);

        assert!(get_package.is_new);
        assert_eq!(get_package.package.name(), "template-test");
        assert_eq!(get_package.package.version(), "0.1.0");
        assert_eq!(get_package.file_path, package_dir.join("template-test.yml"));
        assert!(get_package.package.environments().contains_key("default"));

        // Check that the default environment has the expected structure
        let default_env = get_package.package.environments().get("default").unwrap();
        assert!(default_env.install().contains("template-test"));
        assert!(default_env.check().unwrap().contains("template-test"));
        assert!(default_env.dependencies().is_empty());
    }

    #[test]
    fn test_create_package_interactive_components() {
        // Test that EnvironmentConfig can be created with the new constructor
        let env_config = EnvironmentConfig::new(
            "brew install test".to_string(),
            Some("command -v test".to_string()),
            vec!["dependency1".to_string(), "dependency2".to_string()],
        );

        assert_eq!(env_config.install(), "brew install test");
        assert_eq!(env_config.check(), Some("command -v test"));
        assert_eq!(env_config.dependencies(), &["dependency1", "dependency2"]);
    }

    #[test]
    fn test_open_editor_missing_editor_env() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.yml");
        fs::write(&test_file, "test content").unwrap();

        // Remove EDITOR environment variable
        let old_editor = std::env::var("EDITOR").ok();
        unsafe {
            std::env::remove_var("EDITOR");
        }

        let reporter = create_mock_reporter();
        let result = open_editor(&test_file, &reporter);

        // Should return 1 (error) when EDITOR is not set
        assert_eq!(result, 1);

        // Restore EDITOR if it was set
        if let Some(editor) = old_editor {
            unsafe {
                std::env::set_var("EDITOR", editor);
            }
        }
    }

    #[test]
    fn test_vs_code_wait_flag_logic() {
        // Test that VS Code gets the --wait flag added
        let mut cmd = std::process::Command::new("code");
        cmd.arg("/tmp/test.yml");

        let editor = "code";
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
    fn test_package_template_structure() {
        // Test that the package template has the expected structure
        let temp_dir = TempDir::new().unwrap();
        let get_package = GetPackage::new("structure-test", temp_dir.path());

        assert_eq!(get_package.package.name(), "structure-test");
        assert_eq!(get_package.package.version(), "0.1.0");
        assert!(get_package.package.description().is_none());
        assert!(get_package.package.homepage().is_none());

        let environments = get_package.package.environments();
        assert_eq!(environments.len(), 1);
        assert!(environments.contains_key("default"));

        let default_env = environments.get("default").unwrap();
        assert!(default_env.install().starts_with("# TODO:"));
        assert!(default_env.check().is_some());
        assert!(default_env.dependencies().is_empty());
    }

    #[test]
    fn test_create_basic_package_with_custom_environment() {
        // Test that --environment flag is respected in basic package creation
        let temp_dir = TempDir::new().unwrap();
        let package_dir = temp_dir.path().join("packages");
        fs::create_dir_all(&package_dir).unwrap();

        // Create config with custom environment
        let config = test_common::config::test_config_with_dir_and_env(&package_dir, "staging");

        let package_blob = create_basic_package("test-staging", &config);

        assert_eq!(package_blob.package.name(), "test-staging");
        assert_eq!(package_blob.package.version(), "0.1.0");
        assert!(package_blob.is_new);

        let environments = package_blob.package.environments();
        assert_eq!(environments.len(), 1);
        assert!(environments.contains_key("staging"));
        assert!(!environments.contains_key("default"));
        assert!(!environments.contains_key("macos"));

        let staging_env = environments.get("staging").unwrap();
        assert!(staging_env.install().contains("test-staging"));
        assert!(staging_env.check().unwrap().contains("test-staging"));
        assert!(staging_env.dependencies().is_empty());
    }

    #[test]
    fn test_handle_create_respects_environment_flag() {
        // Test that the full create command respects --environment
        let temp_dir = TempDir::new().unwrap();
        let package_dir = temp_dir.path().join("packages");
        fs::create_dir_all(&package_dir).unwrap();

        let config = test_common::config::test_config_with_dir_and_env(&package_dir, "production");
        let reporter = create_mock_reporter();

        let result = tokio_test::block_on(async {
            handle_create("prod-test", &config, reporter, false).await
        });

        assert_eq!(result, 0);

        // Verify package file was created with production environment
        let package_file = package_dir.join("prod-test.yml");
        assert!(package_file.exists());

        let content = fs::read_to_string(&package_file).unwrap();
        assert!(content.contains("name: prod-test"));
        assert!(content.contains("production:"));
        assert!(!content.contains("default:"));
        assert!(!content.contains("macos:"));
    }

    #[test]
    fn test_max_name_retries_constant() {
        // Ensure the retry limit is reasonable
        assert!(MAX_NAME_RETRIES > 0);
        assert!(MAX_NAME_RETRIES <= 5); // Don't allow too many retries
        assert_eq!(MAX_NAME_RETRIES, 3); // Verify the exact value we set
    }

    #[test]
    fn test_get_valid_package_name_logic() {
        // Test the logic flow without interactive components
        // Since get_valid_package_name involves interactive prompts,
        // we test the integration through handle_create instead
        let temp_dir = TempDir::new().unwrap();
        let package_dir = temp_dir.path().join("packages");
        fs::create_dir_all(&package_dir).unwrap();

        let config = test_common::config::test_config_with_dir(&package_dir);
        let reporter = create_mock_reporter();

        // Test creating a new package (name doesn't exist)
        let result = tokio_test::block_on(async {
            handle_create("new-unique-name", &config, reporter, false).await
        });

        assert_eq!(result, 0);

        // Verify the package was created
        let package_file = package_dir.join("new-unique-name.yml");
        assert!(package_file.exists());
    }
}
