use selfie::{domain::config::AppConfig, ports::config_loader::ApplyToConfg};

use crate::cli::ClapCli;

impl ApplyToConfg for ClapCli {
    fn apply_to_config(&self, mut config: AppConfig) -> AppConfig {
        // Override environment if specified
        if let Some(env) = self.environment.as_ref() {
            *config.environment_mut() = env.clone();
        }

        // Override package directory if specified
        if let Some(dir) = self.package_directory.as_ref() {
            *config.package_directory_mut() = dir.clone();
        }

        // Apply UI settings
        *config.verbose_mut() = self.verbose;
        *config.use_colors_mut() = !self.no_color;

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use selfie::domain::config::AppConfigBuilder;
    use std::path::PathBuf;

    // Helper to create CLI args
    struct FakeArgs {
        environment: Option<&'static str>,
        package_directory: Option<&'static str>,
        verbose: bool,
        no_color: bool,
    }

    impl FakeArgs {
        fn into_cli(self) -> ClapCli {
            let mut args = vec!["selfie"];

            if let Some(env) = self.environment {
                args.push("--environment");
                args.push(env);
            }

            if let Some(dir) = self.package_directory {
                args.push("--package-directory");
                args.push(dir);
            }

            if self.verbose {
                args.push("--verbose");
            }

            if self.no_color {
                args.push("--no-color");
            }

            // Add a required subcommand
            args.push("config");
            args.push("validate");

            ClapCli::parse_from(args)
        }
    }

    #[test]
    fn test_apply_cli_args_environment_override() {
        let config = AppConfigBuilder::default()
            .environment("original-env")
            .package_directory("/original/path")
            .build();

        let args = FakeArgs {
            environment: Some("cli-env"),
            package_directory: None,
            verbose: false,
            no_color: false,
        }
        .into_cli();

        let updated = args.apply_to_config(config);

        assert_eq!(updated.environment(), "cli-env");
        assert_eq!(
            updated.package_directory(),
            &PathBuf::from("/original/path")
        );
        assert!(!updated.verbose());
        assert!(updated.use_colors());
    }

    #[test]
    fn test_apply_cli_args_package_dir_override() {
        let config = AppConfigBuilder::default()
            .environment("original-env")
            .package_directory("/original/path")
            .build();

        let args = FakeArgs {
            environment: None,
            package_directory: Some("/cli/path"),
            verbose: false,
            no_color: false,
        }
        .into_cli();

        let updated = args.apply_to_config(config);

        assert_eq!(updated.environment(), "original-env");
        assert_eq!(updated.package_directory(), &PathBuf::from("/cli/path"));
        assert!(!updated.verbose());
        assert!(updated.use_colors());
    }

    #[test]
    fn test_apply_cli_args_ui_settings() {
        let config = AppConfigBuilder::default()
            .environment("original-env")
            .package_directory("/original/path")
            .verbose(false)
            .use_colors(true)
            .build();

        let args = FakeArgs {
            environment: None,
            package_directory: None,
            verbose: true,
            no_color: true,
        }
        .into_cli();

        let updated = args.apply_to_config(config);

        assert_eq!(updated.environment(), "original-env");
        assert_eq!(
            updated.package_directory(),
            &PathBuf::from("/original/path")
        );
        assert!(updated.verbose());
        assert!(!updated.use_colors());
    }

    #[test]
    fn test_apply_cli_args_multiple_overrides() {
        let config = AppConfigBuilder::default()
            .environment("original-env")
            .package_directory("/original/path")
            .verbose(false)
            .use_colors(true)
            .build();

        let args = FakeArgs {
            environment: Some("cli-env"),
            package_directory: Some("/cli/path"),
            verbose: true,
            no_color: true,
        }
        .into_cli();

        let updated = args.apply_to_config(config);

        assert_eq!(updated.environment(), "cli-env");
        assert_eq!(updated.package_directory(), &PathBuf::from("/cli/path"));
        assert!(updated.verbose());
        assert!(!updated.use_colors());
    }

    #[test]
    fn test_apply_cli_args_no_overrides() {
        let config = AppConfigBuilder::default()
            .environment("original-env")
            .package_directory("/original/path")
            .verbose(false)
            .use_colors(true)
            .build();

        let args = FakeArgs {
            environment: None,
            package_directory: None,
            verbose: false,
            no_color: false,
        }
        .into_cli();

        let updated = args.apply_to_config(config);

        // Config should remain unchanged
        assert_eq!(updated.environment(), "original-env");
        assert_eq!(
            updated.package_directory(),
            &PathBuf::from("/original/path")
        );
        assert!(!updated.verbose());
        assert!(updated.use_colors());
    }

    #[test]
    fn test_apply_cli_args_preserves_other_settings() {
        // Create config with non-default values for settings not affected by CLI
        let config = AppConfigBuilder::default()
            .environment("original-env")
            .package_directory("/original/path")
            .command_timeout_unchecked(120)
            .stop_on_error(false)
            .max_parallel_unchecked(8)
            .use_colors(true)
            .build();

        let args = FakeArgs {
            environment: Some("cli-env"),
            package_directory: Some("/cli/path"),
            verbose: true,
            no_color: true,
        }
        .into_cli();

        let updated = args.apply_to_config(config);

        // CLI-modifiable settings should be updated
        assert_eq!(updated.environment(), "cli-env");
        assert_eq!(updated.package_directory(), &PathBuf::from("/cli/path"));
        assert!(updated.verbose());
        assert!(!updated.use_colors());

        // Other settings should be preserved
        assert_eq!(updated.command_timeout().as_secs(), 120);
        assert!(!updated.stop_on_error());
        assert_eq!(updated.max_parallel().get(), 8);
    }
}
