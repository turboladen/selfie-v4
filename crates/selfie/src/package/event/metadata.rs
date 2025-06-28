use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationType {
    ConfigValidate,
    PackageCheck,
    PackageCreate,
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
            Self::PackageCreate => f.write_str("package_create"),
            Self::PackageInfo => f.write_str("package_info"),
            Self::PackageInstall => f.write_str("package_install"),
            Self::PackageList => f.write_str("package_list"),
            Self::PackageValidate => f.write_str("package_validate"),
        }
    }
}
