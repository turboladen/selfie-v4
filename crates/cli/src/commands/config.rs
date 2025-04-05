use selfie::{config::AppConfig, progress_reporter::port::ProgressReporter};
use tracing::info;

use crate::commands::{TableReporter, report_with_style};

pub(crate) fn handle_validate<R: ProgressReporter>(
    original_config: &AppConfig,
    reporter: R,
) -> i32 {
    info!("Validating configuration");

    let result = original_config.validate();

    if result.issues().has_errors() {
        reporter.report_error("Validation failed.");

        let mut table_reporter = TableReporter::new();
        table_reporter
            .setup(vec!["Category", "Field", "Message", "Suggestion"])
            .add_errors(&result.issues().errors(), &reporter)
            .add_warnings(&result.issues().warnings(), &reporter)
            .print();
        1
    } else if result.issues().has_warnings() {
        let mut table_reporter = TableReporter::new();
        table_reporter
            .setup(vec!["Category", "Field", "Message", "Suggestion"])
            .add_warnings(&result.issues().warnings(), &reporter)
            .print();
        0
    } else {
        reporter.report_success("Configuration is valid.");
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

        0
    }
}
