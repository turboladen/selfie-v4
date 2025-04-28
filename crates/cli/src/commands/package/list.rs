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

    fn handle_packages_output(
        &self,
        list_packages_output: selfie::package::port::ListPackagesOutput,
    ) -> i32 {
        let sorted_errors = self.get_sorted_errors(&list_packages_output);
        let sorted_packages = self.get_sorted_packages(&list_packages_output);

        if sorted_packages.is_empty() {
            self.report_no_packages_found(&sorted_errors);
            return 0;
        }

        self.display_packages_table(&sorted_packages);
        self.report_invalid_packages(&sorted_errors);
        0
    }

    fn get_sorted_errors<'b>(
        &self,
        output: &'b selfie::package::port::ListPackagesOutput,
    ) -> Vec<&'b selfie::package::port::PackageParseError> {
        let mut sorted_errors: Vec<_> = output.invalid_packages().collect();
        sorted_errors.sort_by(|a, b| a.package_path().cmp(b.package_path()));
        sorted_errors
    }

    fn get_sorted_packages<'b>(
        &self,
        output: &'b selfie::package::port::ListPackagesOutput,
    ) -> Vec<&'b selfie::package::Package> {
        let mut sorted_packages: Vec<_> = output.valid_packages().collect();
        sorted_packages.sort_by(|a, b| a.name().cmp(b.name()));
        sorted_packages
    }

    fn report_no_packages_found(&self, errors: &[&selfie::package::port::PackageParseError]) {
        self.reporter.report_info("No packages found.");
        self.report_invalid_packages(errors);
    }

    fn report_invalid_packages(&self, errors: &[&selfie::package::port::PackageParseError]) {
        for error in errors {
            self.reporter
                .report_error(format!("{}: {}", error.package_path().display(), error));
        }
    }

    fn display_packages_table(&self, packages: &[&selfie::package::Package]) {
        let mut package_reporter = PackageListTableReporter::new();
        package_reporter.setup(vec!["Name", "Version", "Environments"]);

        for package in packages {
            package_reporter.add_row(self.format_package_row(package));
        }

        package_reporter.print();
    }

    fn format_package_row(&self, package: &selfie::package::Package) -> Vec<String> {
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

        vec![package_name, version, self.format_environments(package)]
    }

    fn format_environments(&self, package: &selfie::package::Package) -> String {
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
            .join(",  ")
    }
}

impl HandleCommand for ListCommand<'_> {
    fn handle_command(&self) -> i32 {
        let repo = YamlPackageRepository::new(RealFileSystem, self.config.package_directory());

        match repo.list_packages() {
            Ok(list_packages_output) => self.handle_packages_output(list_packages_output),
            Err(e) => {
                PackageListErrorReporter::new(e, self.reporter).report_error();
                1
            }
        }
    }
}
