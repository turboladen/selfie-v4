use comfy_table::Table;
use console::style;
use selfie::{
    config::AppConfig,
    package::{
        event::{EnvironmentStatus, EnvironmentStatusData, PackageEvent, PackageInfoData},
        service::PackageService,
    },
};

use crate::{formatters::format_key, terminal_progress_reporter::TerminalProgressReporter};

use super::common;

pub(crate) async fn handle_info(
    package_name: &str,
    config: &AppConfig,
    reporter: TerminalProgressReporter,
) -> i32 {
    tracing::debug!("Finding package info for: {}", package_name);

    // Create the package service implementation
    let service = common::create_package_service(config);

    // Call the service's info method to get an event stream
    match service.info(package_name).await {
        Ok(event_stream) => {
            // Process the event stream with custom handling for structured data
            common::process_events_with_custom_handler(
                event_stream,
                reporter,
                handle_info_event,
                config,
            )
            .await
        }
        Err(e) => {
            reporter.report_error(format!("Failed to get package info: {e}"));
            1
        }
    }
}

fn handle_info_event(event: &PackageEvent, config: &AppConfig) -> Option<bool> {
    match event {
        PackageEvent::PackageInfoLoaded { package_info, .. } => {
            let table = create_package_info_table(package_info, config);
            println!("{table}");
            Some(true) // Continue processing
        }
        PackageEvent::EnvironmentStatusChecked {
            environment_status, ..
        } => {
            let table = create_environment_table(environment_status, config);
            println!("\n{table}");
            Some(true) // Continue processing
        }
        _ => None, // Use default handling for other events
    }
}

fn create_package_info_table(package_info: &PackageInfoData, config: &AppConfig) -> Table {
    let mut table = common::create_formatted_table();

    // Helper functions for formatting
    let format_key_fn = |name: &str| -> String { format_key(name, config.use_colors()) };

    let format_value =
        |value: &str| -> String { common::format_field_value(value, config.use_colors()) };

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
    let env_names = common::format_environment_names(
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
    let mut env_table = common::create_formatted_table();

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
    let format_env_key =
        |key: &str| -> String { common::format_field_key(key, config.use_colors()) };

    let format_env_value =
        |value: &str| -> String { common::format_field_value(value, config.use_colors()) };

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
            let msg = format!("Unknown ({reason})");
            if use_colors {
                style(msg).yellow().italic().to_string()
            } else {
                msg
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selfie::package::event::{EnvironmentStatus, EnvironmentStatusData, PackageInfoData};
    use test_common::{ALT_TEST_ENV, TEST_ENV, TEST_VERSION, test_config, test_config_with_colors};

    fn create_test_package_info() -> PackageInfoData {
        PackageInfoData {
            name: "test-package".to_string(),
            version: TEST_VERSION.to_string(),
            description: Some("A test package".to_string()),
            homepage: Some("https://example.com".to_string()),
            environments: vec![TEST_ENV.to_string(), ALT_TEST_ENV.to_string()],
            current_environment: TEST_ENV.to_string(),
        }
    }

    fn create_test_environment_status(is_current: bool) -> EnvironmentStatusData {
        EnvironmentStatusData {
            environment_name: if is_current { TEST_ENV } else { ALT_TEST_ENV }.to_string(),
            is_current,
            install_command: "apt install test-package".to_string(),
            check_command: Some("which test-package".to_string()),
            dependencies: vec!["dependency1".to_string(), "dependency2".to_string()],
            status: if is_current {
                Some(EnvironmentStatus::Installed)
            } else {
                None
            },
        }
    }

    #[test]
    fn test_create_package_info_table() {
        let config = test_config();
        let package_info = create_test_package_info();

        let table = create_package_info_table(&package_info, &config);
        // Just test that the function doesn't panic
        let _table_str = table.to_string();
    }

    #[test]
    fn test_create_package_info_table_with_colors() {
        let config = test_config_with_colors();
        let package_info = create_test_package_info();

        let table = create_package_info_table(&package_info, &config);
        // Just test that it doesn't panic with colors enabled
        let _table_str = table.to_string();
    }

    #[test]
    fn test_create_environment_table() {
        let config = test_config();
        let env_status = create_test_environment_status(true);

        let table = create_environment_table(&env_status, &config);
        // Just test that the function doesn't panic
        let _table_str = table.to_string();
    }

    #[test]
    fn test_format_status_functions() {
        let status = EnvironmentStatus::Installed;
        let result = format_status(&status, false);
        assert!(result.contains("Installed"));

        let status = EnvironmentStatus::NotInstalled;
        let result = format_status(&status, false);
        assert!(result.contains("Not installed"));
    }

    #[test]
    fn test_format_environment_names() {
        let config = test_config();
        let environments = vec![TEST_ENV.to_string(), ALT_TEST_ENV.to_string()];
        let result = common::format_environment_names(&environments, TEST_ENV, &config);
        // Just test that it doesn't panic
        assert!(!result.is_empty());
    }

    #[test]
    fn test_create_table() {
        let table = common::create_formatted_table();
        // Just test that table creation doesn't panic
        let _table_str = table.to_string();
    }
}
