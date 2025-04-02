use console::style;
use selfie::{config::AppConfig, progress_reporter::port::ProgressReporter};
use tracing::info;

pub(crate) fn handle_validate<R: ProgressReporter>(
    original_config: &AppConfig,
    reporter: R,
) -> i32 {
    fn report_with_style<S: ProgressReporter>(
        reporter: &S,
        param1: impl std::fmt::Display,
        param2: impl std::fmt::Display,
    ) {
        reporter.report(format!(
            "  {} {}",
            style(param1).italic().dim(),
            style(param2).bold()
        ));
    }
    info!("Validating configuration");

    match original_config.validate(|msg| reporter.report_info(msg)) {
        Ok(_) => {
            reporter.report_success("Configuration validation successful.");
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
        }
        Err(_) => todo!(),
    }

    0
}
