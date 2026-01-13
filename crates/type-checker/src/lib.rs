//! Type checker for Nevermind
//!
//! This crate implements a Hindley-Milner type checker with support for:
//! - Type inference
//! - Polymorphism via type schemes
//! - Constraint solving via unification
//! - Rich error reporting

pub mod types;
pub mod ty;
pub mod environment;
pub mod unification;
pub mod checker;
pub mod error;

pub use types::{Type, TypeVarRef};
pub use ty::{TypeVar, TypeScheme};
pub use environment::TypeEnvironment;
pub use unification::Unifier;
pub use checker::TypeChecker;
pub use error::{TypeError, TypeErrorKind, Result};

/// Type checking context
pub struct TypeContext {
    /// Next type variable ID
    next_var_id: usize,
}

impl TypeContext {
    pub fn new() -> Self {
        Self {
            next_var_id: 0,
        }
    }

    pub fn fresh_var(&mut self) -> TypeVar {
        let id = self.next_var_id;
        self.next_var_id += 1;
        TypeVar::new(id)
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_fresh_vars() {
        let mut ctx = TypeContext::new();

        let var1 = ctx.fresh_var();
        let var2 = ctx.fresh_var();

        assert_ne!(var1, var2);
    }
}
