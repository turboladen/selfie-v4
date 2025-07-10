//! Terminal progress reporting and output formatting
//!
//! This module provides a consistent interface for displaying progress updates,
//! status messages, and other user feedback in the terminal. It handles emoji
//! fallbacks for terminals that don't support Unicode and provides colored
//! output when appropriate.
//!
//! # Features
//!
//! - Consistent emoji prefixes for different message types
//! - Graceful fallback to text indicators when emojis aren't supported
//! - Colored output with automatic color detection
//! - Support for different message severity levels
//! - Structured formatting for various UI contexts
//!
//! # Examples
//!
//! ```rust
//! use crate::terminal_progress_reporter::TerminalProgressReporter;
//!
//! let reporter = TerminalProgressReporter::new(true); // Enable colors
//! reporter.report_success("Package installed successfully");
//! reporter.report_error("Failed to install package");
//! reporter.report_progress("Installing dependencies...");
//! ```

use std::fmt::Display;

use console::{Emoji, style};

// Define emojis with fallbacks for terminals that don't support Unicode
static ERROR_EMOJI: Emoji<'_, '_> = Emoji("❌ ", "[E] ");
static INFO_EMOJI: Emoji<'_, '_> = Emoji("ℹ️ ", "[I] ");
static PROGRESS_EMOJI: Emoji<'_, '_> = Emoji("• ", " • ");
static SUGGESTION_EJOJI: Emoji<'_, '_> = Emoji("✨", "OK ");
static SUCCESS_EMOJI: Emoji<'_, '_> = Emoji("✅ ", "OK ");
static WARN_EMOJI: Emoji<'_, '_> = Emoji("⚠️ ", "[W] ");

/// Types of status messages that can be displayed to the user
///
/// Each message type has its own visual styling, emoji/text prefix,
/// and color scheme to help users quickly understand the nature
/// of the information being presented.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum MessageType {
    /// Error messages for failures and critical issues
    Error,
    /// Informational messages for general status updates
    Info,
    /// Progress indicators for ongoing operations
    Progress,
    /// Success messages for completed operations
    Success,
    /// Helpful suggestions and recommendations
    Suggestion,
    /// Warning messages for potential issues
    Warning,
}

/// Terminal progress reporter for consistent CLI output formatting
///
/// Provides a unified interface for displaying various types of messages
/// to the user with appropriate styling, colors, and emoji indicators.
/// Automatically handles fallbacks for terminals with limited capabilities.
#[derive(Debug, Clone, Copy)]
pub struct TerminalProgressReporter {
    /// Whether to use colored output (respects user preference and terminal capabilities)
    use_colors: bool,
}

impl TerminalProgressReporter {
    /// Create a new terminal progress reporter
    ///
    /// # Arguments
    ///
    /// * `use_colors` - Whether to enable colored output formatting
    ///
    /// # Examples
    ///
    /// ```rust
    /// let reporter = TerminalProgressReporter::new(true);  // With colors
    /// let reporter = TerminalProgressReporter::new(false); // Plain text only
    /// ```
    #[must_use]
    pub fn new(use_colors: bool) -> Self {
        Self { use_colors }
    }
}

impl TerminalProgressReporter {
    /// Format a status line with appropriate styling and prefix
    ///
    /// Creates a formatted status line with the appropriate emoji/text prefix
    /// and color styling based on the message type and color settings.
    ///
    /// # Arguments
    ///
    /// * `message_type` - The type of message to format
    /// * `message` - The message content to display
    ///
    /// # Returns
    ///
    /// A formatted string ready for display in the terminal
    pub(crate) fn status_line(self, message_type: MessageType, message: impl Display) -> String {
        let prefix = match message_type {
            MessageType::Error => ERROR_EMOJI,
            MessageType::Info => INFO_EMOJI,
            MessageType::Progress => PROGRESS_EMOJI,
            MessageType::Success => SUCCESS_EMOJI,
            MessageType::Suggestion => SUGGESTION_EJOJI,
            MessageType::Warning => WARN_EMOJI,
        };

        let formatted_message = if self.use_colors {
            match message_type {
                MessageType::Error => style(message).for_stderr().red().bold().to_string(),
                MessageType::Info => style(message).blue().to_string(),
                MessageType::Progress => style(message).dim().to_string(),
                MessageType::Success => style(message).green().to_string(),
                MessageType::Suggestion => {
                    return format!(
                        "{prefix} {}: {}",
                        style("Suggestion").yellow().bold(),
                        &message
                    );
                }
                MessageType::Warning => style(message).for_stderr().yellow().bold().to_string(),
            }
        } else {
            message.to_string()
        };

        format!("{prefix}{formatted_message}")
    }

    /// Format a message with the specified indentation
    ///
    /// Adds leading whitespace to create visual hierarchy in terminal output.
    /// Useful for nested information or sub-items in lists.
    ///
    /// # Arguments
    ///
    /// * `indent` - Number of spaces to indent the message
    /// * `message` - The message content to format
    ///
    /// # Returns
    ///
    /// The message with appropriate leading whitespace
    pub(crate) fn format(indent: usize, message: impl Display) -> String {
        format!("{:indent$}{}", "", message, indent = indent)
    }

    /// Format an error message with appropriate styling
    ///
    /// Creates a formatted error message with red coloring (if enabled)
    /// and an error emoji/indicator prefix.
    pub(crate) fn format_error(self, message: impl Display) -> String {
        self.status_line(MessageType::Error, message)
    }

    /// Format an informational message with appropriate styling
    ///
    /// Creates a formatted info message with blue coloring (if enabled)
    /// and an info emoji/indicator prefix.
    pub(crate) fn format_info(self, message: impl Display) -> String {
        self.status_line(MessageType::Info, message)
    }

    /// Format a progress message with appropriate styling
    ///
    /// Creates a formatted progress message with dim coloring (if enabled)
    /// and a progress emoji/indicator prefix.
    pub(crate) fn format_progress(self, message: impl Display) -> String {
        self.status_line(MessageType::Progress, message)
    }

    /// Format a suggestion message with appropriate styling
    ///
    /// Creates a formatted suggestion message with yellow coloring (if enabled)
    /// and a suggestion emoji/indicator prefix.
    pub(crate) fn format_suggestion(self, message: impl Display) -> String {
        self.status_line(MessageType::Suggestion, message)
    }

    /// Format a success message with appropriate styling
    ///
    /// Creates a formatted success message with green coloring (if enabled)
    /// and a success emoji/indicator prefix.
    pub(crate) fn format_success(self, message: impl Display) -> String {
        self.status_line(MessageType::Success, message)
    }

    /// Format a warning message with appropriate styling
    ///
    /// Creates a formatted warning message with yellow coloring (if enabled)
    /// and a warning emoji/indicator prefix.
    pub(crate) fn format_warning(self, message: impl Display) -> String {
        self.status_line(MessageType::Warning, message)
    }

    /// Print a message with the specified indentation to stdout
    ///
    /// Convenience method for printing indented messages without specific styling.
    ///
    /// # Arguments
    ///
    /// * `indent` - Number of spaces to indent the message
    /// * `message` - The message content to print
    pub(crate) fn report(indent: usize, message: impl Display) {
        println!("{}", Self::format(indent, message));
    }

    /// Print a formatted progress message to stdout
    ///
    /// Displays a progress message with appropriate styling and prefix.
    /// Useful for showing ongoing operation status to the user.
    pub(crate) fn report_progress(self, message: impl Display) {
        println!("{}", self.format_progress(message));
    }

    /// Print a formatted success message to stdout
    ///
    /// Displays a success message with green styling and success indicator.
    /// Used to confirm successful completion of operations.
    pub(crate) fn report_success(self, message: impl Display) {
        println!("{}", self.format_success(message));
    }

    /// Print a formatted suggestion message to stdout
    ///
    /// Displays a suggestion message with yellow styling and suggestion indicator.
    /// Used to provide helpful recommendations to the user.
    pub(crate) fn report_suggestion(self, message: impl Display) {
        println!("{}", self.format_suggestion(message));
    }

    /// Print a formatted informational message to stdout
    ///
    /// Displays an info message with blue styling and info indicator.
    /// Used for general status updates and non-critical information.
    pub(crate) fn report_info(self, message: impl Display) {
        println!("{}", self.format_info(message));
    }

    /// Print a formatted warning message to stdout
    ///
    /// Displays a warning message with yellow styling and warning indicator.
    /// Used to alert users to potential issues that don't prevent operation.
    pub(crate) fn report_warning(self, message: impl Display) {
        println!("{}", self.format_warning(message));
    }

    /// Print a formatted error message to stderr
    ///
    /// Displays an error message with red styling and error indicator.
    /// Uses stderr for proper error stream handling in scripts and pipelines.
    pub(crate) fn report_error(self, message: impl Display) {
        eprintln!("{}", self.format_error(message));
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
}
