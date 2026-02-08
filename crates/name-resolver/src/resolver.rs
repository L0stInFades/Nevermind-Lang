//! Name resolution for Nevermind AST

use std::collections::HashSet;

use nevermind_ast::{Expr, Stmt, Pattern};

use crate::symbol_table::SymbolTable;
use crate::symbol::Symbol;
use crate::error::{NameError, Result};

/// The name resolver
pub struct NameResolver {
    /// The symbol table
    symbol_table: SymbolTable,

    /// Collected errors
    errors: Vec<NameError>,

    /// Set of visited functions to detect recursion
    visited_functions: HashSet<String>,
}

impl NameResolver {
    /// Create a new name resolver
    pub fn new() -> Self {
        let mut resolver = Self {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
            visited_functions: HashSet::new(),
        };
        // Register built-in functions
        resolver.register_builtins();
        resolver
    }

    /// Register built-in functions in the global scope
    fn register_builtins(&mut self) {
        let builtins = [
            ("print", 1),
            ("println", 1),
            ("len", 1),
            ("str", 1),
            ("int", 1),
            ("float", 1),
            ("bool", 1),
            ("type", 1),
            ("input", 1),
            ("range", 2),
            ("abs", 1),
            ("min", 2),
            ("max", 2),
        ];
        for (name, param_count) in builtins {
            let symbol = Symbol::function(
                name.to_string(),
                param_count,
                nevermind_common::Span::dummy(),
            );
            let _ = self.symbol_table.declare(name.to_string(), symbol);
        }
    }

    /// Resolve a list of statements
    pub fn resolve(&mut self, stmts: &[Stmt]) -> Result<(), Vec<NameError>> {
        for stmt in stmts {
            if let Err(err) = self.resolve_statement(stmt) {
                self.errors.push(err);
            }
        }

        // After resolving all statements, check for undefined variables
        self.check_undefined();

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Resolve a statement
    fn resolve_statement(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Let {
                name,
                is_mutable,
                value,
                span,
                ..
            } => {
                // First resolve the initializer expression
                self.resolve_expression(value)?;

                // Then declare the variable
                let symbol = Symbol::variable(name.clone(), *is_mutable, span.clone());
                self.symbol_table.declare(name.clone(), symbol)?;
                Ok(())
            }

            Stmt::Function {
                name,
                params,
                body,
                span,
                ..
            } => {
                // Declare the function in the current scope
                let func_symbol = Symbol::function(name.clone(), params.len(), span.clone());
                self.symbol_table.declare(name.clone(), func_symbol)?;

                // Enter a new function scope
                self.symbol_table.enter_function();

                // Declare parameters
                for (i, param) in params.iter().enumerate() {
                    let param_symbol = Symbol::parameter(
                        param.name.clone(),
                        i,
                        nevermind_common::Span::dummy(),
                    );
                    self.symbol_table.declare(param.name.clone(), param_symbol)?;
                }

                // Resolve the function body
                self.resolve_expression(body)?;

                // Exit function scope
                self.symbol_table.exit_function()?;
                Ok(())
            }

            Stmt::TypeAlias { name, span, .. } => {
                // Declare the type
                let type_symbol = Symbol::type_(name.clone(), span.clone());
                self.symbol_table.declare(name.clone(), type_symbol)
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                // Resolve condition
                self.resolve_expression(condition)?;

                // Resolve then branch
                self.symbol_table.enter_scope();
                for stmt in then_branch {
                    self.resolve_statement(stmt)?;
                }
                self.symbol_table.exit_scope()?;

                // Resolve else branch if present
                if let Some(else_branch) = else_branch {
                    self.symbol_table.enter_scope();
                    for stmt in else_branch {
                        self.resolve_statement(stmt)?;
                    }
                    self.symbol_table.exit_scope()?;
                }

                Ok(())
            }

            Stmt::While {
                condition, body, ..
            } => {
                // Resolve condition
                self.resolve_expression(condition)?;

                // Resolve body in a loop scope
                self.symbol_table.enter_loop();
                for stmt in body {
                    self.resolve_statement(stmt)?;
                }
                self.symbol_table.exit_loop()?;
                Ok(())
            }

            Stmt::For {
                variable, iter, body, ..
            } => {
                // Resolve iterator
                self.resolve_expression(iter)?;

                // Enter loop scope
                self.symbol_table.enter_loop();

                // Declare loop variable
                self.resolve_pattern(variable)?;
                self.symbol_table.declare(
                    self.pattern_name(variable)?,
                    Symbol::loop_variable(self.pattern_name(variable)?, nevermind_common::Span::dummy()),
                )?;

                // Resolve body
                for stmt in body {
                    self.resolve_statement(stmt)?;
                }

                self.symbol_table.exit_loop()?;
                Ok(())
            }

            Stmt::Match {
                scrutinee, arms, ..
            } => {
                // Resolve scrutinee
                self.resolve_expression(scrutinee)?;

                // Resolve each arm
                for arm in arms {
                    // Each arm creates a new scope for the pattern
                    self.symbol_table.enter_scope();
                    self.resolve_pattern(&arm.pattern)?;

                    // Resolve guard if present
                    if let Some(guard) = &arm.guard {
                        self.resolve_expression(guard)?;
                    }

                    // Resolve body
                    self.resolve_expression(&arm.body)?;

                    self.symbol_table.exit_scope()?;
                }

                Ok(())
            }

            Stmt::Return { value, span, .. } => {
                if !self.symbol_table.in_function() {
                    return Err(NameError::invalid_return(span.clone()));
                }

                if let Some(value) = value {
                    self.resolve_expression(value)?;
                }

                Ok(())
            }

            Stmt::Break { span, .. } => {
                if !self.symbol_table.in_loop() {
                    return Err(NameError::invalid_break(span.clone()));
                }
                Ok(())
            }

            Stmt::Continue { span, .. } => {
                if !self.symbol_table.in_loop() {
                    return Err(NameError::invalid_continue(span.clone()));
                }
                Ok(())
            }

            Stmt::ExprStmt { expr, .. } => {
                self.resolve_expression(expr)
            }

            Stmt::Import { .. } => {
                // TODO: Handle imports properly
                Ok(())
            }

            Stmt::Class { name, members, .. } => {
                // Declare the class as a type
                let class_symbol = Symbol::type_(name.clone(), nevermind_common::Span::dummy());
                self.symbol_table.declare(name.clone(), class_symbol)?;

                // Enter class scope
                self.symbol_table.enter_scope();

                // Resolve members
                for member in members {
                    match member {
                        nevermind_ast::stmt::ClassMember::Field { name, .. } => {
                            let field_symbol = Symbol::variable(
                                name.clone(),
                                false,
                                nevermind_common::Span::dummy(),
                            );
                            self.symbol_table.declare(name.clone(), field_symbol)?;
                        }
                        nevermind_ast::stmt::ClassMember::Method { name, params, body, .. } => {
                            let method_symbol = Symbol::function(name.clone(), params.len(), nevermind_common::Span::dummy());
                            self.symbol_table.declare(name.clone(), method_symbol)?;

                            // Resolve method body
                            self.symbol_table.enter_function();

                            // Declare parameters
                            for (i, param) in params.iter().enumerate() {
                                let param_symbol = Symbol::parameter(
                                    param.name.clone(),
                                    i,
                                    nevermind_common::Span::dummy(),
                                );
                                self.symbol_table.declare(param.name.clone(), param_symbol)?;
                            }

                            self.resolve_expression(body)?;
                            self.symbol_table.exit_function()?;
                        }
                    }
                }

                self.symbol_table.exit_scope()?;
                Ok(())
            }
        }
    }

    /// Resolve an expression
    fn resolve_expression(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Literal(_) => Ok(()),

            Expr::Variable { name, .. } => {
                self.symbol_table.resolve(name)?;
                Ok(())
            }

            Expr::Binary { left, right, .. } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
                Ok(())
            }

            Expr::Comparison { left, right, .. } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
                Ok(())
            }

            Expr::Logical { left, right, .. } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
                Ok(())
            }

            Expr::Unary { expr, .. } => {
                self.resolve_expression(expr)
            }

            Expr::Call { callee, args, .. } => {
                self.resolve_expression(callee)?;

                for arg in args {
                    self.resolve_expression(arg)?;
                }

                Ok(())
            }

            Expr::Pipeline { stages, .. } => {
                for stage in stages {
                    self.resolve_expression(stage)?;
                }
                Ok(())
            }

            Expr::Lambda { params, body, .. } => {
                // Enter a new scope for the lambda
                self.symbol_table.enter_scope();

                // Declare parameters
                for (i, param) in params.iter().enumerate() {
                    let param_symbol = Symbol::parameter(
                        param.name.clone(),
                        i,
                        nevermind_common::Span::dummy(),
                    );
                    self.symbol_table.declare(param.name.clone(), param_symbol)?;
                }

                // Resolve body
                self.resolve_expression(body)?;

                // Exit scope
                self.symbol_table.exit_scope()?;
                Ok(())
            }

            Expr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.resolve_expression(condition)?;
                self.resolve_expression(then_branch)?;
                self.resolve_expression(else_branch)?;
                Ok(())
            }

            Expr::Block { statements, .. } => {
                self.symbol_table.enter_scope();

                for stmt in statements {
                    self.resolve_statement(stmt)?;
                }

                self.symbol_table.exit_scope()?;
                Ok(())
            }

            Expr::List { elements, .. } => {
                for elem in elements {
                    self.resolve_expression(elem)?;
                }
                Ok(())
            }

            Expr::Map { entries, .. } => {
                for (key, value) in entries {
                    self.resolve_expression(key)?;
                    self.resolve_expression(value)?;
                }
                Ok(())
            }

            Expr::Match { scrutinee, arms, .. } => {
                self.resolve_expression(scrutinee)?;

                for arm in arms {
                    self.symbol_table.enter_scope();
                    self.resolve_pattern(&arm.pattern)?;

                    if let Some(guard) = &arm.guard {
                        self.resolve_expression(guard)?;
                    }

                    self.resolve_expression(&arm.body)?;
                    self.symbol_table.exit_scope()?;
                }

                Ok(())
            }

            Expr::Index { array, index, .. } => {
                self.resolve_expression(array)?;
                self.resolve_expression(index)
            }

            Expr::Assign { target, value, .. } => {
                self.resolve_expression(target)?;
                self.resolve_expression(value)
            }

            Expr::MemberAccess { object, .. } => {
                self.resolve_expression(object)
            }
        }
    }

    /// Resolve a pattern
    fn resolve_pattern(&mut self, pattern: &Pattern) -> Result<()> {
        match pattern {
            Pattern::Literal { .. } => Ok(()),
            Pattern::Variable { name, .. } => {
                // Declare pattern variable
                let symbol = Symbol::variable(name.clone(), false, nevermind_common::Span::dummy());
                self.symbol_table.declare(name.clone(), symbol)
            }
            Pattern::Wildcard { .. } => Ok(()),
            Pattern::Tuple { patterns, .. } => {
                for pat in patterns {
                    self.resolve_pattern(pat)?;
                }
                Ok(())
            }
            Pattern::List { patterns, .. } => {
                for pat in patterns {
                    self.resolve_pattern(pat)?;
                }
                Ok(())
            }
            Pattern::ListCons { head, tail, .. } => {
                self.resolve_pattern(head)?;
                self.resolve_pattern(tail)?;
                Ok(())
            }
            Pattern::Struct { fields, .. } => {
                for field in fields {
                    self.resolve_pattern(&field.pattern)?;
                }
                Ok(())
            }
            Pattern::Or { patterns, .. } => {
                for pat in patterns {
                    self.resolve_pattern(pat)?;
                }
                Ok(())
            }
            Pattern::Range { start, end, .. } => {
                self.resolve_pattern(start)?;
                self.resolve_pattern(end)?;
                Ok(())
            }
            Pattern::Constructor { args, .. } => {
                for arg in args {
                    self.resolve_pattern(arg)?;
                }
                Ok(())
            }
        }
    }

    /// Extract a name from a pattern (for error messages and scoping)
    fn pattern_name(&self, pattern: &Pattern) -> Result<String> {
        match pattern {
            Pattern::Variable { name, .. } => Ok(name.clone()),
            Pattern::Wildcard { .. } => Ok("_".to_string()),
            _ => Ok("<pattern>".to_string()),
        }
    }

    /// Check for undefined variables (this is called after all resolution is done)
    fn check_undefined(&mut self) {
        // This is a placeholder for additional validation
        // In practice, undefined variables are caught during resolve()
    }
}

impl Default for NameResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nevermind_ast::{Expr, Stmt, Parameter};
    use nevermind_ast::Literal;

    #[test]
    fn test_resolve_variable() {
        let mut resolver = NameResolver::new();

        // Create a simple variable declaration
        let stmt = Stmt::Let {
            id: 1,
            is_mutable: false,
            name: "x".to_string(),
            type_annotation: None,
            value: Expr::Literal(Literal::Integer(42, nevermind_common::Span::dummy())),
            span: nevermind_common::Span::dummy(),
        };

        resolver.resolve_statement(&stmt).unwrap();

        // Variable should be defined
        assert!(resolver.symbol_table.is_defined("x"));
    }

    #[test]
    fn test_resolve_undefined_variable() {
        let mut resolver = NameResolver::new();

        // Create a variable reference without declaration
        let expr = Expr::Variable {
            id: 1,
            name: "undefined_var".to_string(),
            span: nevermind_common::Span::dummy(),
        };

        let result = resolver.resolve_expression(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_function() {
        let mut resolver = NameResolver::new();

        // Create a simple function declaration
        let stmt = Stmt::Function {
            id: 1,
            name: "add".to_string(),
            params: vec![
                Parameter {
                    id: 2,
                    name: "a".to_string(),
                    type_annotation: None,
                    default_value: None,
                },
                Parameter {
                    id: 3,
                    name: "b".to_string(),
                    type_annotation: None,
                    default_value: None,
                },
            ],
            return_type: None,
            body: Expr::Literal(Literal::Integer(0, nevermind_common::Span::dummy())),
            span: nevermind_common::Span::dummy(),
        };

        resolver.resolve_statement(&stmt).unwrap();

        // Function should be defined
        assert!(resolver.symbol_table.is_defined("add"));
    }

    #[test]
    fn test_nested_scopes() {
        let mut resolver = NameResolver::new();

        // Declare variable in outer scope
        let stmt1 = Stmt::Let {
            id: 1,
            is_mutable: false,
            name: "x".to_string(),
            type_annotation: None,
            value: Expr::Literal(Literal::Integer(10, nevermind_common::Span::dummy())),
            span: nevermind_common::Span::dummy(),
        };

        resolver.resolve_statement(&stmt1).unwrap();

        // Create a block with inner scope
        let block_expr = Expr::Block {
            id: 2,
            statements: vec![
                Stmt::Let {
                    id: 3,
                    is_mutable: false,
                    name: "y".to_string(),
                    type_annotation: None,
                    value: Expr::Literal(Literal::Integer(20, nevermind_common::Span::dummy())),
                    span: nevermind_common::Span::dummy(),
                },
            ],
            span: nevermind_common::Span::dummy(),
        };

        resolver.resolve_expression(&block_expr).unwrap();

        // Outer variable should still be defined
        assert!(resolver.symbol_table.is_defined("x"));
    }
}
