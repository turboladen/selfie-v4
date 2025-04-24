use std::fmt::Display;

use console::style;
use selfie::{
    config::AppConfig,
    fs::real::RealFileSystem,
    package::{port::PackageRepository, repository::YamlPackageRepository},
};

use crate::{
    commands::{ReportError, package::PackageRepoErrorReporter, report_with_style},
    tables::ValidationTableReporter,
    terminal_progress_reporter::TerminalProgressReporter,
};

pub(crate) fn handle_validate(
    package_name: &str,
    config: &AppConfig,
    reporter: TerminalProgressReporter,
) -> i32 {
    fn format_table_key<T: Display>(key: T, use_colors: bool) -> String {
        if use_colors {
            style(key).magenta().bold().to_string()
        } else {
            key.to_string()
        }
    }
    reporter.report_info(format!(
        "Validating package '{}' in environment: {}",
        package_name,
        config.environment()
    ));

    let repo = YamlPackageRepository::new(RealFileSystem, config.package_directory());

    match repo.get_package(package_name) {
        Ok(package) => {
            let validation_result = package.validate(config.environment());

            if validation_result.issues().has_errors() {
                reporter.report_error("Validation failed.");

                let mut table_reporter = ValidationTableReporter::new();
                table_reporter
                    .setup(vec!["Category", "Field", "Message", "Suggestion"])
                    .add_validation_errors(&validation_result.issues().errors(), &reporter)
                    .add_validation_warnings(&validation_result.issues().warnings(), &reporter)
                    .print();
                1
            } else if validation_result.issues().has_warnings() {
                let mut table_reporter = ValidationTableReporter::new();
                table_reporter
                    .setup(vec!["Category", "Field", "Message", "Suggestion"])
                    .add_validation_warnings(&validation_result.issues().warnings(), &reporter)
                    .print();
                0
            } else {
                reporter.report_success("Package is valid.");

                report_with_style("name:", package.name());
                report_with_style("version:", package.version());
                report_with_style("homepage:", package.homepage().unwrap_or_default());
                report_with_style("description:", package.description().unwrap_or_default());
                report_with_style("environments:", "");

                for (name, env_config) in package.environments() {
                    report_with_style(format!("- {name}"), "");

                    let mut env_table = ValidationTableReporter::new();
                    env_table.setup(vec!["Key", "Value"]);

                    let install_key = format_table_key("install", config.use_colors());
                    let check_key = format_table_key("check", config.use_colors());
                    let dependencies_key = format_table_key("dependencies", config.use_colors());

                    env_table.add_row(vec![&install_key, env_config.install()]);
                    env_table.add_row(vec![&check_key, env_config.check().unwrap_or_default()]);
                    env_table.add_row(vec![
                        &dependencies_key,
                        &env_config.dependencies().join(", "),
                    ]);
                    env_table.print();
                }

                0
            }
        }
        Err(e) => {
            let repo: &dyn PackageRepository = &repo;

            PackageRepoErrorReporter::new(e, repo, reporter).report_error();
            1
        }
    }
}
