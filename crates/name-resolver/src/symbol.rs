//! Symbol definitions for name resolution

use std::fmt;

use nevermind_ast::Type;
use nevermind_common::Span;

/// The kind of symbol
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    /// Variable declaration
    Variable {
        /// Whether the variable is mutable
        is_mutable: bool,
    },

    /// Function declaration
    Function {
        /// Number of parameters
        param_count: usize,
    },

    /// Function parameter
    Parameter {
        /// Parameter index
        index: usize,
    },

    /// Type declaration (type alias, class, etc.)
    Type,

    /// Loop variable (for loops)
    LoopVariable,
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolKind::Variable { is_mutable } => {
                if *is_mutable {
                    write!(f, "mutable variable")
                } else {
                    write!(f, "variable")
                }
            }
            SymbolKind::Function { param_count } => {
                write!(f, "function with {} parameter(s)", param_count)
            }
            SymbolKind::Parameter { index } => {
                write!(f, "parameter at index {}", index)
            }
            SymbolKind::Type => write!(f, "type"),
            SymbolKind::LoopVariable => write!(f, "loop variable"),
        }
    }
}

/// A symbol in the symbol table
#[derive(Clone)]
pub struct Symbol {
    /// The name of the symbol
    pub name: String,

    /// The kind of symbol
    pub kind: SymbolKind,

    /// The span where this symbol was defined
    pub span: Span,

    /// The type of the symbol (if known)
    pub type_: Option<Type>,
}

impl Symbol {
    /// Create a new symbol
    pub fn new(name: String, kind: SymbolKind, span: Span) -> Self {
        Self {
            name,
            kind,
            span,
            type_: None,
        }
    }

    /// Create a new symbol with a known type
    pub fn with_type(name: String, kind: SymbolKind, span: Span, type_: Type) -> Self {
        Self {
            name,
            kind,
            span,
            type_: Some(type_),
        }
    }

    /// Create a variable symbol
    pub fn variable(name: String, is_mutable: bool, span: Span) -> Self {
        Self::new(name, SymbolKind::Variable { is_mutable }, span)
    }

    /// Create a function symbol
    pub fn function(name: String, param_count: usize, span: Span) -> Self {
        Self::new(name, SymbolKind::Function { param_count }, span)
    }

    /// Create a parameter symbol
    pub fn parameter(name: String, index: usize, span: Span) -> Self {
        Self::new(name, SymbolKind::Parameter { index }, span)
    }

    /// Create a type symbol
    pub fn type_(name: String, span: Span) -> Self {
        Self::new(name, SymbolKind::Type, span)
    }

    /// Create a loop variable symbol
    pub fn loop_variable(name: String, span: Span) -> Self {
        Self::new(name, SymbolKind::LoopVariable, span)
    }

    /// Check if this symbol is mutable
    pub fn is_mutable(&self) -> bool {
        matches!(self.kind, SymbolKind::Variable { is_mutable: true })
    }

    /// Check if this symbol is a function
    pub fn is_function(&self) -> bool {
        matches!(self.kind, SymbolKind::Function { .. })
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Symbol")
            .field("name", &self.name)
            .field("kind", &self.kind)
            .field("type_", &self.type_)
            .finish()
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} '{}'", self.kind, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_creation() {
        let span = Span::dummy();
        let var = Symbol::variable("x".to_string(), true, span.clone());

        assert_eq!(var.name, "x");
        assert!(var.is_mutable());
        assert!(!var.is_function());
    }

    #[test]
    fn test_function_symbol() {
        let span = Span::dummy();
        let func = Symbol::function("foo".to_string(), 2, span);

        assert_eq!(func.name, "foo");
        assert!(func.is_function());
        assert!(!func.is_mutable());
    }
}
