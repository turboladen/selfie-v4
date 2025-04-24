use console::style;
use selfie::{
    config::AppConfig,
    fs::real::RealFileSystem,
    package::{port::PackageRepository, repository::YamlPackageRepository},
};

use crate::{
    commands::{HandleCommand, ReportError},
    tables::PackageListTableReporter,
    terminal_progress_reporter::TerminalProgressReporter,
};

use super::PackageListErrorReporter;

pub(crate) struct ListCommand<'a> {
    config: &'a AppConfig,
    reporter: TerminalProgressReporter,
}

impl<'a> ListCommand<'a> {
    pub(crate) fn new(config: &'a AppConfig, reporter: TerminalProgressReporter) -> Self {
        Self { config, reporter }
    }
}

impl HandleCommand for ListCommand<'_> {
    fn handle_command(&self) -> i32 {
        let repo = YamlPackageRepository::new(RealFileSystem, self.config.package_directory());

        match repo.list_packages() {
            Ok(list_packages_output) => {
                let mut sorted_errors: Vec<_> = list_packages_output.invalid_packages().collect();
                sorted_errors.sort_by(|a, b| a.package_path().cmp(b.package_path()));

                let mut sorted_packages: Vec<_> = list_packages_output.valid_packages().collect();
                sorted_packages.sort_by(|a, b| a.name().cmp(b.name()));

                if sorted_packages.is_empty() {
                    self.reporter.report_info("No packages found.");

                    for error in sorted_errors {
                        self.reporter.report_error(format!(
                            "{}: {}",
                            error.package_path().display(),
                            error
                        ));
                    }

                    return 0;
                }

                let mut package_reporter = PackageListTableReporter::new();
                package_reporter.setup(vec!["Name", "Version", "Environments"]);

                for package in sorted_packages {
                    let package_name = if self.config.use_colors() {
                        style(package.name()).magenta().bold().to_string()
                    } else {
                        package.name().to_string()
                    };

                    let version = if self.config.use_colors() {
                        style(format!("v{}", package.version())).dim().to_string()
                    } else {
                        format!("v{}", package.version())
                    };

                    package_reporter.add_row(vec![
                        package_name,
                        version,
                        package
                            .environments()
                            .keys()
                            .map(|env_name| {
                                if env_name == self.config.environment() {
                                    let env = format!("*{env_name}");

                                    if self.config.use_colors() {
                                        style(env).bold().green().to_string()
                                    } else {
                                        env
                                    }
                                } else if self.config.use_colors() {
                                    style(env_name).dim().green().to_string()
                                } else {
                                    env_name.to_string()
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(",  "),
                    ]);
                }

                package_reporter.print();

                for error in sorted_errors {
                    self.reporter.report_error(format!(
                        "{}: {}",
                        error.package_path().display(),
                        error
                    ));
                }

                0
            }
            Err(e) => {
                PackageListErrorReporter::new(e, self.reporter).report_error();
                1
            }
        }
    }
}
