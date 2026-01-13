//! Unification algorithm for type inference

use std::collections::HashMap;
use crate::types::{Type, TypeVarRef};
use crate::error::{Result, TypeError, TypeErrorKind};
use nevermind_common::Span;

/// A substitution mapping type variables to types
pub type Substitution = HashMap<usize, Type>;

/// A unifier for finding substitutions between types
pub struct Unifier {
    /// Current substitution
    subst: Substitution,
}

impl Unifier {
    /// Create a new unifier
    pub fn new() -> Self {
        Self {
            subst: Substitution::new(),
        }
    }

    /// Get the current substitution
    pub fn get_subst(&self) -> &Substitution {
        &self.subst
    }

    /// Get the current substitution mutably
    pub fn get_subst_mut(&mut self) -> &mut Substitution {
        &mut self.subst
    }

    /// Unify two types
    pub fn unify(&mut self, ty1: &Type, ty2: &Type, span: &Span) -> Result<()> {
        let ty1 = self.apply(ty1);
        let ty2 = self.apply(ty2);

        match (ty1, ty2) {
            // Unifying a type variable with a type
            (Type::Var(var), ty) | (ty, Type::Var(var)) => {
                self.bind_var(var, ty, span)
            }

            // Unifying two base types
            (Type::Int, Type::Int) |
            (Type::Float, Type::Float) |
            (Type::String, Type::String) |
            (Type::Bool, Type::Bool) |
            (Type::Null, Type::Null) |
            (Type::Unit, Type::Unit) => Ok(()),

            // Unifying two user-defined types
            (Type::User(name1), Type::User(name2)) if name1 == name2 => Ok(()),

            // Unifying two function types
            (Type::Function(params1, ret1), Type::Function(params2, ret2)) => {
                if params1.len() != params2.len() {
                    return Err(TypeError::arity_mismatch(params2.len(), params1.len(), span.clone()));
                }

                // Unify parameters
                for (p1, p2) in params1.iter().zip(params2.iter()) {
                    self.unify(p1, p2, span)?;
                }

                // Unify return types
                self.unify(&ret1, &ret2, span)
            }

            // Unifying two list types
            (Type::List(elem1), Type::List(elem2)) => {
                self.unify(&elem1, &elem2, span)
            }

            // Unifying two map types
            (Type::Map(value1), Type::Map(value2)) => {
                self.unify(&value1, &value2, span)
            }

            // Unifying two tuple types
            (Type::Tuple(elems1), Type::Tuple(elems2)) => {
                if elems1.len() != elems2.len() {
                    return Err(TypeError::type_mismatch(
                        Type::Tuple(elems1.clone()),
                        Type::Tuple(elems2.clone()),
                        span.clone(),
                    ));
                }

                for (e1, e2) in elems1.iter().zip(elems2.iter()) {
                    self.unify(e1, e2, span)?;
                }

                Ok(())
            }

            // Type mismatch
            (ty1, ty2) => {
                Err(TypeError::type_mismatch(ty1, ty2, span.clone()))
            }
        }
    }

    /// Bind a type variable to a type
    fn bind_var(&mut self, var: TypeVarRef, ty: Type, span: &Span) -> Result<()> {
        // Occurs check: ensure the variable doesn't occur in the type
        if self.occurs(&var, &ty) {
            return Err(TypeError::new(
                TypeErrorKind::OccursCheckFailed(var.id()),
                format!("infinite type: t{}", var.id()),
                span.clone(),
            ));
        }

        // Add the binding
        self.subst.insert(var.id(), ty);
        Ok(())
    }

    /// Check if a type variable occurs in a type (occurs check)
    fn occurs(&self, var: &TypeVarRef, ty: &Type) -> bool {
        let ty = self.apply(ty);

        match ty {
            Type::Var(v) => v.id() == var.id(),
            Type::Function(params, ret) => {
                self.occurs(var, &ret) || params.iter().any(|p| self.occurs(var, p))
            }
            Type::List(elem) => self.occurs(var, &elem),
            Type::Map(value) => self.occurs(var, &value),
            Type::Tuple(elems) => elems.iter().any(|e| self.occurs(var, e)),
            _ => false,
        }
    }

    /// Apply the current substitution to a type
    pub fn apply(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(var) => {
                if let Some(replacement) = self.subst.get(&var.id()) {
                    self.apply(replacement)
                } else {
                    ty.clone()
                }
            }
            Type::Function(params, ret) => {
                Type::Function(
                    params.iter().map(|p| self.apply(p)).collect(),
                    Box::new(self.apply(ret)),
                )
            }
            Type::List(elem) => Type::List(Box::new(self.apply(elem))),
            Type::Map(value) => Type::Map(Box::new(self.apply(value))),
            Type::Tuple(elems) => {
                Type::Tuple(elems.iter().map(|e| self.apply(e)).collect())
            }
            _ => ty.clone(),
        }
    }

    /// Compose two substitutions
    pub fn compose(&mut self, other: Substitution) {
        // Collect current substitutions
        let current: Vec<(usize, Type)> = self.subst.drain().collect();

        // Apply the new substitution to each current substitution
        for (var, ty) in current {
            let new_ty = self.apply_from_subst(&ty, &other);
            self.subst.insert(var, new_ty);
        }

        // Add the new bindings
        for (var, ty) in other {
            self.subst.entry(var).or_insert(ty);
        }
    }

    /// Apply a substitution to a type
    fn apply_from_subst(&self, ty: &Type, subst: &Substitution) -> Type {
        match ty {
            Type::Var(var) => {
                if let Some(replacement) = subst.get(&var.id()) {
                    self.apply(replacement)
                } else if let Some(replacement) = self.subst.get(&var.id()) {
                    self.apply(replacement)
                } else {
                    ty.clone()
                }
            }
            Type::Function(params, ret) => {
                Type::Function(
                    params.iter().map(|p| self.apply_from_subst(p, subst)).collect(),
                    Box::new(self.apply_from_subst(ret, subst)),
                )
            }
            Type::List(elem) => Type::List(Box::new(self.apply_from_subst(elem, subst))),
            Type::Map(value) => Type::Map(Box::new(self.apply_from_subst(value, subst))),
            Type::Tuple(elems) => {
                Type::Tuple(elems.iter().map(|e| self.apply_from_subst(e, subst)).collect())
            }
            _ => ty.clone(),
        }
    }
}

impl Default for Unifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unify_same_types() {
        let mut unifier = Unifier::new();
        let span = Span::dummy();

        assert!(unifier.unify(&Type::Int, &Type::Int, &span).is_ok());
        assert!(unifier.unify(&Type::Bool, &Type::Bool, &span).is_ok());
    }

    #[test]
    fn test_unify_different_types() {
        let mut unifier = Unifier::new();
        let span = Span::dummy();

        assert!(unifier.unify(&Type::Int, &Type::Bool, &span).is_err());
    }

    #[test]
    fn test_unify_var_with_type() {
        let mut unifier = Unifier::new();
        let span = Span::dummy();

        let var = Type::Var(TypeVarRef::new(0));
        assert!(unifier.unify(&var, &Type::Int, &span).is_ok());

        // Check that the substitution was recorded
        assert_eq!(unifier.get_subst().get(&0), Some(&Type::Int));
    }

    #[test]
    fn test_apply_substitution() {
        let mut unifier = Unifier::new();
        let span = Span::dummy();

        // Bind t0 to Int
        unifier.unify(&Type::Var(TypeVarRef::new(0)), &Type::Int, &span).unwrap();

        // Apply substitution to a type containing t0
        let ty = Type::Function(vec![Type::Var(TypeVarRef::new(0))], Box::new(Type::Var(TypeVarRef::new(0))));
        let result = unifier.apply(&ty);

        if let Type::Function(params, ret) = result {
            assert_eq!(params[0], Type::Int);
            assert_eq!(*ret, Type::Int);
        } else {
            panic!("Expected function type");
        }
    }

    #[test]
    fn test_unify_function_types() {
        let mut unifier = Unifier::new();
        let span = Span::dummy();

        let ty1 = Type::function(vec![Type::Int], Type::Bool);
        let ty2 = Type::function(vec![Type::Var(TypeVarRef::new(0))], Type::Var(TypeVarRef::new(1)));

        assert!(unifier.unify(&ty1, &ty2, &span).is_ok());

        // Check substitutions
        assert_eq!(unifier.get_subst().get(&0), Some(&Type::Int));
        assert_eq!(unifier.get_subst().get(&1), Some(&Type::Bool));
    }

    #[test]
    fn test_arity_mismatch() {
        let mut unifier = Unifier::new();
        let span = Span::dummy();

        let ty1 = Type::function(vec![Type::Int], Type::Bool);
        let ty2 = Type::function(vec![Type::Int, Type::Int], Type::Bool);

        let result = unifier.unify(&ty1, &ty2, &span);
        assert!(result.is_err());
    }
}
