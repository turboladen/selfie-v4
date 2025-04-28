use selfie::{
    commands::{ShellCommandRunner, runner::CommandRunner},
    config::AppConfig,
    fs::real::RealFileSystem,
    package::{Package, port::PackageRepository, repository::YamlPackageRepository},
};

use crate::{
    commands::package::handle_package_repo_error,
    terminal_progress_reporter::TerminalProgressReporter,
};

pub(crate) async fn handle_check(
    package_name: &str,
    config: &AppConfig,
    reporter: TerminalProgressReporter,
) -> i32 {
    tracing::debug!("Running check command for package: {}", package_name);

    let repo = YamlPackageRepository::new(RealFileSystem, config.package_directory());
    let command_runner = ShellCommandRunner::new("/bin/sh", config.command_timeout());

    match repo.get_package(package_name) {
        Ok(package) => {
            execute_package_check(&package, package_name, config, &command_runner, reporter).await
        }
        Err(e) => {
            handle_package_repo_error(e, &repo, reporter);
            1
        }
    }
}

async fn execute_package_check(
    package: &Package,
    package_name: &str,
    config: &AppConfig,
    command_runner: &ShellCommandRunner,
    reporter: TerminalProgressReporter,
) -> i32 {
    // Get the environment configuration for the current environment
    let current_env = config.environment();

    if let Some(env_config) = package.environments().get(current_env) {
        if let Some(check_cmd) = env_config.check() {
            execute_check_command(
                package_name,
                current_env,
                check_cmd,
                config,
                command_runner,
                reporter,
            )
            .await
        } else {
            // No check command defined for this environment
            reporter.report_info(format!(
                "No check command defined for package '{}' in environment '{}'",
                package_name, current_env
            ));
            // Return success since there's no check command
            0
        }
    } else {
        // Environment not defined for this package
        reporter.report_warning(format!(
            "Package '{}' does not support environment '{}'",
            package_name, current_env
        ));
        1
    }
}

async fn execute_check_command(
    package_name: &str,
    environment: &str,
    check_cmd: &str,
    config: &AppConfig,
    command_runner: &ShellCommandRunner,
    reporter: TerminalProgressReporter,
) -> i32 {
    // Inform user what we're doing
    reporter.report_info(format!(
        "Checking package '{}' in environment '{}'",
        package_name, environment
    ));

    // Run the check command
    match command_runner
        .execute_with_timeout(check_cmd, config.command_timeout())
        .await
    {
        Ok(output) => process_command_output(output, package_name, config, reporter),
        Err(error) => {
            reporter.report_error(format!("Failed to execute check command: {}", error));
            1
        }
    }
}

fn process_command_output(
    output: selfie::commands::runner::CommandOutput,
    package_name: &str,
    config: &AppConfig,
    reporter: TerminalProgressReporter,
) -> i32 {
    // If verbose mode is enabled, print the command output
    if config.verbose() {
        if !output.stdout_str().trim().is_empty() {
            println!("{}", output.stdout_str());
        }
        if !output.stderr_str().trim().is_empty() {
            eprintln!("{}", output.stderr_str());
        }
    }

    // Return success/failure based on the command's exit code
    if output.is_success() {
        reporter.report_success(format!("Package '{}' is installed", package_name));
        0
    } else {
        reporter.report_info(format!(
            "Package '{}' not installed (exit code: {})",
            package_name,
            output.exit_code()
        ));
        output.exit_code()
    }
}
