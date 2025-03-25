// Shell command runner adapter implementation

use std::{
    process::{Output, Stdio},
    time::{Duration, Instant},
};

use async_trait::async_trait;
use futures::TryFutureExt;
use tokio::{io::AsyncReadExt, process::Command, sync::mpsc};

use super::runner::{CommandError, CommandOutput, CommandRunner, OutputChunk};

/// Shell command runner implementation
///
#[derive(Clone)]
pub struct ShellCommandRunner {
    /// Path to the shell executable
    ///
    shell: String,

    /// Default timeout for commands
    ///
    default_timeout: Duration,
}

impl ShellCommandRunner {
    /// Create a new shell command runner
    ///
    #[must_use]
    pub fn new(shell: &str, default_timeout: Duration) -> Self {
        Self {
            shell: shell.to_string(),
            default_timeout,
        }
    }
}

#[async_trait]
impl CommandRunner for ShellCommandRunner {
    /// Checks if `command` is available.
    ///
    async fn is_command_available(&self, command: &str) -> bool {
        // Shell-agnostic way to check if a command exists
        let check_cmd = format!("command -v {command} >/dev/null 2>&1");

        match self.execute(&check_cmd).await {
            Ok(output) => output.is_success(),
            Err(_) => false,
        }
    }

    /// Execute a command using the default timeout.
    ///
    async fn execute(&self, command: &str) -> Result<CommandOutput, CommandError> {
        self.execute_with_timeout(command, self.default_timeout)
            .await
    }

    /// Execute a command without streaming stdout and stderr.
    ///
    async fn execute_with_timeout(
        &self,
        command: &str,
        timeout: Duration,
    ) -> Result<CommandOutput, CommandError> {
        let start_time = Instant::now();

        let mut cmd = Command::new(&self.shell);
        cmd.arg("-c").arg(command).stdin(Stdio::null());

        let duration = start_time.elapsed();

        // Execute the command within the context of a timeout
        let output = tokio::time::timeout(timeout, cmd.output().map_err(CommandError::from))
            .await
            .map_err(|_| CommandError::Timeout(timeout))??;

        Ok(CommandOutput { output, duration })
    }

    async fn execute_streaming<F>(
        &self,
        command: &str,
        timeout: Duration,
        mut callback: F,
    ) -> Result<CommandOutput, CommandError>
    where
        F: FnMut(OutputChunk) + Send + 'static,
    {
        let start_time = Instant::now();

        let mut cmd = Command::new(&self.shell);
        cmd.arg("-c")
            .arg(command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => return Err(CommandError::from(e)),
        };

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| CommandError::StdoutSpawn(command.to_string()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| CommandError::StderrSpawn(command.to_string()))?;

        let mut stdout = tokio::io::BufReader::new(stdout);
        let mut stderr = tokio::io::BufReader::new(stderr);

        let mut full_stdout = Vec::new();
        let mut full_stderr = Vec::new();

        let mut stdout_buf = vec![0; 1024]; // Buffer of 1024 bytes
        let mut stderr_buf = vec![0; 1024]; // Buffer of 1024 bytes

        let (tx, mut rx) = mpsc::channel(32);

        // Spawn a task to handle the callback
        tokio::spawn(async move {
            while let Some(chunk) = rx.recv().await {
                callback(chunk);
            }
        });

        let timeout_future = tokio::time::sleep(timeout);
        tokio::pin!(timeout_future);

        loop {
            tokio::select! {
                () = &mut timeout_future => {
                    let _ = child.kill().await;
                    return Err(CommandError::Timeout(timeout));
                },
                result = stdout.read(&mut stdout_buf) => {
                    handle_chunked_read_result(result, &mut full_stdout, &mut stdout_buf, &tx, OutputChunk::Stdout).await?;
                },
                result = stderr.read(&mut stderr_buf) => {
                    handle_chunked_read_result(result, &mut full_stderr, &mut stderr_buf, &tx, OutputChunk::Stderr).await?;
                },
                status = child.wait() => {
                    let status = status.map_err(CommandError::from)?;
                    let duration = start_time.elapsed();

                    return Ok(CommandOutput {
                        output: Output {
                            status,
                            stdout: full_stdout,
                            stderr: full_stderr,
                        },
                        duration,
                    });
                }
            }
        }
    }
}

async fn handle_chunked_read_result(
    result: Result<usize, tokio::io::Error>,
    full_output: &mut Vec<u8>,
    buffer: &mut Vec<u8>,
    tx: &mpsc::Sender<OutputChunk>,
    output_type: fn(String) -> OutputChunk,
) -> Result<(), CommandError> {
    match result {
        Ok(0) => {} // End of stream
        Ok(n) => {
            full_output.extend_from_slice(&buffer[..n]);
            let chunk = String::from_utf8_lossy(&buffer[..n]).to_string();
            tx.send(output_type(chunk))
                .await
                .map_err(|e| CommandError::Callback(e.0))?;
            buffer.clear();
        }
        Err(e) => return Err(CommandError::IoError(e)),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests will actually run commands on the system
    // They could be skipped in CI environments if necessary
    #[tokio::test]
    async fn test_shell_command_runner_basic() {
        let runner = ShellCommandRunner::new("/bin/sh", Duration::from_secs(10));

        // Test a basic echo command
        let result = runner.execute("echo hello").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.stdout_str().contains("hello"));
        assert!(output.is_success());

        // Test command failure
        let result = runner.execute("exit 1").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_success());
        assert_eq!(output.exit_code(), 1);
    }

    #[tokio::test]
    async fn test_command_availability() {
        let runner = ShellCommandRunner::new("/bin/sh", Duration::from_secs(10));

        // "echo" should be available in most environments
        assert!(runner.is_command_available("echo").await);

        // A random string should not be a valid command
        let random_cmd = "xyzabc123notarealcommand";
        assert!(!runner.is_command_available(random_cmd).await);
    }

    // This test relies on timing and could be flaky
    // Consider skipping or adjusting in CI environments
    #[tokio::test]
    async fn test_timeout() {
        let runner = ShellCommandRunner::new("/bin/sh", Duration::from_millis(100));

        // Command that should timeout (sleep for 1s)
        // Note: This is a simple test and may be flaky since timeouts aren't enforced
        // in a separate thread in our implementation
        let result = runner
            .execute_with_timeout("sleep 1", Duration::from_millis(10))
            .await;
        assert!(matches!(result, Err(CommandError::Timeout(_))));
    }
}
