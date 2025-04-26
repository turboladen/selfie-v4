use selfie::config::AppConfig;
use tracing::info;

use crate::terminal_progress_reporter::TerminalProgressReporter;

pub(crate) fn handle_install(
    package_name: &str,
    config: &AppConfig,
    reporter: TerminalProgressReporter,
) -> i32 {
    info!("Installing package: {}", package_name);

    // TODO: Implement package installation
    reporter.report_info(format!(
        "Package '{}' will be installed in: {}",
        package_name,
        config.package_directory().display()
    ));
    0
}
