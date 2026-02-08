//! MIR statements (top-level)

use super::{NodeId, MirExpr, MirBlock, MirPattern};
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

    /// If statement (statement-level, not expression-level)
    If {
        condition: MirExpr,
        then_body: Vec<MirStmt>,
        else_body: Option<Vec<MirStmt>>,
        id: NodeId,
    },

    /// While loop
    While {
        condition: MirExpr,
        body: Vec<MirStmt>,
        id: NodeId,
    },

    /// For loop
    For {
        variable: String,
        iter: MirExpr,
        body: Vec<MirStmt>,
        id: NodeId,
    },

    /// Return statement
    Return {
        value: Option<MirExpr>,
        id: NodeId,
    },

    /// Break statement
    Break {
        id: NodeId,
    },

    /// Continue statement
    Continue {
        id: NodeId,
    },

    /// Match statement
    Match {
        scrutinee: MirExpr,
        arms: Vec<MirMatchArm>,
        id: NodeId,
    },
}

impl MirStmt {
    /// Get the node ID
    pub fn get_id(&self) -> NodeId {
        match self {
            MirStmt::Function { id, .. } => *id,
            MirStmt::Let { id, .. } => *id,
            MirStmt::Expr(expr) => expr.get_id(),
            MirStmt::If { id, .. } => *id,
            MirStmt::While { id, .. } => *id,
            MirStmt::For { id, .. } => *id,
            MirStmt::Return { id, .. } => *id,
            MirStmt::Break { id } => *id,
            MirStmt::Continue { id } => *id,
            MirStmt::Match { id, .. } => *id,
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

/// A match arm in a MIR match statement
#[derive(Debug, Clone)]
pub struct MirMatchArm {
    pub pattern: MirPattern,
    pub guard: Option<MirExpr>,
    pub body: Vec<MirStmt>,
}
