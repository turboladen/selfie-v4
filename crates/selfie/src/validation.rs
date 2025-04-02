use core::fmt;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ValidationIssues(Vec<ValidationIssue>);

impl ValidationIssues {
    pub fn all_issues(&self) -> &[ValidationIssue] {
        &self.0
    }

    /// Returns true if the validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        !self.has_errors()
    }

    pub fn has_issues(&self) -> bool {
        !self.0.is_empty()
    }

    /// Returns true if the validation has errors (warnings are okay)
    pub fn has_errors(&self) -> bool {
        self.0
            .iter()
            .any(|issue| matches!(issue.level, ValidationLevel::Error))
    }

    /// Get all errors (not warnings)
    ///
    pub fn errors(&self) -> Vec<&ValidationIssue> {
        self.0
            .iter()
            .filter(|issue| issue.level == ValidationLevel::Error)
            .collect()
    }

    /// Get all warnings (not errors)
    pub fn warnings(&self) -> Vec<&ValidationIssue> {
        self.0
            .iter()
            .filter(|issue| issue.level == ValidationLevel::Warning)
            .collect()
    }

    /// Get issues by category
    pub fn issues_by_category(&self, category: &ValidationErrorCategory) -> Vec<&ValidationIssue> {
        self.0
            .iter()
            .filter(|issue| issue.category == *category)
            .collect()
    }
}

impl From<Vec<ValidationIssue>> for ValidationIssues {
    fn from(value: Vec<ValidationIssue>) -> Self {
        Self(value)
    }
}

/// A single validation issue (error or warning)
///
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationIssue {
    /// The category of the issue
    ///
    pub(crate) category: ValidationErrorCategory,

    /// The field or context where the issue was found
    ///
    pub(crate) field: String,

    /// Detailed description of the issue
    ///
    pub(crate) message: String,

    /// Is this a warning (false = error)
    ///
    pub(crate) level: ValidationLevel,

    /// Suggested fix for the issue
    ///
    pub(crate) suggestion: Option<String>,
}

impl ValidationIssue {
    /// Create a new validation error
    ///
    pub(super) fn error(
        category: ValidationErrorCategory,
        field: &str,
        message: &str,
        suggestion: Option<&str>,
    ) -> Self {
        Self {
            category,
            field: field.to_string(),
            message: message.to_string(),
            level: ValidationLevel::Error,
            suggestion: suggestion.map(|s| s.to_string()),
        }
    }

    /// Create a new validation warning
    pub(super) fn warning(
        category: ValidationErrorCategory,
        field: &str,
        message: &str,
        suggestion: Option<&str>,
    ) -> Self {
        Self {
            category,
            field: field.to_string(),
            message: message.to_string(),
            level: ValidationLevel::Warning,
            suggestion: suggestion.map(|s| s.to_string()),
        }
    }

    pub fn category(&self) -> ValidationErrorCategory {
        self.category
    }

    pub fn field(&self) -> &str {
        &self.field
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn level(&self) -> ValidationLevel {
        self.level
    }

    pub fn suggestion(&self) -> Option<&String> {
        self.suggestion.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidationLevel {
    Error,
    Warning,
}

/// Categories of package validation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValidationErrorCategory {
    /// Missing required fields
    ///
    RequiredField,

    /// Invalid field values
    ///
    InvalidValue,

    /// Environment-specific errors
    ///
    Environment,

    /// Shell command syntax errors
    ///
    CommandSyntax,

    /// URL format errors
    ///
    UrlFormat,

    /// File system errors
    ///
    FileSystem,

    /// Availability and compatibility errors
    ///
    Availability,

    /// Other errors
    ///
    Other,
}

impl fmt::Display for ValidationErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RequiredField => f.write_str("required_field"),
            Self::InvalidValue => f.write_str("invalid_value"),
            Self::Environment => f.write_str("environment"),
            Self::CommandSyntax => f.write_str("command_syntax"),
            Self::UrlFormat => f.write_str("url_format"),
            Self::FileSystem => f.write_str("file_system"),
            Self::Availability => f.write_str("availability"),
            Self::Other => f.write_str("other"),
        }
    }
}
