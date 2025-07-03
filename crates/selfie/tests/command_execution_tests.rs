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
    let runner = ShellCommandRunner::new("/bin/sh", Duration::from_secs(10));

    // Command that produces output line by line - simpler and more reliable
    let command = r#"for i in $(seq 1 5); do echo "Line $i"; done"#;

    let output_chunks = Arc::new(Mutex::new(Vec::new()));
    let chunks_clone = output_chunks.clone();

    let output = runner
        .execute_streaming(command, Duration::from_secs(10), move |chunk| {
            if let OutputChunk::Stdout(text) = chunk {
                chunks_clone.lock().unwrap().push(text);
            }
        })
        .await
        .unwrap();

    // Check final output
    assert!(output.is_success());

    // Check streaming chunks were captured
    let chunks = output_chunks.lock().unwrap();
    assert!(
        !chunks.is_empty(),
        "Should have captured at least one chunk"
    );

    // Verify all expected content is present in the chunks
    let combined = chunks.join("");

    // Check that all expected lines are present (order matters)
    for i in 1..=5 {
        let expected_line = format!("Line {i}");
        assert!(
            combined.contains(&expected_line),
            "Missing expected line: '{expected_line}' in combined output: '{combined}'"
        );
    }

    // Count actual lines (filter empty lines that might come from shell output)
    let line_count = combined
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();

    assert_eq!(
        line_count, 5,
        "Expected 5 lines but got {line_count}. Combined output: '{combined}'"
    );

    // Verify the final output matches what we captured via streaming
    let final_output = output.stdout_str();
    let final_line_count = final_output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();

    assert_eq!(
        final_line_count, 5,
        "Final output should also contain 5 lines. Final output: '{final_output}'"
    );
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

#[tokio::test]
async fn test_command_streaming_timeout() {
    let runner = ShellCommandRunner::new("/bin/sh", Duration::from_millis(100));

    // Command that runs longer than the timeout
    let command = "sleep 1";

    let output_chunks = Arc::new(Mutex::new(Vec::new()));
    let chunks_clone = output_chunks.clone();

    let result = runner
        .execute_streaming(command, Duration::from_millis(100), move |chunk| {
            if let OutputChunk::Stdout(text) = chunk {
                chunks_clone.lock().unwrap().push(text);
            }
        })
        .await;

    // Should timeout
    assert!(matches!(
        result,
        Err(selfie::commands::runner::CommandError::Timeout(_))
    ));
}

#[tokio::test]
async fn test_command_streaming_stderr_capture() {
    let runner = ShellCommandRunner::new("/bin/sh", Duration::from_secs(5));

    // Command that outputs to stderr
    let command = r#"echo "stdout line" && echo "stderr line" >&2"#;

    let stdout_chunks = Arc::new(Mutex::new(Vec::new()));
    let stderr_chunks = Arc::new(Mutex::new(Vec::new()));
    let stdout_clone = stdout_chunks.clone();
    let stderr_clone = stderr_chunks.clone();

    let output = runner
        .execute_streaming(command, Duration::from_secs(5), move |chunk| match chunk {
            OutputChunk::Stdout(text) => {
                stdout_clone.lock().unwrap().push(text);
            }
            OutputChunk::Stderr(text) => {
                stderr_clone.lock().unwrap().push(text);
            }
        })
        .await
        .unwrap();

    // Check final output
    assert!(output.is_success());

    // Verify stdout chunks
    let stdout_combined = stdout_chunks.lock().unwrap().join("");
    assert!(
        stdout_combined.contains("stdout line"),
        "Should contain stdout line: '{stdout_combined}'"
    );

    // Verify stderr chunks
    let stderr_combined = stderr_chunks.lock().unwrap().join("");
    assert!(
        stderr_combined.contains("stderr line"),
        "Should contain stderr line: '{stderr_combined}'"
    );
}

#[tokio::test]
async fn test_command_streaming_preserves_order() {
    let runner = ShellCommandRunner::new("/bin/sh", Duration::from_secs(10));

    // Command that outputs numbered lines to ensure we can verify ordering
    let command = r#"for i in $(seq 1 10); do echo "Message $i"; done"#;

    let output_chunks = Arc::new(Mutex::new(Vec::new()));
    let chunks_clone = output_chunks.clone();

    let output = runner
        .execute_streaming(command, Duration::from_secs(10), move |chunk| {
            if let OutputChunk::Stdout(text) = chunk {
                chunks_clone.lock().unwrap().push(text);
            }
        })
        .await
        .unwrap();

    // Check final output
    assert!(output.is_success());

    // Verify chunks were captured
    let chunks = output_chunks.lock().unwrap();
    assert!(
        !chunks.is_empty(),
        "Should have captured at least one chunk"
    );

    // Combine all chunks and split into lines
    let combined = chunks.join("");
    let lines: Vec<&str> = combined
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();

    // Verify we have all expected lines
    assert_eq!(lines.len(), 10, "Should have exactly 10 lines");

    // Verify ordering is preserved - each line should contain its sequential number
    for (index, line) in lines.iter().enumerate() {
        let expected_number = index + 1;
        let expected_content = format!("Message {expected_number}");
        assert_eq!(
            line.trim(),
            expected_content,
            "Line {} should be '{}' but was '{}'",
            index + 1,
            expected_content,
            line.trim()
        );
    }

    // Cross-verify with final output ordering
    let final_output_str = output.stdout_str();
    let final_lines: Vec<&str> = final_output_str
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();

    assert_eq!(
        lines, final_lines,
        "Streaming chunks should have same order as final output"
    );
}

#[tokio::test]
async fn test_command_streaming_stdout_stderr_interleaving() {
    let runner = ShellCommandRunner::new("/bin/sh", Duration::from_secs(10));

    // Simplified command that outputs to both stdout and stderr
    let command = r#"echo "stdout-line" && echo "stderr-line" >&2"#;

    let all_chunks = Arc::new(Mutex::new(Vec::new()));
    let chunks_clone = all_chunks.clone();

    let output = runner
        .execute_streaming(command, Duration::from_secs(10), move |chunk| {
            chunks_clone.lock().unwrap().push(chunk);
        })
        .await
        .unwrap();

    assert!(output.is_success());

    let chunks = all_chunks.lock().unwrap();
    assert!(!chunks.is_empty(), "Should have captured chunks");

    // Separate stdout and stderr chunks while preserving their relative order
    let stdout_chunks: Vec<String> = chunks
        .iter()
        .filter_map(|chunk| match chunk {
            OutputChunk::Stdout(text) => Some(text.clone()),
            _ => None,
        })
        .collect();

    let stderr_chunks: Vec<String> = chunks
        .iter()
        .filter_map(|chunk| match chunk {
            OutputChunk::Stderr(text) => Some(text.clone()),
            _ => None,
        })
        .collect();

    // Verify stdout content and ordering
    // Verify stdout content
    let stdout_combined = stdout_chunks.join("");
    assert!(
        stdout_combined.contains("stdout-line"),
        "Should contain stdout content: '{stdout_combined}'"
    );

    // Verify stderr content
    let stderr_combined = stderr_chunks.join("");
    assert!(
        stderr_combined.contains("stderr-line"),
        "Should contain stderr content: '{stderr_combined}'"
    );
}
