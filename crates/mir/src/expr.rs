//! MIR expressions

use super::NodeId;
use nevermind_type_checker::Type;

/// Mid-level IR expression
#[derive(Debug, Clone)]
pub enum MirExpr {
    /// Literal value
    Literal {
        value: Literal,
        ty: Type,
        id: NodeId,
    },

    /// Variable reference
    Variable {
        name: String,
        ty: Type,
        id: NodeId,
    },

    /// Binary operation
    Binary {
        op: BinOp,
        left: Box<MirExpr>,
        right: Box<MirExpr>,
        ty: Type,
        id: NodeId,
    },

    /// Unary operation
    Unary {
        op: UnaryOp,
        operand: Box<MirExpr>,
        ty: Type,
        id: NodeId,
    },

    /// Function call
    Call {
        callee: Box<MirExpr>,
        args: Vec<MirExpr>,
        ty: Type,
        id: NodeId,
    },

    /// Block expression
    Block {
        statements: Vec<MirExprStmt>,
        expr: Option<Box<MirExpr>>,
        ty: Type,
        id: NodeId,
    },
}

impl MirExpr {
    /// Get the expression's type
    pub fn get_type(&self) -> &Type {
        match self {
            MirExpr::Literal { ty, .. } => ty,
            MirExpr::Variable { ty, .. } => ty,
            MirExpr::Binary { ty, .. } => ty,
            MirExpr::Unary { ty, .. } => ty,
            MirExpr::Call { ty, .. } => ty,
            MirExpr::Block { ty, .. } => ty,
        }
    }

    /// Get the node ID
    pub fn get_id(&self) -> NodeId {
        match self {
            MirExpr::Literal { id, .. } => *id,
            MirExpr::Variable { id, .. } => *id,
            MirExpr::Binary { id, .. } => *id,
            MirExpr::Unary { id, .. } => *id,
            MirExpr::Call { id, .. } => *id,
            MirExpr::Block { id, .. } => *id,
        }
    }

    /// Check if this is a literal value
    pub fn is_literal(&self) -> bool {
        matches!(self, MirExpr::Literal { .. })
    }

    /// Check if this is a variable reference
    pub fn is_variable(&self) -> bool {
        matches!(self, MirExpr::Variable { .. })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logical
    And,
    Or,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// Literal values
#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
}

/// MIR statements used inside blocks
#[derive(Debug, Clone)]
pub enum MirExprStmt {
    /// Variable definition: let x = value
    Let {
        name: String,
        value: MirExpr,
        ty: Type,
        id: NodeId,
    },

    /// Assignment: x = value
    Assign {
        target: String,
        value: MirExpr,
        id: NodeId,
    },

    /// Expression statement
    Expr(MirExpr),

    /// Return statement
    Return {
        value: Option<Box<MirExpr>>,
        id: NodeId,
    },
}

impl MirExprStmt {
    pub fn get_id(&self) -> NodeId {
        match self {
            MirExprStmt::Let { id, .. } => *id,
            MirExprStmt::Assign { id, .. } => *id,
            MirExprStmt::Expr(expr) => expr.get_id(),
            MirExprStmt::Return { id, .. } => *id,
        }
    }
}

/// A block of statements with an optional expression
#[derive(Debug, Clone)]
pub struct MirBlock {
    pub statements: Vec<MirExprStmt>,
    pub expr: Option<Box<MirExpr>>,
}

impl MirBlock {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
            expr: None,
        }
    }

    pub fn with_expr(mut self, expr: MirExpr) -> Self {
        self.expr = Some(Box::new(expr));
        self
    }

    pub fn add_stmt(mut self, stmt: MirExprStmt) -> Self {
        self.statements.push(stmt);
        self
    }
}