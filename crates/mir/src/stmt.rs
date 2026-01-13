//! MIR statements (top-level)

use super::{NodeId, MirExpr, MirBlock};
use nevermind_type_checker::Type;

/// Top-level MIR statements
#[derive(Debug, Clone)]
pub enum MirStmt {
    /// Function definition
    Function {
        name: String,
        params: Vec<Param>,
        body: MirBlock,
        return_type: Type,
        id: NodeId,
    },

    /// Variable declaration
    Let {
        name: String,
        value: MirExpr,
        ty: Type,
        id: NodeId,
    },

    /// Expression statement
    Expr(MirExpr),
}

impl MirStmt {
    /// Get the node ID
    pub fn get_id(&self) -> NodeId {
        match self {
            MirStmt::Function { id, .. } => *id,
            MirStmt::Let { id, .. } => *id,
            MirStmt::Expr(expr) => expr.get_id(),
        }
    }
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub id: NodeId,
}