#![allow(clippy::result_large_err)]

//! Lexer for Nevermind

pub mod lexer;
pub mod token;

pub use lexer::Lexer;
pub use token::{Token, TokenType};
