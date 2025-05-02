pub mod error;
pub mod metadata;

use std::{
    fmt::{self, Debug},
    pin::Pin,
};

use futures::Stream;
use tokio::sync::mpsc;

use self::{
    error::StreamedError,
    metadata::{EventMetadata, OperationType},
};

pub type EventStream<T> = Pin<Box<dyn Stream<Item = PackageEvent<T>> + Send>>;

#[derive(Debug, Clone)]
pub(crate) struct EventSender<T> {
    metadata: EventMetadata<T>,
    tx: mpsc::Sender<PackageEvent<T>>,
}

impl<T: Debug + Clone> EventSender<T> {
    pub(crate) fn new(
        tx: mpsc::Sender<PackageEvent<T>>,
        operation_type: OperationType,
        command_metadata: T,
    ) -> Self {
        Self {
            tx,
            metadata: EventMetadata::new(operation_type, command_metadata),
        }
    }

    pub(crate) async fn send_started(&self) {
        let metadata = self.metadata.touch_and_clone();

        tracing::trace!(
            operation_type = metadata.operation_type().to_string(),
            command_metadata = ?metadata.command_metadata(),
            "operation started",
        );
        let _ = self.tx.send(PackageEvent::Started { metadata }).await;
    }

    pub(crate) async fn send_progress(
        &self,
        step: u32,
        total_steps: u32,
        message: impl fmt::Display,
    ) {
        let metadata = self.metadata.touch_and_clone();
        let msg = message.to_string();
        tracing::info!(
            operation_type = metadata.operation_type().to_string(),
            command_metadata = ?metadata.command_metadata(),
            message = &msg,
        );
        let _ = self
            .tx
            .send(PackageEvent::Progress {
                metadata,
                step,
                total_steps,
                percent_complete: step as f32 / total_steps as f32,
                message: msg,
            })
            .await;
    }

    pub(crate) async fn send_completed(
        &self,
        message: Result<impl fmt::Display, impl fmt::Display>,
    ) {
        let metadata = self.metadata.touch_and_clone();
        let message = message.map(|m| m.to_string()).map_err(|e| e.to_string());

        tracing::info!(
            operation_type = metadata.operation_type().to_string(),
            command_metadata = ?metadata.command_metadata(),
            "operation started",
        );
        let _ = self
            .tx
            .send(PackageEvent::Completed { metadata, message })
            .await;
    }

    pub(crate) async fn send_trace(&self, message: impl fmt::Display) {
        let metadata = self.metadata.touch_and_clone();
        let message = message.to_string();

        tracing::trace!(
            operation_type = metadata.operation_type().to_string(),
            command_metadata = ?metadata.command_metadata(),
            message = &message,
        );
        let _ = self
            .tx
            .send(PackageEvent::Trace { metadata, message })
            .await;
    }

    pub(crate) async fn send_debug(&self, message: impl fmt::Display) {
        let metadata = self.metadata.touch_and_clone();
        let message = message.to_string();

        tracing::debug!(
            operation_type = metadata.operation_type().to_string(),
            command_metadata = ?metadata.command_metadata(),
            message = &message,
        );
        let _ = self
            .tx
            .send(PackageEvent::Debug { metadata, message })
            .await;
    }

    pub(crate) async fn send_info(&self, message: ConsoleOutput) {
        let metadata = self.metadata.touch_and_clone();

        tracing::info!(
            operation_type = metadata.operation_type().to_string(),
            command_metadata = ?metadata.command_metadata(),
            message = ?&message,
        );
        let _ = self.tx.send(PackageEvent::Info { metadata, message }).await;
    }

    pub(crate) async fn send_warning(&self, message: impl fmt::Display) {
        let metadata = self.metadata.touch_and_clone();
        let msg = message.to_string();
        tracing::warn!(
            operation_type = metadata.operation_type().to_string(),
            command_metadata = ?metadata.command_metadata(),
            message = &msg,
        );
        let _ = self
            .tx
            .send(PackageEvent::Warning {
                metadata,
                message: msg,
            })
            .await;
    }

    pub(crate) async fn send_error<E>(&self, error: E, message: impl fmt::Display)
    where
        StreamedError: From<E>,
    {
        let metadata = self.metadata.touch_and_clone();
        let msg = message.to_string();
        tracing::error!(
            operation_type = metadata.operation_type().to_string(),
            command_metadata = ?metadata.command_metadata(),
            message = &msg,
        );
        let _ = self
            .tx
            .send(PackageEvent::Error {
                metadata,
                error: StreamedError::from(error),
                message: msg,
            })
            .await;
    }
}

/// Events that can be emitted during package operations
#[derive(Debug, Clone)]
pub enum PackageEvent<T> {
    /// Operation has started
    Started { metadata: EventMetadata<T> },

    /// Progress update
    Progress {
        metadata: EventMetadata<T>,
        step: u32,
        total_steps: u32,
        percent_complete: f32,
        message: String,
    },

    // InputRequested {
    //     metadata: EventMetadata<T>,
    //     prompt: String,
    //     options: Option<Vec<String>>,
    // },
    /// Operation completed successfully
    Completed {
        metadata: EventMetadata<T>,
        // result: Option<serde_json::Value>,
        message: Result<String, String>,
    },

    /// Operation was canceled
    Canceled {
        metadata: EventMetadata<T>,
        reason: String,
    },

    Trace {
        metadata: EventMetadata<T>,
        message: String,
    },

    Debug {
        metadata: EventMetadata<T>,
        message: String,
    },

    /// Informational message
    Info {
        metadata: EventMetadata<T>,
        // message: String,
        message: ConsoleOutput,
    },

    /// Warning message
    Warning {
        metadata: EventMetadata<T>,
        message: String,
    },

    /// Error occurred but operation continues
    Error {
        metadata: EventMetadata<T>,
        error: StreamedError,
        message: String,
    },
}

#[derive(Debug, Clone)]
pub enum ConsoleOutput {
    Stdout(String),
    Stderr(String),
}
