use selfie::{
    config::AppConfig,
    fs::real::RealFileSystem,
    package::{port::PackageRepository, repository::YamlPackageRepository},
    progress_reporter::port::ProgressReporter,
};
use tracing::info;

use crate::commands::{TableReporter, report_with_style};

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
    info!(
        "Listing packages from {}",
        config.package_directory().display()
    );
    // TODO: Implement package listing
    reporter.report_info("Listing packages (not yet implemented)");
    0
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

                let mut table_reporter = TableReporter::new();
                table_reporter
                    .setup(vec!["Category", "Field", "Message", "Suggestion"])
                    .add_errors(&validation_result.issues().errors(), &reporter)
                    .add_warnings(&validation_result.issues().warnings(), &reporter)
                    .print();
                1
            } else if validation_result.issues().has_warnings() {
                let mut table_reporter = TableReporter::new();
                table_reporter
                    .setup(vec!["Category", "Field", "Message", "Suggestion"])
                    .add_warnings(&validation_result.issues().warnings(), &reporter)
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

                for (name, config) in package.environments() {
                    report_with_style(&reporter, format!("- {name}"), "");

                    let mut env = TableReporter::new();
                    env.setup(vec!["key", "value"]);
                    env.add_row(vec!["install", config.install()]);
                    env.add_row(vec!["check", config.check().unwrap_or_default()]);
                    env.add_row(vec!["dependencies", &config.dependencies().join(", ")]);
                    env.print();
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
