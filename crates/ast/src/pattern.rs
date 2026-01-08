//! Pattern matching patterns

use crate::expr::Literal;
use nevermind_common::Span;

/// A pattern in a match expression or function parameter
#[derive(Debug, Clone)]
pub enum Pattern {
    /// Wildcard pattern (matches anything, _)
    Wildcard {
        span: Span,
    },

    /// Variable binding pattern
    Variable {
        name: String,
        span: Span,
    },

    /// Literal pattern
    Literal {
        value: Literal,
        span: Span,
    },

    /// Or pattern (matches if any pattern matches)
    Or {
        patterns: Vec<Pattern>,
        span: Span,
    },

    /// Tuple pattern
    Tuple {
        patterns: Vec<Pattern>,
        span: Span,
    },

    /// List pattern
    List {
        patterns: Vec<Pattern>,
        span: Span,
    },

    /// List pattern with head and tail
    ListCons {
        head: Box<Pattern>,
        tail: Box<Pattern>,
        span: Span,
    },

    /// Struct/Object pattern
    Struct {
        name: String,
        fields: Vec<StructPatternField>,
        span: Span,
    },

    /// Range pattern
    Range {
        start: Box<Pattern>,
        end: Box<Pattern>,
        span: Span,
    },
}

/// A field in a struct pattern
#[derive(Debug, Clone)]
pub struct StructPatternField {
    pub name: String,
    pub pattern: Pattern,
    pub shorthand: bool,
}

impl Pattern {
    /// Get the span of this pattern
    pub fn span(&self) -> &Span {
        match self {
            Pattern::Wildcard { span } => span,
            Pattern::Variable { span, .. } => span,
            Pattern::Literal { span, .. } => span,
            Pattern::Or { span, .. } => span,
            Pattern::Tuple { span, .. } => span,
            Pattern::List { span, .. } => span,
            Pattern::ListCons { span, .. } => span,
            Pattern::Struct { span, .. } => span,
            Pattern::Range { span, .. } => span,
        }
    }

    /// Check if this pattern can fail to match
    pub fn is_refutable(&self) -> bool {
        match self {
            Pattern::Wildcard { .. } => false,
            Pattern::Variable { .. } => false,
            Pattern::Literal { .. } => true,
            Pattern::Or { patterns, .. } => patterns.iter().any(|p| p.is_refutable()),
            Pattern::Tuple { patterns, .. } => patterns.iter().any(|p| p.is_refutable()),
            Pattern::List { patterns, .. } => patterns.iter().any(|p| p.is_refutable()),
            Pattern::ListCons { .. } => true,
            Pattern::Struct { .. } => true,
            Pattern::Range { .. } => true,
        }
    }

    /// Collect all variable names bound by this pattern
    pub fn collect_variables(&self) -> Vec<String> {
        match self {
            Pattern::Wildcard { .. } => vec![],
            Pattern::Variable { name, .. } => vec![name.clone()],
            Pattern::Literal { .. } => vec![],
            Pattern::Or { patterns, .. } => {
                patterns.iter()
                    .flat_map(|p| p.collect_variables())
                    .collect()
            }
            Pattern::Tuple { patterns, .. } => {
                patterns.iter()
                    .flat_map(|p| p.collect_variables())
                    .collect()
            }
            Pattern::List { patterns, .. } => {
                patterns.iter()
                    .flat_map(|p| p.collect_variables())
                    .collect()
            }
            Pattern::ListCons { head, tail, .. } => {
                let mut vars = head.collect_variables();
                vars.extend(tail.collect_variables());
                vars
            }
            Pattern::Struct { fields, .. } => {
                fields.iter()
                    .flat_map(|f| f.pattern.collect_variables())
                    .collect()
            }
            Pattern::Range { .. } => vec![],
        }
    }
}
