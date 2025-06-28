//!
//! Helps break down the pieces of running the `package list` command.
//!

use crate::{
    commands::runner::CommandRunner,
    config::AppConfig,
    package::{
        event::{ConsoleOutput, EventSender, OperationResult},
        port::PackageRepository,
    },
};

pub(super) async fn handle_list<PR, CR>(
    repo: &PR,
    config: &AppConfig,
    _command_runner: &CR,
    sender: &EventSender,
    progress: &mut crate::package::service::ProgressTracker,
) -> OperationResult
where
    PR: PackageRepository,
    CR: CommandRunner,
{
    // Step 1: List all packages
    progress.next(sender, "Loading package list").await;

    let list_output = match repo.list_packages() {
        Ok(output) => {
            sender.send_debug("Successfully loaded package list").await;
            output
        }
        Err(err) => {
            let error_msg = format!("Failed to list packages: {}", err);
            let repo_error = crate::package::port::PackageRepoError::PackageListError(err);
            sender.send_error(repo_error, &error_msg).await;
            return OperationResult::Failure(error_msg);
        }
    };

    // Step 2: Process valid packages
    progress
        .next(sender, "Processing package information")
        .await;

    let valid_packages: Vec<_> = list_output.valid_packages().collect();
    let invalid_packages: Vec<_> = list_output.invalid_packages().collect();

    if valid_packages.is_empty() && invalid_packages.is_empty() {
        sender
            .send_info(ConsoleOutput::Stdout("No packages found.".to_string()))
            .await;
    } else {
        // Send header
        sender
            .send_info(ConsoleOutput::Stdout(format!(
                "Found {} valid package(s){}",
                valid_packages.len(),
                if invalid_packages.is_empty() {
                    "".to_string()
                } else {
                    format!(" and {} invalid package(s)", invalid_packages.len())
                }
            )))
            .await;

        // Send valid packages
        for package in &valid_packages {
            let environments: Vec<String> = package
                .environments()
                .keys()
                .map(|env_name| {
                    if env_name == config.environment() {
                        format!("*{}", env_name)
                    } else {
                        env_name.to_string()
                    }
                })
                .collect();

            sender
                .send_info(ConsoleOutput::Stdout(format!(
                    "  {} (v{}) - Environments: {}",
                    package.name(),
                    package.version(),
                    environments.join(", ")
                )))
                .await;
        }

        // Send invalid packages as warnings
        for invalid_package in &invalid_packages {
            sender
                .send_warning(format!(
                    "Invalid package at {}: {}",
                    invalid_package.package_path().display(),
                    invalid_package
                ))
                .await;
        }
    }

    // Step 3: Complete operation
    progress.next(sender, "Finalizing package list").await;

    let success_msg = format!(
        "Package listing completed with {} valid package(s){}",
        valid_packages.len(),
        if invalid_packages.is_empty() {
            "".to_string()
        } else {
            format!(" and {} invalid package(s)", invalid_packages.len())
        }
    );

    sender
        .send_debug("Package listing completed successfully")
        .await;

    OperationResult::Success(success_msg)
}
