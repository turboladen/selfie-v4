pub(crate) mod config;
pub(crate) mod package;

use comfy_table::Table;
use selfie::{
    config::AppConfig, progress_reporter::port::ProgressReporter, validation::ValidationIssue,
};
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

struct TableReporter {
    table: Table,
}

impl TableReporter {
    fn new() -> Self {
        use comfy_table::Table;

        Self {
            table: Table::new(),
        }
    }

    fn setup(&mut self, header: Vec<&'static str>) -> &mut Self {
        use comfy_table::{
            ContentArrangement,
            modifiers::{UTF8_ROUND_CORNERS, UTF8_SOLID_INNER_BORDERS},
            presets::UTF8_FULL,
        };
        self.table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .apply_modifier(UTF8_SOLID_INNER_BORDERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(header);

        self
    }

    fn add_errors(
        &mut self,
        error_issues: &[&ValidationIssue],
        reporter: &impl ProgressReporter,
    ) -> &mut Self {
        for error in error_issues {
            self.table.add_row(vec![
                reporter.format_error(error.category().to_string()),
                error.field().to_string(),
                error.message().to_string(),
                error
                    .suggestion()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
            ]);
        }

        self
    }

    fn add_warnings(
        &mut self,
        warning_issues: &[&ValidationIssue],
        reporter: &impl ProgressReporter,
    ) -> &mut Self {
        for warning in warning_issues {
            self.table.add_row(vec![
                reporter.format_warning(warning.category().to_string()),
                warning.field().to_string(),
                warning.message().to_string(),
                warning
                    .suggestion()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
            ]);
        }

        self
    }

    fn print(&self) {
        eprintln!("{}", &self.table);
    }
}
