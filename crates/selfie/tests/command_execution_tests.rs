use selfie::commands::{
    ShellCommandRunner,
    runner::{CommandRunner, OutputChunk},
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

#[tokio::test]
async fn test_command_execution_with_long_output() {
    let runner = ShellCommandRunner::new("/bin/sh", Duration::from_secs(5));

    // Generate a command that produces a lot of output
    let command = "for i in $(seq 1 1000); do echo \"Line $i\"; done";

    let output = runner.execute(command).await.unwrap();

    // Should capture all output lines
    let output_lines = output.stdout_str().lines().count();
    assert_eq!(output_lines, 1000);
}

#[tokio::test]
async fn test_command_streaming_captures_all_output() {
    let runner = ShellCommandRunner::new("/bin/sh", Duration::from_secs(5));

    // Command that produces output with some delay
    let command = r#"for i in $(seq 1 10); do echo "Line $i"; done"#;

    let output_chunks = Arc::new(Mutex::new(Vec::new()));
    let chunks_clone = output_chunks.clone();

    let output = runner
        .execute_streaming(command, Duration::from_secs(5), move |chunk| {
            if let OutputChunk::Stdout(text) = chunk {
                chunks_clone.lock().unwrap().push(text);
            }
        })
        .await
        .unwrap();

    // Check final output
    assert!(output.is_success());

    // Check streaming chunks
    let chunks = output_chunks.lock().unwrap();
    assert!(!chunks.is_empty());

    dbg!(&chunks);
    // Join all chunks and count lines
    let combined = chunks.join("");
    let line_count = combined.lines().count();
    assert_eq!(line_count, 10);
}

#[tokio::test]
async fn test_command_timeout() {
    let runner = ShellCommandRunner::new("/bin/sh", Duration::from_secs(1));

    // Command that runs longer than the timeout
    let command = "sleep 5";

    let result = runner.execute(command).await;

    // Should timeout
    assert!(matches!(
        result,
        Err(selfie::commands::runner::CommandError::Timeout(_))
    ));
}
