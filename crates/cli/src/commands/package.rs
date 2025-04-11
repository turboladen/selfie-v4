use std::fmt::Display;

use console::style;
use selfie::{
    config::AppConfig,
    fs::real::RealFileSystem,
    package::{port::PackageRepository, repository::YamlPackageRepository},
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

pub(crate) fn handle_info<R: ProgressReporter>(
    package_name: &str,
    _config: &AppConfig,
    reporter: R,
) -> i32 {
    info!("Getting info for package: {}", package_name);
    // TODO: Implement package info
    reporter.report_info(format!(
        "Displaying info for package: {package_name} (not yet implemented)"
    ));
    0
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
