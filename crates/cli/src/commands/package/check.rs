use std::{borrow::Cow, pin::Pin};

use futures::{Stream, StreamExt};
use selfie::{
    commands::ShellCommandRunner,
    config::AppConfig,
    fs::real::RealFileSystem,
    package::{
        event::{EventStream, PackageEvent, metadata::CheckMetadata},
        repository::YamlPackageRepository,
        service::{PackageService, PackageServiceImpl},
    },
};

use crate::terminal_progress_reporter::TerminalProgressReporter;

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

    // Process the event stream and display progress in the terminal
    process_check_event_stream(event_stream, reporter).await
}

async fn process_check_event_stream(
    mut event_stream: EventStream<CheckMetadata, Cow<'static, str>, Cow<'static, str>>,
    reporter: TerminalProgressReporter,
) -> i32 {
    let mut exit_code = 0;

    // Process each event as it comes in from the stream
    while let Some(event) = event_stream.next().await {
        match event {
            PackageEvent::Started { metadata } => {
                reporter.report_info(format!(
                    "Checking package '{}' in environment '{}'",
                    metadata.command_metadata().package_name(),
                    metadata.command_metadata().environment()
                ));
            }

            PackageEvent::Progress { message, .. } => {
                reporter.report_progress(message);
            }

            PackageEvent::Info { message, .. } => match message {
                selfie::package::event::ConsoleOutput::Stdout(msg) => {
                    println!("{}", msg);
                }
                selfie::package::event::ConsoleOutput::Stderr(msg) => {
                    eprintln!("{}", msg);
                }
            },

            PackageEvent::Trace { message, .. } => {
                tracing::trace!("{}", message);
            }

            PackageEvent::Debug { message, .. } => {
                tracing::debug!("{}", message);
            }

            PackageEvent::Warning { message, .. } => {
                reporter.report_warning(message);
                // Warnings don't necessarily mean failure
            }

            PackageEvent::Error { message, error, .. } => {
                reporter.report_error(format!("{}: {}", message, error));
                exit_code = 1; // Set failure exit code
            }

            PackageEvent::Completed {
                result: message, ..
            } => {
                match message {
                    Ok(msg) => {
                        reporter.report_success(msg);
                        // Success message, but exit code might have been set earlier
                    }
                    Err(err) => {
                        reporter.report_error(err);
                        exit_code = 1;
                    }
                }
            }

            PackageEvent::Canceled { reason, .. } => {
                reporter.report_warning(format!("Operation canceled: {}", reason));
                exit_code = 1;
            }
        }
    }

    exit_code
}
