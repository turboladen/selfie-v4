use std::fmt;

pub trait ProgressReporter {
    fn status_line(&self, message_type: MessageType, message: impl fmt::Display) -> String;

    fn format_progress(&self, message: impl fmt::Display) -> String {
        self.status_line(MessageType::Progress, message)
    }

    fn format_success(&self, message: impl fmt::Display) -> String {
        self.status_line(MessageType::Success, message)
    }

    fn format_info(&self, message: impl fmt::Display) -> String {
        self.status_line(MessageType::Info, message)
    }

    fn format_warning(&self, message: impl fmt::Display) -> String {
        self.status_line(MessageType::Warning, message)
    }

    fn format_error(&self, message: impl fmt::Display) -> String {
        self.status_line(MessageType::Error, message)
    }

    fn report_progress(&self, message: impl fmt::Display);
    fn report_success(&self, message: impl fmt::Display);
    fn report_info(&self, message: impl fmt::Display);
    fn report_warning(&self, message: impl fmt::Display);
    fn report_error(&self, message: impl fmt::Display);
}

/// Types of status messages
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageType {
    Progress,
    Info,
    Success,
    Error,
    Warning,
}
