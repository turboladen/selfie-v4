mod check;
mod install;
mod steps;
mod validate;

use std::path::PathBuf;

use tokio::sync::mpsc;
use tracing::instrument;

use super::{
    event::{
        EventSender, EventStream, OperationContext, OperationResult, PackageEvent,
        metadata::OperationType,
    },
    port::PackageRepository,
};

use crate::{commands::runner::CommandRunner, config::AppConfig, package::port::PackageError};

/// Helper for tracking progress through operation steps
#[derive(Debug, Clone)]
pub(crate) struct ProgressTracker {
    current_step: u32,
    total_steps: u32,
}

impl ProgressTracker {
    pub(crate) fn new(total_steps: u32) -> Self {
        Self {
            current_step: 0,
            total_steps,
        }
    }

    pub(crate) async fn next(&mut self, sender: &EventSender, message: impl std::fmt::Display) {
        self.current_step += 1;
        let enhanced_message = format!("{} ({}/{})", message, self.current_step, self.total_steps);
        sender
            .send_progress(self.current_step, self.total_steps, enhanced_message)
            .await;
    }

    pub(crate) fn current_step(&self) -> u32 {
        self.current_step
    }

    pub(crate) fn total_steps(&self) -> u32 {
        self.total_steps
    }
}

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

    // Helper to execute an operation with standard event handling and dependency injection
    fn execute_operation_with_deps<F, Fut>(
        &self,
        operation_type: OperationType,
        package_name: &str,
        context: OperationContext,
        total_steps: u32,
        handler: F,
    ) -> EventStream
    where
        F: FnOnce(R, CR, AppConfig, EventSender, ProgressTracker) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = OperationResult> + Send,
    {
        let repo = self.package_repository.clone();
        let command_runner = self.command_runner.clone();
        let config = self.config.clone();
        let package_name = package_name.to_string();

        Self::create_event_stream(move |tx| async move {
            let sender = EventSender::new_with_context(
                tx.clone(),
                operation_type,
                package_name.clone(),
                config.environment().to_string(),
                context,
            );

            sender.send_started().await;
            sender
                .send_trace(format!("Current environment: {}", config.environment()))
                .await;

            let progress = ProgressTracker::new(total_steps);
            let result = handler(repo, command_runner, config, sender.clone(), progress).await;
            sender.send_completed(result).await;
        })
    }

    // Helper to execute an operation with standard event handling (simpler version)
    fn execute_operation<F, Fut>(
        &self,
        operation_type: OperationType,
        package_name: &str,
        context: OperationContext,
        total_steps: u32,
        handler: F,
    ) -> EventStream
    where
        F: FnOnce(EventSender, ProgressTracker) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = OperationResult> + Send,
    {
        let config = self.config.clone();
        let package_name = package_name.to_string();

        Self::create_event_stream(move |tx| async move {
            let sender = EventSender::new_with_context(
                tx,
                operation_type,
                package_name,
                config.environment().to_string(),
                context,
            );

            sender.send_started().await;
            sender
                .send_trace(format!("Current environment: {}", config.environment()))
                .await;

            let progress = ProgressTracker::new(total_steps);
            let result = handler(sender.clone(), progress).await;
            sender.send_completed(result).await;
        })
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
        let package_name_owned = package_name.to_string();
        self.execute_operation_with_deps(
            OperationType::PackageCheck,
            package_name,
            OperationContext::default(),
            3, // Load package + check environment + run check command
            move |repo, command_runner, config, sender, mut progress| async move {
                check::handle_check(
                    &package_name_owned,
                    &repo,
                    &config,
                    &command_runner,
                    &sender,
                    &mut progress,
                )
                .await
            },
        )
    }

    #[instrument]
    async fn install(&self, package_name: &str) -> EventStream {
        let package_name_owned = package_name.to_string();
        self.execute_operation_with_deps(
            OperationType::PackageInstall,
            package_name,
            OperationContext::default(),
            5, // fetch_package + find_env + get_command + execute_command + result processing
            move |repo, command_runner, config, sender, mut progress| async move {
                install::handle_install(
                    &package_name_owned,
                    &repo,
                    &config,
                    &command_runner,
                    &sender,
                    &mut progress,
                )
                .await
            },
        )
    }

    async fn validate(
        &self,
        package_name: &str,
        package_path: Option<PathBuf>,
    ) -> Result<EventStream, PackageError> {
        let context = OperationContext {
            package_path,
            target_environment: None,
        };

        let package_name_owned = package_name.to_string();
        Ok(self.execute_operation_with_deps(
            OperationType::PackageValidate,
            package_name,
            context,
            3, // load_package + validate_package + result processing
            move |repo, command_runner, config, sender, mut progress| async move {
                validate::handle_validate(
                    &package_name_owned,
                    &repo,
                    &config,
                    &command_runner,
                    &sender,
                    &mut progress,
                )
                .await
            },
        ))
    }

    async fn list(&self) -> Result<EventStream, PackageError> {
        Ok(self.execute_operation(
            OperationType::PackageList,
            "", // No specific package for list operation
            OperationContext::default(),
            1, // Just one step for listing
            |_sender, mut _progress| async move {
                // TODO: Implement actual listing logic
                OperationResult::Success("List operation not yet implemented".to_string())
            },
        ))
    }

    async fn info(&self, package_name: &str) -> Result<EventStream, PackageError> {
        Ok(self.execute_operation(
            OperationType::PackageInfo,
            package_name,
            OperationContext::default(),
            1, // Just one step for info
            |_sender, mut _progress| async move {
                // TODO: Implement actual info logic
                OperationResult::Success("Info operation not yet implemented".to_string())
            },
        ))
    }

    async fn create(&self, package_name: &str) -> Result<EventStream, PackageError> {
        Ok(self.execute_operation(
            OperationType::PackageCreate,
            package_name,
            OperationContext::default(),
            1, // Just one step for creation
            |_sender, mut _progress| async move {
                // TODO: Implement actual creation logic
                OperationResult::Success("Create operation not yet implemented".to_string())
            },
        ))
    }
}
