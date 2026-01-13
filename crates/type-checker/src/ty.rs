//! Type variables and type schemes for polymorphism

use std::collections::HashSet;
use std::rc::Rc;
use crate::types::Type;
use crate::error::{Result, TypeError, TypeErrorKind};

/// A type variable (for inference and polymorphism)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeVar {
    id: usize,
}

impl TypeVar {
    pub fn new(id: usize) -> Self {
        Self { id }
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

impl std::fmt::Display for TypeVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}", (b'a' + self.id as u8) as char)
    }
}

/// A type scheme: ∀α1...αn. type
///
/// Type schemes represent polymorphic types. For example:
/// - `∀a. a -> a` is the identity function type
/// - `∀a b. a -> b -> a` is the const function type
#[derive(Debug, Clone)]
pub struct TypeScheme {
    /// Type variables that are universally quantified
    pub vars: Vec<TypeVar>,

    /// The type itself
    pub ty: Type,
}

impl TypeScheme {
    /// Create a new type scheme
    pub fn new(vars: Vec<TypeVar>, ty: Type) -> Self {
        Self { vars, ty }
    }

    /// Create a monomorphic type scheme (no quantified variables)
    pub fn monomorphic(ty: Type) -> Self {
        Self {
            vars: vec![],
            ty,
        }
    }

    /// Generalize a type with respect to a set of free variables
    ///
    /// This creates a type scheme by quantifying over all free type variables
    /// in the type that are not in the provided set.
    pub fn generalize(ty: Type, free_vars: &HashSet<usize>) -> Self {
        let ty_free_vars = Type::free_vars(&ty);

        // Quantify over variables that are free in the type but not in the environment
        let vars: Vec<TypeVar> = ty_free_vars
            .difference(free_vars)
            .cloned()
            .map(TypeVar::new)
            .collect();

        Self { vars, ty }
    }

    /// Instantiate this type scheme by replacing quantified variables with fresh type variables
    pub fn instantiate(&self, ctx: &mut crate::TypeContext) -> Type {
        // Create fresh type variables for each quantified variable
        let subst: std::collections::HashMap<usize, Type> = self.vars
            .iter()
            .map(|var| {
                let fresh_var = ctx.fresh_var();
                (var.id(), Type::Var(crate::types::TypeVarRef::new(fresh_var.id())))
            })
            .collect();

        // Apply substitution
        self.substitute(&subst)
    }

    /// Apply a substitution to this type scheme
    fn substitute(&self, subst: &std::collections::HashMap<usize, Type>) -> Type {
        self.ty.substitute(subst)
    }
}

impl Type {
    /// Get the free type variables in a type
    pub fn free_vars(ty: &Type) -> HashSet<usize> {
        match ty {
            Type::Var(var) => {
                let mut set = HashSet::new();
                set.insert(var.id());
                set
            }
            Type::Function(params, ret) => {
                let mut set = Type::free_vars(ret);
                for param in params {
                    set = &set | &Type::free_vars(param);
                }
                set
            }
            Type::List(elem) => Type::free_vars(elem),
            Type::Map(value) => Type::free_vars(value),
            Type::Tuple(elems) => {
                let mut set = HashSet::new();
                for elem in elems {
                    set = &set | &Type::free_vars(elem);
                }
                set
            }
            Type::Int | Type::Float | Type::String | Type::Bool | Type::Null | Type::Unit | Type::User(_) => {
                HashSet::new()
            }
        }
    }

    /// Apply a substitution to a type
    pub fn substitute(&self, subst: &std::collections::HashMap<usize, Type>) -> Type {
        match self {
            Type::Var(var) => {
                if let Some(replacement) = subst.get(&var.id()) {
                    replacement.clone()
                } else {
                    self.clone()
                }
            }
            Type::Function(params, ret) => {
                Type::Function(
                    params.iter().map(|p| p.substitute(subst)).collect(),
                    Box::new(ret.substitute(subst)),
                )
            }
            Type::List(elem) => Type::List(Box::new(elem.substitute(subst))),
            Type::Map(value) => Type::Map(Box::new(value.substitute(subst))),
            Type::Tuple(elems) => {
                Type::Tuple(elems.iter().map(|e| e.substitute(subst)).collect())
            }
            _ => self.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_var_display() {
        let var = TypeVar::new(0);
        assert_eq!(format!("{}", var), "'a");

        let var = TypeVar::new(1);
        assert_eq!(format!("{}", var), "'b");
    }

    #[test]
    fn test_monomorphic_scheme() {
        let ty = Type::Int;
        let scheme = TypeScheme::monomorphic(ty.clone());

        assert!(scheme.vars.is_empty());
        assert_eq!(scheme.ty, ty);
    }

    #[test]
    fn test_generalize() {
        let ty = Type::function(
            vec![Type::var(0), Type::var(0)],
            Type::var(0),
        );

        let mut free_vars = HashSet::new();
        free_vars.insert(1);

        let scheme = TypeScheme::generalize(ty, &free_vars);

        // Should quantify over t0 but not t1
        assert_eq!(scheme.vars.len(), 1);
        assert_eq!(scheme.vars[0].id(), 0);
    }

    #[test]
    fn test_instantiate() {
        let ty = Type::function(
            vec![Type::Var(crate::types::TypeVarRef::new(0))],
            Type::Var(crate::types::TypeVarRef::new(0)),
        );

        let scheme = TypeScheme::new(vec![TypeVar::new(0)], ty);

        let mut ctx = crate::TypeContext::new();
        let instantiated = scheme.instantiate(&mut ctx);

        // Should replace t0 with a fresh variable, but still be a function type
        if let Type::Function(params, ret) = &instantiated {
            // Check that params and ret are both type variables
            assert!(params[0].is_var());
            assert!(ret.is_var());
        } else {
            panic!("Expected function type");
        }
    }

    #[test]
    fn test_free_vars() {
        let ty = Type::function(
            vec![Type::var(0), Type::var(1)],
            Type::var(0),
        );

        let free_vars = Type::free_vars(&ty);

        assert_eq!(free_vars.len(), 2);
        assert!(free_vars.contains(&0));
        assert!(free_vars.contains(&1));
    }
}
