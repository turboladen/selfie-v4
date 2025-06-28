//!
//! Helps break down the pieces of running the `package validate` command.
//!

use crate::{
    commands::runner::CommandRunner,
    config::AppConfig,
    package::{
        event::{EventSender, OperationResult},
        port::PackageRepository,
    },
};

pub(super) async fn handle_validate<PR, CR>(
    package_name: &str,
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
    // Step 1: Fetch package
    progress.next(sender, "Loading package definition").await;

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

    // Step 2: Validate the package for the current environment
    progress.next(sender, "Validating package definition").await;

    let validation_result = package.validate(config.environment());
    let issues = validation_result.issues();

    // Step 3: Process validation results
    progress.next(sender, "Processing validation results").await;

    if issues.has_errors() {
        let error_count = issues.errors().len();
        let warning_count = issues.warnings().len();

        for error in issues.errors() {
            sender
                .send_warning(format!("Validation error: {:?}", error))
                .await;
        }

        for warning in issues.warnings() {
            sender
                .send_warning(format!("Validation warning: {:?}", warning))
                .await;
        }

        let error_msg = format!(
            "Package '{}' validation failed with {} error(s) and {} warning(s) (completed {}/{} steps)",
            package_name,
            error_count,
            warning_count,
            progress.current_step(),
            progress.total_steps()
        );
        OperationResult::Failure(error_msg)
    } else if issues.has_warnings() {
        let warning_count = issues.warnings().len();

        for warning in issues.warnings() {
            sender
                .send_warning(format!("Validation warning: {:?}", warning))
                .await;
        }

        let success_msg = format!(
            "Package '{}' validation completed with {} warning(s) ({}/{} steps)",
            package_name,
            warning_count,
            progress.current_step(),
            progress.total_steps()
        );
        OperationResult::Success(success_msg)
    } else {
        let success_msg = format!(
            "Package '{}' validation completed successfully ({}/{} steps)",
            package_name,
            progress.current_step(),
            progress.total_steps()
        );
        sender
            .send_debug("Package definition is valid for the current environment")
            .await;
        OperationResult::Success(success_msg)
    }
}
