//! Statement nodes

use crate::{NodeId, Pattern, TypeAnnotation};
use crate::expr::{Expr, Parameter, Literal};
use nevermind_common::Span;

/// A statement
#[derive(Debug, Clone)]
pub enum Stmt {
    /// Variable declaration (let or var)
    Let {
        id: NodeId,
        is_mutable: bool,
        name: String,
        type_annotation: Option<TypeAnnotation>,
        value: Expr,
        span: Span,
    },

    /// Function declaration
    Function {
        id: NodeId,
        name: String,
        params: Vec<Parameter>,
        return_type: Option<TypeAnnotation>,
        body: Expr,
        span: Span,
    },

    /// Type alias declaration
    TypeAlias {
        id: NodeId,
        name: String,
        type_params: Vec<String>,
        definition: TypeAnnotation,
        span: Span,
    },

    /// If statement
    If {
        id: NodeId,
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
        span: Span,
    },

    /// While loop
    While {
        id: NodeId,
        condition: Expr,
        body: Vec<Stmt>,
        span: Span,
    },

    /// For loop
    For {
        id: NodeId,
        variable: Pattern,
        iter: Expr,
        body: Vec<Stmt>,
        span: Span,
    },

    /// Match statement
    Match {
        id: NodeId,
        scrutinee: Expr,
        arms: Vec<MatchArm>,
        span: Span,
    },

    /// Return statement
    Return {
        id: NodeId,
        value: Option<Expr>,
        span: Span,
    },

    /// Break statement
    Break {
        id: NodeId,
        span: Span,
    },

    /// Continue statement
    Continue {
        id: NodeId,
        span: Span,
    },

    /// Expression statement (expression evaluated for side effects)
    ExprStmt {
        id: NodeId,
        expr: Expr,
        span: Span,
    },

    /// Import statement
    Import {
        id: NodeId,
        module: String,
        symbols: Option<Vec<String>>,
        span: Span,
    },

    /// Class declaration
    Class {
        id: NodeId,
        name: String,
        extends: Option<String>,
        members: Vec<ClassMember>,
        span: Span,
    },
}

/// A match arm (used in both match expressions and statements)
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
}

/// A class member
#[derive(Debug, Clone)]
pub enum ClassMember {
    Field {
        name: String,
        type_annotation: TypeAnnotation,
        default_value: Option<Expr>,
    },
    Method {
        name: String,
        params: Vec<Parameter>,
        return_type: Option<TypeAnnotation>,
        body: Expr,
    },
}

impl Stmt {
    /// Get the span of this statement
    pub fn span(&self) -> &Span {
        match self {
            Stmt::Let { span, .. } => span,
            Stmt::Function { span, .. } => span,
            Stmt::TypeAlias { span, .. } => span,
            Stmt::If { span, .. } => span,
            Stmt::While { span, .. } => span,
            Stmt::For { span, .. } => span,
            Stmt::Match { span, .. } => span,
            Stmt::Return { span, .. } => span,
            Stmt::Break { span, .. } => span,
            Stmt::Continue { span, .. } => span,
            Stmt::ExprStmt { span, .. } => span,
            Stmt::Import { span, .. } => span,
            Stmt::Class { span, .. } => span,
        }
    }
}
