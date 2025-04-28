pub(crate) mod config;
pub(crate) mod package;

use package::list::ListCommand;
use selfie::config::AppConfig;
use tracing::debug;

use crate::{
    cli::{ClapCommands, ConfigSubcommands, PackageSubcommands},
    terminal_progress_reporter::TerminalProgressReporter,
};

pub(crate) trait HandleCommand {
    fn handle_command(&self) -> i32;
}

pub(crate) trait ReportError<C> {
    fn report_error(self);
}

/// Primary command dispatcher that routes to the appropriate command handler
pub(crate) async fn dispatch_command(
    command: &ClapCommands,
    config: &AppConfig,
    original_config: AppConfig,
    reporter: TerminalProgressReporter,
) -> i32 {
    debug!("Dispatching command: {:?}", command);

    match command {
        ClapCommands::Package(package_cmd) => {
            dispatch_package_command(&package_cmd.command, config, reporter).await
        }
        ClapCommands::Config(config_cmd) => {
            dispatch_config_command(&config_cmd.command, original_config, reporter)
        }
    }
}

/// Handle package management commands
async fn dispatch_package_command(
    command: &PackageSubcommands,
    config: &AppConfig,
    reporter: TerminalProgressReporter,
) -> i32 {
    debug!("Handling package command: {:?}", command);

    match command {
        PackageSubcommands::Install { package_name } => {
            package::install::handle_install(package_name, config, reporter)
        }
        PackageSubcommands::Check { package_name } => {
            package::check::handle_check(package_name, config, reporter).await
        }
        PackageSubcommands::List => ListCommand::new(config, reporter).handle_command(),
        PackageSubcommands::Info { package_name } => {
            package::info::handle_info(package_name, config, reporter).await
        }
        PackageSubcommands::Create { package_name } => {
            package::create::handle_create(package_name, config, reporter)
        }
        PackageSubcommands::Validate { package_name } => {
            package::validate::handle_validate(package_name, config, reporter)
        }
    }
}

/// Handle configuration management commands
fn dispatch_config_command(
    command: &ConfigSubcommands,
    original_config: AppConfig,
    reporter: TerminalProgressReporter,
) -> i32 {
    debug!("Handling config command: {:?}", command);

    match command {
        ConfigSubcommands::Validate => config::handle_validate(&original_config, reporter),
    }
}

fn report_with_style(param1: impl std::fmt::Display, param2: impl std::fmt::Display) {
    TerminalProgressReporter::report(
        2,
        format!(
            "{} {}",
            console::style(param1).italic().dim(),
            console::style(param2).bold()
        ),
    );
}
