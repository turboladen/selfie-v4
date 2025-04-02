use comfy_table::Table;
use selfie::{
    config::AppConfig,
    fs::real::RealFileSystem,
    package::{port::PackageRepository, repository::YamlPackageRepository},
    progress_reporter::port::ProgressReporter,
};
use tracing::{info};

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
        "Displaying info for package: {} (not yet implemented)",
        package_name
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
        "Creating package: {} (not yet implemented)",
        package_name
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

    let repo = YamlPackageRepository::new(RealFileSystem, config.package_directory(), &reporter);

    match repo.get_package(package_name) {
        Ok(package) => {
            let validation_result = package.validate(config.environment());

            if validation_result.has_errors() {
                reporter.report_error("Validation failed.");
                let mut table = Table::new();
                table.set_header(vec!["Category", "Field", "Message", "Suggestion"]);

                for error in validation_result.errors() {
                    table.add_row(vec![
                        reporter.format_error(error.category().to_string()),
                        error.field().to_string(),
                        error.message().to_string(),
                        error
                            .suggestion()
                            .map(ToString::to_string)
                            .unwrap_or_default(),
                    ]);
                }
                for warning in validation_result.warnings() {
                    table.add_row(vec![
                        reporter.format_warning(warning.category().to_string()),
                        warning.field().to_string(),
                        warning.message().to_string(),
                        warning
                            .suggestion()
                            .map(ToString::to_string)
                            .unwrap_or_default(),
                    ]);
                }
                eprintln!("{table}");
                1
            } else {
                reporter.report_success("Package is valid.");

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

