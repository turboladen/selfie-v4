use selfie::{config::AppConfig, progress_reporter::port::ProgressReporter};
use tracing::info;

use crate::commands::TableReporter;

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
        reporter.report_success("Package is valid.");

        0
    }
}
