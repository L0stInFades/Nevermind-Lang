//! Type environment for managing type schemes in nested scopes

use std::collections::HashMap;
use std::collections::HashSet;
use crate::types::Type;
use crate::ty::TypeScheme;
use crate::error::{Result, TypeError, TypeErrorKind};
use nevermind_common::Span;

/// A type environment that maps names to type schemes
#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    /// Stack of scopes
    scopes: Vec<Scope>,
}

/// A single scope in the environment
#[derive(Debug, Clone)]
struct Scope {
    /// Mapping from names to type schemes
    bindings: HashMap<String, TypeScheme>,
}

impl TypeEnvironment {
    /// Create a new empty environment
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
        }
    }

    /// Create a new environment with predefined bindings
    pub fn with_predefined() -> Self {
        use crate::ty::TypeVar;

        let mut env = Self::new();

        // Add built-in functions
        // print: forall a. (a) -> Unit
        let print_var = TypeVar::new(9000);
        let print_type = Type::Function(
            vec![Type::Var(crate::types::TypeVarRef::new(print_var.id()))],
            Box::new(Type::Unit),
        );
        let _ = env.insert("print".to_string(), TypeScheme::new(vec![print_var.clone()], print_type));

        // println: forall a. (a) -> Unit
        let println_var = TypeVar::new(9001);
        let println_type = Type::Function(
            vec![Type::Var(crate::types::TypeVarRef::new(println_var.id()))],
            Box::new(Type::Unit),
        );
        let _ = env.insert("println".to_string(), TypeScheme::new(vec![println_var.clone()], println_type));

        // len: forall a. (a) -> Int
        let len_var = TypeVar::new(9002);
        let len_type = Type::Function(
            vec![Type::Var(crate::types::TypeVarRef::new(len_var.id()))],
            Box::new(Type::Int),
        );
        let _ = env.insert("len".to_string(), TypeScheme::new(vec![len_var.clone()], len_type));

        // input: (String) -> String
        let input_type = Type::Function(vec![Type::String], Box::new(Type::String));
        let _ = env.insert("input".to_string(), TypeScheme::monomorphic(input_type));

        // range: (Int, Int) -> List[Int]
        let range_type = Type::Function(
            vec![Type::Int, Type::Int],
            Box::new(Type::List(Box::new(Type::Int))),
        );
        let _ = env.insert("range".to_string(), TypeScheme::monomorphic(range_type));

        // str: forall a. (a) -> String
        let str_var = TypeVar::new(9003);
        let str_type = Type::Function(
            vec![Type::Var(crate::types::TypeVarRef::new(str_var.id()))],
            Box::new(Type::String),
        );
        let _ = env.insert("str".to_string(), TypeScheme::new(vec![str_var.clone()], str_type));

        // int: forall a. (a) -> Int
        let int_var = TypeVar::new(9004);
        let int_type = Type::Function(
            vec![Type::Var(crate::types::TypeVarRef::new(int_var.id()))],
            Box::new(Type::Int),
        );
        let _ = env.insert("int".to_string(), TypeScheme::new(vec![int_var.clone()], int_type));

        env
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    /// Exit the current scope
    pub fn exit_scope(&mut self) -> Result<()> {
        if self.scopes.len() <= 1 {
            return Err(TypeError::new(
                TypeErrorKind::InvalidScope,
                "Cannot exit the global scope".to_string(),
                Span::dummy(),
            ));
        }

        self.scopes.pop();
        Ok(())
    }

    /// Insert a binding in the current scope
    pub fn insert(&mut self, name: String, scheme: TypeScheme) -> Result<()> {
        let current_scope = self.scopes.last_mut().unwrap();

        if current_scope.bindings.contains_key(&name) {
            return Err(TypeError::new(
                TypeErrorKind::DuplicateDefinition(name.clone()),
                format!("Name '{}' is already defined in this scope", name),
                Span::dummy(),
            ));
        }

        current_scope.bindings.insert(name, scheme);
        Ok(())
    }

    /// Insert or update a binding in the current scope (allows overwriting)
    pub fn insert_or_update(&mut self, name: String, scheme: TypeScheme) {
        let current_scope = self.scopes.last_mut().unwrap();
        current_scope.bindings.insert(name, scheme);
    }

    /// Look up a name in the environment
    pub fn lookup(&self, name: &str) -> Option<&TypeScheme> {
        for scope in self.scopes.iter().rev() {
            if let Some(scheme) = scope.bindings.get(name) {
                return Some(scheme);
            }
        }

        None
    }

    /// Check if a name is defined in the current scope
    pub fn in_current_scope(&self, name: &str) -> bool {
        if let Some(scope) = self.scopes.last() {
            scope.bindings.contains_key(name)
        } else {
            false
        }
    }

    /// Get the depth of the environment
    pub fn depth(&self) -> usize {
        self.scopes.len()
    }

    /// Get the free type variables in the environment
    pub fn free_vars(&self) -> HashSet<usize> {
        let mut free_vars = HashSet::new();

        for scope in &self.scopes {
            for scheme in scope.bindings.values() {
                free_vars = &free_vars | &Type::free_vars(&scheme.ty);
            }
        }

        free_vars
    }
}

impl Scope {
    fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }
}

impl Default for TypeEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ty::TypeVar;

    #[test]
    fn test_empty_environment() {
        let env = TypeEnvironment::new();
        assert!(env.lookup("x").is_none());
    }

    #[test]
    fn test_insert_and_lookup() {
        let mut env = TypeEnvironment::new();
        let scheme = TypeScheme::monomorphic(Type::Int);

        env.insert("x".to_string(), scheme).unwrap();

        let found = env.lookup("x");
        assert!(found.is_some());
        assert_eq!(found.unwrap().ty, Type::Int);
    }

    #[test]
    fn test_nested_scopes() {
        let mut env = TypeEnvironment::new();

        // Insert in outer scope
        env.insert("x".to_string(), TypeScheme::monomorphic(Type::Int)).unwrap();

        // Enter inner scope
        env.enter_scope();

        // Can still see x
        assert!(env.lookup("x").is_some());

        // Shadow x
        env.insert("x".to_string(), TypeScheme::monomorphic(Type::Bool)).unwrap();

        // Should see the shadowed value
        let found = env.lookup("x").unwrap();
        assert_eq!(found.ty, Type::Bool);

        // Exit inner scope
        env.exit_scope().unwrap();

        // Should see outer value again
        let found = env.lookup("x").unwrap();
        assert_eq!(found.ty, Type::Int);
    }

    #[test]
    fn test_duplicate_definition() {
        let mut env = TypeEnvironment::new();

        env.insert("x".to_string(), TypeScheme::monomorphic(Type::Int)).unwrap();

        let result = env.insert("x".to_string(), TypeScheme::monomorphic(Type::Bool));
        assert!(result.is_err());
    }

    #[test]
    fn test_exit_global_scope() {
        let mut env = TypeEnvironment::new();
        let result = env.exit_scope();
        assert!(result.is_err());
    }

    #[test]
    fn test_free_vars() {
        let mut env = TypeEnvironment::new();

        env.insert("x".to_string(), TypeScheme::monomorphic(Type::var(0))).unwrap();
        env.insert("y".to_string(), TypeScheme::monomorphic(Type::var(1))).unwrap();

        let free_vars = env.free_vars();

        assert_eq!(free_vars.len(), 2);
        assert!(free_vars.contains(&0));
        assert!(free_vars.contains(&1));
    }
}
