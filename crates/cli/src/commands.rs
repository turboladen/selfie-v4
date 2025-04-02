use comfy_table::Table;
use console::style;
use selfie::{
    config::AppConfig,
    fs::real::RealFileSystem,
    package::{port::PackageRepository, repository::YamlPackageRepository},
    progress_reporter::port::ProgressReporter,
};
use tracing::{debug, info};

use crate::cli::{ClapCommands, ConfigSubcommands, PackageSubcommands};

/// Primary command dispatcher that routes to the appropriate command handler
pub fn dispatch_command<R: ProgressReporter>(
    command: &ClapCommands,
    config: &AppConfig,
    original_config: AppConfig,
    reporter: R,
) -> i32 {
    debug!("Dispatching command: {:?}", command);

    match command {
        ClapCommands::Package(package_cmd) => {
            dispatch_package_command(&package_cmd.command, config, reporter)
        }
        ClapCommands::Config(config_cmd) => {
            dispatch_config_command(&config_cmd.command, original_config, reporter)
        }
    }
}

/// Handle package management commands
fn dispatch_package_command<R: ProgressReporter>(
    command: &PackageSubcommands,
    config: &AppConfig,
    reporter: R,
) -> i32 {
    debug!("Handling package command: {:?}", command);

    match command {
        PackageSubcommands::Install { package_name } => {
            handle_package_install(package_name, config, reporter)
        }
        PackageSubcommands::List => handle_package_list(config, reporter),
        PackageSubcommands::Info { package_name } => {
            handle_package_info(package_name, config, reporter)
        }
        PackageSubcommands::Create { package_name } => {
            handle_package_create(package_name, config, reporter)
        }
        PackageSubcommands::Validate { package_name } => {
            handle_package_validate(package_name, config, reporter)
        }
    }
}

/// Handle configuration management commands
fn dispatch_config_command<R: ProgressReporter>(
    command: &ConfigSubcommands,
    original_config: AppConfig,
    reporter: R,
) -> i32 {
    debug!("Handling config command: {:?}", command);

    match command {
        ConfigSubcommands::Validate => handle_config_validate(&original_config, reporter),
    }
}

// Command handler implementations

fn handle_package_install<R: ProgressReporter>(
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

fn handle_package_list<R: ProgressReporter>(config: &AppConfig, reporter: R) -> i32 {
    info!(
        "Listing packages from {}",
        config.package_directory().display()
    );
    // TODO: Implement package listing
    reporter.report_info("Listing packages (not yet implemented)");
    0
}

fn handle_package_info<R: ProgressReporter>(
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

fn handle_package_create<R: ProgressReporter>(
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

fn handle_package_validate<R: ProgressReporter>(
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

fn handle_config_validate<R: ProgressReporter>(original_config: &AppConfig, reporter: R) -> i32 {
    fn report_with_style<S: ProgressReporter>(
        reporter: &S,
        param1: impl std::fmt::Display,
        param2: impl std::fmt::Display,
    ) {
        reporter.report(format!(
            "  {} {}",
            style(param1).italic().dim(),
            style(param2).bold()
        ));
    }
    info!("Validating configuration");

    match original_config.validate(|msg| reporter.report_info(msg)) {
        Ok(_) => {
            reporter.report_success("Configuration validation successful.");
            report_with_style(&reporter, "environment:", original_config.environment());
            report_with_style(
                &reporter,
                "package_directory:",
                original_config.package_directory().display(),
            );
            report_with_style(
                &reporter,
                "command_timeout:",
                format!("{} seconds", original_config.command_timeout().as_secs()),
            );
            report_with_style(
                &reporter,
                "max_parallel_installations:",
                original_config.max_parallel_installations().get(),
            );
            report_with_style(&reporter, "stop_on_error:", original_config.stop_on_error());
            report_with_style(&reporter, "verbose:", original_config.verbose());
            report_with_style(&reporter, "use_colors:", original_config.use_colors());
        }
        Err(_) => todo!(),
    }

    0
}
