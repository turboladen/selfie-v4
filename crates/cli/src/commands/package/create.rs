use selfie::config::AppConfig;
use tracing::info;

use crate::terminal_progress_reporter::TerminalProgressReporter;

pub(crate) fn handle_create(
    package_name: &str,
    _config: &AppConfig,
    reporter: TerminalProgressReporter,
) -> i32 {
    info!("Creating package: {}", package_name);
    // TODO: Implement package creation
    reporter.report_info(format!(
        "Creating package: {package_name} (not yet implemented)"
    ));
    0
}
