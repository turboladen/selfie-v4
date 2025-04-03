use selfie::progress_reporter::{port::ProgressReporter, terminal::TerminalProgressReporter};

#[test]
fn test_terminal_reporter_formatting() {
    // Test with colors enabled
    let reporter = TerminalProgressReporter::new(true);

    // Check formatting of different message types
    let success_msg = reporter.format_success("Test successful");
    let error_msg = reporter.format_error("Test failed");
    let info_msg = reporter.format_info("Test information");

    // Verify prefixes are included
    assert!(success_msg.contains("Test successful"));
    assert!(error_msg.contains("Test failed"));
    assert!(info_msg.contains("Test information"));

    // Prefix indicators should be present
    assert!(success_msg.contains("✅") || success_msg.contains("OK"));
    assert!(error_msg.contains("❌") || error_msg.contains("[E]"));
    assert!(info_msg.contains("ℹ️") || info_msg.contains("[I]"));
}

#[test]
fn test_terminal_reporter_without_colors() {
    // Test with colors disabled
    let reporter = TerminalProgressReporter::new(false);

    let success_msg = reporter.format_success("Test successful");

    // Message should be plain text without ANSI color codes
    assert!(!success_msg.contains("\x1b["));
}
