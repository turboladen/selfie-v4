//! Command-line interface definitions and argument parsing
//!
//! This module defines the CLI structure using the clap crate for argument parsing.
//! It provides a hierarchical command structure with global options and subcommands
//! for different package management operations.
//!
//! # Structure
//!
//! The CLI follows a nested command pattern:
//! - Global options (environment, verbosity, etc.)
//! - Top-level commands (package, config)
//! - Subcommands (install, check, list, etc.)
//!
//! # Examples
//!
//! ```bash
//! selfie --environment=macos package install node
//! selfie --verbose config validate
//! selfie package list
//! ```

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// Selfie - A personal package manager
///
/// Defines the top-level command-line interface including global options
/// that can be used with any subcommand. Global options override values
/// from the configuration file when provided.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct ClapCli {
    /// Override the target environment from configuration file
    ///
    /// Specifies which environment configuration to use for package operations.
    /// This overrides the environment setting in the config file.
    ///
    /// Example: --environment=macos, --environment=linux
    #[clap(long, short = 'e', global = true)]
    pub(crate) environment: Option<String>,

    /// Override the package directory from configuration file
    ///
    /// Specifies the directory where package definition files are located.
    /// This overrides the `package_directory` setting in the config file.
    ///
    /// Example: --package-directory=/path/to/packages
    #[clap(long, short = 'p', global = true)]
    pub(crate) package_directory: Option<PathBuf>,

    /// Enable verbose output for debugging and detailed information
    ///
    /// Shows additional debug information including command execution details,
    /// configuration loading process, and internal operation steps.
    #[clap(long, short = 'v', global = true, default_value_t = false)]
    pub(crate) verbose: bool,

    /// Disable colored output in terminal
    ///
    /// Forces plain text output without ANSI color codes. Useful for
    /// scripts, CI environments, or terminals that don't support colors.
    #[clap(long, global = true, default_value_t = false)]
    pub(crate) no_color: bool,

    /// The main command to execute
    #[clap(subcommand)]
    pub(crate) command: ClapCommands,
}

/// Top-level commands available in the selfie CLI
///
/// The CLI is organized into main command categories that group related
/// operations together. Each command category has its own subcommands
/// and specific options.
#[derive(Subcommand, Debug, Clone)]
pub(crate) enum ClapCommands {
    /// Package management operations
    ///
    /// Commands for installing, checking, listing, and managing packages.
    /// This is the primary interface for package operations.
    Package(PackageCommands),

    /// Configuration management operations
    ///
    /// Commands for validating and managing the selfie configuration file.
    /// These operations work with the application settings and validation.
    Config(ConfigCommands),
}

/// Package command group container
///
/// This structure holds the package-related subcommands. It serves as
/// an organizational container for all package management operations.
#[derive(Args, Debug, Clone)]
pub(crate) struct PackageCommands {
    /// The specific package operation to perform
    #[clap(subcommand)]
    pub(crate) command: PackageSubcommands,
}

/// Specific package management operations
///
/// These subcommands provide the core package management functionality
/// including installation, checking, validation, and information retrieval.
#[derive(Subcommand, Debug, Clone)]
pub(crate) enum PackageSubcommands {
    /// Install a package using its configured installation method
    ///
    /// Executes the package's installation command for the current environment.
    /// If the package is already installed (based on its check command), the
    /// installation may be skipped unless forced.
    ///
    /// Example: `selfie package install node`
    Install {
        /// Name of the package to install
        ///
        /// Must correspond to a package definition file in the package directory.
        /// The package name should match the filename (without extension).
        package_name: String,
    },

    /// Check if a package is already installed
    ///
    /// Runs the package's configured check command to determine if it's
    /// already installed in the current environment. This is useful for
    /// verification and before attempting installation.
    ///
    /// Example: `selfie package check node`
    Check {
        /// Name of the package to check for installation
        ///
        /// Must correspond to a package definition file in the package directory.
        package_name: String,
    },

    /// List all available packages in the package directory
    ///
    /// Discovers and displays all package definition files, showing basic
    /// information about each package including name, version, and description.
    ///
    /// Example: `selfie package list`
    List,

    /// Show detailed information about a package
    ///
    /// Displays comprehensive information about a package including its
    /// configuration, available environments, dependencies, and current
    /// installation status.
    ///
    /// Example: `selfie package info node`
    Info {
        /// Name of the package to get information about
        ///
        /// Must correspond to a package definition file in the package directory.
        package_name: String,
    },

    /// Create a new package definition file
    ///
    /// Creates a new package definition file with a basic template structure
    /// in the package directory. This provides a starting point for defining
    /// custom packages.
    ///
    /// Example: `selfie package create my-tool`
    Create {
        /// Name of the new package to create
        ///
        /// This will be used as the filename for the package definition.
        /// The package name should be unique within the package directory.
        package_name: String,

        /// Enable interactive mode for package creation
        ///
        /// Walks through prompts to configure package details like version,
        /// description, environments, and dependencies interactively.
        #[arg(short, long)]
        interactive: bool,
    },

    /// Edit a package definition file
    ///
    /// Opens an existing package definition file for editing, or creates a new one
    /// if it doesn't exist. Uses the editor specified in the EDITOR environment
    /// variable, with fallbacks to common editors like VS Code, vim, or nano.
    ///
    /// Example: `selfie package edit my-tool`
    Edit {
        /// Name of the package to edit or create
        ///
        /// If the package exists, it will be opened for editing.
        /// If it doesn't exist, a new template will be created and opened.
        package_name: String,
    },

    /// Remove a package definition file
    ///
    /// Permanently removes a package definition file from the package directory.
    /// This operation requires confirmation and will warn if the package is a
    /// dependency of other packages.
    ///
    /// Example: `selfie package remove my-tool`
    Remove {
        /// Name of the package to remove
        ///
        /// The package definition file will be permanently deleted from the
        /// package directory. This operation cannot be undone.
        package_name: String,
    },

    /// Validate a package definition file
    ///
    /// Performs comprehensive validation of a package definition including
    /// schema validation, environment configuration checks, and command
    /// syntax verification.
    ///
    /// Example: `selfie package validate node`
    Validate {
        /// Name of the package to validate
        ///
        /// Must correspond to a package definition file in the package directory.
        package_name: String,
    },
}

/// Configuration command group container
///
/// This structure holds the configuration-related subcommands. It serves as
/// an organizational container for all configuration management operations.
#[derive(Args, Debug, Clone)]
pub(crate) struct ConfigCommands {
    /// The specific configuration operation to perform
    #[clap(subcommand)]
    pub(crate) command: ConfigSubcommands,
}

/// Configuration management operations
///
/// These subcommands provide functionality for managing and validating
/// the selfie application configuration.
#[derive(Subcommand, Debug, Clone)]
pub(crate) enum ConfigSubcommands {
    /// Validate the selfie configuration file
    ///
    /// Performs comprehensive validation of the configuration file including
    /// schema validation, path verification, and environment setting checks.
    /// This helps ensure the configuration is valid before using it for
    /// package operations.
    ///
    /// Example: `selfie config validate`
    Validate,
}
