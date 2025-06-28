//! Helps break down the pieces of running the `package check` command.
//!
//! This module now uses hexagonal architecture by delegating to application services
//! that contain the core business logic, separated from infrastructure concerns.

use std::borrow::Cow;

use crate::{
    commands::runner::CommandRunner,
    config::AppConfig,
    package::{
        application::adapters::CheckServiceAdapter,
        event::{EventSender, metadata::CheckMetadata},
        port::PackageRepository,
    },
};

pub(super) const TOTAL_STEPS: u32 = 3;

pub(super) async fn handle_check<PR, CR>(
    package_name: &str,
    repo: &PR,
    config: &AppConfig,
    command_runner: &CR,
    sender: &EventSender<CheckMetadata, Cow<'static, str>, Cow<'static, str>>,
) -> Result<Cow<'static, str>, Cow<'static, str>>
where
    PR: PackageRepository + Clone,
    CR: CommandRunner + Clone,
{
    // Create adapter that bridges to hexagonal architecture
    let adapter = CheckServiceAdapter::new(repo.clone(), command_runner.clone(), config.clone());

    // Delegate to the application service through the adapter
    adapter.handle_check(package_name, sender).await
}
