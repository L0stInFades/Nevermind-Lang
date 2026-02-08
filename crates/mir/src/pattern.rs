//! MIR patterns

use super::NodeId;
use nevermind_type_checker::Type;

/// MIR patterns (used in pattern matching)
#[derive(Debug, Clone)]
pub enum MirPattern {
    /// Wildcard pattern: _
    Wildcard {
        id: NodeId,
    },

    /// Variable pattern: x
    Variable {
        name: String,
        ty: Type,
        id: NodeId,
    },

    /// Literal pattern: 42, "hello", true
    Literal {
        value: super::expr::Literal,
        id: NodeId,
    },

    /// List pattern: [a, b, ...rest]
    List {
        patterns: Vec<MirPattern>,
        rest: Option<String>,
        id: NodeId,
    },

    /// Constructor pattern: Some(x), None, Ok(value)
    Constructor {
        name: String,
        args: Vec<MirPattern>,
        id: NodeId,
    },
}
