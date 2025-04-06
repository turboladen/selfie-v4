pub(crate) mod config;
pub(crate) mod package;

use selfie::{config::AppConfig, progress_reporter::port::ProgressReporter};
use tracing::debug;

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
            package::handle_install(package_name, config, reporter)
        }
        PackageSubcommands::List => package::handle_list(config, reporter),
        PackageSubcommands::Info { package_name } => {
            package::handle_info(package_name, config, reporter)
        }
        PackageSubcommands::Create { package_name } => {
            package::handle_create(package_name, config, reporter)
        }
        PackageSubcommands::Validate { package_name } => {
            package::handle_validate(package_name, config, reporter)
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
        ConfigSubcommands::Validate => config::handle_validate(&original_config, reporter),
    }
}

fn report_with_style<S: ProgressReporter>(
    reporter: &S,
    param1: impl std::fmt::Display,
    param2: impl std::fmt::Display,
) {
    reporter.report(format!(
        "  {} {}",
        console::style(param1).italic().dim(),
        console::style(param2).bold()
    ));
}
