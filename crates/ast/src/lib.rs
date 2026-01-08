//! Abstract Syntax Tree definitions for Nevermind

pub mod expr;
pub mod stmt;
pub mod pattern;
pub mod types;
pub mod op;

pub use expr::Expr;
pub use stmt::Stmt;
pub use pattern::Pattern;
pub use types::{Type, TypeAnnotation};
pub use op::{BinaryOp, UnaryOp, LogicalOp};

use nevermind_common::Span;

/// A unique identifier for AST nodes
pub type NodeId = usize;

/// Generate a new unique node ID
pub fn new_node_id() -> NodeId {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
