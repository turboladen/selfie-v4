use std::path::PathBuf;

use crate::validation::{ValidationErrorCategory, ValidationIssue, ValidationLevel};

use super::Package;

/// Results of a package validation
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ValidationResult {
    /// The package that was validated
    ///
    pub(crate) package_name: String,

    /// The package file path
    ///
    pub(crate) package_path: Option<PathBuf>,

    /// List of validation issues found
    ///
    pub(crate) issues: Vec<ValidationIssue>,
}

impl ValidationResult {
    pub fn issues(&self) -> &[ValidationIssue] {
        &self.issues
    }

    /// Returns true if the validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        !self.has_errors()
    }

    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }

    /// Returns true if the validation has errors (warnings are okay)
    pub fn has_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|issue| matches!(issue.level, ValidationLevel::Error))
    }

    /// Get all errors (not warnings)
    ///
    pub fn errors(&self) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|issue| issue.level == ValidationLevel::Error)
            .collect()
    }

    /// Get all warnings (not errors)
    pub fn warnings(&self) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|issue| issue.level == ValidationLevel::Warning)
            .collect()
    }

    /// Get issues by category
    pub fn issues_by_category(&self, category: &ValidationErrorCategory) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|issue| issue.category == *category)
            .collect()
    }
}

impl Package {
    /// Perform all basic domain validations
    pub fn validate(&self, current_env: &str) -> ValidationResult {
        let mut issues = Vec::new();

        issues.extend(self.validate_required_fields());
        issues.extend(self.validate_urls());
        issues.extend(self.validate_environments_contents(current_env));
        issues.extend(self.validate_command_syntax());

        ValidationResult {
            package_name: self.name.clone(),
            package_path: Some(self.path.clone()),
            issues,
        }
    }

    pub(crate) fn validate_required_fields(&self) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Check name
        if let Err(issue) = self.validate_name() {
            issues.push(issue);
        }

        // Check version
        if let Err(issue) = self.validate_version() {
            issues.push(issue);
        }

        // Check environments
        if let Err(issue) = self.validate_environments_exists() {
            issues.push(issue);
        }

        issues
    }

    fn validate_name(&self) -> Result<(), ValidationIssue> {
        /// Check if a string is a valid package name
        fn is_valid_package_name(name: &str) -> bool {
            // Package names should only contain alphanumeric chars, hyphens, and underscores
            !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        }

        if self.name.is_empty() {
            return Err(ValidationIssue::error(
                ValidationErrorCategory::RequiredField,
                "name",
                "Package name is required",
                Some("Add 'name: your-package-name' to the package file."),
            ));
        } else if !is_valid_package_name(&self.name) {
            return Err(ValidationIssue::error(
                ValidationErrorCategory::InvalidValue,
                "name",
                "Package name contains invalid characters",
                Some("Use only alphanumeric characters, hyphens, and underscores."),
            ));
        }

        Ok(())
    }

    fn validate_version(&self) -> Result<(), ValidationIssue> {
        fn is_valid_version(version: &str) -> bool {
            // Simple check for semver format: major.minor.patch
            let semver_regex = regex::Regex::new(r"^\d+\.\d+\.\d+").unwrap();
            semver_regex.is_match(version)
        }

        if self.version.is_empty() {
            return Err(ValidationIssue::error(
                ValidationErrorCategory::RequiredField,
                "version",
                "Package version is required",
                Some("Add 'version: \"0.1.0\"' to the package file."),
            ));
        } else if !is_valid_version(&self.version) {
            return Err(ValidationIssue::warning(
                ValidationErrorCategory::InvalidValue,
                "version",
                "Package version should follow semantic versioning",
                Some("Consider using a semantic version like '1.0.0'."),
            ));
        }

        Ok(())
    }

    fn validate_environments_exists(&self) -> Result<(), ValidationIssue> {
        if self.environments.is_empty() {
            Err(ValidationIssue::error(
                ValidationErrorCategory::RequiredField,
                "environments",
                "At least one environment must be defined",
                Some("Add an 'environments' section with at least one environment."),
            ))
        } else {
            Ok(())
        }
    }

    fn validate_environments_contents(&self, current_env: &str) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Check if current environment is configured
        if !current_env.is_empty() && !self.environments.contains_key(current_env) {
            issues.push(ValidationIssue::warning(
                ValidationErrorCategory::Environment,
                "environments",
                &format!("Current environment '{}' is not configured", current_env),
                Some(&format!(
                    "Add an environment section for '{}' if needed for this environment.",
                    current_env
                )),
            ));
        }

        // Validate each environment's required fields
        for (env_name, env_config) in &self.environments {
            if env_config.install.is_empty() {
                issues.push(ValidationIssue::error(
                    ValidationErrorCategory::RequiredField,
                    &format!("environments.{}.install", env_name),
                    "Install command is required",
                    Some("Add an install command like 'brew install package-name'."),
                ));
            }

            // Validate dependencies (check for empty names)
            for (i, dep) in env_config.dependencies.iter().enumerate() {
                if dep.is_empty() {
                    issues.push(ValidationIssue::error(
                        ValidationErrorCategory::InvalidValue,
                        &format!("environments.{}.dependencies[{}]", env_name, i),
                        "Dependency name cannot be empty",
                        Some("Remove the empty dependency or provide a valid name."),
                    ));
                }
            }
        }

        issues
    }

    /// Validate URL fields
    fn validate_urls(&self) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Check homepage URL if present
        if let Some(homepage) = &self.homepage {
            match url::Url::parse(homepage) {
                Ok(url) => {
                    // Check scheme
                    if url.scheme() != "http" && url.scheme() != "https" {
                        issues.push(ValidationIssue::warning(
                            ValidationErrorCategory::UrlFormat,
                            "homepage",
                            &format!(
                                "URL should use http or https scheme, found: {}",
                                url.scheme()
                            ),
                            Some("Use https:// prefix for the URL."),
                        ));
                    }
                }
                Err(err) => {
                    issues.push(ValidationIssue::error(
                        ValidationErrorCategory::UrlFormat,
                        "homepage",
                        &format!("Invalid URL format: {}", err),
                        Some("Provide a valid URL with http:// or https:// prefix."),
                    ));
                }
            }
        }

        issues
    }
    /// Basic command syntax validation that doesn't require external dependencies
    pub(crate) fn validate_command_syntax(&self) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for (env_name, env_config) in &self.environments {
            // Check install command syntax
            issues.extend(Self::validate_single_command(
                &env_config.install,
                &format!("environments.{}.install", env_name),
            ));

            // Check check command syntax if present
            if let Some(check_cmd) = &env_config.check {
                issues.extend(Self::validate_single_command(
                    check_cmd,
                    &format!("environments.{}.check", env_name),
                ));
            }
        }

        issues
    }

    /// Validate a single command for syntax issues
    fn validate_single_command(command: &str, field_name: &str) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Check for unmatched quotes
        let mut in_single_quotes = false;
        let mut in_double_quotes = false;

        for c in command.chars() {
            match c {
                '\'' if !in_double_quotes => in_single_quotes = !in_single_quotes,
                '"' if !in_single_quotes => in_double_quotes = !in_double_quotes,
                _ => {}
            }
        }

        if in_single_quotes {
            issues.push(ValidationIssue::error(
                ValidationErrorCategory::CommandSyntax,
                field_name,
                "Unmatched single quote in command",
                Some("Add a closing single quote (') to the command."),
            ));
        }

        if in_double_quotes {
            issues.push(ValidationIssue::error(
                ValidationErrorCategory::CommandSyntax,
                field_name,
                "Unmatched double quote in command",
                Some("Add a closing double quote (\") to the command."),
            ));
        }

        // Check for invalid pipe usage
        if command.contains("| |") {
            issues.push(ValidationIssue::error(
                ValidationErrorCategory::CommandSyntax,
                field_name,
                "Invalid pipe usage in command",
                Some("Remove duplicate pipe symbols."),
            ));
        }

        // Check for invalid redirections
        for redirect in &[">", ">>", "<"] {
            if command.contains(&format!("{} ", redirect))
                && !command.contains(&format!("{} /", redirect))
                && !command.contains(&format!("{} ~/", redirect))
            {
                issues.push(ValidationIssue::warning(
                    ValidationErrorCategory::CommandSyntax,
                    field_name,
                    &format!("Potential invalid redirection with {}", redirect),
                    Some("Ensure the redirection path is valid and absolute."),
                ));
            }
        }

        // Check for command injection risks with backticks
        if command.contains('`') {
            issues.push(ValidationIssue::warning(
                ValidationErrorCategory::CommandSyntax,
                field_name,
                "Contains command substitution with backticks",
                Some("Consider using $() for command substitution instead of backticks."),
            ));
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use crate::package::{EnvironmentConfig, builder::PackageBuilder};

    use super::*;

    #[test]
    fn test_validate_valid_package() {
        let package = PackageBuilder::default()
            .name("test-package")
            .version("1.0.0")
            .environment("test-env", |b| b.install("test install"))
            .build();

        assert!(package.validate("test-env").is_valid());
    }

    #[test]
    fn test_validate_urls() {
        // Test invalid URL
        let package = PackageBuilder::default()
            .name("test-package")
            .version("1.0.0")
            .homepage("not-a-valid-url")
            .environment("test-env", |b| b.install("test install"))
            .build();

        let issues = package.validate_urls();
        assert_eq!(issues.len(), 1);
        assert!(issues[0].category == ValidationErrorCategory::UrlFormat);

        // Test valid URL but wrong scheme (ftp)
        let package = PackageBuilder::default()
            .name("test-package")
            .version("1.0.0")
            .homepage("ftp://example.com")
            .environment("test-env", |b| b.install("test install"))
            .build();

        let issues = package.validate_urls();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].level(), ValidationLevel::Warning);
        assert!(issues[0].message.contains("scheme"));

        // Test valid URL with correct scheme
        let package = PackageBuilder::default()
            .name("test-package")
            .version("1.0.0")
            .homepage("https://example.com")
            .environment("test-env", |b| b.install("test install"))
            .build();

        let issues = package.validate_urls();
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_validate_environments() {
        // Test missing current environment
        let package = PackageBuilder::default()
            .name("test-package")
            .version("1.0.0")
            .environment("other-env", |b| b.install("test install"))
            .build();

        let issues = package.validate_environments_contents("test-env");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].level(), ValidationLevel::Warning);
        assert!(issues[0].message.contains("not configured"));

        // Test empty install command
        let mut package = PackageBuilder::default()
            .name("test-package")
            .version("1.0.0")
            .build();

        let env_config = EnvironmentConfig {
            install: String::new(),
            check: None,
            dependencies: vec![],
        };

        package
            .environments
            .insert("test-env".to_string(), env_config);

        let issues = package.validate_environments_contents("test-env");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].level(), ValidationLevel::Error);
        assert!(issues[0].message.contains("required"));
    }

    #[test]
    fn test_validate_command_syntax() {
        // Test unmatched quote
        let package = PackageBuilder::default()
            .name("test-package")
            .version("1.0.0")
            .environment("test-env", |b| b.install("echo 'unmatched"))
            .build();

        let issues = package.validate_command_syntax();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].level(), ValidationLevel::Error);
        assert!(issues[0].message.contains("Unmatched single quote"));

        // Test invalid pipe
        let package = PackageBuilder::default()
            .name("test-package")
            .version("1.0.0")
            .environment("test-env", |b| b.install("echo test | | grep test"))
            .build();

        let issues = package.validate_command_syntax();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].level(), ValidationLevel::Error);
        assert!(issues[0].message.contains("Invalid pipe usage"));

        // Test backticks (warning)
        let package = PackageBuilder::default()
            .name("test-package")
            .version("1.0.0")
            .environment("test-env", |b| b.install("echo `date`"))
            .build();

        let issues = package.validate_command_syntax();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].level(), ValidationLevel::Warning);
        assert!(issues[0].message.contains("backticks"));
    }

    #[test]
    fn test_full_validate() {
        // Test a valid package
        let package = PackageBuilder::default()
            .name("test-package")
            .version("1.0.0")
            .homepage("https://example.com")
            .description("A test package")
            .environment("test-env", |b| b.install("echo test"))
            .build();

        let result = package.validate("test-env");
        assert!(!result.has_issues());

        // Test an invalid package with multiple issues
        let package = PackageBuilder::default()
            .name("")
            .version("")
            .homepage("invalid-url")
            .environment("other-env", |b| b.install("echo `test`"))
            .build();

        let result = package.validate("test-env");
        assert!(result.issues().len() >= 4); // At least 4 issues should be found
    }
}
