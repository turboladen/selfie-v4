use std::fmt::Display;

use comfy_table::{ContentArrangement, Table, modifiers, presets};
use console::style;
use selfie::{
    commands::{ShellCommandRunner, runner::CommandRunner},
    config::AppConfig,
    fs::real::RealFileSystem,
    package::{
        port::{PackageRepoError, PackageRepository},
        repository::YamlPackageRepository,
    },
    progress_reporter::port::ProgressReporter,
};
use tracing::info;

use crate::{
    commands::report_with_style,
    tables::{PackageListTableReporter, ValidationTableReporter},
};

pub(crate) fn handle_install<R: ProgressReporter>(
    package_name: &str,
    config: &AppConfig,
    reporter: R,
) -> i32 {
    info!("Installing package: {}", package_name);

    // TODO: Implement package installation
    reporter.report_info(format!(
        "Package '{}' will be installed in: {}",
        package_name,
        config.package_directory().display()
    ));
    0
}

pub(crate) fn handle_list<R: ProgressReporter>(config: &AppConfig, reporter: R) -> i32 {
    let repo = YamlPackageRepository::new(RealFileSystem, config.package_directory());

    match repo.list_packages() {
        Ok(list_packages_output) => {
            let mut sorted_errors: Vec<_> = list_packages_output.invalid_packages().collect();
            sorted_errors.sort_by(|a, b| a.package_path().cmp(b.package_path()));

            let mut sorted_packages: Vec<_> = list_packages_output.valid_packages().collect();
            sorted_packages.sort_by(|a, b| a.name().cmp(b.name()));

            if sorted_packages.is_empty() {
                reporter.report_info("No packages found.");

                for error in sorted_errors {
                    reporter.report_error(format!("{}: {}", error.package_path().display(), error));
                }

                return 0;
            }

            let mut package_reporter = PackageListTableReporter::new();
            package_reporter.setup(vec!["Name", "Version", "Environments"]);

            for package in sorted_packages {
                let package_name = if config.use_colors() {
                    style(package.name()).magenta().bold().to_string()
                } else {
                    package.name().to_string()
                };

                let version = if config.use_colors() {
                    style(format!("v{}", package.version())).dim().to_string()
                } else {
                    format!("v{}", package.version())
                };

                package_reporter.add_row(vec![
                    package_name,
                    version,
                    package
                        .environments()
                        .keys()
                        .map(|env_name| {
                            if env_name == config.environment() {
                                let env = format!("*{env_name}");

                                if config.use_colors() {
                                    style(env).bold().green().to_string()
                                } else {
                                    env
                                }
                            } else if config.use_colors() {
                                style(env_name).dim().green().to_string()
                            } else {
                                env_name.to_string()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(",  "),
                ]);
            }

            package_reporter.print();

            for error in sorted_errors {
                reporter.report_error(format!("{}: {}", error.package_path().display(), error));
            }

            0
        }
        Err(error) => {
            reporter.report_error(error);

            1
        }
    }
}

pub(crate) async fn handle_info<R: ProgressReporter>(
    package_name: &str,
    config: &AppConfig,
    reporter: R,
) -> i32 {
    tracing::debug!("Finding package info for: {}", package_name);

    let repo = YamlPackageRepository::new(RealFileSystem, config.package_directory());
    let command_runner = ShellCommandRunner::new("/bin/sh", config.command_timeout());

    match repo.get_package(package_name) {
        Ok(package) => {
            // Create main table for package metadata
            let mut table = Table::new();

            table
                .load_preset(presets::UTF8_FULL_CONDENSED)
                .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
                .set_content_arrangement(ContentArrangement::Dynamic);

            // Helper function for formatting field names in the left column
            let format_field = |name: &str| -> String {
                if config.use_colors() {
                    style(name).bold().cyan().to_string()
                } else {
                    style(name).bold().to_string()
                }
            };

            // Helper function for formatting values in the right column
            let format_value = |value: &str| -> String {
                if config.use_colors() {
                    style(value).white().to_string()
                } else {
                    value.to_string()
                }
            };

            // Add the basic package info rows
            table.add_row(vec![format_field("Name"), format_value(package.name())]);
            table.add_row(vec![
                format_field("Version"),
                format_value(package.version()),
            ]);

            if let Some(desc) = package.description() {
                table.add_row(vec![format_field("Description"), format_value(desc)]);
            }

            if let Some(homepage) = package.homepage() {
                table.add_row(vec![
                    format_field("Homepage"),
                    if config.use_colors() {
                        style(homepage).underlined().blue().to_string()
                    } else {
                        homepage.to_string()
                    },
                ]);
            }

            // Format the environment names as a comma-separated list
            let env_names = package
                .environments()
                .keys()
                .map(|name| {
                    if name == config.environment() {
                        if config.use_colors() {
                            format!("{}", style(format!("*{}", name)).green().bold())
                        } else {
                            format!("*{}", name)
                        }
                    } else {
                        name.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");

            // Add environments row with list of environment names
            table.add_row(vec![format_field("Environments"), format_value(&env_names)]);

            // Print the main table
            println!("{table}");

            println!(); // Space between tables

            // For each environment, create a separate table with header
            for (env_name, env_config) in package.environments() {
                // Create environment details table
                let mut env_table = Table::new();
                env_table
                    .load_preset(presets::UTF8_FULL_CONDENSED)
                    .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
                    .set_content_arrangement(ContentArrangement::Dynamic);

                // Create a header for the environment table
                let env_header = if env_name == config.environment() {
                    if config.use_colors() {
                        style(format!("Environment: *{}", env_name))
                            .bold()
                            .green()
                            .to_string()
                    } else {
                        format!("Environment: *{}", env_name)
                    }
                } else {
                    if config.use_colors() {
                        style(format!("Environment: {}", env_name))
                            .bold()
                            .to_string()
                    } else {
                        format!("Environment: {}", env_name)
                    }
                };

                // Add a header row
                env_table.set_header(vec![env_header, String::new()]);

                // Format environment detail keys
                let format_env_key = |key: &str| -> String {
                    if config.use_colors() {
                        style(key).magenta().to_string()
                    } else {
                        key.to_string()
                    }
                };

                // Add installation status if this is the current environment
                if env_name == config.environment() {
                    // Only run check for current environment
                    if let Some(check_cmd) = env_config.check() {
                        // Run the check command asynchronously
                        match command_runner.execute(check_cmd).await {
                            Ok(output) => {
                                let status = if output.is_success() {
                                    if config.use_colors() {
                                        style("Installed ✓").green().bold().to_string()
                                    } else {
                                        "Installed ✓".to_string()
                                    }
                                } else {
                                    if config.use_colors() {
                                        style("Not installed ✗").yellow().to_string()
                                    } else {
                                        "Not installed ✗".to_string()
                                    }
                                };

                                env_table.add_row(vec![format_env_key("Status"), status]);
                            }
                            Err(_) => {
                                // Error executing check command
                                let status = if config.use_colors() {
                                    style("Unknown (check failed)")
                                        .yellow()
                                        .italic()
                                        .to_string()
                                } else {
                                    "Unknown (check failed)".to_string()
                                };

                                env_table.add_row(vec![format_env_key("Status"), status]);
                            }
                        }
                    } else {
                        // No check command available
                        let status = if config.use_colors() {
                            style("Unknown (no check command)")
                                .dim()
                                .italic()
                                .to_string()
                        } else {
                            "Unknown (no check command)".to_string()
                        };

                        env_table.add_row(vec![format_env_key("Status"), status]);
                    }
                }

                // Add environment detail rows
                env_table.add_row(vec![
                    format_env_key("Install"),
                    format_value(env_config.install()),
                ]);

                if let Some(check) = env_config.check() {
                    env_table.add_row(vec![format_env_key("Check"), format_value(check)]);
                }

                if !env_config.dependencies().is_empty() {
                    env_table.add_row(vec![
                        format_env_key("Dependencies"),
                        format_value(&env_config.dependencies().join(", ")),
                    ]);
                }

                // Print the environment details table
                println!("{}", env_table);
                println!(); // Add space between environment tables
            }

            0
        }
        Err(e) => {
            match e {
                PackageRepoError::PackageNotFound {
                    name,
                    packages_path,
                } => {
                    let msg = format!("Package `{name}` Not Found\n");
                    reporter.report_error(msg);

                    // Print where we looked
                    reporter.report(format!(
                        "Searched in: {}",
                        config.package_directory().display()
                    ));

                    // Try to find similar package names to suggest
                    if let Ok(repo_output) = repo.list_packages() {
                        let available_packages: Vec<&str> =
                            repo_output.valid_packages().map(|p| p.name()).collect();

                        if !available_packages.is_empty() {
                            // Add available packages information
                            if available_packages.len() <= 5 {
                                reporter.report(format!(
                                    "Available packages: {}",
                                    available_packages.join(", ")
                                ));
                            } else {
                                reporter.report(format!(
                                    "Available packages: {}, and {} more...",
                                    available_packages[..5].join(", "),
                                    available_packages.len() - 5
                                ));
                            }
                        }
                    }

                    // Add help with suggestion
                    let help_title = if config.use_colors() {
                        style("\nSuggestion:").yellow().bold().to_string()
                    } else {
                        "\nSuggestion:".to_string()
                    };

                    reporter.report(format!(
                        "{} Run 'selfie package list' to see all available packages",
                        help_title
                    ));
                }
                PackageRepoError::MultiplePackagesFound(_) => todo!(),
                PackageRepoError::ParseError {
                    source,
                    packages_path,
                } => todo!(),
                PackageRepoError::IoError(error) => todo!(),
                PackageRepoError::DirectoryNotFound(_) => todo!(),
            }

            1
        }
    }
}

pub(crate) fn handle_create<R: ProgressReporter>(
    package_name: &str,
    _config: &AppConfig,
    reporter: R,
) -> i32 {
    info!("Creating package: {}", package_name);
    // TODO: Implement package creation
    reporter.report_info(format!(
        "Creating package: {package_name} (not yet implemented)"
    ));
    0
}

pub(crate) fn handle_validate<R: ProgressReporter>(
    package_name: &str,
    config: &AppConfig,
    reporter: R,
) -> i32 {
    fn format_table_key<T: Display>(key: T, use_colors: bool) -> String {
        if use_colors {
            style(key).magenta().bold().to_string()
        } else {
            key.to_string()
        }
    }
    info!("Validating package: {}", package_name);

    reporter.report_info(format!(
        "Validating package '{}' in environment: {}",
        package_name,
        config.environment()
    ));

    let repo = YamlPackageRepository::new(RealFileSystem, config.package_directory());

    match repo.get_package(package_name) {
        Ok(package) => {
            let validation_result = package.validate(config.environment());

            if validation_result.issues().has_errors() {
                reporter.report_error("Validation failed.");

                let mut table_reporter = ValidationTableReporter::new();
                table_reporter
                    .setup(vec!["Category", "Field", "Message", "Suggestion"])
                    .add_validation_errors(&validation_result.issues().errors(), &reporter)
                    .add_validation_warnings(&validation_result.issues().warnings(), &reporter)
                    .print();
                1
            } else if validation_result.issues().has_warnings() {
                let mut table_reporter = ValidationTableReporter::new();
                table_reporter
                    .setup(vec!["Category", "Field", "Message", "Suggestion"])
                    .add_validation_warnings(&validation_result.issues().warnings(), &reporter)
                    .print();
                0
            } else {
                reporter.report_success("Package is valid.");

                report_with_style(&reporter, "name:", package.name());
                report_with_style(&reporter, "version:", package.version());
                report_with_style(
                    &reporter,
                    "homepage:",
                    package.homepage().unwrap_or_default(),
                );
                report_with_style(
                    &reporter,
                    "description:",
                    package.description().unwrap_or_default(),
                );
                report_with_style(&reporter, "environments:", "");

                for (name, env_config) in package.environments() {
                    report_with_style(&reporter, format!("- {name}"), "");

                    let mut env_table = ValidationTableReporter::new();
                    env_table.setup(vec!["Key", "Value"]);

                    let install_key = format_table_key("install", config.use_colors());
                    let check_key = format_table_key("check", config.use_colors());
                    let dependencies_key = format_table_key("dependencies", config.use_colors());

                    env_table.add_row(vec![&install_key, env_config.install()]);
                    env_table.add_row(vec![&check_key, env_config.check().unwrap_or_default()]);
                    env_table.add_row(vec![
                        &dependencies_key,
                        &env_config.dependencies().join(", "),
                    ]);
                    env_table.print();
                }

                0
            }
        }
        Err(e) => {
            reporter.report_error("Unable to validate package.");
            reporter.report_progress(format!("  {e}"));
            1
        }
    }
}
