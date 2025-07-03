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
    progress: &mut crate::package::service::ProgressTracker,
) -> Result<Package, &'static str>
where
    PR: PackageRepository,
{
    progress
        .next(sender, format!("Fetching package: {package_name}"))
        .await;

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
    progress: &mut crate::package::service::ProgressTracker,
) -> Result<&'a EnvironmentConfig, Cow<'static, str>> {
    progress
        .next(
            sender,
            format!("Checking if package supports current environment: {environment}"),
        )
        .await;

    if let Some(env_config) = package.environments().get(environment) {
        sender
            .send_trace("Current environment supported by package")
            .await;
        Ok(env_config)
    } else {
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

/// Step to get a specific command from environment config
pub async fn get_command<'a>(
    env_config: &'a EnvironmentConfig,
    command_type: &str,
    command_getter: impl FnOnce(&EnvironmentConfig) -> Option<&str>,
    sender: &EventSender,
    progress: &mut crate::package::service::ProgressTracker,
) -> Result<&'a str, Cow<'static, str>> {
    progress
        .next(
            sender,
            format!("Checking if package has `{command_type}` command"),
        )
        .await;

    if let Some(cmd) = command_getter(env_config) {
        sender
            .send_trace(format!("Package has `{command_type}` command"))
            .await;
        Ok(cmd)
    } else {
        progress
            .next(
                sender,
                format!("Package does not have `{command_type}` command"),
            )
            .await;
        Err(format!("No {command_type} command defined").into())
    }
}

/// Step to execute a command
pub async fn execute_command<CR>(
    command_runner: &CR,
    cmd: &str,
    command_type: &str,
    config: &AppConfig,
    sender: &EventSender,
    progress: &mut crate::package::service::ProgressTracker,
) -> Result<bool, Cow<'static, str>>
where
    CR: CommandRunner,
{
    let is_final_execution = progress.current_step() + 1 == progress.total_steps();
    let step_message = if is_final_execution {
        format!("Executing final `{command_type}` command: `{cmd}`")
    } else {
        format!("Executing package's `{command_type}` command: `{cmd}`")
    };

    progress.next(sender, step_message).await;

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
                if is_final_execution {
                    sender
                        .send_debug(format!(
                            "Final command execution completed successfully (step {}/{})",
                            progress.current_step(),
                            progress.total_steps()
                        ))
                        .await;
                }
                Ok(true)
            } else {
                sender
                    .send_warning(format!(
                        "Command failed at step {}/{}: exit code {}",
                        progress.current_step(),
                        progress.total_steps(),
                        output.exit_code()
                    ))
                    .await;
                Ok(false)
            }
        }
        Err(error) => {
            sender
                .send_error(
                    error,
                    format!(
                        "Failed to execute {command_type} command at step {}/{}",
                        progress.current_step(),
                        progress.total_steps()
                    ),
                )
                .await;
            Err(format!(
                "Command execution failed: {} (step {}/{})",
                command_type,
                progress.current_step(),
                progress.total_steps()
            )
            .into())
        }
    }
}
