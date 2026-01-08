//! Expression nodes

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{NodeId, op::{BinaryOp, UnaryOp, LogicalOp, ComparisonOp}};
use crate::{Pattern, TypeAnnotation};
use nevermind_common::Span;

/// An expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    /// Literal value
    Literal(Literal),

    /// Variable reference
    Variable {
        id: NodeId,
        name: String,
        span: Span,
    },

    /// Binary operation
    Binary {
        id: NodeId,
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        span: Span,
    },

    /// Comparison operation
    Comparison {
        id: NodeId,
        left: Box<Expr>,
        op: ComparisonOp,
        right: Box<Expr>,
        span: Span,
    },

    /// Logical operation
    Logical {
        id: NodeId,
        left: Box<Expr>,
        op: LogicalOp,
        right: Box<Expr>,
        span: Span,
    },

    /// Unary operation
    Unary {
        id: NodeId,
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },

    /// Function call
    Call {
        id: NodeId,
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },

    /// Pipe operation (expression |> function)
    Pipeline {
        id: NodeId,
        stages: Vec<Expr>,
        span: Span,
    },

    /// Lambda/function expression
    Lambda {
        id: NodeId,
        params: Vec<Parameter>,
        body: Box<Expr>,
        span: Span,
    },

    /// If expression
    If {
        id: NodeId,
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
        span: Span,
    },

    /// Block expression
    Block {
        id: NodeId,
        statements: Vec<crate::Stmt>,
        span: Span,
    },

    /// List literal
    List {
        id: NodeId,
        elements: Vec<Expr>,
        span: Span,
    },

    /// Map/dict literal
    Map {
        id: NodeId,
        entries: Vec<(Expr, Expr)>,
        span: Span,
    },

    /// Match expression
    Match {
        id: NodeId,
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },
}

/// A function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub id: NodeId,
    pub name: String,
    pub type_annotation: Option<TypeAnnotation>,
    pub default_value: Option<Box<Expr>>,
}

/// A match arm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,
    pub body: Box<Expr>,
}

impl Expr {
    /// Get the span of this expression
    pub fn span(&self) -> &Span {
        match self {
            Expr::Literal(lit) => &lit.span,
            Expr::Variable { span, .. } => span,
            Expr::Binary { span, .. } => span,
            Expr::Comparison { span, .. } => span,
            Expr::Logical { span, .. } => span,
            Expr::Unary { span, .. } => span,
            Expr::Call { span, .. } => span,
            Expr::Pipeline { span, .. } => span,
            Expr::Lambda { span, .. } => span,
            Expr::If { span, .. } => span,
            Expr::Block { span, .. } => span,
            Expr::List { span, .. } => span,
            Expr::Map { span, .. } => span,
            Expr::Match { span, .. } => span,
        }
    }
}

/// Literal values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    /// Integer literal
    Integer(i64),

    /// Floating-point literal
    Float(f64),

    /// String literal
    String(String),

    /// Character literal
    Char(char),

    /// Boolean literal
    Boolean(bool),

    /// Null literal
    Null,
}

impl Literal {
    /// Get the span of this literal
    pub fn span(&self) -> Span {
        Span::dummy()
    }

    /// Get the type of this literal
    pub fn type_name(&self) -> &'static str {
        match self {
            Literal::Integer(_) => "Int",
            Literal::Float(_) => "Float",
            Literal::String(_) => "String",
            Literal::Char(_) => "Char",
            Literal::Boolean(_) => "Bool",
            Literal::Null => "Null",
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(lit) => write!(f, "{}", lit),
            Expr::Variable { name, .. } => write!(f, "{}", name),
            Expr::Binary { left, op, right, .. } => {
                write!(f, "({} {} {})", left, op.symbol(), right)
            }
            Expr::Comparison { left, op, right, .. } => {
                write!(f, "({} {} {})", left, op.symbol(), right)
            }
            Expr::Logical { left, op, right, .. } => {
                write!(f, "({} {} {})", left, op.symbol(), right)
            }
            Expr::Unary { op, expr, .. } => {
                write!(f, "({}{})", op.symbol(), expr)
            }
            Expr::Call { callee, args, .. } => {
                write!(f, "({}(", callee)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Expr::Pipeline { stages, .. } => {
                for (i, stage) in stages.iter().enumerate() {
                    if i > 0 {
                        write!(f, " |> ")?;
                    }
                    write!(f, "{}", stage)?;
                }
                Ok(())
            }
            Expr::If { condition, then_branch, else_branch, .. } => {
                write!(f, "(if {} then {} else {})", condition, then_branch, else_branch)
            }
            Expr::List { elements, .. } => {
                write!(f, "[")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, "]")
            }
            _ => write!(f, "(expression)"),
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Integer(n) => write!(f, "{}", n),
            Literal::Float(n) => write!(f, "{}", n),
            Literal::String(s) => write!(f, "\"{}\"", s),
            Literal::Char(c) => write!(f, "'{}'", c),
            Literal::Boolean(b) => write!(f, "{}", b),
            Literal::Null => write!(f, "null"),
        }
    }
}
