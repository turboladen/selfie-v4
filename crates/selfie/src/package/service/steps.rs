use std::borrow::Cow;

use crate::{
    commands::runner::CommandRunner,
    config::AppConfig,
    package::{EnvironmentConfig, Package, event::EventSender, port::PackageRepository},
};

/// Step to fetch a package from the repository
pub async fn fetch_package<PR>(
    repo: &PR,
    package_name: &str,
    sender: &EventSender,
    step: &mut u32,
    total_steps: u32,
) -> Result<Package, &'static str>
where
    PR: PackageRepository,
{
    sender
        .send_progress(
            *step,
            total_steps,
            format!("Fetching package: {package_name}"),
        )
        .await;
    *step += 1;

    match repo.get_package(package_name) {
        Ok(package) => {
            sender.send_trace("Package found").await;
            Ok(package)
        }
        Err(e) => {
            sender
                .send_error(e, "Error fetching package from repository")
                .await;
            Err("Unable to fetch package")
        }
    }
}

/// Step to find environment configuration for a package
pub async fn find_environment_config<'a>(
    package: &'a Package,
    environment: &str,
    sender: &EventSender,
    step: &mut u32,
    total_steps: u32,
) -> Result<&'a EnvironmentConfig, Cow<'static, str>> {
    sender
        .send_progress(
            *step,
            total_steps,
            format!(
                "Checking if package supports current environment: {}",
                environment
            ),
        )
        .await;
    *step += 1;

    match package.environments().get(environment) {
        Some(env_config) => {
            sender
                .send_trace("Current environment supported by package")
                .await;
            Ok(env_config)
        }
        None => {
            sender
                .send_warning(format!(
                    "Package '{}' does not support environment '{}'",
                    package.name(),
                    environment
                ))
                .await;
            Err("Environment not supported".into())
        }
    }
}

/// Step to get a specific command from environment config
pub async fn get_command<'a>(
    env_config: &'a EnvironmentConfig,
    package_name: &str,
    command_type: &str,
    command_getter: impl FnOnce(&EnvironmentConfig) -> Option<&str>,
    sender: &EventSender,
    step: &mut u32,
    total_steps: u32,
) -> Result<&'a str, Cow<'static, str>> {
    sender
        .send_progress(
            *step,
            total_steps,
            format!("Checking if package has `{command_type}` command"),
        )
        .await;
    *step += 1;

    match command_getter(env_config) {
        Some(cmd) => {
            sender
                .send_trace(format!("Package has `{command_type}` command"))
                .await;
            Ok(cmd)
        }
        None => {
            sender
                .send_progress(
                    *step,
                    total_steps,
                    format!("Package does not have `{command_type}` command"),
                )
                .await;
            Err(format!("No {command_type} command defined").into())
        }
    }
}

/// Step to execute a command
pub async fn execute_command<CR>(
    command_runner: &CR,
    cmd: &str,
    command_type: &str,
    config: &AppConfig,
    sender: &EventSender,
    step: &mut u32,
    total_steps: u32,
) -> Result<bool, Cow<'static, str>>
where
    CR: CommandRunner,
{
    sender
        .send_progress(
            *step,
            total_steps,
            format!("Executing package's `{command_type}` command: `{cmd}`"),
        )
        .await;
    *step += 1;

    match command_runner
        .execute_with_timeout(cmd, config.command_timeout())
        .await
    {
        Ok(output) => {
            if config.verbose() {
                if !output.stdout_str().trim().is_empty() {
                    sender
                        .send_info(crate::package::event::ConsoleOutput::Stdout(
                            output.stdout_str().to_string(),
                        ))
                        .await;
                }
                if !output.stderr_str().trim().is_empty() {
                    sender
                        .send_info(crate::package::event::ConsoleOutput::Stderr(
                            output.stderr_str().to_string(),
                        ))
                        .await;
                }
            }

            if output.is_success() {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Err(error) => {
            sender
                .send_error(error, format!("Failed to execute {command_type} command"))
                .await;
            Err(format!("Command execution failed: {command_type}").into())
        }
    }
}
