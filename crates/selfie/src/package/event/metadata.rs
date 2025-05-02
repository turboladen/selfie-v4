use std::{fmt, time::Instant};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Metadata included with every event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata<T> {
    /// Unique ID for the operation that generated this event
    operation_id: Uuid,

    /// Timestamp when the event was created
    #[serde(skip, default = "default_instant")]
    timestamp: Instant,

    /// Operation type (install, validate, etc.)
    operation_type: OperationType,

    /// Command-specific metadata
    command_metadata: T,
}

impl<T: Clone> EventMetadata<T> {
    pub fn new(operation_type: OperationType, command_metadata: T) -> Self {
        Self {
            operation_id: Uuid::new_v4(),
            timestamp: Instant::now(),
            operation_type,
            command_metadata,
        }
    }

    pub fn touch_and_clone(&self) -> Self {
        let mut s = self.clone();
        s.timestamp = Instant::now();
        s
    }

    pub fn operation_id(&self) -> Uuid {
        self.operation_id
    }

    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }

    pub fn operation_type(&self) -> OperationType {
        self.operation_type
    }

    pub fn command_metadata(&self) -> &T {
        &self.command_metadata
    }
}

fn default_instant() -> Instant {
    Instant::now()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationType {
    ConfigValidate,
    PackageCheck,
    PackageInfo,
    PackageInstall,
    PackageList,
    PackageValidate,
}

impl fmt::Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigValidate => f.write_str("config_validate"),
            Self::PackageCheck => f.write_str("package_check"),
            Self::PackageInfo => f.write_str("package_info"),
            Self::PackageInstall => f.write_str("package_install"),
            Self::PackageList => f.write_str("package_list"),
            Self::PackageValidate => f.write_str("package_validate"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CheckMetadata {
    environment: String,
    package_name: String,
}

impl CheckMetadata {
    pub fn new(environment: String, package_name: String) -> Self {
        Self {
            environment,
            package_name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstallMetadata {
    environment: String,
    package_name: String,
}

impl InstallMetadata {
    pub fn new(environment: String, package_name: String) -> Self {
        Self {
            environment,
            package_name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InfoMetadata;

#[derive(Debug, Clone)]
pub struct ValidateMetadata;

#[derive(Debug, Clone)]
pub struct ListMetadata;

#[derive(Debug, Clone)]
pub struct CreateMetadata;
