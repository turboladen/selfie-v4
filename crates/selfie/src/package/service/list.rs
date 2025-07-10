//!
//! Helps break down the pieces of running the `package list` command.
//!

use crate::{
    commands::runner::CommandRunner,
    config::AppConfig,
    package::{
        event::{
            EventSender, InvalidPackageInfo, OperationResult, PackageListData, PackageListItem,
        },
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
            let error_msg = format!("Failed to list packages: {err}");
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

    // Convert to structured data and sort by name
    let mut valid_package_items: Vec<PackageListItem> = valid_packages
        .iter()
        .map(|package| PackageListItem {
            name: package.name().to_string(),
            version: package.version().to_string(),
            environments: package.environments().keys().cloned().collect(),
        })
        .collect();

    // Sort packages alphabetically by name
    valid_package_items.sort_by(|a, b| a.name.cmp(&b.name));

    let invalid_package_items: Vec<InvalidPackageInfo> = invalid_packages
        .iter()
        .map(|invalid_package| InvalidPackageInfo {
            path: invalid_package.package_path().display().to_string(),
            error: invalid_package.to_string(),
        })
        .collect();

    let package_list_data = PackageListData {
        valid_packages: valid_package_items,
        invalid_packages: invalid_package_items,
        current_environment: config.environment().to_string(),
        package_directory: config.package_directory().display().to_string(),
    };

    // Send structured data event
    sender.send_package_list(package_list_data).await;

    // Step 3: Complete operation
    progress.next(sender, "Finalizing package list").await;

    let success_msg = format!(
        "Package listing completed with {} valid package(s){}",
        valid_packages.len(),
        if invalid_packages.is_empty() {
            String::new()
        } else {
            format!(" and {} invalid package(s)", invalid_packages.len())
        }
    );

    sender
        .send_debug("Package listing completed successfully")
        .await;

    OperationResult::Success(success_msg)
}
