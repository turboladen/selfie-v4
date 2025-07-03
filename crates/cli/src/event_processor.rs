//! Event processing utilities for CLI commands
//!
//! This module provides a reusable event processor that can handle package events
//! from the selfie library and present them consistently across different CLI commands.
//!
//! # Usage
//!
//! ## Event Processing with Custom Handlers
//!
//! All commands use `process_events_with_handler` which allows custom handling
//! of specific events while providing default behavior for standard events:
//!
//! ```rust,ignore
//! async fn handle_command_with_custom_progress(
//!     reporter: TerminalProgressReporter,
//!     event_stream: EventStream
//! ) -> i32 {
//!     let processor = EventProcessor::new(reporter);
//!
//!     processor.process_events_with_handler(event_stream, |event, reporter| {
//!         match event {
//!             PackageEvent::Progress { percent_complete, step, total_steps, message, .. } => {
//!                 // Custom progress display with percentage
//!                 reporter.report_progress(format!(
//!                     "[{:.0}%] Step {}/{}: {}",
//!                     percent_complete * 100.0, step, total_steps, message
//!                 ));
//!                 Some(true) // Continue processing
//!             }
//!             PackageEvent::Warning { message, .. } => {
//!                 // Treat warnings as errors in this command
//!                 reporter.report_error(message);
//!                 Some(false) // Stop processing
//!             }
//!             _ => None, // Use default handling for other events
//!         }
//!     }).await
//! }
//! ```
//!
//! The custom handler should return:
//! - `Some(true)` to continue processing after handling the event
//! - `Some(false)` to stop processing after handling the event
//! - `None` to use the default handling for the event

use futures::StreamExt;
use selfie::package::{
    event::{ConsoleOutput, EventStream, OperationResult, PackageEvent, error::StreamedError},
    port::{PackageListError, PackageRepoError},
};

use crate::{
    commands::package::handle_directory_not_found,
    terminal_progress_reporter::TerminalProgressReporter,
};

/// A reusable event processor for handling package operation events
///
/// This processor standardizes how events from the selfie library are handled
/// and displayed in the CLI, reducing boilerplate across different commands.
#[derive(Debug)]
pub struct EventProcessor {
    reporter: TerminalProgressReporter,
}

impl EventProcessor {
    /// Create a new event processor with the given reporter
    pub fn new(reporter: TerminalProgressReporter) -> Self {
        Self { reporter }
    }

    /// Process events from the stream with a custom event handler
    ///
    /// This allows commands to provide custom handling for specific event types
    /// while still getting the default behavior for standard events.
    pub async fn process_events_with_handler<F>(
        self,
        mut stream: EventStream,
        mut custom_handler: F,
    ) -> i32
    where
        F: FnMut(&PackageEvent, &TerminalProgressReporter) -> Option<bool>,
    {
        let mut exit_code = 0;

        while let Some(event) = stream.next().await {
            // Try custom handler first
            if let Some(should_continue) = custom_handler(&event, &self.reporter) {
                if !should_continue {
                    break;
                }
                continue;
            }

            // Fall back to default handling
            if self.handle_event(event, &mut exit_code) {
                break;
            }
        }

        exit_code
    }

    /// Handle a single event and update the exit code as needed
    ///
    /// Returns true if processing should stop (early termination)
    fn handle_event(&self, event: PackageEvent, exit_code: &mut i32) -> bool {
        match event {
            PackageEvent::Started { operation_info } => {
                self.reporter.report_info(format!(
                    "{} package '{}' in environment '{}'",
                    operation_info.operation_type.to_string().to_title_case(),
                    operation_info.package_name,
                    operation_info.environment
                ));
            }

            PackageEvent::Progress { message, .. } => {
                self.reporter.report_progress(message);
            }

            PackageEvent::Info { output, .. } => {
                self.handle_console_output(output);
            }

            PackageEvent::Trace { message, .. } => {
                tracing::trace!("{}", message);
            }

            PackageEvent::Debug { message, .. } => {
                tracing::debug!("{}", message);
            }

            PackageEvent::Warning { message, .. } => {
                self.reporter.report_warning(message);
                // Warnings don't set failure exit code by default
            }

            PackageEvent::Error { message, error, .. } => {
                // Check for specific error types that need special handling
                match &error {
                    StreamedError::PackageRepoError(PackageRepoError::PackageListError(
                        PackageListError::PackageDirectoryNotFound(path),
                    )) => {
                        handle_directory_not_found(path, self.reporter);
                    }
                    _ => {
                        self.reporter.report_error(format!("{message}: {error}"));
                    }
                }
                *exit_code = 1;
            }

            PackageEvent::Completed { result, .. } => match result {
                OperationResult::Success(msg) => {
                    self.reporter.report_success(msg);
                }
                OperationResult::Failure(err) => {
                    self.reporter.report_error(err);
                    *exit_code = 1;
                }
            },

            PackageEvent::Canceled { reason, .. } => {
                self.reporter
                    .report_warning(format!("Operation canceled: {reason}"));
                *exit_code = 1;
                return true; // Stop processing after cancellation
            }

            PackageEvent::PackageInfoLoaded { .. } => {
                // These structured events are handled by command-specific handlers
                // If no custom handler processed them, just continue
            }

            PackageEvent::EnvironmentStatusChecked { .. } => {
                // These structured events are handled by command-specific handlers
                // If no custom handler processed them, just continue
            }

            PackageEvent::PackageListLoaded { .. } => {
                // These structured events are handled by command-specific handlers
                // If no custom handler processed them, just continue
            }

            PackageEvent::CheckResultCompleted { .. } => {
                // These structured events are handled by command-specific handlers
                // If no custom handler processed them, just continue
            }

            PackageEvent::ValidationResultCompleted { .. } => {
                // These structured events are handled by command-specific handlers
                // If no custom handler processed them, just continue
            }
        }

        false // Continue processing
    }

    /// Handle console output appropriately
    fn handle_console_output(&self, output: ConsoleOutput) {
        match output {
            ConsoleOutput::Stdout(msg) => {
                println!("{msg}");
            }
            ConsoleOutput::Stderr(msg) => {
                eprintln!("{msg}");
            }
        }
    }
}

/// Extension trait to add title case conversion to strings
trait ToTitleCase {
    fn to_title_case(&self) -> String;
}

impl ToTitleCase for str {
    fn to_title_case(&self) -> String {
        let mut chars = self.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => {
                first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;

    #[test]
    fn test_to_title_case() {
        assert_eq!("check".to_title_case(), "Check");
        assert_eq!("install".to_title_case(), "Install");
        assert_eq!("VALIDATE".to_title_case(), "Validate");
        assert_eq!("".to_title_case(), "");
    }

    #[test]
    fn test_event_processor_creation() {
        let reporter = TerminalProgressReporter::new(false);
        let processor = EventProcessor::new(reporter);

        // Just verify it can be created
        assert!(std::mem::size_of_val(&processor) > 0);
    }

    #[tokio::test]
    async fn test_process_empty_stream() {
        let reporter = TerminalProgressReporter::new(false);
        let processor = EventProcessor::new(reporter);

        let events: Vec<PackageEvent> = vec![];
        let event_stream = Box::pin(stream::iter(events));
        let exit_code = processor
            .process_events_with_handler(event_stream, |_event, _reporter| None)
            .await;

        // Empty stream should return success
        assert_eq!(exit_code, 0);
    }

    #[tokio::test]
    async fn test_custom_handler_behavior() {
        let reporter = TerminalProgressReporter::new(false);
        let processor = EventProcessor::new(reporter);

        let events: Vec<PackageEvent> = vec![];
        let event_stream = Box::pin(stream::iter(events));

        // Test that custom handler gets called with None for empty stream
        let mut handler_called = false;
        let exit_code = processor
            .process_events_with_handler(event_stream, |_event, _reporter| {
                handler_called = true;
                Some(true)
            })
            .await;

        assert_eq!(exit_code, 0);
        // Handler should not be called for empty stream
        assert!(!handler_called);
    }

    #[tokio::test]
    async fn test_integration_with_actual_service() {
        use selfie::{
            commands::ShellCommandRunner,
            fs::real::RealFileSystem,
            package::{
                repository::YamlPackageRepository,
                service::{PackageService, PackageServiceImpl},
            },
        };

        // Create a minimal config for testing
        let config = selfie::config::AppConfigBuilder::default()
            .environment("test")
            .package_directory("/tmp/nonexistent")
            .command_timeout_unchecked(1)
            .use_colors(false)
            .build();

        let repo = YamlPackageRepository::new(RealFileSystem, config.package_directory().clone());
        let command_runner = ShellCommandRunner::new("/bin/sh", config.command_timeout());
        let service = PackageServiceImpl::new(repo, command_runner, config);

        let reporter = TerminalProgressReporter::new(false);
        let processor = EventProcessor::new(reporter);

        // Test with a nonexistent package - should get events but ultimately fail
        let event_stream = service.check("nonexistent-test-package").await;
        let exit_code = processor
            .process_events_with_handler(event_stream, |_event, _reporter| None)
            .await;

        // Should return error exit code since package doesn't exist
        assert_eq!(exit_code, 1);
    }

    #[tokio::test]
    async fn test_custom_handler_with_real_events() {
        use selfie::{
            commands::ShellCommandRunner,
            fs::real::RealFileSystem,
            package::{
                repository::YamlPackageRepository,
                service::{PackageService, PackageServiceImpl},
            },
        };

        // Create a minimal config for testing
        let config = selfie::config::AppConfigBuilder::default()
            .environment("test")
            .package_directory("/tmp/nonexistent")
            .command_timeout_unchecked(1)
            .use_colors(false)
            .build();

        let repo = YamlPackageRepository::new(RealFileSystem, config.package_directory().clone());
        let command_runner = ShellCommandRunner::new("/bin/sh", config.command_timeout());
        let service = PackageServiceImpl::new(repo, command_runner, config);

        let reporter = TerminalProgressReporter::new(false);
        let processor = EventProcessor::new(reporter);

        let mut progress_events_seen = 0;
        let mut started_events_seen = 0;
        let mut completed_events_seen = 0;

        let event_stream = service.check("nonexistent-test-package").await;
        let exit_code = processor
            .process_events_with_handler(event_stream, |event, _reporter| {
                match event {
                    PackageEvent::Started { .. } => {
                        started_events_seen += 1;
                        None // Use default handling
                    }
                    PackageEvent::Progress { .. } => {
                        progress_events_seen += 1;
                        None // Use default handling
                    }
                    PackageEvent::Completed { .. } => {
                        completed_events_seen += 1;
                        None // Use default handling
                    }
                    _ => None, // Use default handling for all other events
                }
            })
            .await;

        // Should have seen some events even though it failed
        assert_eq!(started_events_seen, 1);
        assert!(progress_events_seen > 0);
        assert_eq!(completed_events_seen, 1);
        assert_eq!(exit_code, 1);
    }

    #[tokio::test]
    async fn test_custom_handler_early_termination() {
        use selfie::{
            commands::ShellCommandRunner,
            fs::real::RealFileSystem,
            package::{
                repository::YamlPackageRepository,
                service::{PackageService, PackageServiceImpl},
            },
        };

        // Create a minimal config for testing
        let config = selfie::config::AppConfigBuilder::default()
            .environment("test")
            .package_directory("/tmp/nonexistent")
            .command_timeout_unchecked(1)
            .use_colors(false)
            .build();

        let repo = YamlPackageRepository::new(RealFileSystem, config.package_directory().clone());
        let command_runner = ShellCommandRunner::new("/bin/sh", config.command_timeout());
        let service = PackageServiceImpl::new(repo, command_runner, config);

        let reporter = TerminalProgressReporter::new(false);
        let processor = EventProcessor::new(reporter);

        let mut events_after_started = 0;

        let event_stream = service.check("nonexistent-test-package").await;
        let exit_code = processor
            .process_events_with_handler(event_stream, |event, _reporter| {
                if let PackageEvent::Started { .. } = event {
                    Some(false) // Stop processing immediately after started
                } else {
                    events_after_started += 1;
                    None // Should not reach here
                }
            })
            .await;

        // Should have stopped processing early, so no events after Started
        assert_eq!(events_after_started, 0);
        assert_eq!(exit_code, 0); // Exit code should still be 0 since we stopped early
    }

    #[tokio::test]
    async fn test_title_case_with_different_operations() {
        // Test the ToTitleCase trait with operation names that might come from the system
        assert_eq!("package_check".to_title_case(), "Package_check");
        assert_eq!("package_install".to_title_case(), "Package_install");
        assert_eq!("package_validate".to_title_case(), "Package_validate");
        assert_eq!("PACKAGE_LIST".to_title_case(), "Package_list");
    }
}
