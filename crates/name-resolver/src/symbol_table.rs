//! Symbol table for name resolution

use std::fmt;

use crate::scope::Scope;
use crate::symbol::Symbol;
use crate::error::{NameError, Result};

/// A symbol table managing nested scopes
#[derive(Clone)]
pub struct SymbolTable {
    /// Stack of scopes
    scopes: Vec<Scope>,

    /// Loop depth (for break/continue validation)
    loop_depth: usize,

    /// Function depth (for return validation)
    function_depth: usize,
}

impl SymbolTable {
    /// Create a new symbol table
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::global()],
            loop_depth: 0,
            function_depth: 0,
        }
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        let level = self.scopes.len() as u32;
        let parent = self.scopes.last().cloned();
        self.scopes.push(Scope::new(parent, level));
    }

    /// Exit the current scope
    pub fn exit_scope(&mut self) -> Result<()> {
        if self.scopes.len() <= 1 {
            return Err(NameError::invalid_scope(
                "cannot exit global scope",
                nevermind_common::Span::dummy(),
            ));
        }

        self.scopes.pop();
        Ok(())
    }

    /// Enter a loop scope
    pub fn enter_loop(&mut self) {
        self.enter_scope();
        self.loop_depth += 1;
    }

    /// Exit a loop scope
    pub fn exit_loop(&mut self) -> Result<()> {
        if self.loop_depth == 0 {
            return Err(NameError::invalid_scope(
                "not in a loop",
                nevermind_common::Span::dummy(),
            ));
        }

        self.loop_depth -= 1;
        self.exit_scope()
    }

    /// Enter a function scope
    pub fn enter_function(&mut self) {
        self.enter_scope();
        self.function_depth += 1;
    }

    /// Exit a function scope
    pub fn exit_function(&mut self) -> Result<()> {
        if self.function_depth == 0 {
            return Err(NameError::invalid_scope(
                "not in a function",
                nevermind_common::Span::dummy(),
            ));
        }

        self.function_depth -= 1;
        self.exit_scope()
    }

    /// Declare a symbol in the current scope
    pub fn declare(&mut self, name: String, symbol: Symbol) -> Result<()> {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, symbol)
        } else {
            Err(NameError::invalid_scope(
                "no active scope",
                nevermind_common::Span::dummy(),
            ))
        }
    }

    /// Resolve a symbol name in the current scope chain
    pub fn resolve(&self, name: &str) -> Result<&Symbol> {
        if let Some(scope) = self.scopes.last() {
            scope
                .lookup(name)
                .ok_or_else(|| {
                    NameError::undefined_variable(name.to_string(), nevermind_common::Span::dummy())
                })
        } else {
            Err(NameError::invalid_scope(
                "no active scope",
                nevermind_common::Span::dummy(),
            ))
        }
    }

    /// Resolve a symbol name in the current scope only (not parent scopes)
    pub fn resolve_local(&self, name: &str) -> Result<&Symbol> {
        if let Some(scope) = self.scopes.last() {
            scope
                .lookup_local(name)
                .ok_or_else(|| {
                    NameError::undefined_variable(name.to_string(), nevermind_common::Span::dummy())
                })
        } else {
            Err(NameError::invalid_scope(
                "no active scope",
                nevermind_common::Span::dummy(),
            ))
        }
    }

    /// Check if a name is defined in the current scope
    pub fn in_current_scope(&self, name: &str) -> bool {
        self.scopes
            .last()
            .map(|scope| scope.lookup_local(name).is_some())
            .unwrap_or(false)
    }

    /// Check if a name is defined in any scope
    pub fn is_defined(&self, name: &str) -> bool {
        self.scopes
            .last()
            .map(|scope| scope.lookup(name).is_some())
            .unwrap_or(false)
    }

    /// Get the current scope depth
    pub fn depth(&self) -> usize {
        self.scopes.len()
    }

    /// Check if we're currently inside a loop
    pub fn in_loop(&self) -> bool {
        self.loop_depth > 0
    }

    /// Check if we're currently inside a function
    pub fn in_function(&self) -> bool {
        self.function_depth > 0
    }

    /// Get the current loop depth
    pub fn loop_depth(&self) -> usize {
        self.loop_depth
    }

    /// Get the current function depth
    pub fn function_depth(&self) -> usize {
        self.function_depth
    }

    /// Get a reference to the current scope
    pub fn current_scope(&self) -> Option<&Scope> {
        self.scopes.last()
    }

    /// Get a mutable reference to the current scope
    pub fn current_scope_mut(&mut self) -> Option<&mut Scope> {
        self.scopes.last_mut()
    }
}

impl fmt::Debug for SymbolTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SymbolTable")
            .field("depth", &self.depth())
            .field("loop_depth", &self.loop_depth)
            .field("function_depth", &self.function_depth)
            .field("scopes", &self.scopes)
            .finish()
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SymbolKind;

    #[test]
    fn test_symbol_table_creation() {
        let table = SymbolTable::new();
        assert_eq!(table.depth(), 1);
        assert_eq!(table.loop_depth(), 0);
        assert_eq!(table.function_depth(), 0);
    }

    #[test]
    fn test_enter_exit_scope() {
        let mut table = SymbolTable::new();

        table.enter_scope();
        assert_eq!(table.depth(), 2);

        table.exit_scope().unwrap();
        assert_eq!(table.depth(), 1);
    }

    #[test]
    fn test_declare_and_resolve() {
        let mut table = SymbolTable::new();
        let span = nevermind_common::Span::dummy();

        let symbol = Symbol::new("x".to_string(), SymbolKind::Variable { is_mutable: false }, span);
        table.declare("x".to_string(), symbol).unwrap();

        let resolved = table.resolve("x").unwrap();
        assert_eq!(resolved.name, "x");
    }

    #[test]
    fn test_resolve_undefined() {
        let table = SymbolTable::new();
        let result = table.resolve("nonexistent");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err().kind, NameErrorKind::UndefinedVariable(_)));
    }

    #[test]
    fn test_nested_scope_resolution() {
        let mut table = SymbolTable::new();
        let span = nevermind_common::Span::dummy();

        // Declare in global scope
        let global_symbol = Symbol::new("x".to_string(), SymbolKind::Variable { is_mutable: false }, span.clone());
        table.declare("x".to_string(), global_symbol).unwrap();

        // Enter nested scope
        table.enter_scope();

        // Should still find the global symbol
        let resolved = table.resolve("x").unwrap();
        assert_eq!(resolved.name, "x");

        // But it's not in the current scope
        assert!(!table.in_current_scope("x"));

        // It is defined somewhere though
        assert!(table.is_defined("x"));
    }

    #[test]
    fn test_loop_scope() {
        let mut table = SymbolTable::new();

        assert!(!table.in_loop());

        table.enter_loop();
        assert!(table.in_loop());
        assert_eq!(table.loop_depth(), 1);

        table.exit_loop().unwrap();
        assert!(!table.in_loop());
    }

    #[test]
    fn test_function_scope() {
        let mut table = SymbolTable::new();

        assert!(!table.in_function());

        table.enter_function();
        assert!(table.in_function());
        assert_eq!(table.function_depth(), 1);

        table.exit_function().unwrap();
        assert!(!table.in_function());
    }

    #[test]
    fn test_cannot_exit_global_scope() {
        let mut table = SymbolTable::new();
        let result = table.exit_scope();

        assert!(result.is_err());
    }
}
