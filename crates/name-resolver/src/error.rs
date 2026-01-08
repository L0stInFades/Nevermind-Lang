//! Error types for name resolution

use std::fmt;

use thiserror::Error;

use nevermind_common::Span;

/// Kinds of name resolution errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameErrorKind {
    /// Undefined variable or function
    UndefinedVariable(String),

    /// Duplicate definition in the same scope
    DuplicateDefinition(String),

    /// Invalid scope operation (e.g., exit without enter)
    InvalidScope,

    /// Return statement outside of function
    InvalidReturn,

    /// Break statement outside of loop
    InvalidBreak,

    /// Continue statement outside of loop
    InvalidContinue,

    /// Incorrect number of arguments
    ArgumentCountMismatch {
        expected: usize,
        found: usize,
    },
}

impl fmt::Display for NameErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NameErrorKind::UndefinedVariable(name) => {
                write!(f, "undefined variable or function '{}'", name)
            }
            NameErrorKind::DuplicateDefinition(name) => {
                write!(f, "duplicate definition of '{}'", name)
            }
            NameErrorKind::InvalidScope => {
                write!(f, "invalid scope operation")
            }
            NameErrorKind::InvalidReturn => {
                write!(f, "return statement outside of function")
            }
            NameErrorKind::InvalidBreak => {
                write!(f, "break statement outside of loop")
            }
            NameErrorKind::InvalidContinue => {
                write!(f, "continue statement outside of loop")
            }
            NameErrorKind::ArgumentCountMismatch { expected, found } => {
                write!(f, "expected {} argument(s), found {}", expected, found)
            }
        }
    }
}

/// A name resolution error
#[derive(Error, Debug, Clone)]
pub struct NameError {
    /// The kind of error
    pub kind: NameErrorKind,

    /// The error message
    pub message: String,

    /// The span where the error occurred
    pub span: Span,

    /// Additional contextual information
    pub context: Vec<ErrorContext>,
}

/// Contextual information about a name error
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// A note or hint
    pub message: String,

    /// The location of this context (optional)
    pub span: Option<Span>,
}

impl NameError {
    /// Create a new name error
    pub fn new(kind: NameErrorKind, message: impl Into<String>, span: Span) -> Self {
        Self {
            kind,
            message: message.into(),
            span,
            context: Vec::new(),
        }
    }

    /// Add contextual information to this error
    pub fn with_context(mut self, message: impl Into<String>, span: Option<Span>) -> Self {
        self.context.push(ErrorContext {
            message: message.into(),
            span,
        });
        self
    }

    /// Create an undefined variable error
    pub fn undefined_variable(name: String, span: Span) -> Self {
        Self::new(
            NameErrorKind::UndefinedVariable(name.clone()),
            format!("Cannot find value '{}' in this scope", name),
            span,
        )
    }

    /// Create a duplicate definition error
    pub fn duplicate_definition(name: String, span: Span) -> Self {
        Self::new(
            NameErrorKind::DuplicateDefinition(name.clone()),
            format!("Name '{}' is already defined in this scope", name),
            span,
        )
    }

    /// Create an invalid scope error
    pub fn invalid_scope(message: impl Into<String>, span: Span) -> Self {
        Self::new(
            NameErrorKind::InvalidScope,
            message,
            span,
        )
    }

    /// Create an invalid return error
    pub fn invalid_return(span: Span) -> Self {
        Self::new(
            NameErrorKind::InvalidReturn,
            "return statement can only be used inside a function".to_string(),
            span,
        )
    }

    /// Create an invalid break error
    pub fn invalid_break(span: Span) -> Self {
        Self::new(
            NameErrorKind::InvalidBreak,
            "break statement can only be used inside a loop".to_string(),
            span,
        )
    }

    /// Create an invalid continue error
    pub fn invalid_continue(span: Span) -> Self {
        Self::new(
            NameErrorKind::InvalidContinue,
            "continue statement can only be used inside a loop".to_string(),
            span,
        )
    }

    /// Create an argument count mismatch error
    pub fn argument_count_mismatch(expected: usize, found: usize, span: Span) -> Self {
        Self::new(
            NameErrorKind::ArgumentCountMismatch { expected, found },
            format!("Expected {} argument(s) but found {}", expected, found),
            span,
        )
    }

    /// Format the error for display
    pub fn display(&self, source: Option<&str>) -> String {
        let mut output = String::new();

        // Error header
        output.push_str(&format!("{}: error: {}\n", self.span, self.message));

        // Show source code snippet if available
        if let Some(source) = source {
            if let Some(snippet) = self.format_source_snippet(source) {
                output.push_str(&snippet);
                output.push('\n');
            }
        }

        // Show context
        for ctx in &self.context {
            if let Some(span) = &ctx.span {
                output.push_str(&format!("{}: note: {}\n", span, ctx.message));
            } else {
                output.push_str(&format!("note: {}\n", ctx.message));
            }
        }

        output
    }

    /// Format a source code snippet for this error
    fn format_source_snippet(&self, source: &str) -> Option<String> {
        let lines: Vec<&str> = source.lines().collect();

        let line_num = self.span.start.line;
        if line_num == 0 || line_num > lines.len() {
            return None;
        }

        let line = lines[line_num - 1];

        let mut output = String::new();

        // Show the line
        output.push_str(&format!("  {} | {}\n", line_num, line));

        // Show the error marker
        let col_start = self.span.start.column;
        let col_end = if self.span.start.line == self.span.end.line {
            self.span.end.column
        } else {
            col_start + 1
        };

        output.push_str(&format!("  {} | ", " ".repeat(line_num.to_string().len())));
        output.push_str(&" ".repeat(col_start - 1));
        output.push_str(&"^".repeat(col_end.saturating_sub(col_start)));
        output.push('\n');

        Some(output)
    }
}

impl fmt::Display for NameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: error: {}", self.span, self.message)
    }
}

/// Result type for name resolution operations
pub type Result<T, E = NameError> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let span = Span::dummy();
        let error = NameError::undefined_variable("x".to_string(), span.clone());

        assert!(matches!(error.kind, NameErrorKind::UndefinedVariable(_)));
        assert_eq!(error.message, "Cannot find value 'x' in this scope");
        assert_eq!(error.span, span);
    }

    #[test]
    fn test_error_with_context() {
        let span = Span::dummy();
        let error = NameError::undefined_variable("foo".to_string(), span.clone())
            .with_context("did you mean 'bar'?", None);

        assert_eq!(error.context.len(), 1);
        assert_eq!(error.context[0].message, "did you mean 'bar'?");
    }

    #[test]
    fn test_error_display() {
        let span = Span::dummy();
        let error = NameError::undefined_variable("my_var".to_string(), span);

        let display = error.display(None);
        assert!(display.contains("Cannot find value 'my_var'"));
    }
}
