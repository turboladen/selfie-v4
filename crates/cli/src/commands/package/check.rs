use selfie::{
    commands::ShellCommandRunner,
    config::AppConfig,
    fs::real::RealFileSystem,
    package::{
        event::{CheckResult, CheckResultData, PackageEvent},
        repository::YamlPackageRepository,
        service::{PackageService, PackageServiceImpl},
    },
};

use crate::{
    event_processor::EventProcessor, formatters::format_key,
    terminal_progress_reporter::TerminalProgressReporter,
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

    // Process the event stream with custom handling for structured data
    let processor = EventProcessor::new(reporter);
    processor
        .process_events_with_handler(event_stream, |event, _reporter| {
            handle_check_event(event, config)
        })
        .await
}

fn handle_check_event(event: &PackageEvent, config: &AppConfig) -> Option<bool> {
    match event {
        PackageEvent::CheckResultCompleted { check_result, .. } => {
            display_check_result_card(check_result, config);
            Some(true) // Continue processing
        }
        PackageEvent::Progress {
            percent_complete,
            step,
            total_steps,
            message,
            ..
        } => {
            // Custom progress format for check command
            println!(
                "• [{:.0}%] Step {}/{}: {}",
                percent_complete * 100.0,
                step,
                total_steps,
                message
            );
            Some(true) // Continue processing
        }
        _ => None, // Use default handling for other events
    }
}

fn display_check_result_card(check_result: &CheckResultData, config: &AppConfig) {
    println!();
    println!("📋 Check Results:");

    let format_key_fn =
        |field: &str| -> String { format!("   {}: ", format_key(field, config.use_colors())) };

    println!("{}{}", format_key_fn("Package"), check_result.package_name);
    println!(
        "{}{}",
        format_key_fn("Environment"),
        check_result.environment
    );

    if let Some(cmd) = &check_result.check_command {
        println!("{}{}", format_key_fn("Command"), cmd);
    }

    // Format status with appropriate icon and color
    let status_line = match &check_result.result {
        CheckResult::Success => {
            if config.use_colors() {
                format!(
                    "{}{}",
                    format_key_fn("Status"),
                    console::style("✅ Installed").green().bold()
                )
            } else {
                format!("{}✅ Installed", format_key_fn("Status"))
            }
        }
        CheckResult::Failed {
            stderr, exit_code, ..
        } => {
            let status = if config.use_colors() {
                format!(
                    "{}{}",
                    format_key_fn("Status"),
                    console::style("❌ Not installed").red().bold()
                )
            } else {
                format!("{}❌ Not installed", format_key_fn("Status"))
            };

            if !stderr.is_empty() {
                format!("{}\n{}{}", status, format_key_fn("Details"), stderr.trim())
            } else if let Some(code) = exit_code {
                format!("{}\n{}Exit code {}", status, format_key_fn("Details"), code)
            } else {
                status
            }
        }
        CheckResult::NoCheckCommand => {
            if config.use_colors() {
                format!(
                    "   {}: {}",
                    console::style("Status").cyan().bold(),
                    console::style("⚠️ No check command defined").yellow()
                )
            } else {
                "   Status: ⚠️ No check command defined".to_string()
            }
        }
        CheckResult::CommandNotFound => {
            if config.use_colors() {
                format!(
                    "   {}: {}",
                    console::style("Status").cyan().bold(),
                    console::style("❌ Command not found").red().bold()
                )
            } else {
                "   Status: ❌ Command not found".to_string()
            }
        }
        CheckResult::Error(error) => {
            if config.use_colors() {
                format!(
                    "   {}: {}\n   {}: {}",
                    console::style("Status").cyan().bold(),
                    console::style("❌ Error").red().bold(),
                    console::style("Details").cyan().bold(),
                    error
                )
            } else {
                format!("   Status: ❌ Error\n   Details: {}", error)
            }
        }
    };

    println!("{}", status_line);
}

#[cfg(test)]
mod tests {
    use super::*;
    use selfie::config::AppConfigBuilder;
    use selfie::package::event::{CheckResult, CheckResultData};

    fn create_test_config() -> selfie::config::AppConfig {
        AppConfigBuilder::default()
            .environment("test-env")
            .package_directory("/tmp/test-packages")
            .use_colors(false)
            .build()
    }

    fn create_colored_config() -> selfie::config::AppConfig {
        AppConfigBuilder::default()
            .environment("test-env")
            .package_directory("/tmp/test-packages")
            .use_colors(true)
            .build()
    }

    #[test]
    fn test_display_check_result_card_success() {
        let config = create_test_config();
        let check_result = CheckResultData {
            package_name: "test-package".to_string(),
            environment: "test-env".to_string(),
            check_command: Some("which test-command".to_string()),
            result: CheckResult::Success,
        };

        // Just test that the function doesn't panic
        display_check_result_card(&check_result, &config);
    }

    #[test]
    fn test_display_check_result_card_failed() {
        let config = create_test_config();
        let check_result = CheckResultData {
            package_name: "test-package".to_string(),
            environment: "test-env".to_string(),
            check_command: Some("which missing-command".to_string()),
            result: CheckResult::Failed {
                stdout: "".to_string(),
                stderr: "command not found".to_string(),
                exit_code: Some(1),
            },
        };

        // Just test that the function doesn't panic
        display_check_result_card(&check_result, &config);
    }

    #[test]
    fn test_display_check_result_card_no_command() {
        let config = create_test_config();
        let check_result = CheckResultData {
            package_name: "test-package".to_string(),
            environment: "test-env".to_string(),
            check_command: None,
            result: CheckResult::NoCheckCommand,
        };

        // Just test that the function doesn't panic
        display_check_result_card(&check_result, &config);
    }

    #[test]
    fn test_display_check_result_card_with_colors() {
        let config = create_colored_config();
        let check_result = CheckResultData {
            package_name: "test-package".to_string(),
            environment: "test-env".to_string(),
            check_command: Some("which test-command".to_string()),
            result: CheckResult::Success,
        };

        // Just test that the function doesn't panic with colors enabled
        display_check_result_card(&check_result, &config);
    }

    #[test]
    fn test_display_check_result_card_error() {
        let config = create_test_config();
        let check_result = CheckResultData {
            package_name: "test-package".to_string(),
            environment: "test-env".to_string(),
            check_command: Some("some-command".to_string()),
            result: CheckResult::Error("Network timeout".to_string()),
        };

        // Just test that the function doesn't panic
        display_check_result_card(&check_result, &config);
    }

    #[test]
    fn test_display_check_result_card_command_not_found() {
        let config = create_test_config();
        let check_result = CheckResultData {
            package_name: "test-package".to_string(),
            environment: "test-env".to_string(),
            check_command: Some("missing-cmd".to_string()),
            result: CheckResult::CommandNotFound,
        };

        // Just test that the function doesn't panic
        display_check_result_card(&check_result, &config);
    }
}
