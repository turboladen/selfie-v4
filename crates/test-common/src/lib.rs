//! Common test utilities shared across all selfie crates.
//!
//! This crate provides standardized test helpers to eliminate code duplication
//! while maintaining test clarity and ergonomics.

pub mod config;
pub mod constants;
pub mod package;

// Re-export the most commonly used items for convenience
pub use config::{
    test_config, test_config_for_env, test_config_verbose, test_config_with_colors,
    test_config_with_dir,
};
pub use constants::*;
pub use package::{multi_env_test_package, simple_test_package, test_package_with_check};

// Re-export commonly used external dependencies for convenience
pub use selfie::{
    config::AppConfigBuilder,
    package::{Package, PackageBuilder},
};
pub use tempfile::TempDir;
