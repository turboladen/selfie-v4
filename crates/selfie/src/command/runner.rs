pub mod shell;

pub use self::shell::ShellCommandRunner;

use std::{borrow::Cow, fmt, process::Output, time::Duration};

use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug)]
pub enum OutputChunk {
    Stdout(String),
    Stderr(String),
}

impl fmt::Display for OutputChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stdout(s) => f.write_str(s),
            Self::Stderr(s) => f.write_str(s),
        }
    }
}

/// Port for command execution
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait CommandRunner: Send + Sync {
    /// Check if a command is available in the current environment.
    ///
    async fn is_command_available(&self, command: &str) -> bool;

    /// Execute a command, wait for it to complete, then return its output.
    ///
    async fn execute(&self, command: &str) -> Result<CommandOutput, CommandError>;

    /// Execute a command with a timeout and return its output.
    ///
    async fn execute_with_timeout(
        &self,
        command: &str,
        timeout: Duration,
    ) -> Result<CommandOutput, CommandError>;

    /// Execute a command that streams stdout and stderr to the `output_callback` function.
    ///
    async fn execute_streaming<F>(
        &self,
        command: &str,
        timeout: Duration,
        output_callback: F,
    ) -> Result<CommandOutput, CommandError>
    where
        F: FnMut(OutputChunk) + Send + 'static;
}

/// Result of executing a command
#[derive(Debug, Clone, PartialEq)]
pub struct CommandOutput {
    /// The process output.
    ///
    pub(crate) output: Output,

    /// How long the command took to execute
    ///
    pub(crate) duration: Duration,
}

impl CommandOutput {
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        self.output.status.code().unwrap_or(-1)
    }

    #[must_use]
    pub fn stdout(&self) -> &[u8] {
        &self.output.stdout
    }

    #[must_use]
    pub fn stdout_str(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.output.stdout)
    }

    #[must_use]
    pub fn stderr(&self) -> &[u8] {
        &self.output.stderr
    }

    #[must_use]
    pub fn stderr_str(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.output.stdout)
    }

    pub fn is_success(&self) -> bool {
        self.output.status.success()
    }
}

/// Errors that can occur during command execution
#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Command timed out after {0:?}")]
    Timeout(Duration),

    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed spawning stdout during command: {0}")]
    StdoutSpawn(String),

    #[error("Failed spawning stderr during command: {0}")]
    StderrSpawn(String),

    #[error("Error while processing command: {0}")]
    Callback(OutputChunk),
}
