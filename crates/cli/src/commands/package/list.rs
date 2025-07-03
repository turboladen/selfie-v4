use comfy_table::{ContentArrangement, Table, modifiers, presets};
use console::style;
use selfie::{
    commands::ShellCommandRunner,
    config::AppConfig,
    fs::real::RealFileSystem,
    package::{
        event::PackageEvent,
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
        let repo =
            YamlPackageRepository::new(RealFileSystem, self.config.package_directory().clone());
        let command_runner = ShellCommandRunner::new("/bin/sh", self.config.command_timeout());

        // Create the package service implementation
        let service = PackageServiceImpl::new(repo, command_runner, self.config.clone());

        // Call the service's list method to get an event stream
        match service.list().await {
            Ok(event_stream) => {
                // Process the event stream with custom handling for structured data
                let processor = EventProcessor::new(self.reporter);
                processor
                    .process_events_with_handler(event_stream, |event, _reporter| {
                        handle_list_event(event, self.config)
                    })
                    .await
            }
            Err(e) => {
                self.reporter
                    .report_error(format!("Failed to list packages: {e}"));
                1
            }
        }
    }
}

fn handle_list_event(event: &PackageEvent, config: &AppConfig) -> Option<bool> {
    match event {
        PackageEvent::PackageListLoaded { package_list, .. } => {
            // Show package directory path
            println!("📁 Package directory: {}", package_list.package_directory);

            if package_list.valid_packages.is_empty() && package_list.invalid_packages.is_empty() {
                println!("No packages found.");
            } else {
                display_packages_table(&package_list.valid_packages, config);

                // Report invalid packages as separate messages after the table
                for invalid_package in &package_list.invalid_packages {
                    eprintln!(
                        "⚠️  Invalid package at {}: {}",
                        invalid_package.path, invalid_package.error
                    );
                }
            }
            Some(true) // Continue processing
        }
        _ => None, // Use default handling for other events
    }
}

fn display_packages_table(
    packages: &[selfie::package::event::PackageListItem],
    config: &AppConfig,
) {
    if packages.is_empty() {
        return;
    }

    let mut table = create_table();
    table.set_header(vec!["Name", "Version", "Environments"]);

    for package in packages {
        let package_name = if config.use_colors() {
            style(&package.name).magenta().bold().to_string()
        } else {
            package.name.clone()
        };

        let version = if config.use_colors() {
            style(format!("v{}", package.version)).dim().to_string()
        } else {
            format!("v{}", package.version)
        };

        let environments = format_environments(&package.environments, config.environment(), config);

        table.add_row(vec![package_name, version, environments]);
    }

    println!("{table}");
}

fn create_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL_CONDENSED)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table
}

fn format_environments(
    environments: &[String],
    current_environment: &str,
    config: &AppConfig,
) -> String {
    environments
        .iter()
        .map(|env_name| {
            if env_name == current_environment {
                let env = format!("*{env_name}");
                if config.use_colors() {
                    style(env).bold().green().to_string()
                } else {
                    env
                }
            } else if config.use_colors() {
                style(env_name).dim().green().to_string()
            } else {
                env_name.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use selfie::package::event::PackageListItem;
    use test_common::{ALT_TEST_ENV, TEST_ENV, TEST_VERSION, test_config, test_config_with_colors};

    fn create_mock_reporter() -> TerminalProgressReporter {
        TerminalProgressReporter::new(false)
    }

    #[test]
    fn test_list_command_new() {
        let config = test_config();
        let reporter = create_mock_reporter();

        let command = ListCommand::new(&config, reporter);
        // Just test that construction doesn't panic
        assert_eq!(command.config.environment(), "test-env");
    }

    #[test]
    fn test_display_packages_table_empty() {
        let config = test_config();
        let packages = vec![];

        // Should not panic with empty list
        display_packages_table(&packages, &config);
    }

    #[test]
    fn test_display_packages_table_single_package() {
        let config = test_config();
        let packages = vec![PackageListItem {
            name: "test-package".to_string(),
            version: TEST_VERSION.to_string(),
            environments: vec![TEST_ENV.to_string()],
        }];

        // Should not panic
        display_packages_table(&packages, &config);
    }

    #[test]
    fn test_display_packages_table_with_colors() {
        let config = test_config_with_colors();
        let packages = vec![PackageListItem {
            name: "test-package".to_string(),
            version: TEST_VERSION.to_string(),
            environments: vec![TEST_ENV.to_string()],
        }];

        // Should not panic with colors enabled
        display_packages_table(&packages, &config);
    }

    #[test]
    fn test_create_table() {
        let table = create_table();
        // Just test that table creation doesn't panic
        let _table_str = table.to_string();
    }

    #[test]
    fn test_format_environments() {
        let config = test_config();
        let environments = vec![TEST_ENV.to_string(), ALT_TEST_ENV.to_string()];

        let result = format_environments(&environments, TEST_ENV, &config);

        // Just test that it doesn't panic and returns something
        assert!(!result.is_empty());
    }
}
