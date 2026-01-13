//! Error types for type checking

use std::fmt;
use thiserror::Error;
use nevermind_common::Span;
use crate::types::Type;

/// Kinds of type errors
#[derive(Debug, Clone, PartialEq)]
pub enum TypeErrorKind {
    /// Type mismatch
    TypeMismatch {
        expected: Type,
        found: Type,
    },

    /// Undefined variable or function
    UndefinedVariable(String),

    /// Duplicate definition
    DuplicateDefinition(String),

    /// Invalid scope operation
    InvalidScope,

    /// Arity mismatch (wrong number of arguments)
    ArityMismatch {
        expected: usize,
        found: usize,
    },

    /// Not a function
    NotAFunction(Type),

    /// Cannot infer type
    CannotInfer(String),

    /// Recursive type
    RecursiveType,

    /// Occurs check failed (infinite type)
    OccursCheckFailed(usize),
}

impl fmt::Display for TypeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeErrorKind::TypeMismatch { expected, found } => {
                write!(f, "type mismatch: expected {}, found {}", expected.display_name(), found.display_name())
            }
            TypeErrorKind::UndefinedVariable(name) => {
                write!(f, "cannot find value '{}' in this scope", name)
            }
            TypeErrorKind::DuplicateDefinition(name) => {
                write!(f, "name '{}' is already defined in this scope", name)
            }
            TypeErrorKind::InvalidScope => {
                write!(f, "invalid scope operation")
            }
            TypeErrorKind::ArityMismatch { expected, found } => {
                write!(f, "this function takes {} argument{} but {} {} supplied",
                    expected,
                    if *expected == 1 { "" } else { "s" },
                    found,
                    if *found == 1 { "was" } else { "were" })
            }
            TypeErrorKind::NotAFunction(ty) => {
                write!(f, "{} is not a function", ty.display_name())
            }
            TypeErrorKind::CannotInfer(msg) => {
                write!(f, "cannot infer type: {}", msg)
            }
            TypeErrorKind::RecursiveType => {
                write!(f, "recursive types are not supported")
            }
            TypeErrorKind::OccursCheckFailed(id) => {
                write!(f, "infinite type: t{}", id)
            }
        }
    }
}

/// A type error
#[derive(Error, Debug, Clone)]
pub struct TypeError {
    /// The kind of error
    pub kind: TypeErrorKind,

    /// The error message
    pub message: String,

    /// The span where the error occurred
    pub span: Span,

    /// Additional context information
    pub context: Vec<ErrorContext>,
}

/// Contextual information about a type error
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// A note or hint
    pub message: String,

    /// The location of this context (optional)
    pub span: Option<Span>,
}

impl TypeError {
    /// Create a new type error
    pub fn new(kind: TypeErrorKind, message: String, span: Span) -> Self {
        Self {
            kind,
            message,
            span,
            context: Vec::new(),
        }
    }

    /// Add contextual information to this error
    pub fn with_context(mut self, message: String, span: Option<Span>) -> Self {
        self.context.push(ErrorContext {
            message,
            span,
        });
        self
    }

    /// Create a type mismatch error
    pub fn type_mismatch(expected: Type, found: Type, span: Span) -> Self {
        Self::new(
            TypeErrorKind::TypeMismatch {
                expected: expected.clone(),
                found: found.clone(),
            },
            format!("expected {}, found {}", expected.display_name(), found.display_name()),
            span,
        )
    }

    /// Create an undefined variable error
    pub fn undefined_variable(name: String, span: Span) -> Self {
        Self::new(
            TypeErrorKind::UndefinedVariable(name.clone()),
            format!("cannot find value '{}' in this scope", name),
            span,
        )
    }

    /// Create a duplicate definition error
    pub fn duplicate_definition(name: String, span: Span) -> Self {
        Self::new(
            TypeErrorKind::DuplicateDefinition(name.clone()),
            format!("name '{}' is already defined in this scope", name),
            span,
        )
    }

    /// Create an arity mismatch error
    pub fn arity_mismatch(expected: usize, found: usize, span: Span) -> Self {
        Self::new(
            TypeErrorKind::ArityMismatch { expected, found },
            format!("expected {} argument{}, found {}",
                expected,
                if expected == 1 { "" } else { "s" },
                found),
            span,
        )
    }

    /// Create a "not a function" error
    pub fn not_a_function(ty: Type, span: Span) -> Self {
        Self::new(
            TypeErrorKind::NotAFunction(ty.clone()),
            format!("{} is not a function", ty.display_name()),
            span,
        )
    }

    /// Create a "cannot infer" error
    pub fn cannot_infer(msg: String, span: Span) -> Self {
        Self::new(
            TypeErrorKind::CannotInfer(msg.clone()),
            format!("cannot infer type: {}", msg),
            span,
        )
    }

    /// Format the error for display
    pub fn display(&self, source: Option<&str>) -> String {
        let mut output = String::new();

        // Error header
        output.push_str(&format!("error: {}\n", self.message));

        // Show location
        if self.span.start.line > 0 {
            let file = self.span.start.file
                .as_ref()
                .and_then(|p| p.to_str())
                .unwrap_or("<anon>");
            output.push_str(&format!("  --> {}:{}:{}\n",
                file,
                self.span.start.line,
                self.span.start.column));
        }

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
                let file = span.start.file
                    .as_ref()
                    .and_then(|p| p.to_str())
                    .unwrap_or("<anon>");
                output.push_str(&format!("  --> {}:{}:{}: note: {}\n",
                    file,
                    span.start.line,
                    span.start.column,
                    ctx.message));
            } else {
                output.push_str(&format!("  note: {}\n", ctx.message));
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
        output.push_str(&format!("  {} |\n", line_num));
        output.push_str(&format!("  {} | {}\n", line_num, line));

        // Show the error marker
        let col_start = self.span.start.column;
        let col_end = if self.span.start.line == self.span.end.line {
            self.span.end.column
        } else {
            col_start + 1
        };

        output.push_str(&format!("  {} |", " ".repeat(line_num.to_string().len())));
        output.push_str(&" ".repeat(col_start));
        output.push_str(&"^".repeat(col_end.saturating_sub(col_start).max(1)));
        output.push('\n');

        Some(output)
    }
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: error: {}", self.span, self.message)
    }
}

/// Result type for type checking operations
pub type Result<T, E = TypeError> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let span = Span::dummy();
        let error = TypeError::type_mismatch(Type::Int, Type::Bool, span.clone());

        assert!(matches!(error.kind, TypeErrorKind::TypeMismatch { .. }));
    }

    #[test]
    fn test_error_with_context() {
        let span = Span::dummy();
        let error = TypeError::undefined_variable("x".to_string(), span.clone())
            .with_context("did you mean 'y'?".to_string(), None);

        assert_eq!(error.context.len(), 1);
        assert_eq!(error.context[0].message, "did you mean 'y'?");
    }

    #[test]
    fn test_arity_mismatch() {
        let span = Span::dummy();
        let error = TypeError::arity_mismatch(2, 3, span);

        assert!(matches!(error.kind, TypeErrorKind::ArityMismatch { .. }));
    }
}
