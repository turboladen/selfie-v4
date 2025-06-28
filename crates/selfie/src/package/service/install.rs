//!
//! Helps break down the pieces of running the `package install` command.
//!

use std::borrow::Cow;

use crate::{
    commands::runner::CommandRunner,
    config::AppConfig,
    package::{
        event::{EventSender, metadata::InstallMetadata},
        port::PackageRepository,
    },
};

use super::steps;

pub(super) async fn handle_install<PR, CR>(
    package_name: &str,
    repo: &PR,
    config: &AppConfig,
    command_runner: &CR,
    sender: &EventSender<InstallMetadata, Cow<'static, str>, Cow<'static, str>>,
    step: &mut u32,
    total_steps: u32,
) -> Result<Cow<'static, str>, Cow<'static, str>>
where
    PR: PackageRepository,
    CR: CommandRunner,
{
    // Step 1: Fetch package (reusing shared step)
    let package = steps::fetch_package(repo, package_name, sender, step, total_steps).await?;

    // Step 2: Find environment configuration (reusing shared step)
    let env_config =
        steps::find_environment_config(&package, config.environment(), sender, step, total_steps)
            .await?;

    // Step 3: Get install command (reusing shared step with custom getter function)
    let install_cmd = steps::get_command(
        env_config,
        package_name,
        "install",
        |ec| Some(ec.install()),
        sender,
        step,
        total_steps,
    )
    .await?;

    // Step 4: Execute install command (reusing shared step)
    let is_success = match steps::execute_command(
        command_runner,
        install_cmd,
        "install",
        config,
        sender,
        step,
        total_steps,
    )
    .await
    {
        Ok(success) => success,
        Err(e) => return Err(e),
    };

    // Step 5: Process result
    if is_success {
        sender
            .send_progress(*step, total_steps, "Package installed successfully")
            .await;
        Ok("Installation completed".into())
    } else {
        sender
            .send_warning(format!(
                "Package '{}' installation command failed",
                package_name
            ))
            .await;
        Err("Installation failed".into())
    }
}
