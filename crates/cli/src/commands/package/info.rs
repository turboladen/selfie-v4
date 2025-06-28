use comfy_table::{ContentArrangement, Table, modifiers, presets};
use console::style;
use selfie::{
    commands::ShellCommandRunner,
    config::AppConfig,
    fs::real::RealFileSystem,
    package::{
        event::{EnvironmentStatus, EnvironmentStatusData, PackageEvent, PackageInfoData},
        repository::YamlPackageRepository,
        service::{PackageService, PackageServiceImpl},
    },
};

use crate::{
    event_processor::EventProcessor, formatters::format_key,
    terminal_progress_reporter::TerminalProgressReporter,
};

pub(crate) async fn handle_info(
    package_name: &str,
    config: &AppConfig,
    reporter: TerminalProgressReporter,
) -> i32 {
    tracing::debug!("Finding package info for: {}", package_name);

    // Create the repository and command runner
    let repo = YamlPackageRepository::new(RealFileSystem, config.package_directory().to_path_buf());
    let command_runner = ShellCommandRunner::new("/bin/sh", config.command_timeout());

    // Create the package service implementation
    let service = PackageServiceImpl::new(repo, command_runner, config.clone());

    // Call the service's info method to get an event stream
    match service.info(package_name).await {
        Ok(event_stream) => {
            // Process the event stream with custom handling for structured data
            let processor = EventProcessor::new(reporter);
            processor
                .process_events_with_handler(event_stream, |event, _reporter| {
                    handle_info_event(event, config)
                })
                .await
        }
        Err(e) => {
            reporter.report_error(format!("Failed to get package info: {}", e));
            1
        }
    }
}

fn handle_info_event(event: &PackageEvent, config: &AppConfig) -> Option<bool> {
    match event {
        PackageEvent::PackageInfoLoaded { package_info, .. } => {
            let table = create_package_info_table(package_info, config);
            println!("{}", table);
            Some(true) // Continue processing
        }
        PackageEvent::EnvironmentStatusChecked {
            environment_status, ..
        } => {
            let table = create_environment_table(environment_status, config);
            println!("\n{}", table);
            Some(true) // Continue processing
        }
        _ => None, // Use default handling for other events
    }
}

fn create_package_info_table(package_info: &PackageInfoData, config: &AppConfig) -> Table {
    let mut table = create_table();

    // Helper functions for formatting
    let format_key_fn = |name: &str| -> String { format_key(name, config.use_colors()) };

    let format_value = |value: &str| -> String {
        if config.use_colors() {
            style(value).white().to_string()
        } else {
            value.to_string()
        }
    };

    // Add the basic package info rows
    table.add_row(vec![
        format_key_fn("Name"),
        format_value(&package_info.name),
    ]);
    table.add_row(vec![
        format_key_fn("Version"),
        format_value(&package_info.version),
    ]);

    if let Some(desc) = &package_info.description {
        table.add_row(vec![format_key_fn("Description"), format_value(desc)]);
    }

    if let Some(homepage) = &package_info.homepage {
        let homepage_value = if config.use_colors() {
            style(homepage).underlined().blue().to_string()
        } else {
            homepage.to_string()
        };
        table.add_row(vec![format_key_fn("Homepage"), homepage_value]);
    }

    // Format the environment names as a comma-separated list
    let env_names = format_environment_names(
        &package_info.environments,
        &package_info.current_environment,
        config,
    );
    table.add_row(vec![
        format_key_fn("Environments"),
        format_value(&env_names),
    ]);

    table
}

fn create_environment_table(env_status: &EnvironmentStatusData, config: &AppConfig) -> Table {
    let mut env_table = create_table();

    // Create a header for the environment table
    let env_header = if env_status.is_current {
        let msg = format!("Environment: *{}", env_status.environment_name);
        if config.use_colors() {
            style(msg).bold().green().to_string()
        } else {
            msg
        }
    } else {
        let msg = format!("Environment: {}", env_status.environment_name);
        if config.use_colors() {
            style(msg).bold().to_string()
        } else {
            msg
        }
    };

    // Add a header row
    env_table.set_header(vec![env_header, String::new()]);

    // Format environment detail keys
    let format_env_key = |key: &str| -> String {
        if config.use_colors() {
            style(key).magenta().to_string()
        } else {
            key.to_string()
        }
    };

    let format_env_value = |value: &str| -> String {
        if config.use_colors() {
            style(value).white().to_string()
        } else {
            value.to_string()
        }
    };

    // Add installation status if this is the current environment and we have status
    if env_status.is_current {
        if let Some(status) = &env_status.status {
            let status_text = format_status(status, config.use_colors());
            env_table.add_row(vec![format_env_key("Status"), status_text]);
        }
    }

    // Add environment detail rows
    env_table.add_row(vec![
        format_env_key("Install"),
        format_env_value(&env_status.install_command),
    ]);

    if let Some(check) = &env_status.check_command {
        env_table.add_row(vec![format_env_key("Check"), format_env_value(check)]);
    }

    if !env_status.dependencies.is_empty() {
        env_table.add_row(vec![
            format_env_key("Dependencies"),
            format_env_value(&env_status.dependencies.join(", ")),
        ]);
    }

    env_table
}

fn create_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL_CONDENSED)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table
}

fn format_environment_names(
    environments: &[String],
    current_environment: &str,
    config: &AppConfig,
) -> String {
    environments
        .iter()
        .map(|name| {
            if name == current_environment {
                if config.use_colors() {
                    format!("{}", style(format!("*{name}")).green().bold())
                } else {
                    format!("*{name}")
                }
            } else {
                name.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_status(status: &EnvironmentStatus, use_colors: bool) -> String {
    match status {
        EnvironmentStatus::Installed => {
            if use_colors {
                style("Installed ✓").green().bold().to_string()
            } else {
                "Installed ✓".to_string()
            }
        }
        EnvironmentStatus::NotInstalled => {
            if use_colors {
                style("Not installed ✗").yellow().to_string()
            } else {
                "Not installed ✗".to_string()
            }
        }
        EnvironmentStatus::Unknown(reason) => {
            let msg = format!("Unknown ({})", reason);
            if use_colors {
                style(msg).yellow().italic().to_string()
            } else {
                msg
            }
        }
    }
}
