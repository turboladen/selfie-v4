// src/cli.rs
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// Selfie - A personal package manager
///
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct ClapCli {
    /// Override the environment from config
    ///
    #[clap(long, short = 'e', global = true)]
    pub(crate) environment: Option<String>,

    /// Override the package directory from config
    ///
    #[clap(long, short = 'p', global = true)]
    pub(crate) package_directory: Option<PathBuf>,

    /// Show detailed output
    ///
    #[clap(long, short = 'v', global = true, default_value_t = false)]
    pub(crate) verbose: bool,

    /// Enable colored output
    ///
    #[clap(long, global = true, default_value_t = false)]
    pub(crate) no_color: bool,

    /// Subcommand to execute
    ///
    #[clap(subcommand)]
    pub(crate) command: ClapCommands,
}

// Clap-specific command structure definitions here...
#[derive(Subcommand, Debug, Clone)]
pub(crate) enum ClapCommands {
    /// Selfie: package management commands
    ///
    Package(PackageCommands),

    /// Selfie: configuration management commands
    ///
    Config(ConfigCommands),
}

#[derive(Args, Debug, Clone)]
pub(crate) struct PackageCommands {
    #[clap(subcommand)]
    pub(crate) command: PackageSubcommands,
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum PackageSubcommands {
    /// Install a package
    Install {
        /// Name of the package to install
        package_name: String,
    },

    /// List available packages
    List,

    /// Show information about a package
    Info {
        /// Name of the package to get information about
        package_name: String,
    },

    /// Create a new package
    Create {
        /// Name of the package to create
        package_name: String,
    },

    /// Validate a package
    Validate {
        /// Name of the package to validate
        package_name: String,

        /// Package file path (optional)
        #[clap(long)]
        package_path: Option<PathBuf>,
    },
}

#[derive(Args, Debug, Clone)]
pub(crate) struct ConfigCommands {
    #[clap(subcommand)]
    pub(crate) command: ConfigSubcommands,
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum ConfigSubcommands {
    /// Validate the selfie configuration
    Validate,
}
