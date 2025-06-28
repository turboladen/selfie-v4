mod check;
mod install;
mod steps;
mod validate;

use std::path::PathBuf;

use tokio::sync::mpsc;
use tracing::instrument;

use super::{
    event::{EventSender, EventStream, OperationResult, PackageEvent, metadata::OperationType},
    port::PackageRepository,
};

use crate::{commands::runner::CommandRunner, config::AppConfig, package::port::PackageError};

/// Primary port for package operations
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait PackageService: Send + Sync {
    /// Run package's `check` command
    async fn check(&self, package_name: &str) -> EventStream;

    /// Install a package
    async fn install(&self, package_name: &str) -> EventStream;

    /// Get information about a package
    async fn info(&self, package_name: &str) -> Result<EventStream, PackageError>;

    /// Validate a package
    async fn validate(
        &self,
        package_name: &str,
        package_path: Option<PathBuf>,
    ) -> Result<EventStream, PackageError>;

    /// List available packages
    async fn list(&self) -> Result<EventStream, PackageError>;

    /// Create a new package
    async fn create(&self, package_name: &str) -> Result<EventStream, PackageError>;
}

/// Implementation of the PackageService
#[derive(Debug)]
pub struct PackageServiceImpl<R, CR> {
    package_repository: R,
    command_runner: CR,
    config: AppConfig,
}

impl<R, CR> PackageServiceImpl<R, CR>
where
    R: PackageRepository + Clone + 'static,
    CR: CommandRunner + Clone + 'static,
{
    pub fn new(package_repository: R, command_runner: CR, config: AppConfig) -> Self {
        Self {
            package_repository,
            command_runner,
            config,
        }
    }

    // Helper to create an event stream
    fn create_event_stream<F, Fut>(f: F) -> EventStream
    where
        F: FnOnce(mpsc::Sender<PackageEvent>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send,
    {
        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            f(tx).await;
        });

        Box::pin(futures::stream::unfold(rx, |mut rx| async move {
            rx.recv().await.map(|event| (event, rx))
        }))
    }
}

#[async_trait::async_trait]
impl<R, CR> PackageService for PackageServiceImpl<R, CR>
where
    R: PackageRepository + Clone + std::fmt::Debug + Send + Sync + 'static,
    CR: CommandRunner + Clone + std::fmt::Debug + Send + Sync + 'static,
{
    #[instrument]
    async fn check(&self, package_name: &str) -> EventStream {
        // Clone what we need for the async task
        let repo = self.package_repository.clone();
        let command_runner = self.command_runner.clone();
        let config = self.config.clone();
        let package_name = package_name.to_string();

        Self::create_event_stream(move |tx| async move {
            let sender = EventSender::new(
                tx,
                OperationType::PackageCheck,
                package_name.clone(),
                config.environment().to_string(),
            );

            sender.send_started().await;
            let current_env = config.environment();

            sender
                .send_trace(format!("Current environment: {}", current_env))
                .await;

            let result =
                check::handle_check(&package_name, &repo, &config, &command_runner, &sender).await;

            sender.send_completed(result).await
        })
    }

    // Implementation for the install method
    #[instrument]
    async fn install(&self, package_name: &str) -> EventStream {
        // Clone what we need for the async task
        let repo = self.package_repository.clone();
        let command_runner = self.command_runner.clone();
        let config = self.config.clone();
        let package_name = package_name.to_string();

        Self::create_event_stream(move |tx| async move {
            let sender = EventSender::new(
                tx,
                OperationType::PackageInstall,
                package_name.clone(),
                config.environment().to_string(),
            );

            sender.send_started().await;
            sender
                .send_trace(format!("Current environment: {}", config.environment()))
                .await;

            let mut step = 1;
            let total_steps = 12345; // Replace with actual calculation

            let result = install::handle_install(
                &package_name,
                &repo,
                &config,
                &command_runner,
                &sender,
                &mut step,
                total_steps,
            )
            .await;

            sender.send_completed(result).await;
        })
    }

    // Implement other methods similarly...
    async fn validate(
        &self,
        package_name: &str,
        _package_path: Option<PathBuf>,
    ) -> Result<EventStream, PackageError> {
        let repo = self.package_repository.clone();
        let command_runner = self.command_runner.clone();
        let config = self.config.clone();
        let package_name = package_name.to_string();

        Ok(Self::create_event_stream(move |tx| async move {
            let sender = EventSender::new(
                tx,
                OperationType::PackageValidate,
                package_name.clone(),
                config.environment().to_string(),
            );
            sender.send_started().await;
            let current_env = config.environment();

            sender
                .send_trace(format!("Current environment: {}", current_env))
                .await;

            let result =
                validate::handle_validate(&package_name, &repo, &config, &command_runner, &sender)
                    .await;

            sender.send_completed(result).await
        }))
    }

    async fn list(&self) -> Result<EventStream, PackageError> {
        // Clone what we need
        let config = self.config.clone();

        Ok(Self::create_event_stream(move |tx| async move {
            let sender = EventSender::new(
                tx,
                OperationType::PackageList,
                "".to_string(), // No specific package for list operation
                config.environment().to_string(),
            );

            sender.send_started().await;

            // TODO: Implement actual listing logic
            let result = OperationResult::Success("List operation not yet implemented".to_string());
            sender.send_completed(result).await;
        }))
    }

    async fn info(&self, package_name: &str) -> Result<EventStream, PackageError> {
        // Implementation similar to other methods
        let package_name = package_name.to_string();
        let config = self.config.clone();

        Ok(Self::create_event_stream(move |tx| async move {
            let sender = EventSender::new(
                tx,
                OperationType::PackageInfo,
                package_name.clone(),
                config.environment().to_string(),
            );

            sender.send_started().await;

            // TODO: Implement actual info logic
            let result = OperationResult::Success("Info operation not yet implemented".to_string());
            sender.send_completed(result).await;
        }))
    }

    async fn create(&self, package_name: &str) -> Result<EventStream, PackageError> {
        let package_name = package_name.to_string();
        let config = self.config.clone();

        Ok(Self::create_event_stream(move |tx| async move {
            let sender = EventSender::new(
                tx,
                OperationType::PackageCreate,
                package_name.clone(),
                config.environment().to_string(),
            );

            sender.send_started().await;

            // TODO: Implement actual creation logic
            let result =
                OperationResult::Success("Create operation not yet implemented".to_string());
            sender.send_completed(result).await;
        }))
    }
}
