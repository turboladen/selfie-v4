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

pub(super) const TOTAL_STEPS: u32 = 3;

pub(super) async fn handle_validate<PR, CR>(
    package_name: &str,
    repo: &PR,
    config: &AppConfig,
    command_runner: &CR,
    sender: &EventSender,
) -> OperationResult
where
    PR: PackageRepository,
    CR: CommandRunner,
{
    let mut step = 1;

    // Step 1: Fetch package
    sender
        .send_progress(step, TOTAL_STEPS, "Loading package definition")
        .await;

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
    step += 1;

    // Step 2: Validate the package for the current environment
    sender
        .send_progress(step, TOTAL_STEPS, "Validating package definition")
        .await;

    let validation_result = package.validate(config.environment());
    let issues = validation_result.issues();
    step += 1;

    // Step 3: Process validation results
    sender
        .send_progress(step, TOTAL_STEPS, "Processing validation results")
        .await;

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
            "Package '{}' validation failed with {} error(s) and {} warning(s)",
            package_name, error_count, warning_count
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
            "Package '{}' validation completed with {} warning(s)",
            package_name, warning_count
        );
        OperationResult::Success(success_msg)
    } else {
        let success_msg = format!(
            "Package '{}' validation completed successfully",
            package_name
        );
        sender
            .send_debug("Package definition is valid for the current environment")
            .await;
        OperationResult::Success(success_msg)
    }
}
