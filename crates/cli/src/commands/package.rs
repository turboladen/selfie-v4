pub(crate) mod check;
pub(crate) mod create;
pub(crate) mod info;
pub(crate) mod install;
pub(crate) mod list;
pub(crate) mod validate;

use std::path::Path;

use crate::terminal_progress_reporter::TerminalProgressReporter;

pub(crate) fn handle_directory_not_found(path: &Path, reporter: TerminalProgressReporter) {
    reporter.report_error("✗ Package Directory Not Found");

    reporter.report_info("The package directory does not exist:");
    reporter.report_info(format!("{}", path.display()));

    reporter
        .report_suggestion("Create the directory or configure a different package directory path");
}
