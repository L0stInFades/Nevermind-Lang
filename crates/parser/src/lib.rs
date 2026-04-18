#![allow(clippy::result_large_err)]

//! Parser for Nevermind

mod error;
mod expr_parser;
mod parser;
mod pattern_parser;

pub use error::ParseError;
pub use parser::Parser;
