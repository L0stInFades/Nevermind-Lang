//! Abstract Syntax Tree definitions for Nevermind

pub mod expr;
pub mod op;
pub mod pattern;
pub mod stmt;
pub mod types;

pub use expr::{Expr, Literal, MatchArm, Parameter};
pub use op::{BinaryOp, ComparisonOp, LogicalOp, UnaryOp};
pub use pattern::Pattern;
pub use stmt::Stmt;
pub use types::{Type, TypeAnnotation};

/// A unique identifier for AST nodes
pub type NodeId = usize;

/// Generate a new unique node ID
pub fn new_node_id() -> NodeId {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
