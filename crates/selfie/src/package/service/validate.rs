//!
//! Helps break down the pieces of running the `package validate` command.
//!

use std::borrow::Cow;

use crate::{
    commands::runner::CommandRunner,
    config::AppConfig,
    package::{
        event::{EventSender, metadata::ValidateMetadata},
        port::PackageRepository,
    },
    validation::ValidationIssues,
};

use super::steps;

pub(super) const TOTAL_STEPS: u32 = 5;

pub(super) async fn handle_validate<PR, CR>(
    package_name: &str,
    repo: &PR,
    config: &AppConfig,
    command_runner: &CR,
    sender: &EventSender<ValidateMetadata, ValidationIssues, Cow<'static, str>>,
) -> Result<ValidationIssues, Cow<'static, str>>
where
    PR: PackageRepository,
    CR: CommandRunner,
{
    let mut step = 1;

    // Step 1: Fetch package
    let package = steps::fetch_package(repo, package_name, sender, &mut step, TOTAL_STEPS).await?;

    // Step 2: Validate the package for the current environment.
    sender
        .send_progress(step, TOTAL_STEPS, "Validating package...")
        .await;
    let validation_result = package.validate(config.environment());
    step += 1;

    let issues = validation_result.issues();
    if issues.has_errors() {
        // handle_validation_errors(issues, &reporter)
    } else if issues.has_warnings() {
        // handle_validation_warnings(issues, &reporter)
    } else {
        // display_valid_package_info(&package, config, &reporter)
    }

    todo!()
    // let env_config = steps::find_environment_config(
    //     &package,
    //     config.environment(),
    //     sender,
    //     &mut step,
    //     TOTAL_STEPS,
    // )
    // .await?;
    //
    // // Step 3: Get check command
    // let check_cmd = steps::get_command(
    //     env_config,
    //     package_name,
    //     "check",
    //     |ec| ec.check(),
    //     sender,
    //     &mut step,
    //     TOTAL_STEPS,
    // )
    // .await?;
    //
    // // Step 4: Execute check command
    // let is_success = steps::execute_command(
    //     command_runner,
    //     check_cmd,
    //     "check",
    //     config,
    //     sender,
    //     &mut step,
    //     TOTAL_STEPS,
    // )
    // .await?;
    //
    // // Step 5: Process result
    // if is_success {
    //     sender
    //         .send_progress(step, TOTAL_STEPS, "Package is installed")
    //         .await;
    //
    //     Ok("`check` completed".into())
    // } else {
    //     sender
    //         .send_progress(
    //             step,
    //             TOTAL_STEPS,
    //             format!(
    //                 "Package '{}' not installed (check command failed)",
    //                 package_name
    //             ),
    //         )
    //         .await;
    //     Err("Package not installed".into())
    // }
}
