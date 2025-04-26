pub(crate) mod create;
pub(crate) mod info;
pub(crate) mod install;
pub(crate) mod list;
pub(crate) mod validate;

use std::path::Path;

use selfie::package::port::{
    PackageError, PackageListError, PackageParseError, PackageRepoError, PackageRepository,
};

use crate::terminal_progress_reporter::TerminalProgressReporter;

use self::list::ListCommand;

use super::ReportError;

pub(crate) struct PackageRepoErrorReporter<'a> {
    error: PackageRepoError,
    repo: &'a dyn PackageRepository,
    reporter: TerminalProgressReporter,
}

impl<'a> PackageRepoErrorReporter<'a> {
    pub(crate) fn new(
        error: PackageRepoError,
        repo: &'a dyn PackageRepository,
        reporter: TerminalProgressReporter,
    ) -> Self {
        Self {
            error,
            repo,
            reporter,
        }
    }
}

impl ReportError<ListCommand<'_>> for PackageRepoErrorReporter<'_> {
    fn report_error(self) {
        match self.error {
            PackageRepoError::PackageError(e) => {
                PackageErrorReporter::new(e, self.repo, self.reporter).report_error();
            }
            PackageRepoError::PackageListError(e) => {
                PackageListErrorReporter::new(e, self.reporter).report_error();
            }
        }
    }
}

pub(crate) struct PackageErrorReporter<'a> {
    package_error: PackageError,
    repo: &'a dyn PackageRepository,
    reporter: TerminalProgressReporter,
}

impl<'a> PackageErrorReporter<'a> {
    pub(crate) fn new(
        package_error: PackageError,
        repo: &'a dyn PackageRepository,
        reporter: TerminalProgressReporter,
    ) -> Self {
        Self {
            package_error,
            repo,
            reporter,
        }
    }
}

impl ReportError<ListCommand<'_>> for PackageErrorReporter<'_> {
    fn report_error(self) {
        match self.package_error {
            PackageError::PackageNotFound {
                ref name,
                ref packages_path,
            } => {
                handle_package_not_found(name, packages_path, self.repo, self.reporter);
            }
            PackageError::MultiplePackagesFound {
                ref name,
                ref packages_path,
            } => {
                handle_multiple_packages_found(name, packages_path, self.reporter);
            }
            PackageError::ParseError {
                ref name,
                ref packages_path,
                ref source,
            } => {
                handle_parse_error(name, source, packages_path, self.reporter);
            }
        }
    }
}

pub(crate) struct PackageListErrorReporter {
    error: PackageListError,
    reporter: TerminalProgressReporter,
}

impl PackageListErrorReporter {
    pub(crate) fn new(error: PackageListError, reporter: TerminalProgressReporter) -> Self {
        Self { error, reporter }
    }
}

impl ReportError<ListCommand<'_>> for PackageListErrorReporter {
    fn report_error(self) {
        match self.error {
            PackageListError::IoError(error) => {
                handle_io_error(error, self.reporter);
            }
            PackageListError::PackageDirectoryNotFound(ref dir) => {
                handle_directory_not_found(dir, self.reporter);
            }
        }
    }
}

fn handle_package_repo_error(
    e: PackageRepoError,
    repo: &dyn PackageRepository,
    reporter: TerminalProgressReporter,
) {
    match e {
        PackageRepoError::PackageError(pe) => match pe {
            PackageError::PackageNotFound {
                ref name,
                packages_path,
            } => {
                handle_package_not_found(name, &packages_path, repo, reporter);
            }
            PackageError::MultiplePackagesFound {
                ref name,
                packages_path,
            } => {
                handle_multiple_packages_found(name, &packages_path, reporter);
            }
            PackageError::ParseError {
                ref name,
                packages_path,
                source,
            } => {
                handle_parse_error(name, &source, &packages_path, reporter);
            }
        },
        PackageRepoError::PackageListError(ple) => match ple {
            PackageListError::IoError(error) => {
                handle_io_error(error, reporter);
            }
            PackageListError::PackageDirectoryNotFound(dir) => {
                handle_directory_not_found(&dir, reporter);
            }
        },
    }
}

fn handle_package_not_found(
    name: &str,
    packages_path: &Path,
    repo: &dyn PackageRepository,
    reporter: TerminalProgressReporter,
) {
    let msg = format!("Package Not Found: {name}");

    // Print the error header
    reporter.report_error(msg);

    // Print where we looked
    reporter.report_info(format!("Searched in: {}", packages_path.display()));

    // Try to find similar package names to suggest
    if let Ok(available_packages) = repo.available_packages() {
        if !available_packages.is_empty() {
            // Add available packages information
            let msg = if available_packages.len() <= 5 {
                format!("Available packages: {}", available_packages.join(", "))
            } else {
                format!(
                    "Available packages: {}, and {} more...",
                    available_packages[..5].join(", "),
                    available_packages.len() - 5
                )
            };
            reporter.report_info(msg);
        }
    }

    // Add help with suggestion
    reporter.report_suggestion("Run 'selfie package list' to see all available packages");
}

fn handle_multiple_packages_found(
    name: &str,
    packages_path: &Path,
    reporter: TerminalProgressReporter,
) {
    reporter.report_error("✗ Multiple Packages Found");
    reporter.report_info(format!(
        "Multiple package files found with name '{name}' in package directory '{}'",
        packages_path.display()
    ));
    reporter
        .report_info("This can happen if you have both .yaml and .yml files for the same package.");

    reporter
        .report_suggestion("Use only one file extension (.yaml or .yml) for your package files");
}

fn handle_parse_error(
    package_name: &str,
    source: &PackageParseError,
    packages_path: &Path,
    reporter: TerminalProgressReporter,
) {
    reporter.report_error("✗ Package Parse Error");
    reporter.report_info(format!("Failed to parse package file for '{package_name}'"));
    reporter.report_info(format!("Error: {source}"));
    reporter.report_info(format!("Location: {}", packages_path.display()));

    reporter
        .report_suggestion("Check the format of your package file and make sure it's valid YAML");
}

fn handle_io_error(error: std::io::Error, reporter: TerminalProgressReporter) {
    reporter.report_error("✗ I/O Error");
    reporter.report_info("Failed to read package information due to an I/O error:");
    reporter.report_info(format!("{error}"));

    reporter.report_suggestion(
        "Check if the file system is accessible and you have proper permissions",
    );
}

fn handle_directory_not_found(path: &Path, reporter: TerminalProgressReporter) {
    reporter.report_error("✗ Package Directory Not Found");

    reporter.report_info("The package directory does not exist:");
    reporter.report_info(format!("{}", path.display()));

    reporter
        .report_suggestion("Create the directory or configure a different package directory path");
}
