//! Nevermind Common - Shared types and utilities

pub mod error;
pub mod source;
pub mod span;

pub use error::{Error, ErrorKind, Result};
pub use source::SourceLocation;
pub use span::Span;
