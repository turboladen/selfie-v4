use std::{collections::HashMap, path::PathBuf};

use super::{EnvironmentConfig, Package};

#[derive(Default)]
pub struct PackageBuilder {
    name: String,
    version: String,
    homepage: Option<String>,
    description: Option<String>,
    environments: HashMap<String, EnvironmentConfig>,
    path: PathBuf,
}

impl PackageBuilder {
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }

    pub fn homepage(mut self, homepage: &str) -> Self {
        self.homepage = Some(homepage.to_string());
        self
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn environment<T, F>(mut self, name: T, env_builder: F) -> Self
    where
        T: ToString,
        F: Fn(EnvironmentConfigBuilder) -> EnvironmentConfigBuilder,
    {
        self.environments.insert(
            name.to_string(),
            env_builder(EnvironmentConfigBuilder::default()).build(),
        );
        self
    }

    pub fn build(self) -> Package {
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

#[derive(Default)]
pub struct EnvironmentConfigBuilder {
    install: String,
    check: Option<String>,
    dependencies: Vec<String>,
}
impl EnvironmentConfigBuilder {
    pub fn install<T: ToString>(mut self, install: T) -> Self {
        self.install = install.to_string();
        self
    }

    pub fn check<T: ToString>(mut self, check: Option<T>) -> Self {
        self.check = check.map(|c| c.to_string());
        self
    }

    pub fn dependencies<T: ToString>(mut self, dependencies: Vec<T>) -> Self {
        self.dependencies = dependencies.into_iter().map(|d| d.to_string()).collect();
        self
    }

    pub fn build(self) -> EnvironmentConfig {
        EnvironmentConfig {
            install: self.install,
            check: self.check,
            dependencies: self.dependencies,
        }
    }
}
