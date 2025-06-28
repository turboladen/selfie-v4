//!
//! Helps break down the pieces of running the `package install` command.
//!

use crate::{
    commands::runner::CommandRunner,
    config::AppConfig,
    package::{
        event::{EventSender, OperationResult},
        port::PackageRepository,
    },
};

use super::steps;

pub(super) async fn handle_install<PR, CR>(
    package_name: &str,
    repo: &PR,
    config: &AppConfig,
    command_runner: &CR,
    sender: &EventSender,
    progress: &mut crate::package::service::ProgressTracker,
) -> OperationResult
where
    PR: PackageRepository,
    CR: CommandRunner,
{
    // Step 1: Fetch package (reusing shared step)
    let package = match steps::fetch_package(repo, package_name, sender, progress).await {
        Ok(pkg) => pkg,
        Err(err) => {
            let error_msg = format!("Failed to fetch package '{}': {}", package_name, err);
            return OperationResult::Failure(error_msg);
        }
    };

    // Step 2: Find environment configuration (reusing shared step)
    let env_config = match steps::find_environment_config(
        &package,
        config.environment(),
        sender,
        progress,
    )
    .await
    {
        Ok(config) => config,
        Err(err) => {
            let error_msg = format!("Environment configuration error: {}", err);
            return OperationResult::Failure(error_msg);
        }
    };

    // Step 3: Get install command (reusing shared step with custom getter function)
    let install_cmd = match steps::get_command(
        env_config,
        "install",
        |ec| Some(ec.install()),
        sender,
        progress,
    )
    .await
    {
        Ok(cmd) => cmd,
        Err(err) => {
            let error_msg = format!("Install command error: {}", err);
            return OperationResult::Failure(error_msg);
        }
    };

    // Step 4: Execute install command (reusing shared step)
    let is_success = match steps::execute_command(
        command_runner,
        install_cmd,
        "install",
        config,
        sender,
        progress,
    )
    .await
    {
        Ok(success) => success,
        Err(err) => {
            let error_msg = format!("Command execution error: {}", err);
            return OperationResult::Failure(error_msg);
        }
    };

    // Step 5: Process result
    if is_success {
        progress
            .next(sender, "Package installed successfully")
            .await;
        OperationResult::Success(format!(
            "Installation completed successfully ({}/{} steps)",
            progress.current_step(),
            progress.total_steps()
        ))
    } else {
        sender
            .send_warning(format!(
                "Package '{}' installation command failed",
                package_name
            ))
            .await;
        OperationResult::Failure(format!(
            "Installation failed at step {}/{}",
            progress.current_step(),
            progress.total_steps()
        ))
    }
}
