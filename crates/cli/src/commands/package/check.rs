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

pub(crate) async fn handle_check(
    package_name: &str,
    config: &AppConfig,
    reporter: TerminalProgressReporter,
) -> i32 {
    tracing::debug!("Running check command for package: {}", package_name);

    // Create the repository and command runner
    let repo = YamlPackageRepository::new(RealFileSystem, config.package_directory().to_path_buf());
    let command_runner = ShellCommandRunner::new("/bin/sh", config.command_timeout());

    // Create the package service implementation with our repository and command runner
    let service = PackageServiceImpl::new(repo, command_runner, config.clone());

    // Call the service's check method to get an event stream
    let event_stream = service.check(package_name).await;

    // Process the event stream using the reusable event processor with custom handling
    let processor = EventProcessor::new(reporter);

    // Example of custom event handling for Progress events
    processor
        .process_events_with_handler(event_stream, |event, reporter| {
            match event {
                // Custom handling for progress events - show percentage
                selfie::package::event::PackageEvent::Progress {
                    percent_complete,
                    step,
                    total_steps,
                    message,
                    ..
                } => {
                    reporter.report_progress(format!(
                        "[{:.0}%] Step {}/{}: {}",
                        percent_complete * 100.0,
                        step,
                        total_steps,
                        message
                    ));
                    Some(true) // Continue processing
                }
                // Let default handler handle all other events
                _ => None,
            }
        })
        .await
}
