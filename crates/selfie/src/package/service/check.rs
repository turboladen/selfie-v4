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

// pub(super) async fn handle_check<PR, CR>(
//     package_name: &str,
//     repo: &PR,
//     config: &AppConfig,
//     command_runner: &CR,
//     sender: &EventSender<CheckMetadata>,
//     step: &mut u32,
//     total_steps: u32,
// ) -> Result<Cow<'static, str>, Cow<'static, str>>
// where
//     PR: PackageRepository,
//     CR: CommandRunner,
// {
//     match repo.get_package(package_name) {
//         Ok(package) => {
//             sender.send_trace("Package found").await;
//
//             let current_env = config.environment();
//
//             // ╭──────────────────────────────────────────────────────────────╮
//             // │ Step 2: Check if package supports current environment        │
//             // ╰──────────────────────────────────────────────────────────────╯
//             sender
//                 .send_progress(
//                     *step,
//                     total_steps,
//                     format!(
//                         "Checking if package supports current environment: {}",
//                         current_env
//                     ),
//                 )
//                 .await;
//             *step += 1;
//
//             if let Some(env_config) = package.environments().get(current_env) {
//                 sender
//                     .send_trace("Current environment supported by package")
//                     .await;
//
//                 // ╭───────────────────────────────────────────╮
//                 // │ Step 3: Get the package's `check` command │
//                 // ╰───────────────────────────────────────────╯
//                 sender
//                     .send_progress(
//                         *step,
//                         total_steps,
//                         "Checking if package has `check` command",
//                     )
//                     .await;
//                 *step += 1;
//
//                 if let Some(check_cmd) = env_config.check() {
//                     sender.send_trace("Package has `check` command").await;
//
//                     // ╭─────────────────────────────────────╮
//                     // │ Step 4: Execute the `check` command │
//                     // ╰─────────────────────────────────────╯
//                     sender
//                         .send_progress(
//                             *step,
//                             total_steps,
//                             "Executing package's `check` command: `{check_cmd}`",
//                         )
//                         .await;
//                     *step += 1;
//
//                     // Run the check command
//                     match command_runner
//                         .execute_with_timeout(check_cmd, config.command_timeout())
//                         .await
//                     {
//                         Ok(output) => {
//                             if config.verbose() {
//                                 if !output.stdout_str().trim().is_empty() {
//                                     sender
//                                         .send_info(ConsoleOutput::Stdout(
//                                             output.stdout_str().to_string(),
//                                         ))
//                                         .await;
//                                 }
//                                 if !output.stderr_str().trim().is_empty() {
//                                     sender
//                                         .send_info(ConsoleOutput::Stderr(
//                                             output.stderr_str().to_string(),
//                                         ))
//                                         .await;
//                                 }
//                             }
//
//                             // ╭───────────────────────────────────────────╮
//                             // │ Step 5: Examine the command's return code │
//                             // ╰───────────────────────────────────────────╯
//                             if output.is_success() {
//                                 sender
//                                     .send_progress(*step, total_steps, "Package is installed")
//                                     .await;
//                             } else {
//                                 sender
//                                     .send_progress(
//                                         *step,
//                                         total_steps,
//                                         format!(
//                                             "Package '{}' not installed (exit code: {})",
//                                             package_name,
//                                             output.exit_code()
//                                         ),
//                                     )
//                                     .await;
//                             }
//                         }
//                         Err(error) => {
//                             sender
//                                 .send_error(error, "Failed to execute check command")
//                                 .await;
//                         }
//                     }
//                 } else {
//                     sender
//                         .send_progress(*step, total_steps, "Package does not have `check` command")
//                         .await;
//                 }
//
//                 Ok("`check` completed".into())
//             } else {
//                 // Environment not defined for this package
//                 sender
//                     .send_warning(format!(
//                         "Package '{}' does not support environment '{}'",
//                         package_name, current_env
//                     ))
//                     .await;
//                 Ok("Nothing to do".into())
//             }
//         }
//         Err(e) => {
//             sender
//                 .send_error(e, "Error fetching package from repository")
//                 .await;
//
//             Err("Unable to run package's `check` command".into())
//         }
//     }
// }

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
    }

    Ok("`check` completed".into())
}
