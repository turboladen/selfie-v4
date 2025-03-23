// src/commands.rs

use anyhow::Result;
use selfie::{
    config::AppConfig,
    progress_reporter::{port::ProgressReporter, terminal::TerminalProgressReporter},
};
use tracing::{debug, info};

use crate::cli::{
    ClapCommands, ConfigCommands, ConfigSubcommands, PackageCommands, PackageSubcommands,
};

/// Primary command dispatcher that routes to the appropriate command handler
pub fn dispatch_command(
    command: &ClapCommands,
    config: &AppConfig,
    reporter: TerminalProgressReporter,
) -> Result<()> {
    debug!("Dispatching command: {:?}", command);

    match command {
        ClapCommands::Package(package_cmd) => {
            dispatch_package_command(&package_cmd.command, config, reporter)
        }
        ClapCommands::Config(config_cmd) => {
            dispatch_config_command(&config_cmd.command, config, reporter)
        }
    }
}

/// Handle package management commands
fn dispatch_package_command(
    command: &PackageSubcommands,
    config: &AppConfig,
    reporter: TerminalProgressReporter,
) -> Result<()> {
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
        PackageSubcommands::Validate {
            package_name,
            package_path,
        } => handle_package_validate(package_name, package_path.as_ref(), config, reporter),
    }
}

/// Handle configuration management commands
fn dispatch_config_command(
    command: &ConfigSubcommands,
    config: &AppConfig,
    reporter: TerminalProgressReporter,
) -> Result<()> {
    debug!("Handling config command: {:?}", command);

    match command {
        ConfigSubcommands::Validate => handle_config_validate(config, reporter),
    }
}

// Command handler implementations

fn handle_package_install(
    package_name: &str,
    config: &AppConfig,
    reporter: TerminalProgressReporter,
) -> Result<()> {
    info!("Installing package: {}", package_name);

    // TODO: Implement package installation
    reporter.report_info(format!(
        "Package '{}' will be installed in: {}",
        package_name,
        config.package_directory().display()
    ));
    Ok(())
}

fn handle_package_list(config: &AppConfig, reporter: TerminalProgressReporter) -> Result<()> {
    info!(
        "Listing packages from {}",
        config.package_directory().display()
    );
    // TODO: Implement package listing
    reporter.report_info("Listing packages (not yet implemented)");
    Ok(())
}

fn handle_package_info(
    package_name: &str,
    _config: &AppConfig,
    reporter: TerminalProgressReporter,
) -> Result<()> {
    info!("Getting info for package: {}", package_name);
    // TODO: Implement package info
    reporter.report_info(format!(
        "Displaying info for package: {} (not yet implemented)",
        package_name
    ));
    Ok(())
}

fn handle_package_create(
    package_name: &str,
    _config: &AppConfig,
    reporter: TerminalProgressReporter,
) -> Result<()> {
    info!("Creating package: {}", package_name);
    // TODO: Implement package creation
    reporter.report_info(format!(
        "Creating package: {} (not yet implemented)",
        package_name
    ));
    Ok(())
}

fn handle_package_validate(
    package_name: &str,
    package_path: Option<&std::path::PathBuf>,
    config: &AppConfig,
    reporter: TerminalProgressReporter,
) -> Result<()> {
    info!("Validating package: {}", package_name);

    if let Some(path) = package_path {
        reporter.report_info(format!(
            "Validating package '{}' at path: {}",
            package_name,
            path.display()
        ));
    } else {
        reporter.report_info(format!(
            "Validating package '{}' in environment: {}",
            package_name,
            config.environment()
        ));
    }
    // TODO: Implement package validation
    Ok(())
}

fn handle_config_validate(config: &AppConfig, reporter: TerminalProgressReporter) -> Result<()> {
    info!("Validating configuration");

    reporter.report_success("Configuration validation successful:");
    reporter.report_info(format!("  Environment: {}", config.environment()));
    reporter.report_info(format!(
        "  Package Directory: {}",
        config.package_directory().display()
    ));
    reporter.report_info(format!(
        "  Command Timeout: {} seconds",
        config.command_timeout().as_secs()
    ));
    reporter.report_info(format!(
        "  Max Parallel Installations: {}",
        config.max_parallel().get()
    ));
    reporter.report_info(format!("  Stop On Error: {}", config.stop_on_error()));
    reporter.report_info(format!("  Verbose Output: {}", config.verbose()));
    reporter.report_info(format!("  Color Output: {}", config.use_colors()));

    Ok(())
}
