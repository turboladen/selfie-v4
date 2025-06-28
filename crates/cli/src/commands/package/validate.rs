use selfie::{
    commands::ShellCommandRunner,
    config::AppConfig,
    fs::real::RealFileSystem,
    package::{
        repository::YamlPackageRepository,
        service::{PackageService, PackageServiceImpl},
    },
};

use crate::{
    event_processor::EventProcessor, terminal_progress_reporter::TerminalProgressReporter,
};

pub(crate) async fn handle_validate(
    package_name: &str,
    config: &AppConfig,
    reporter: TerminalProgressReporter,
) -> i32 {
    tracing::debug!("Running validate command for package: {}", package_name);

    // Create the repository and command runner
    let repo = YamlPackageRepository::new(RealFileSystem, config.package_directory().to_path_buf());
    let command_runner = ShellCommandRunner::new("/bin/sh", config.command_timeout());

    // Create the package service implementation
    let service = PackageServiceImpl::new(repo, command_runner, config.clone());

    // Call the service's validate method to get an event stream
    match service.validate(package_name, None).await {
        Ok(event_stream) => {
            // Process the event stream using the reusable event processor
            let processor = EventProcessor::new(reporter);
            processor.process_events(event_stream).await
        }
        Err(e) => {
            reporter.report_error(format!("Failed to validate package: {}", e));
            1
        }
    }
}
