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

pub(crate) struct ListCommand<'a> {
    config: &'a AppConfig,
    reporter: TerminalProgressReporter,
}

impl<'a> ListCommand<'a> {
    pub(crate) fn new(config: &'a AppConfig, reporter: TerminalProgressReporter) -> Self {
        Self { config, reporter }
    }
}

impl ListCommand<'_> {
    pub(crate) async fn handle_command(&self) -> i32 {
        // Create the repository and command runner
        let repo = YamlPackageRepository::new(
            RealFileSystem,
            self.config.package_directory().to_path_buf(),
        );
        let command_runner = ShellCommandRunner::new("/bin/sh", self.config.command_timeout());

        // Create the package service implementation
        let service = PackageServiceImpl::new(repo, command_runner, self.config.clone());

        // Call the service's list method to get an event stream
        match service.list().await {
            Ok(event_stream) => {
                // Process the event stream using the reusable event processor
                let processor = EventProcessor::new(self.reporter);
                processor.process_events(event_stream).await
            }
            Err(e) => {
                self.reporter
                    .report_error(format!("Failed to list packages: {}", e));
                1
            }
        }
    }
}
