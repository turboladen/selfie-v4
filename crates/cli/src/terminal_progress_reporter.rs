use std::fmt::Display;

use console::{Emoji, style};

// Define emojis with fallbacks
static ERROR_EMOJI: Emoji<'_, '_> = Emoji("❌ ", "[E] ");
static INFO_EMOJI: Emoji<'_, '_> = Emoji("ℹ️ ", "[I] ");
static PROGRESS_EMOJI: Emoji<'_, '_> = Emoji("• ", " • ");
static SUGGESTION_EJOJI: Emoji<'_, '_> = Emoji("✨", "OK ");
static SUCCESS_EMOJI: Emoji<'_, '_> = Emoji("✅ ", "OK ");
static WARN_EMOJI: Emoji<'_, '_> = Emoji("⚠️ ", "[W] ");

/// Types of status messages
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum MessageType {
    Error,
    Info,
    Progress,
    Success,
    Suggestion,
    Warning,
}

#[derive(Debug, Clone, Copy)]
pub struct TerminalProgressReporter {
    use_colors: bool,
}

impl TerminalProgressReporter {
    #[must_use]
    pub fn new(use_colors: bool) -> Self {
        Self { use_colors }
    }
}

impl TerminalProgressReporter {
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

    pub(crate) fn format(indent: usize, message: impl Display) -> String {
        format!("{:indent$}{}", "", message, indent = indent)
    }

    pub(crate) fn format_error(self, message: impl Display) -> String {
        self.status_line(MessageType::Error, message)
    }

    pub(crate) fn format_info(self, message: impl Display) -> String {
        self.status_line(MessageType::Info, message)
    }

    pub(crate) fn format_progress(self, message: impl Display) -> String {
        self.status_line(MessageType::Progress, message)
    }

    pub(crate) fn format_suggestion(self, message: impl Display) -> String {
        self.status_line(MessageType::Suggestion, message)
    }

    pub(crate) fn format_success(self, message: impl Display) -> String {
        self.status_line(MessageType::Success, message)
    }

    pub(crate) fn format_warning(self, message: impl Display) -> String {
        self.status_line(MessageType::Warning, message)
    }

    pub(crate) fn report(indent: usize, message: impl Display) {
        println!("{}", Self::format(indent, message));
    }

    pub(crate) fn report_progress(self, message: impl Display) {
        println!("{}", self.format_progress(message));
    }

    pub(crate) fn report_success(self, message: impl Display) {
        println!("{}", self.format_success(message));
    }

    pub(crate) fn report_suggestion(self, message: impl Display) {
        println!("{}", self.format_suggestion(message));
    }

    pub(crate) fn report_info(self, message: impl Display) {
        println!("{}", self.format_info(message));
    }

    pub(crate) fn report_warning(self, message: impl Display) {
        println!("{}", self.format_warning(message));
    }

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
