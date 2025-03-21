// src/commands.rs

use anyhow::Result;
use selfie::config::AppConfig;
use tracing::{debug, info};

use crate::cli::{
    ClapCommands, ConfigCommands, ConfigSubcommands, PackageCommands, PackageSubcommands,
};

/// Primary command dispatcher that routes to the appropriate command handler
pub fn dispatch_command(command: &ClapCommands, config: &AppConfig) -> Result<()> {
    debug!("Dispatching command: {:?}", command);
    
    match command {
        ClapCommands::Package(package_cmd) => dispatch_package_command(&package_cmd.command, config),
        ClapCommands::Config(config_cmd) => dispatch_config_command(&config_cmd.command, config),
    }
}

/// Handle package management commands
fn dispatch_package_command(command: &PackageSubcommands, config: &AppConfig) -> Result<()> {
    debug!("Handling package command: {:?}", command);
    
    match command {
        PackageSubcommands::Install { package_name } => handle_package_install(package_name, config),
        PackageSubcommands::List => handle_package_list(config),
        PackageSubcommands::Info { package_name } => handle_package_info(package_name, config),
        PackageSubcommands::Create { package_name } => handle_package_create(package_name, config),
        PackageSubcommands::Validate { package_name, package_path } => {
            handle_package_validate(package_name, package_path.as_ref(), config)
        }
    }
}

/// Handle configuration management commands
fn dispatch_config_command(command: &ConfigSubcommands, config: &AppConfig) -> Result<()> {
    debug!("Handling config command: {:?}", command);
    
    match command {
        ConfigSubcommands::Validate => handle_config_validate(config),
    }
}

// Command handler implementations

fn handle_package_install(package_name: &str, config: &AppConfig) -> Result<()> {
    info!("Installing package: {}", package_name);
    // TODO: Implement package installation
    println!("Package '{}' will be installed in: {}", package_name, config.package_directory().display());
    Ok(())
}

fn handle_package_list(config: &AppConfig) -> Result<()> {
    info!("Listing packages from {}", config.package_directory().display());
    // TODO: Implement package listing
    println!("Listing packages (not yet implemented)");
    Ok(())
}

fn handle_package_info(package_name: &str, config: &AppConfig) -> Result<()> {
    info!("Getting info for package: {}", package_name);
    // TODO: Implement package info
    println!("Displaying info for package: {} (not yet implemented)", package_name);
    Ok(())
}

fn handle_package_create(package_name: &str, config: &AppConfig) -> Result<()> {
    info!("Creating package: {}", package_name);
    // TODO: Implement package creation
    println!("Creating package: {} (not yet implemented)", package_name);
    Ok(())
}

fn handle_package_validate(package_name: &str, package_path: Option<&std::path::PathBuf>, config: &AppConfig) -> Result<()> {
    info!("Validating package: {}", package_name);
    
    if let Some(path) = package_path {
        println!("Validating package '{}' at path: {}", package_name, path.display());
    } else {
        println!("Validating package '{}' in environment: {}", package_name, config.environment());
    }
    // TODO: Implement package validation
    Ok(())
}

fn handle_config_validate(config: &AppConfig) -> Result<()> {
    info!("Validating configuration");
    
    println!("Configuration validation successful:");
    println!("  Environment: {}", config.environment());
    println!("  Package Directory: {}", config.package_directory().display());
    println!("  Command Timeout: {} seconds", config.command_timeout().as_secs());
    println!("  Max Parallel Installations: {}", config.max_parallel().get());
    println!("  Stop On Error: {}", config.stop_on_error());
    println!("  Verbose Output: {}", config.verbose());
    println!("  Color Output: {}", config.use_colors());
    
    Ok(())
}
