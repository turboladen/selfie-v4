//! Helps break down the pieces of running the `package check` command.

use crate::{
    commands::runner::CommandRunner,
    config::AppConfig,
    package::{
        event::{EventSender, OperationResult},
        port::PackageRepository,
    },
};

pub(super) async fn handle_check<PR, CR>(
    package_name: &str,
    repo: &PR,
    config: &AppConfig,
    command_runner: &CR,
    sender: &EventSender,
    progress: &mut crate::package::service::ProgressTracker,
) -> OperationResult
where
    PR: PackageRepository + Clone,
    CR: CommandRunner + Clone,
{
    progress.next(sender, "Loading package definition").await;

    // Step 1: Load package from repository
    let package = match repo.get_package(package_name) {
        Ok(pkg) => {
            sender
                .send_debug(format!("Successfully loaded package: {}", package_name))
                .await;
            pkg
        }
        Err(err) => {
            let error_msg = format!("Failed to load package '{}': {}", package_name, err);
            sender.send_error(err, &error_msg).await;
            return OperationResult::Failure(error_msg);
        }
    };

    progress.next(sender, "Checking package environment").await;

    // Step 2: Get environment-specific check command
    let current_env = config.environment();
    let env_config = match package.environments().get(current_env) {
        Some(config) => config,
        None => {
            let error_msg = format!(
                "No configuration found for package '{}' in environment '{}'",
                package_name, current_env
            );
            sender.send_warning(&error_msg).await;
            return OperationResult::Failure(error_msg);
        }
    };

    let check_command = match &env_config.check {
        Some(cmd) => {
            sender
                .send_debug(format!(
                    "Found check command for environment '{}': {}",
                    current_env, cmd
                ))
                .await;
            cmd
        }
        None => {
            let error_msg = format!(
                "No check command defined for package '{}' in environment '{}'",
                package_name, current_env
            );
            sender.send_warning(&error_msg).await;
            return OperationResult::Failure(error_msg);
        }
    };

    progress.next(sender, "Running package check command").await;

    // Step 3: Execute the check command
    match command_runner.execute(check_command).await {
        Ok(output) => {
            if output.is_success() {
                let success_msg = format!(
                    "Package '{}' check completed successfully ({}/{} steps)",
                    package_name,
                    progress.current_step(),
                    progress.total_steps()
                );
                sender
                    .send_debug(format!("Check command output: {}", output.stdout_str()))
                    .await;
                OperationResult::Success(success_msg)
            } else {
                let error_msg = format!(
                    "Package '{}' check failed at step {}/{}",
                    package_name,
                    progress.current_step(),
                    progress.total_steps()
                );
                sender
                    .send_warning(format!("Check command stderr: {}", output.stderr_str()))
                    .await;
                OperationResult::Failure(error_msg)
            }
        }
        Err(err) => {
            let error_msg = format!(
                "Failed to execute check command for package '{}' at step {}/{}: {}",
                package_name,
                progress.current_step(),
                progress.total_steps(),
                err
            );
            sender.send_error(err, &error_msg).await;
            OperationResult::Failure(error_msg)
        }
    }
}
