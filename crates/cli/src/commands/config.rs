use selfie::config::AppConfig;
use tracing::info;

use crate::{
    commands::report_with_style, tables::ValidationTableReporter,
    terminal_progress_reporter::TerminalProgressReporter,
};

pub(crate) fn handle_validate(
    original_config: &AppConfig,
    reporter: TerminalProgressReporter,
) -> i32 {
    info!("Validating configuration");

    let result = original_config.validate();

    if result.issues().has_errors() {
        reporter.report_error("Validation failed.");

        let mut table_reporter = ValidationTableReporter::new();
        table_reporter
            .setup(vec!["Category", "Field", "Message", "Suggestion"])
            .add_validation_errors(&result.issues().errors(), &reporter)
            .add_validation_warnings(&result.issues().warnings(), &reporter)
            .print();
        1
    } else if result.issues().has_warnings() {
        let mut table_reporter = ValidationTableReporter::new();
        table_reporter
            .setup(vec!["Category", "Field", "Message", "Suggestion"])
            .add_validation_warnings(&result.issues().warnings(), &reporter)
            .print();
        0
    } else {
        reporter.report_success("Configuration is valid.");
        report_with_style("environment:", original_config.environment());
        report_with_style(
            "package_directory:",
            original_config.package_directory().display(),
        );
        report_with_style(
            "command_timeout:",
            format!("{} seconds", original_config.command_timeout().as_secs()),
        );
        report_with_style(
            "max_parallel_installations:",
            original_config.max_parallel_installations().get(),
        );
        report_with_style("stop_on_error:", original_config.stop_on_error());
        report_with_style("verbose:", original_config.verbose());
        report_with_style("use_colors:", original_config.use_colors());

        0
    }
}
