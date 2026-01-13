//! Main type checker implementing Hindley-Milner type inference

use ::nevermind_ast::Expr;
use ::nevermind_ast::Stmt;
use ::nevermind_ast::Pattern;
use ::nevermind_ast::Literal;
use crate::types::Type;
use crate::ty::TypeScheme;
use crate::environment::TypeEnvironment;
use crate::unification::Unifier;
use crate::error::{Result, TypeError};
use crate::TypeContext;

/// The main type checker
pub struct TypeChecker {
    /// Type environment
    env: TypeEnvironment,

    /// Type context (for fresh variables)
    ctx: TypeContext,

    /// Unifier (for type inference)
    unifier: Unifier,
}

impl TypeChecker {
    /// Create a new type checker
    pub fn new() -> Self {
        Self {
            env: TypeEnvironment::with_predefined(),
            ctx: TypeContext::new(),
            unifier: Unifier::new(),
        }
    }

    /// Get the current environment
    pub fn env(&self) -> &TypeEnvironment {
        &self.env
    }

    /// Get the current context
    pub fn ctx(&mut self) -> &mut TypeContext {
        &mut self.ctx
    }

    /// Type check a list of statements
    pub fn check(&mut self, stmts: &[Stmt]) -> Result<Type> {
        let mut last_type = Type::Unit;

        for stmt in stmts {
            last_type = self.check_statement(stmt)?;
        }

        Ok(last_type)
    }

    /// Type check a statement
    fn check_statement(&mut self, stmt: &Stmt) -> Result<Type> {
        match stmt {
            Stmt::Let { name, is_mutable, value, span, .. } => {
                // Infer the type of the initializer
                let ty = self.infer_expression(value)?;

                // Get the free variables in the environment
                let free_vars = self.env.free_vars();

                // Generalize the type
                let scheme = TypeScheme::generalize(ty, &free_vars);

                // Add the binding to the environment
                self.env.insert(name.clone(), scheme)?;

                Ok(Type::Unit)
            }

            Stmt::Function { name, params, body, span, .. } => {
                // Create function type
                let param_types: Vec<Type> = params.iter()
                    .map(|_| {
                        let var = self.ctx.fresh_var();
                        Type::Var(crate::types::TypeVarRef::new(var.id()))
                    })
                    .collect();

                // Enter a new scope for the function body
                self.env.enter_scope();

                // Bind parameters
                for (i, param) in params.iter().enumerate() {
                    let scheme = TypeScheme::monomorphic(param_types[i].clone());
                    self.env.insert(param.name.clone(), scheme)?;
                }

                // Type check the body
                let return_type = self.infer_expression(body)?;

                // Exit the function scope
                self.env.exit_scope()?;

                // Create the function type
                let func_type = Type::Function(param_types, Box::new(return_type));

                // Generalize and add to environment
                let free_vars = self.env.free_vars();
                let scheme = TypeScheme::generalize(func_type, &free_vars);
                self.env.insert(name.clone(), scheme)?;

                Ok(Type::Unit)
            }

            Stmt::TypeAlias { .. } => {
                // TODO: Handle type aliases
                Ok(Type::Unit)
            }

            Stmt::If { condition, then_branch, else_branch, span, .. } => {
                // Type check condition
                let cond_ty = self.infer_expression(condition)?;
                self.unifier.unify(&cond_ty, &Type::Bool, &ast_helpers::get_span(condition))?;

                // Type check then branch
                self.env.enter_scope();
                let mut then_ty = Type::Unit;
                for stmt in then_branch {
                    then_ty = self.check_statement(stmt)?;
                }
                self.env.exit_scope()?;

                // Type check else branch if present
                let else_ty = if let Some(else_branch) = else_branch {
                    self.env.enter_scope();
                    let mut ty = Type::Unit;
                    for stmt in else_branch {
                        ty = self.check_statement(stmt)?;
                    }
                    self.env.exit_scope()?;
                    Some(ty)
                } else {
                    None
                };

                // If there's an else branch, unify then and else types
                if let Some(else_ty) = else_ty {
                    self.unifier.unify(&then_ty, &else_ty, span)?;
                    Ok(then_ty)
                } else {
                    Ok(Type::Unit)
                }
            }

            Stmt::While { condition, body, .. } => {
                // Type check condition
                let cond_ty = self.infer_expression(condition)?;
                self.unifier.unify(&cond_ty, &Type::Bool, &ast_helpers::get_span(condition))?;

                // Type check body
                self.env.enter_scope();
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                self.env.exit_scope()?;

                Ok(Type::Unit)
            }

            Stmt::For { variable, iter, body, .. } => {
                // Type check iterator
                let iter_ty = self.infer_expression(iter)?;

                // Expect a list type
                let elem_ty = if let Type::List(elem) = iter_ty {
                    *elem
                } else {
                    return Err(TypeError::type_mismatch(
                        Type::List(Box::new(Type::var(0))),
                        iter_ty,
                        ast_helpers::get_span(iter),
                    ));
                };

                // Type check body
                self.env.enter_scope();

                // Bind loop variable
                let scheme = TypeScheme::monomorphic(elem_ty.clone());
                self.check_pattern(variable, &elem_ty)?;
                if let Pattern::Variable { name, .. } = variable {
                    self.env.insert(name.clone(), scheme)?;
                }

                for stmt in body {
                    self.check_statement(stmt)?;
                }

                self.env.exit_scope()?;

                Ok(Type::Unit)
            }

            Stmt::Match { scrutinee, arms, span, .. } => {
                // Type check scrutinee
                let scrutinee_ty = self.infer_expression(scrutinee)?;

                // Type check each arm
                let mut arm_types = Vec::new();

                for arm in arms {
                    self.env.enter_scope();

                    // Type check pattern
                    self.check_pattern(&arm.pattern, &scrutinee_ty)?;

                    // Type check guard if present
                    if let Some(guard) = &arm.guard {
                        let guard_ty = self.infer_expression(guard)?;
                        self.unifier.unify(&guard_ty, &Type::Bool, &ast_helpers::get_span(guard))?;
                    }

                    // Type check body
                    let arm_ty = self.infer_expression(&arm.body)?;
                    arm_types.push(arm_ty);

                    self.env.exit_scope()?;
                }

                // All arms must have the same type
                if let Some(first_ty) = arm_types.first() {
                    for ty in &arm_types[1..] {
                        self.unifier.unify(first_ty, ty, span)?;
                    }
                }

                Ok(arm_types.into_iter().next().unwrap_or(Type::Unit))
            }

            Stmt::Return { value, .. } => {
                // TODO: Check that return type matches function signature
                if let Some(value) = value {
                    self.infer_expression(value)?;
                }
                Ok(Type::Unit)
            }

            Stmt::Break { .. } | Stmt::Continue { .. } => {
                Ok(Type::Unit)
            }

            Stmt::ExprStmt { expr, .. } => {
                self.infer_expression(expr)
            }

            Stmt::Import { .. } => {
                // TODO: Handle imports
                Ok(Type::Unit)
            }

            Stmt::Class { .. } => {
                // TODO: Handle classes
                Ok(Type::Unit)
            }
        }
    }

    /// Infer the type of an expression
    fn infer_expression(&mut self, expr: &Expr) -> Result<Type> {
        match expr {
            Expr::Literal(lit) => {
                let ty = match lit {
                    Literal::Integer(_, _) => Type::Int,
                    Literal::Float(_, _) => Type::Float,
                    Literal::String(_, _) => Type::String,
                    Literal::Char(_, _) => Type::Int, // Characters are integers
                    Literal::Boolean(_, _) => Type::Bool,
                    Literal::Null(_) => Type::Null,
                };
                Ok(ty)
            }

            Expr::Variable { name, span, .. } => {
                // Look up the variable in the environment
                if let Some(scheme) = self.env.lookup(name) {
                    // Instantiate the type scheme
                    Ok(scheme.instantiate(&mut self.ctx))
                } else {
                    Err(TypeError::undefined_variable(name.clone(), span.clone()))
                }
            }

            Expr::Binary { left, op, right, span, id: _ } => {
                let left_ty = self.infer_expression(left)?;
                let right_ty = self.infer_expression(right)?;

                // Type check based on operator
                self.unifier.unify(&left_ty, &right_ty, span)?;
                // Numeric operators return the same type as operands
                Ok(left_ty.clone())
            }

            Expr::Comparison { left, op, right, span, id: _ } => {
                let left_ty = self.infer_expression(left)?;
                let right_ty = self.infer_expression(right)?;

                self.unifier.unify(&left_ty, &right_ty, span)?;
                Ok(Type::Bool)
            }

            Expr::Logical { left, op, right, span, id: _ } => {
                let left_ty = self.infer_expression(left)?;
                let right_ty = self.infer_expression(right)?;

                self.unifier.unify(&left_ty, &Type::Bool, span)?;
                self.unifier.unify(&right_ty, &Type::Bool, span)?;
                Ok(Type::Bool)
            }

            Expr::Unary { op, expr, span, id: _ } => {
                let expr_ty = self.infer_expression(expr)?;
                Ok(expr_ty)
            }

            Expr::Call { callee, args, span, id: _ } => {
                // Infer the type of the callee
                let callee_ty = self.infer_expression(callee)?;

                // Create fresh type variables for arguments and return type
                let var = self.ctx.fresh_var();
                let return_var = Type::Var(crate::types::TypeVarRef::new(var.id()));

                let mut arg_types = Vec::new();
                for _ in args {
                    let var = self.ctx.fresh_var();
                    arg_types.push(Type::Var(crate::types::TypeVarRef::new(var.id())));
                }

                // Expected function type
                let expected_ty = Type::Function(arg_types.clone(), Box::new(return_var.clone()));

                // Unify callee type with expected function type
                self.unifier.unify(&callee_ty, &expected_ty, span)?;

                // Type check arguments
                for (i, arg) in args.iter().enumerate() {
                    let arg_ty = self.infer_expression(arg)?;
                    self.unifier.unify(&arg_ty, &arg_types[i], span)?;
                }

                Ok(return_var)
            }

            Expr::Pipeline { stages, span, id: _ } => {
                // Type check each stage
                let mut current_ty = self.infer_expression(&stages[0])?;

                for stage in &stages[1..] {
                    let stage_ty = self.infer_expression(stage)?;

                    // Create a function type: current_ty -> ?
                    let var = self.ctx.fresh_var();
                    let expected_func = Type::Function(
                        vec![current_ty.clone()],
                        Box::new(Type::Var(crate::types::TypeVarRef::new(var.id())))
                    );

                    self.unifier.unify(&stage_ty, &expected_func, span)?;

                    // Update current type to the return type of the function
                    current_ty = Type::Var(crate::types::TypeVarRef::new(var.id()));
                }

                Ok(current_ty)
            }

            Expr::Lambda { params, body, .. } => {
                // Enter a new scope
                self.env.enter_scope();

                // Create fresh type variables for parameters
                let param_types: Vec<Type> = params.iter()
                    .map(|_| {
                        let var = self.ctx.fresh_var();
                        Type::Var(crate::types::TypeVarRef::new(var.id()))
                    })
                    .collect();

                // Bind parameters
                for (i, param) in params.iter().enumerate() {
                    let scheme = TypeScheme::monomorphic(param_types[i].clone());
                    self.env.insert(param.name.clone(), scheme)?;
                }

                // Type check body
                let body_ty = self.infer_expression(body)?;

                // Exit scope
                self.env.exit_scope()?;

                Ok(Type::Function(param_types, Box::new(body_ty)))
            }

            Expr::If { condition, then_branch, else_branch, .. } => {
                // Type check condition
                let cond_ty = self.infer_expression(condition)?;
                self.unifier.unify(&cond_ty, &Type::Bool, &ast_helpers::get_span(condition))?;

                // Type check branches
                let then_ty = self.infer_expression(then_branch)?;
                let else_ty = self.infer_expression(else_branch)?;

                // Unify branch types
                self.unifier.unify(&then_ty, &else_ty, &ast_helpers::get_span(expr))?;

                Ok(then_ty)
            }

            Expr::Block { statements, .. } => {
                self.env.enter_scope();

                let mut ty = Type::Unit;
                for stmt in statements {
                    ty = self.check_statement(stmt)?;
                }

                self.env.exit_scope()?;

                Ok(ty)
            }

            Expr::List { elements, .. } => {
                if elements.is_empty() {
                    // Empty list has a fresh type variable
                    let var = self.ctx.fresh_var();
                    Ok(Type::List(Box::new(Type::Var(crate::types::TypeVarRef::new(var.id())))))
                } else {
                    // Infer the type of the first element
                    let elem_ty = self.infer_expression(&elements[0])?;

                    // All elements must have the same type
                    for elem in &elements[1..] {
                        let ty = self.infer_expression(elem)?;
                        self.unifier.unify(&elem_ty, &ty, &ast_helpers::get_span(elem))?;
                    }

                    Ok(Type::List(Box::new(elem_ty)))
                }
            }

            Expr::Map { entries, .. } => {
                if entries.is_empty() {
                    // Empty map has a fresh type variable
                    let var = self.ctx.fresh_var();
                    Ok(Type::Map(Box::new(Type::Var(crate::types::TypeVarRef::new(var.id())))))
                } else {
                    // Keys must be strings
                    // Values can be any type, but all must be the same
                    let value_ty = self.infer_expression(&entries[0].1)?;

                    for (key, value) in entries {
                        let key_ty = self.infer_expression(key)?;
                        self.unifier.unify(&key_ty, &Type::String, &ast_helpers::get_span(key))?;

                        let ty = self.infer_expression(value)?;
                        self.unifier.unify(&value_ty, &ty, &ast_helpers::get_span(value))?;
                    }

                    Ok(Type::Map(Box::new(value_ty)))
                }
            }

            Expr::Match { scrutinee, arms, .. } => {
                // Type check scrutinee
                let scrutinee_ty = self.infer_expression(scrutinee)?;

                // Type check each arm
                let mut arm_types = Vec::new();

                for arm in arms {
                    self.env.enter_scope();

                    // Type check pattern
                    self.check_pattern(&arm.pattern, &scrutinee_ty)?;

                    // Type check guard if present
                    if let Some(guard) = &arm.guard {
                        let guard_ty = self.infer_expression(guard)?;
                        self.unifier.unify(&guard_ty, &Type::Bool, &ast_helpers::get_span(guard))?;
                    }

                    // Type check body
                    let arm_ty = self.infer_expression(&arm.body)?;
                    arm_types.push(arm_ty);

                    self.env.exit_scope()?;
                }

                // All arms must have the same type
                if let Some(first_ty) = arm_types.first() {
                    for ty in &arm_types[1..] {
                        self.unifier.unify(first_ty, ty, &ast_helpers::get_span(expr))?;
                    }
                    Ok(first_ty.clone())
                } else {
                    Ok(Type::Unit)
                }
            }

            Expr::Index { array, index, .. } => {
                // Infer array and index types
                let array_ty = self.infer_expression(array)?;
                let _index_ty = self.infer_expression(index)?;

                // TODO: Check that array is a list type and return element type
                Ok(array_ty)
            }
        }
    }

    /// Type check a pattern against an expected type
    fn check_pattern(&mut self, pattern: &Pattern, expected_ty: &Type) -> Result<()> {
        match pattern {
            Pattern::Literal { .. } => {
                // Literal patterns match their literal type
                Ok(())
            }

            Pattern::Variable { name, .. } => {
                // Variable patterns bind a variable of the expected type
                let scheme = TypeScheme::monomorphic(expected_ty.clone());
                self.env.insert(name.clone(), scheme)?;
                Ok(())
            }

            Pattern::Wildcard { .. } => {
                // Wildcard patterns match anything
                Ok(())
            }

            Pattern::Tuple { patterns, .. } => {
                if let Type::Tuple(elem_types) = expected_ty {
                    if patterns.len() != elem_types.len() {
                        return Err(TypeError::arity_mismatch(elem_types.len(), patterns.len(), ast_helpers::get_span_pattern(pattern)));
                    }

                    for (pat, ty) in patterns.iter().zip(elem_types.iter()) {
                        self.check_pattern(pat, ty)?;
                    }

                    Ok(())
                } else {
                    Err(TypeError::type_mismatch(
                        Type::Tuple(vec![Type::var(0)]),
                        expected_ty.clone(),
                        ast_helpers::get_span_pattern(pattern),
                    ))
                }
            }

            Pattern::List { patterns, .. } => {
                if let Type::List(elem_ty) = expected_ty {
                    for pat in patterns {
                        self.check_pattern(pat, elem_ty)?;
                    }
                    Ok(())
                } else {
                    Err(TypeError::type_mismatch(
                        Type::List(Box::new(Type::var(0))),
                        expected_ty.clone(),
                        ast_helpers::get_span_pattern(pattern),
                    ))
                }
            }

            Pattern::ListCons { head, tail, .. } => {
                if let Type::List(elem_ty) = expected_ty {
                    self.check_pattern(head, elem_ty)?;
                    self.check_pattern(tail, expected_ty)?;
                    Ok(())
                } else {
                    Err(TypeError::type_mismatch(
                        Type::List(Box::new(Type::var(0))),
                        expected_ty.clone(),
                        ast_helpers::get_span_pattern(pattern),
                    ))
                }
            }

            Pattern::Struct { fields, .. } => {
                // TODO: Check struct patterns
                Ok(())
            }

            Pattern::Or { patterns, .. } => {
                // All alternatives must have the same type
                for pat in patterns {
                    self.check_pattern(pat, expected_ty)?;
                }
                Ok(())
            }

            Pattern::Range { start, end, .. } => {
                // Range patterns must be integers
                self.check_pattern(start, &Type::Int)?;
                self.check_pattern(end, &Type::Int)?;
                Ok(())
            }
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions for getting spans from AST nodes
mod ast_helpers {
    use super::*;
    use nevermind_common::Span;

    pub fn get_span(expr: &Expr) -> Span {
        match expr {
            Expr::Literal(lit) => get_span_literal(lit),
            Expr::Variable { span, .. } => span.clone(),
            Expr::Binary { span, .. } => span.clone(),
            Expr::Comparison { span, .. } => span.clone(),
            Expr::Logical { span, .. } => span.clone(),
            Expr::Unary { span, .. } => span.clone(),
            Expr::Call { span, .. } => span.clone(),
            Expr::Pipeline { span, .. } => span.clone(),
            Expr::Lambda { span, .. } => span.clone(),
            Expr::If { span, .. } => span.clone(),
            Expr::Block { span, .. } => span.clone(),
            Expr::List { span, .. } => span.clone(),
            Expr::Map { span, .. } => span.clone(),
            Expr::Match { span, .. } => span.clone(),
            Expr::Index { span, .. } => span.clone(),
        }
    }

    pub fn get_span_pattern(pat: &Pattern) -> Span {
        match pat {
            Pattern::Literal { span, .. } => span.clone(),
            Pattern::Variable { span, .. } => span.clone(),
            Pattern::Wildcard { span, .. } => span.clone(),
            Pattern::Tuple { span, .. } => span.clone(),
            Pattern::List { span, .. } => span.clone(),
            Pattern::ListCons { span, .. } => span.clone(),
            Pattern::Struct { span, .. } => span.clone(),
            Pattern::Or { span, .. } => span.clone(),
            Pattern::Range { span, .. } => span.clone(),
        }
    }

    fn get_span_literal(lit: &Literal) -> Span {
        match lit {
            Literal::Integer(_, span) |
            Literal::Float(_, span) |
            Literal::String(_, span) |
            Literal::Char(_, span) |
            Literal::Boolean(_, span) |
            Literal::Null(span) => span.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nevermind_ast::Parameter;
    use nevermind_common::Span;

    #[test]
    fn test_literal_types() {
        let mut checker = TypeChecker::new();

        let int_expr = Expr::Literal(Literal::Integer(42, Span::dummy()));
        assert_eq!(checker.infer_expression(&int_expr).unwrap(), Type::Int);

        let bool_expr = Expr::Literal(Literal::Boolean(true, Span::dummy()));
        assert_eq!(checker.infer_expression(&bool_expr).unwrap(), Type::Bool);
    }

    #[test]
    fn test_variable_lookup() {
        let mut checker = TypeChecker::new();

        // Add a variable to the environment
        checker.env.insert("x".to_string(), TypeScheme::monomorphic(Type::Int)).unwrap();

        let var_expr = Expr::Variable {
            id: 1,
            name: "x".to_string(),
            span: Span::dummy(),
        };

        assert_eq!(checker.infer_expression(&var_expr).unwrap(), Type::Int);
    }

    #[test]
    fn test_undefined_variable() {
        let mut checker = TypeChecker::new();

        let var_expr = Expr::Variable {
            id: 1,
            name: "undefined".to_string(),
            span: Span::dummy(),
        };

        assert!(checker.infer_expression(&var_expr).is_err());
    }
}
