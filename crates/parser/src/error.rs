//! Parse errors

use thiserror::Error;

use nevermind_common::{Error, Span};

/// A parse error
#[derive(Error, Debug)]
pub struct ParseError {
    /// The error message
    pub message: String,

    /// The location in the source
    pub span: Span,
}

impl ParseError {
    /// Create a new parse error
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }

    /// Convert to a compiler Error
    pub fn into_error(self) -> Error {
        Error::syntax(self.message, self.span)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.span, self.message)
    }
}

/// A result type for parsing
pub type ParseResult<T> = std::result::Result<T, ParseError>;
