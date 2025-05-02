mod check;
mod install;
mod steps;

use std::path::PathBuf;

use tokio::sync::mpsc;
use tracing::instrument;

use super::{
    event::{
        EventSender, EventStream, PackageEvent,
        metadata::{
            CheckMetadata, CreateMetadata, InfoMetadata, InstallMetadata, ListMetadata,
            OperationType, ValidateMetadata,
        },
    },
    port::PackageRepository,
};

use crate::{commands::runner::CommandRunner, config::AppConfig, package::port::PackageError};

/// Primary port for package operations
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait PackageService: Send + Sync {
    /// Run package's `check` command
    async fn check(&self, package_name: &str) -> EventStream<CheckMetadata>;

    /// Install a package
    async fn install(&self, package_name: &str) -> EventStream<InstallMetadata>;

    /// Get information about a package
    async fn info(&self, package_name: &str) -> Result<EventStream<InfoMetadata>, PackageError>;

    /// Validate a package
    async fn validate(
        &self,
        package_name: &str,
        package_path: Option<PathBuf>,
    ) -> Result<EventStream<ValidateMetadata>, PackageError>;

    /// List available packages
    async fn list(&self) -> Result<EventStream<ListMetadata>, PackageError>;

    /// Create a new package
    async fn create(&self, package_name: &str)
    -> Result<EventStream<CreateMetadata>, PackageError>;
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
    fn create_event_stream<F, Fut, T>(f: F) -> EventStream<T>
    where
        F: FnOnce(mpsc::Sender<PackageEvent<T>>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send,
        T: Send + 'static,
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
    async fn check(&self, package_name: &str) -> EventStream<CheckMetadata> {
        // Clone what we need for the async task
        let repo = self.package_repository.clone();
        let command_runner = self.command_runner.clone();
        let config = self.config.clone();
        let package_name = package_name.to_string();

        Self::create_event_stream(move |tx| async move {
            let sender = EventSender::new(
                tx,
                OperationType::PackageCheck,
                CheckMetadata::new(config.environment().to_string(), package_name.to_string()),
            );

            sender.send_started().await;
            let current_env = config.environment();

            sender
                .send_trace(format!("Current environment: {}", current_env))
                .await;

            let mut step = 1;
            let total_steps = 12345;

            // ╭─────────────────────────╮
            // │ Step 1: Get the package │
            // ╰─────────────────────────╯
            sender
                .send_progress(
                    step,
                    total_steps,
                    format!("Fetching package: {package_name}"),
                )
                .await;
            step += 1;

            let result = check::handle_check(
                &package_name,
                &repo,
                &config,
                &command_runner,
                &sender,
                &mut step,
                total_steps,
            )
            .await;
            // let result = PackageFinder::new(repo, &sender, step, total_steps)
            //     .find_package_then(&package_name, |package| todo!())
            //     .await;

            sender.send_completed(result).await
        })
    }

    // Implementation for the install method
    #[instrument]
    async fn install(&self, package_name: &str) -> EventStream<InstallMetadata> {
        // Clone what we need for the async task
        let repo = self.package_repository.clone();
        let command_runner = self.command_runner.clone();
        let config = self.config.clone();
        let package_name = package_name.to_string();

        Self::create_event_stream(move |tx| async move {
            let sender = EventSender::new(
                tx,
                OperationType::PackageInstall,
                InstallMetadata::new(config.environment().to_string(), package_name.to_string()),
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
    ) -> Result<EventStream<ValidateMetadata>, PackageError> {
        // Implementation similar to install
        // let package_name = package_name.to_string();

        Ok(Self::create_event_stream(move |tx| async move {
            todo!()
            // let _ = tx
            //     .send(PackageEvent::Started {
            //         operation: format!("Validating package '{}'", package_name),
            //     })
            //     .await;
            //
            // // Validation would happen here...
            //
            // let _ = tx.send(PackageEvent::Completed).await;
        }))
    }

    async fn list(&self) -> Result<EventStream<ListMetadata>, PackageError> {
        // Clone what we need
        // let fs = self.file_system.clone();
        // let config = self.config.clone();

        Ok(Self::create_event_stream(move |tx| async move {
            todo!()
            // let _ = tx
            //     .send(PackageEvent::Started {
            //         operation: "Listing available packages".to_string(),
            //     })
            //     .await;
            //
            // // Example implementation
            // match fs.list_directory(config.package_directory()) {
            //     Ok(entries) => {
            //         let _ = tx
            //             .send(PackageEvent::Info(format!(
            //                 "Found {} packages",
            //                 entries.len()
            //             )))
            //             .await;
            //
            //         for entry in entries {
            //             if let Some(name) = entry.file_name().and_then(|n| n.to_str()) {
            //                 let _ = tx.send(PackageEvent::Info(name.to_string())).await;
            //             }
            //         }
            //
            //         let _ = tx.send(PackageEvent::Completed).await;
            //     }
            //     Err(e) => {
            //         let _ = tx
            //             .send(PackageEvent::Error {
            //                 message: format!("Failed to list packages: {}", e),
            //                 recoverable: false,
            //             })
            //             .await;
            //     }
            // }
        }))
    }

    async fn info(&self, package_name: &str) -> Result<EventStream<InfoMetadata>, PackageError> {
        // Implementation similar to other methods
        // let package_name = package_name.to_string();

        Ok(Self::create_event_stream(move |tx| async move {
            todo!()
            // let _ = tx
            //     .send(PackageEvent::Started {
            //         operation: format!("Getting info for package '{}'", package_name),
            //     })
            //     .await;
            //
            // // Info gathering would happen here...
            //
            // let _ = tx.send(PackageEvent::Completed).await;
        }))
    }

    async fn create(
        &self,
        package_name: &str,
    ) -> Result<EventStream<CreateMetadata>, PackageError> {
        // Implementation similar to other methods
        // let package_name = package_name.to_string();

        Ok(Self::create_event_stream(move |tx| async move {
            todo!()
            // let _ = tx
            //     .send(PackageEvent::Started {
            //         operation: format!("Creating package '{}'", package_name),
            //     })
            //     .await;
            //
            // // Package creation would happen here...
            //
            // let _ = tx.send(PackageEvent::Completed).await;
        }))
    }
}
