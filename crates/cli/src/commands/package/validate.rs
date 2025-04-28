use std::fmt::Display;

use selfie::{
    config::AppConfig,
    fs::real::RealFileSystem,
    package::{port::PackageRepository, repository::YamlPackageRepository},
    validation::ValidationIssues,
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
    reporter.report_info(format!(
        "Validating package '{}' in environment: {}",
        package_name,
        config.environment()
    ));

    let repo = YamlPackageRepository::new(RealFileSystem, config.package_directory());

    match repo.get_package(package_name) {
        Ok(package) => {
            let validation_result = package.validate(config.environment());
            let issues = validation_result.issues();

            if issues.has_errors() {
                handle_validation_errors(issues, &reporter)
            } else if issues.has_warnings() {
                handle_validation_warnings(issues, &reporter)
            } else {
                display_valid_package_info(&package, config, &reporter)
            }
        }
        Err(e) => {
            let repo: &dyn PackageRepository = &repo;
            PackageRepoErrorReporter::new(e, repo, reporter).report_error();
            1
        }
    }
}

fn format_table_key<T: Display>(key: T, use_colors: bool) -> String {
    use crate::formatters::{FieldStyle, format_field};
    format_field(key, FieldStyle::Title, use_colors)
}

fn handle_validation_errors(issues: &ValidationIssues, reporter: &TerminalProgressReporter) -> i32 {
    reporter.report_error("Validation failed.");

    let mut table_reporter = ValidationTableReporter::new();
    table_reporter
        .setup(vec!["Category", "Field", "Message", "Suggestion"])
        .add_validation_errors(&issues.errors(), reporter)
        .add_validation_warnings(&issues.warnings(), reporter)
        .print();
    1
}

fn handle_validation_warnings(
    issues: &ValidationIssues,
    reporter: &TerminalProgressReporter,
) -> i32 {
    let mut table_reporter = ValidationTableReporter::new();
    table_reporter
        .setup(vec!["Category", "Field", "Message", "Suggestion"])
        .add_validation_warnings(&issues.warnings(), reporter)
        .print();
    0
}

fn display_valid_package_info(
    package: &selfie::package::Package,
    config: &AppConfig,
    reporter: &TerminalProgressReporter,
) -> i32 {
    reporter.report_success("Package is valid.");

    report_with_style("name:", package.name());
    report_with_style("version:", package.version());
    report_with_style("homepage:", package.homepage().unwrap_or_default());
    report_with_style("description:", package.description().unwrap_or_default());
    report_with_style("environments:", "");

    display_environments(package, config);
    0
}

fn display_environments(package: &selfie::package::Package, config: &AppConfig) {
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
}
