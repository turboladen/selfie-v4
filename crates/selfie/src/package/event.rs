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

pub type EventStream<M, O, E> = Pin<Box<dyn Stream<Item = PackageEvent<M, O, E>> + Send>>;

#[derive(Debug, Clone)]
pub(crate) struct EventSender<M, O, E> {
    metadata: EventMetadata<M>,
    tx: mpsc::Sender<PackageEvent<M, O, E>>,
}

impl<M: Debug + Clone, O, E> EventSender<M, O, E> {
    pub(crate) fn new(
        tx: mpsc::Sender<PackageEvent<M, O, E>>,
        operation_type: OperationType,
        command_metadata: M,
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

    pub(crate) async fn send_completed(&self, result: Result<O, E>) {
        let metadata = self.metadata.touch_and_clone();

        tracing::info!(
            operation_type = metadata.operation_type().to_string(),
            command_metadata = ?metadata.command_metadata(),
            "operation started",
        );
        let _ = self
            .tx
            .send(PackageEvent::Completed { metadata, result })
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

    pub(crate) async fn send_error<SE>(&self, error: SE, message: impl fmt::Display)
    where
        StreamedError: From<SE>,
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
pub enum PackageEvent<M, O, E> {
    /// Operation has started
    Started { metadata: EventMetadata<M> },

    /// Progress update
    Progress {
        metadata: EventMetadata<M>,
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
        metadata: EventMetadata<M>,
        // result: Option<serde_json::Value>,
        result: Result<O, E>,
    },

    /// Operation was canceled
    Canceled {
        metadata: EventMetadata<M>,
        reason: String,
    },

    Trace {
        metadata: EventMetadata<M>,
        message: String,
    },

    Debug {
        metadata: EventMetadata<M>,
        message: String,
    },

    /// Informational message
    Info {
        metadata: EventMetadata<M>,
        // message: String,
        message: ConsoleOutput,
    },

    /// Warning message
    Warning {
        metadata: EventMetadata<M>,
        message: String,
    },

    /// Error occurred but operation continues
    Error {
        metadata: EventMetadata<M>,
        error: StreamedError,
        message: String,
    },
}

#[derive(Debug, Clone)]
pub enum ConsoleOutput {
    Stdout(String),
    Stderr(String),
}
