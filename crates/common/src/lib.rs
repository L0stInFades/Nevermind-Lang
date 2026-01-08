//! Nevermind Common - Shared types and utilities

pub mod source;
pub mod error;
pub mod span;

pub use source::SourceLocation;
pub use error::{Error, ErrorKind, Result};
pub use span::Span;
