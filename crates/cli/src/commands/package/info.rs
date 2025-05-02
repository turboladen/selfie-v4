use comfy_table::{ContentArrangement, Table, modifiers, presets};
use console::style;
use selfie::{
    commands::{ShellCommandRunner, runner::CommandRunner},
    config::AppConfig,
    fs::real::RealFileSystem,
    package::{port::PackageRepository, repository::YamlPackageRepository},
};

use crate::{
    commands::package::handle_package_repo_error,
    formatters::{self, FieldStyle},
    terminal_progress_reporter::TerminalProgressReporter,
};

pub(crate) async fn handle_info(
    package_name: &str,
    config: &AppConfig,
    reporter: TerminalProgressReporter,
) -> i32 {
    tracing::debug!("Finding package info for: {}", package_name);

    let repo = YamlPackageRepository::new(RealFileSystem, config.package_directory().to_path_buf());
    let command_runner = ShellCommandRunner::new("/bin/sh", config.command_timeout());

    match repo.get_package(package_name) {
        Ok(package) => {
            // Helper function for formatting field names in the left column
            let format_field = |name: &str| -> String {
                formatters::format_field(name, FieldStyle::Key, config.use_colors())
            };

            // Helper function for formatting values in the right column
            let format_value = |value: &str| -> String {
                formatters::format_field(value, FieldStyle::Value, config.use_colors())
            };

            // Create and display main package info table
            let table = create_package_info_table(&package, config, format_field, format_value);
            println!("{table}");
            println!(); // Space between tables

            // Create and display environment tables
            let env_tables =
                create_environment_tables(&package, config, &command_runner, format_value).await;
            for table in env_tables {
                println!("{table}");
                println!(); // Add space between environment tables
            }

            0
        }
        Err(e) => {
            let repo: &dyn PackageRepository = &repo;
            handle_package_repo_error(e, repo, reporter);
            1
        }
    }
}

fn create_package_info_table<F, V>(
    package: &selfie::package::Package,
    config: &AppConfig,
    format_field: F,
    format_value: V,
) -> Table
where
    F: Fn(&str) -> String,
    V: Fn(&str) -> String,
{
    let mut table = create_table();

    // Add the basic package info rows
    table.add_row(vec![format_field("Name"), format_value(package.name())]);
    table.add_row(vec![
        format_field("Version"),
        format_value(package.version()),
    ]);

    if let Some(desc) = package.description() {
        table.add_row(vec![format_field("Description"), format_value(desc)]);
    }

    if let Some(homepage) = package.homepage() {
        table.add_row(vec![
            format_field("Homepage"),
            if config.use_colors() {
                style(homepage).underlined().blue().to_string()
            } else {
                homepage.to_string()
            },
        ]);
    }

    // Format the environment names as a comma-separated list
    let env_names = format_environment_names(package, config);

    // Add environments row with list of environment names
    table.add_row(vec![format_field("Environments"), format_value(&env_names)]);

    table
}

fn create_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL_CONDENSED)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table
}

fn format_environment_names(package: &selfie::package::Package, config: &AppConfig) -> String {
    package
        .environments()
        .keys()
        .map(|name| {
            if name == config.environment() {
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

async fn create_environment_tables<V>(
    package: &selfie::package::Package,
    config: &AppConfig,
    command_runner: &ShellCommandRunner,
    format_value: V,
) -> Vec<Table>
where
    V: Fn(&str) -> String,
{
    let mut tables = Vec::new();

    for (env_name, env_config) in package.environments() {
        // Create environment details table
        let mut env_table = create_table();

        // Create a header for the environment table
        let env_header = if env_name == config.environment() {
            let msg = format!("Environment: *{env_name}");
            if config.use_colors() {
                style(msg).bold().green().to_string()
            } else {
                msg
            }
        } else {
            let msg = format!("Environment: {env_name}");
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

        // Add installation status if this is the current environment
        if env_name == config.environment() {
            let status =
                get_installation_status(env_config, command_runner, config.use_colors()).await;
            env_table.add_row(vec![format_env_key("Status"), status]);
        }

        // Add environment detail rows
        env_table.add_row(vec![
            format_env_key("Install"),
            format_value(env_config.install()),
        ]);

        if let Some(check) = env_config.check() {
            env_table.add_row(vec![format_env_key("Check"), format_value(check)]);
        }

        if !env_config.dependencies().is_empty() {
            env_table.add_row(vec![
                format_env_key("Dependencies"),
                format_value(&env_config.dependencies().join(", ")),
            ]);
        }

        tables.push(env_table);
    }

    tables
}
async fn get_installation_status(
    env_config: &selfie::package::EnvironmentConfig,
    command_runner: &ShellCommandRunner,
    use_colors: bool,
) -> String {
    // Only run check for current environment
    if let Some(check_cmd) = env_config.check() {
        // Run the check command asynchronously
        if let Ok(output) = command_runner.execute(check_cmd).await {
            if output.is_success() {
                if use_colors {
                    style("Installed ✓").green().bold().to_string()
                } else {
                    "Installed ✓".to_string()
                }
            } else if use_colors {
                style("Not installed ✗").yellow().to_string()
            } else {
                "Not installed ✗".to_string()
            }
        } else {
            // Error executing check command
            if use_colors {
                style("Unknown (check failed)")
                    .yellow()
                    .italic()
                    .to_string()
            } else {
                "Unknown (check failed)".to_string()
            }
        }
    } else {
        // No check command available
        if use_colors {
            style("Unknown (no check command)")
                .dim()
                .italic()
                .to_string()
        } else {
            "Unknown (no check command)".to_string()
        }
    }
}
