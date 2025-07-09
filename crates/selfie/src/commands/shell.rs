// Shell command runner adapter implementation

use std::{
    path::Path,
    process::{Output, Stdio},
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;

use tokio::{io::AsyncReadExt, process::Command, sync::mpsc};

use super::runner::{CommandError, CommandOutput, CommandRunner, OutputChunk};

/// Shell command runner implementation
///
#[derive(Clone, Debug)]
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

        let working_directory =
            std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());

        // Execute the command within the context of a timeout
        let output = tokio::time::timeout(timeout, cmd.output())
            .await
            .map_err(|_| CommandError::Timeout {
                command: command.to_string(),
                timeout,
                working_directory: working_directory.clone(),
            })?
            .map_err(|e| CommandError::IoError {
                command: command.to_string(),
                working_directory: working_directory.clone(),
                source: Arc::new(e),
            })?;

        let duration = start_time.elapsed();

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
            Err(e) => {
                return Err(CommandError::IoError {
                    command: command.to_string(),
                    working_directory: Path::new(".").to_path_buf(),
                    source: Arc::new(e),
                });
            }
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

        let timeout_future = tokio::time::timeout(timeout, async {
            loop {
                tokio::select! {
                    result = stdout.read(&mut stdout_buf) => {
                        handle_chunked_read_result(result, &mut full_stdout, &mut stdout_buf, &tx, OutputChunk::Stdout).await?;
                    },
                    result = stderr.read(&mut stderr_buf) => {
                        handle_chunked_read_result(result, &mut full_stderr, &mut stderr_buf, &tx, OutputChunk::Stderr).await?;
                    },
                    status = child.wait() => {
                        let status = status.map_err(|e| CommandError::IoError {
                            command: command.to_string(),
                            working_directory: std::env::current_dir()
                                .unwrap_or_else(|_| Path::new(".").to_path_buf()),
                            source: Arc::new(e),
                        })?;
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
        });

        if let Ok(result) = timeout_future.await {
            result
        } else {
            let _ = child.kill().await;
            Err(CommandError::Timeout {
                command: command.to_string(),
                timeout,
                working_directory: std::env::current_dir()
                    .unwrap_or_else(|_| Path::new(".").to_path_buf()),
            })
        }
    }
}

async fn handle_chunked_read_result(
    result: Result<usize, tokio::io::Error>,
    full_output: &mut Vec<u8>,
    buffer: &mut [u8],
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
            // Note: Don't clear the buffer here - tokio reuses it for the next read
        }
        Err(e) => {
            return Err(CommandError::IoError {
                command: "streaming command".to_string(),
                working_directory: std::env::current_dir()
                    .unwrap_or_else(|_| Path::new(".").to_path_buf()),
                source: Arc::new(e),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod original_tests {
    use super::*;
    use std::path::PathBuf;

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
        assert!(matches!(result, Err(CommandError::Timeout { .. })));
    }

    // Error handling tests
    #[tokio::test]
    async fn test_command_timeout_error() {
        let runner = ShellCommandRunner::new("/bin/sh", Duration::from_millis(50));

        // Create a command that will timeout
        let result = runner
            .execute_with_timeout("sleep 1", Duration::from_millis(10))
            .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            CommandError::Timeout { .. } => {
                // Expected timeout error
            }
            _ => panic!("Expected CommandError::Timeout, got: {error:?}"),
        }
    }

    #[tokio::test]
    async fn test_command_io_error() {
        let runner = ShellCommandRunner::new("/bin/sh", Duration::from_secs(5));

        // Try to execute a command that doesn't exist
        let result = runner.execute("nonexistent_command_12345_xyz").await;

        // Command might succeed but with non-zero exit code, or fail
        if let Ok(output) = result {
            // If command executes, it should fail (non-zero exit code)
            assert!(!output.is_success());
        }
        // If result is Err, that's also acceptable for this test
    }

    #[tokio::test]
    async fn test_command_permission_denied() {
        let runner = ShellCommandRunner::new("/bin/sh", Duration::from_secs(5));

        // Try to access a file that should not be accessible
        let result = runner
            .execute("cat /root/.ssh/id_rsa 2>/dev/null || echo 'permission denied'")
            .await;

        // This should either succeed with "permission denied" message or fail
        // Either way, we're testing that the command runner handles the scenario
        if let Ok(output) = result {
            assert!(
                output.stdout_str().contains("permission denied")
                    || !output.stderr_str().is_empty()
            );
        }
        // If it fails, that's also acceptable for this test
    }

    #[tokio::test]
    async fn test_command_invalid_syntax() {
        let runner = ShellCommandRunner::new("/bin/sh", Duration::from_secs(5));

        // Try to execute a command with invalid syntax
        let result = runner.execute("if [ 1 -eq 1 ; then echo 'unclosed'").await;

        // This should fail due to invalid shell syntax
        if let Ok(output) = result {
            // Some shells might handle this gracefully
            assert!(!output.is_success());
        }
        // If it errors, that's also expected
    }

    #[tokio::test]
    async fn test_error_display_formatting() {
        // Test that our error types format correctly
        let timeout_error = CommandError::Timeout {
            command: "test-command".to_string(),
            timeout: Duration::from_millis(100),
            working_directory: PathBuf::from("/tmp"),
        };
        assert!(
            timeout_error
                .to_string()
                .contains("Command timed out after 100ms")
        );
        assert!(timeout_error.to_string().contains("test-command"));

        let io_error = std::io::Error::other("test error");
        let cmd_error = CommandError::IoError {
            command: "test-command".to_string(),
            working_directory: PathBuf::from("/tmp"),
            source: Arc::new(io_error),
        };
        assert!(cmd_error.to_string().contains("test-command"));

        let stdout_error = CommandError::StdoutSpawn("stdout issue".to_string());
        assert_eq!(
            stdout_error.to_string(),
            "Failed spawning stdout during command: stdout issue"
        );

        let stderr_error = CommandError::StderrSpawn("stderr issue".to_string());
        assert_eq!(
            stderr_error.to_string(),
            "Failed spawning stderr during command: stderr issue"
        );
    }

    #[tokio::test]
    async fn test_command_with_large_output() {
        let runner = ShellCommandRunner::new("/bin/sh", Duration::from_secs(5));

        // Generate a large amount of output to test buffering
        let result = runner
            .execute("for i in $(seq 1 1000); do echo \"Line $i\"; done")
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_success());
        assert!(output.stdout_str().lines().count() >= 1000);
    }

    #[tokio::test]
    async fn test_command_output_methods() {
        let runner = ShellCommandRunner::new("/bin/sh", Duration::from_secs(5));

        // Test that our output methods work correctly
        let result = runner.execute("echo 'test output'").await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_success());
        assert!(output.stdout_str().contains("test output"));

        // Test that stderr_str() method exists and returns a string
        let _stderr = output.stderr_str(); // Just verify the method works
    }

    #[tokio::test]
    async fn test_command_exit_code_handling() {
        let runner = ShellCommandRunner::new("/bin/sh", Duration::from_secs(5));

        // Command that exits with non-zero status
        let result = runner.execute("exit 42").await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_success());
        assert_eq!(output.exit_code(), 42);
    }
}
