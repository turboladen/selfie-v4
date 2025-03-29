use std::{collections::HashMap, path::PathBuf};

use super::{EnvironmentConfig, Package};

#[derive(Default)]
pub(crate) struct PackageBuilder {
    name: String,
    version: String,
    homepage: Option<String>,
    description: Option<String>,
    environments: HashMap<String, EnvironmentConfig>,
    path: PathBuf,
}

impl PackageBuilder {
    pub(crate) fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub(crate) fn version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }

    pub(crate) fn homepage(mut self, homepage: &str) -> Self {
        self.homepage = Some(homepage.to_string());
        self
    }

    pub(crate) fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub(crate) fn environment<T>(mut self, name: T, install_command: &str) -> Self
    where
        T: ToString,
    {
        self.environments.insert(
            name.to_string(),
            EnvironmentConfig {
                install: install_command.to_string(),
                check: None,
                dependencies: Vec::new(),
            },
        );
        self
    }

    pub(crate) fn build(self) -> Package {
        Package::new(
            self.name,
            self.version,
            self.homepage,
            self.description,
            self.environments,
            self.path,
        )
    }
}
