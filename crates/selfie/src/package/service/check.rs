//! Helps break down the pieces of running the `package check` command.

use crate::{
    commands::runner::CommandRunner,
    config::AppConfig,
    package::{
        event::{CheckResult, CheckResultData, EventSender, OperationResult},
        port::{PackageRepoError, PackageRepository},
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
                .send_debug(format!("Successfully loaded package: {package_name}"))
                .await;
            pkg
        }
        Err(err) => {
            let error_msg = format!("Failed to load package '{package_name}': {err}");
            sender.send_error(err, &error_msg).await;
            return OperationResult::Failure(error_msg);
        }
    };

    progress.next(sender, "Checking package environment").await;

    // Step 2: Get environment-specific check command with rich error context
    let current_env = config.environment();
    let env_config = if let Some(config) = package.environments().get(current_env) {
        config
    } else {
        use crate::package::port::PackageError;
        let err = Box::new(PackageError::EnvironmentNotFound {
            package_name: package_name.to_string(),
            environment: current_env.to_string(),
            available_environments: package.environments().keys().cloned().collect(),
            package_file: package.path().clone(),
        });
        let error_msg = format!("Environment configuration error: {err}");
        sender
            .send_error(PackageRepoError::PackageError(err), &error_msg)
            .await;
        return OperationResult::Failure(error_msg);
    };

    let check_command = if let Some(check_cmd) = env_config.check.as_ref() {
        check_cmd
    } else {
        use crate::package::port::PackageError;
        // Find other environments that do have check commands
        let other_envs_with_check: Vec<String> = package
            .environments()
            .iter()
            .filter_map(|(env_name, env_config)| {
                if env_config.check.is_some() {
                    Some(env_name.clone())
                } else {
                    None
                }
            })
            .collect();

        let err = Box::new(PackageError::NoCheckCommand {
            package_name: package_name.to_string(),
            environment: current_env.to_string(),
            package_file: package.path().clone(),
            other_envs_with_check,
        });

        // Send structured result for no check command
        let check_result = CheckResultData {
            package_name: package_name.to_string(),
            environment: current_env.to_string(),
            check_command: None,
            result: CheckResult::NoCheckCommand,
        };
        sender.send_check_result(check_result).await;

        let error_msg = format!("Check command configuration error: {err}");
        sender
            .send_error(PackageRepoError::PackageError(err), &error_msg)
            .await;
        return OperationResult::Failure(error_msg);
    };
    sender
        .send_debug(format!(
            "Found check command for environment '{current_env}': {check_command}"
        ))
        .await;

    progress.next(sender, "Running package check command").await;

    // Step 3: Execute the check command
    let check_result = match command_runner.execute(check_command).await {
        Ok(output) => {
            if output.is_success() {
                sender
                    .send_debug(format!("Check command output: {}", output.stdout_str()))
                    .await;
                CheckResultData {
                    package_name: package_name.to_string(),
                    environment: current_env.to_string(),
                    check_command: Some(check_command.to_string()),
                    result: CheckResult::Success,
                }
            } else {
                CheckResultData {
                    package_name: package_name.to_string(),
                    environment: current_env.to_string(),
                    check_command: Some(check_command.to_string()),
                    result: CheckResult::Failed {
                        stdout: output.stdout_str().to_string(),
                        stderr: output.stderr_str().to_string(),
                        exit_code: Some(output.exit_code()),
                    },
                }
            }
        }
        Err(err) => CheckResultData {
            package_name: package_name.to_string(),
            environment: current_env.to_string(),
            check_command: Some(check_command.to_string()),
            result: CheckResult::Error(err.to_string()),
        },
    };

    // Send structured check result
    sender.send_check_result(check_result.clone()).await;

    // Return appropriate operation result
    match &check_result.result {
        CheckResult::Success => {
            let success_msg = format!(
                "Package '{}' check completed successfully ({}/{} steps)",
                package_name,
                progress.current_step(),
                progress.total_steps()
            );
            OperationResult::Success(success_msg)
        }
        CheckResult::Failed { .. } => {
            let error_msg = format!(
                "Package '{}' check failed at step {}/{}",
                package_name,
                progress.current_step(),
                progress.total_steps()
            );
            OperationResult::Failure(error_msg)
        }
        CheckResult::Error(_) => {
            let error_msg = format!(
                "Failed to execute check command for package '{}' at step {}/{}",
                package_name,
                progress.current_step(),
                progress.total_steps()
            );
            OperationResult::Failure(error_msg)
        }
        _ => {
            // This case is already handled above, but included for completeness
            OperationResult::Failure("Unexpected check result".to_string())
        }
    }
}
