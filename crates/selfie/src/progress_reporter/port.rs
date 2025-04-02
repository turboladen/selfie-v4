use std::fmt::Display;

#[cfg_attr(feature = "with_mocks", mockall::automock)]
pub trait ProgressReporter: Send + Sync {
    fn status_line<T: Display + 'static>(&self, message_type: MessageType, message: T) -> String;

    fn format<T: Display + 'static>(&self, message: T) -> String;

    fn format_progress<T: Display + 'static>(&self, message: T) -> String {
        self.status_line(MessageType::Progress, message)
    }

    fn format_success<T: Display + 'static>(&self, message: T) -> String {
        self.status_line(MessageType::Success, message)
    }

    fn format_info<T: Display + 'static>(&self, message: T) -> String {
        self.status_line(MessageType::Info, message)
    }

    fn format_warning<T: Display + 'static>(&self, message: T) -> String {
        self.status_line(MessageType::Warning, message)
    }

    fn format_error<T: Display + 'static>(&self, message: T) -> String {
        self.status_line(MessageType::Error, message)
    }

    fn report<T: Display + 'static>(&self, message: T);
    fn report_progress<T: Display + 'static>(&self, message: T);
    fn report_success<T: Display + 'static>(&self, message: T);
    fn report_info<T: Display + 'static>(&self, message: T);
    fn report_warning<T: Display + 'static>(&self, message: T);
    fn report_error<T: Display + 'static>(&self, message: T);
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
