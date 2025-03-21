pub mod yaml;

pub use self::yaml::Yaml;

use std::path::PathBuf;

use thiserror::Error;

use crate::{config::AppConfig, filesystem::FileSystemError};

/// Port for loading configuration from disk
///
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait ConfigLoader: Send + Sync {
    /// Load configuration from standard locations
    ///
    fn load_config(&self) -> Result<AppConfig, ConfigLoadError>;

    /// Find possible configuration file paths
    ///
    fn find_config_file_paths(&self) -> Result<Vec<std::path::PathBuf>, PathBuf>;
}

#[derive(Error, Debug)]
pub enum ConfigLoadError {
    #[error(transparent)]
    FileSystemError(#[from] FileSystemError),

    #[error("No configuration file found in locations: {searched}")]
    NotFound { searched: PathBuf },

    #[error("Multiple configuration files found: {}", .0.join(", "))]
    MultipleFound(Vec<String>),

    #[error(transparent)]
    ConfigError(#[from] ::config::ConfigError),
}

/// This trait allows for applying runtime CLI arguments on top of the configuration that the app
/// read from the config file.
///
pub trait ApplyToConfg {
    /// Implement this method such that the arguments in `args` are applied after/on top of the
    /// configuration that was loaded from the config file.
    ///
    fn apply_to_config(&self, config: AppConfig) -> AppConfig;
}
