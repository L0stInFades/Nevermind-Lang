//! Scope management for name resolution

use std::collections::HashMap;
use std::fmt;

use crate::symbol::Symbol;
use crate::error::{NameError, NameErrorKind};

/// A lexical scope
#[derive(Clone)]
pub struct Scope {
    /// Parent scope (if any)
    pub parent: Option<Box<Scope>>,

    /// Symbols declared in this scope
    pub symbols: HashMap<String, Symbol>,

    /// Nesting level (0 = global)
    pub level: u32,
}

impl Scope {
    /// Create a new scope
    pub fn new(parent: Option<Scope>, level: u32) -> Self {
        Self {
            parent: parent.map(Box::new),
            symbols: HashMap::new(),
            level,
        }
    }

    /// Create a new global scope (no parent)
    pub fn global() -> Self {
        Self::new(None, 0)
    }

    /// Insert a symbol into this scope
    ///
    /// Returns an error if a symbol with the same name already exists
    /// in this scope (shadowing is allowed in child scopes).
    pub fn insert(&mut self, name: String, symbol: Symbol) -> Result<(), NameError> {
        if self.symbols.contains_key(&name) {
            let existing = &self.symbols[&name];
            return Err(NameError::new(
                NameErrorKind::DuplicateDefinition(name.clone()),
                format!("Cannot declare '{}', already defined in this scope", name),
                symbol.span.clone(),
            ).with_context(format!("previous definition of '{}' is here", name), Some(existing.span.clone())));
        }

        self.symbols.insert(name, symbol);
        Ok(())
    }

    /// Look up a symbol in this scope and all parent scopes
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        // Check current scope first
        if let Some(symbol) = self.symbols.get(name) {
            return Some(symbol);
        }

        // Check parent scopes
        if let Some(parent) = &self.parent {
            parent.lookup(name)
        } else {
            None
        }
    }

    /// Look up a symbol in this scope only (not parent scopes)
    pub fn lookup_local(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    /// Look up a mutable reference to a symbol
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        // Check current scope first
        if self.symbols.contains_key(name) {
            return self.symbols.get_mut(name);
        }

        // Check parent scopes
        if let Some(parent) = &mut self.parent {
            parent.lookup_mut(name)
        } else {
            None
        }
    }

    /// Get the number of symbols in this scope
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if this scope is empty
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Get all symbol names in this scope
    pub fn symbol_names(&self) -> impl Iterator<Item = &String> {
        self.symbols.keys()
    }

    /// Get the nesting level of this scope
    pub fn level(&self) -> u32 {
        self.level
    }

    /// Check if this is a global scope
    pub fn is_global(&self) -> bool {
        self.level == 0
    }
}

impl fmt::Debug for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scope")
            .field("level", &self.level)
            .field("symbols", &self.symbols)
            .field("has_parent", &self.parent.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SymbolKind;

    #[test]
    fn test_scope_insert_and_lookup() {
        let mut scope = Scope::global();
        let span = nevermind_common::Span::dummy();

        let symbol = Symbol::new("x".to_string(), SymbolKind::Variable { is_mutable: false }, span.clone());
        scope.insert("x".to_string(), symbol).unwrap();

        let found = scope.lookup("x");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "x");
    }

    #[test]
    fn test_scope_duplicate_definition() {
        let mut scope = Scope::global();
        let span = nevermind_common::Span::dummy();

        let symbol1 = Symbol::new("x".to_string(), SymbolKind::Variable { is_mutable: false }, span.clone());
        let symbol2 = Symbol::new("x".to_string(), SymbolKind::Variable { is_mutable: false }, span);

        scope.insert("x".to_string(), symbol1).unwrap();
        let result = scope.insert("x".to_string(), symbol2);

        assert!(result.is_err());
    }

    #[test]
    fn test_parent_scope_lookup() {
        let span = nevermind_common::Span::dummy();

        let mut parent = Scope::global();
        let symbol = Symbol::new("x".to_string(), SymbolKind::Variable { is_mutable: false }, span.clone());
        parent.insert("x".to_string(), symbol).unwrap();

        let mut child = Scope::new(Some(parent), 1);
        assert!(child.lookup("x").is_some());
        assert!(child.lookup_local("x").is_none());
    }

    #[test]
    fn test_shadowing() {
        let span = nevermind_common::Span::dummy();

        let mut parent = Scope::global();
        let parent_symbol = Symbol::new("x".to_string(), SymbolKind::Variable { is_mutable: false }, span.clone());
        parent.insert("x".to_string(), parent_symbol).unwrap();

        let mut child = Scope::new(Some(parent), 1);
        let child_symbol = Symbol::new("x".to_string(), SymbolKind::Variable { is_mutable: true }, span);
        child.insert("x".to_string(), child_symbol).unwrap();

        // Child should find its own symbol
        let found = child.lookup("x").unwrap();
        assert!(found.is_mutable());
    }
}
