use std::fmt::Display;

use console::{Emoji, style};

use crate::progress_reporter::port::MessageType;

use super::port::ProgressReporter;

// Define emojis with fallbacks
static ERROR_EMOJI: Emoji<'_, '_> = Emoji("❌ ", "[E] ");
static WARN_EMOJI: Emoji<'_, '_> = Emoji("⚠️ ", "[W] ");
static INFO_EMOJI: Emoji<'_, '_> = Emoji("ℹ️ ", "[I] ");
static PROGRESS_EMOJI: Emoji<'_, '_> = Emoji("• ", " • ");
static SUCCESS_EMOJI: Emoji<'_, '_> = Emoji("✅ ", "OK ");

#[derive(Debug, Clone, Copy)]
pub struct TerminalProgressReporter {
    use_colors: bool,
}

impl TerminalProgressReporter {
    pub fn new(use_colors: bool) -> Self {
        Self { use_colors }
    }
}

impl ProgressReporter for TerminalProgressReporter {
    fn status_line<T: Display + 'static>(
        &self,
        message_type: super::port::MessageType,
        message: T,
    ) -> String {
        let prefix = match message_type {
            MessageType::Progress => PROGRESS_EMOJI,
            MessageType::Info => INFO_EMOJI,
            MessageType::Success => SUCCESS_EMOJI,
            MessageType::Error => ERROR_EMOJI,
            MessageType::Warning => WARN_EMOJI,
        };

        let formatted_message = if self.use_colors {
            match message_type {
                MessageType::Error => style(message).for_stderr().red().bold().to_string(),
                MessageType::Warning => style(message).for_stderr().yellow().bold().to_string(),
                MessageType::Info => style(message).blue().to_string(),
                MessageType::Progress => style(message).dim().to_string(),
                MessageType::Success => style(message).green().to_string(),
            }
        } else {
            message.to_string()
        };

        format!("{}{}", prefix, formatted_message)
    }

    fn format<T: Display + 'static>(&self, message: T) -> String {
        message.to_string()
    }

    fn report<T: Display + 'static>(&self, message: T) {
        println!("{}", self.format(message));
    }

    fn report_progress<T: Display + 'static>(&self, message: T) {
        println!("{}", self.format_progress(message));
    }

    fn report_success<T: Display + 'static>(&self, message: T) {
        println!("{}", self.format_success(message));
    }

    fn report_info<T: Display + 'static>(&self, message: T) {
        println!("{}", self.format_info(message));
    }

    fn report_warning<T: Display + 'static>(&self, message: T) {
        println!("{}", self.format_warning(message));
    }

    fn report_error<T: Display + 'static>(&self, message: T) {
        eprintln!("{}", self.format_error(message));
    }
}
