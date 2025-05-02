//!
//! Helps break down the pieces of running the `package check` command.
//!

use std::borrow::Cow;

use crate::{
    commands::runner::CommandRunner,
    config::AppConfig,
    package::{
        event::{EventSender, metadata::CheckMetadata},
        port::PackageRepository,
    },
};

use super::steps;

pub(super) async fn handle_check<PR, CR>(
    package_name: &str,
    repo: &PR,
    config: &AppConfig,
    command_runner: &CR,
    sender: &EventSender<CheckMetadata>,
    step: &mut u32,
    total_steps: u32,
) -> Result<Cow<'static, str>, Cow<'static, str>>
where
    PR: PackageRepository,
    CR: CommandRunner,
{
    // Step 1: Fetch package
    let package = steps::fetch_package(repo, package_name, sender, step, total_steps).await?;

    // Step 2: Find environment configuration
    let env_config =
        steps::find_environment_config(&package, config.environment(), sender, step, total_steps)
            .await?;

    // Step 3: Get check command
    let check_cmd = steps::get_command(
        env_config,
        package_name,
        "check",
        |ec| ec.check(),
        sender,
        step,
        total_steps,
    )
    .await?;

    // Step 4: Execute check command
    let is_success = steps::execute_command(
        command_runner,
        check_cmd,
        "check",
        config,
        sender,
        step,
        total_steps,
    )
    .await?;

    // Step 5: Process result
    if is_success {
        sender
            .send_progress(*step, total_steps, "Package is installed")
            .await;

        Ok("`check` completed".into())
    } else {
        sender
            .send_progress(
                *step,
                total_steps,
                format!(
                    "Package '{}' not installed (check command failed)",
                    package_name
                ),
            )
            .await;
        Err("Package not installed".into())
    }
}
