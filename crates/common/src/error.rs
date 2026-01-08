//! Error types for Nevermind

use std::fmt;
use std::path::PathBuf;

use thiserror::Error;

use crate::Span;

/// A result type for Nevermind operations
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Categories of errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// Lexical error
    Lexical,

    /// Syntax error
    Syntax,

    /// Type error
    Type,

    /// Name resolution error
    Resolution,

    /// Runtime error
    Runtime,

    /// IO error
    Io,

    /// Compilation error
    Compilation,
}

/// A compiler error with rich information
#[derive(Error, Debug)]
pub struct Error {
    /// The kind of error
    pub kind: ErrorKind,

    /// The error message
    pub message: String,

    /// The span where the error occurred
    pub span: Span,

    /// Additional contextual information
    pub context: Vec<ErrorContext>,

    /// Related errors (e.g., in a chain)
    pub related: Vec<Box<Error>>,
}

/// Contextual information about an error
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// A note or hint
    pub message: String,

    /// The location of this context (optional)
    pub span: Option<Span>,
}

impl Error {
    /// Create a new error
    pub fn new(kind: ErrorKind, message: impl Into<String>, span: Span) -> Self {
        Self {
            kind,
            message: message.into(),
            span,
            context: Vec::new(),
            related: Vec::new(),
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

    /// Add a related error
    pub fn with_related(mut self, error: Error) -> Self {
        self.related.push(Box::new(error));
        self
    }

    /// Create a lexical error
    pub fn lexical(message: impl Into<String>, span: Span) -> Self {
        Self::new(ErrorKind::Lexical, message, span)
    }

    /// Create a syntax error
    pub fn syntax(message: impl Into<String>, span: Span) -> Self {
        Self::new(ErrorKind::Syntax, message, span)
    }

    /// Create a type error
    pub fn type_error(message: impl Into<String>, span: Span) -> Self {
        Self::new(ErrorKind::Type, message, span)
    }

    /// Create a resolution error
    pub fn resolution(message: impl Into<String>, span: Span) -> Self {
        Self::new(ErrorKind::Resolution, message, span)
    }

    /// Create a runtime error
    pub fn runtime(message: impl Into<String>, span: Span) -> Self {
        Self::new(ErrorKind::Runtime, message, span)
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

        // Show related errors
        for related in &self.related {
            output.push_str(&related.display(source));
            output.push('\n');
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
        output.push_str(&"^".repeat(col_end - col_start));
        output.push('\n');

        Some(output)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: error: {}", self.span, self.message)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.related.first()?.as_ref().into()
    }
}

/// Convert IO errors
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::new(
            ErrorKind::Io,
            err.to_string(),
            Span::dummy(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let span = Span::dummy();
        let error = Error::syntax("unexpected token", span.clone());

        assert_eq!(error.kind, ErrorKind::Syntax);
        assert_eq!(error.message, "unexpected token");
        assert_eq!(error.span, span);
    }

    #[test]
    fn test_error_with_context() {
        let span = Span::dummy();
        let error = Error::syntax("unexpected token", span.clone())
            .with_context("expected 'let'", None)
            .with_context("see the documentation for more info", None);

        assert_eq!(error.context.len(), 2);
    }
}
