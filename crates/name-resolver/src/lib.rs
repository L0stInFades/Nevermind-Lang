#![allow(clippy::result_large_err)]

//! Name resolution for Nevermind
//!
//! This crate provides name resolution functionality for the Nevermind compiler.
//! It handles:
//!
//! - Symbol declaration and resolution
//! - Scope management (global, function, block, loop scopes)
//! - Detection of undefined variables
//! - Detection of duplicate definitions
//! - Support for variable shadowing
//! - Validation of control flow (return, break, continue)
//!
//! ## Example
//!
//! ```rust
//! use nevermind_name_resolver::{NameResolver, Symbol, SymbolKind};
//! use nevermind_ast::{Stmt, Expr};
//! use nevermind_common::Span;
//!
//! let mut resolver = NameResolver::new();
//!
//! // Resolve a list of statements
//! let stmts = vec![
//!     // ... your AST statements
//! ];
//!
//! match resolver.resolve(&stmts) {
//!     Ok(()) => println!("Name resolution successful"),
//!     Err(errors) => {
//!         for error in errors {
//!             eprintln!("Error: {}", error);
//!         }
//!     }
//! }
//! ```

pub mod error;
pub mod resolver;
pub mod scope;
pub mod symbol;
pub mod symbol_table;

// Re-export common types for convenience
pub use error::{NameError, NameErrorKind, Result};
pub use resolver::NameResolver;
pub use scope::Scope;
pub use symbol::{Symbol, SymbolKind};
pub use symbol_table::SymbolTable;
