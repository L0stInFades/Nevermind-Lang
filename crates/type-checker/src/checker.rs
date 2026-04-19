//! Main type checker implementing Hindley-Milner type inference

use crate::environment::TypeEnvironment;
use crate::error::{Result, TypeError};
use crate::ty::TypeScheme;
use crate::types::Type;
use crate::unification::Unifier;
use crate::TypeContext;
use nevermind_ast::Expr;
use nevermind_ast::Literal;
use nevermind_ast::Pattern;
use nevermind_ast::Stmt;
use nevermind_common::Span;

#[derive(Clone)]
struct FlowInfo {
    ty: Type,
    always_returns: bool,
    always_produces_value: bool,
}

impl FlowInfo {
    fn new(ty: Type) -> Self {
        Self {
            ty,
            always_returns: false,
            always_produces_value: true,
        }
    }

    fn returning(ty: Type) -> Self {
        Self {
            ty,
            always_returns: true,
            always_produces_value: true,
        }
    }
}

#[derive(Clone)]
struct FunctionContext {
    name: String,
    return_type: Type,
    return_annotation_span: Option<Span>,
}

/// The main type checker
pub struct TypeChecker {
    /// Type environment
    env: TypeEnvironment,

    /// Type context (for fresh variables)
    ctx: TypeContext,

    /// Unifier (for type inference)
    unifier: Unifier,

    /// Stack of functions currently being checked.
    function_contexts: Vec<FunctionContext>,
}

impl TypeChecker {
    /// Create a new type checker
    pub fn new() -> Self {
        Self {
            env: TypeEnvironment::with_predefined(),
            ctx: TypeContext::new(),
            unifier: Unifier::new(),
            function_contexts: Vec::new(),
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
        Ok(self.check_statement_with_flow(stmt)?.ty)
    }

    fn check_statement_with_flow(&mut self, stmt: &Stmt) -> Result<FlowInfo> {
        match stmt {
            Stmt::Export { stmt, .. } => self.check_statement_with_flow(stmt),

            Stmt::Let { name, value, .. } => {
                let ty = self.infer_expression(value)?;
                let free_vars = self.env.free_vars();
                let scheme = TypeScheme::generalize(ty, &free_vars);
                self.env.insert(name.clone(), scheme)?;
                Ok(FlowInfo::new(Type::Unit))
            }

            Stmt::Function {
                name,
                params,
                body,
                return_type: ret_ann,
                ..
            } => {
                let param_types: Vec<Type> = params
                    .iter()
                    .map(|p| {
                        if let Some(ann) = &p.type_annotation {
                            self.resolve_type_annotation(ann)
                        } else {
                            let var = self.ctx.fresh_var();
                            Type::Var(crate::types::TypeVarRef::new(var.id()))
                        }
                    })
                    .collect();

                let declared_return = if let Some(ann) = ret_ann {
                    self.resolve_type_annotation(ann)
                } else {
                    let var = self.ctx.fresh_var();
                    Type::Var(crate::types::TypeVarRef::new(var.id()))
                };

                let free_vars_before = self.env.free_vars();
                let func_type =
                    Type::Function(param_types.clone(), Box::new(declared_return.clone()));
                let func_scheme = TypeScheme::monomorphic(func_type.clone());
                self.env.insert_or_update(name.clone(), func_scheme);

                self.env.enter_scope();
                for (i, param) in params.iter().enumerate() {
                    let scheme = TypeScheme::monomorphic(param_types[i].clone());
                    self.env.insert(param.name.clone(), scheme)?;
                }

                self.function_contexts.push(FunctionContext {
                    name: name.clone(),
                    return_type: declared_return.clone(),
                    return_annotation_span: ret_ann.as_ref().map(|ann| ann.span.clone()),
                });
                let body_result = self.infer_expression_with_flow(body);
                self.function_contexts.pop();
                let body_result = body_result?;

                self.env.exit_scope()?;

                if !body_result.always_returns {
                    let expected_return = self.unifier.apply(&declared_return);
                    let body_ty = self.unifier.apply(&body_result.ty);

                    if !body_result.always_produces_value
                        && expected_return != Type::Unit
                        && !expected_return.is_var()
                    {
                        let mut error = TypeError::missing_return(
                            name.clone(),
                            expected_return.clone(),
                            ast_helpers::get_span(body),
                        );
                        if let Some(span) = ret_ann.as_ref().map(|ann| ann.span.clone()) {
                            error = error.with_context(
                                format!(
                                    "function '{}' declares return type {}",
                                    name,
                                    expected_return.display_name()
                                ),
                                Some(span),
                            );
                        }
                        return Err(error);
                    }

                    if body_ty == Type::Unit
                        && expected_return != Type::Unit
                        && !expected_return.is_var()
                    {
                        let mut error = TypeError::missing_return(
                            name.clone(),
                            expected_return.clone(),
                            ast_helpers::get_span(body),
                        );
                        if let Some(span) = ret_ann.as_ref().map(|ann| ann.span.clone()) {
                            error = error.with_context(
                                format!(
                                    "function '{}' declares return type {}",
                                    name,
                                    expected_return.display_name()
                                ),
                                Some(span),
                            );
                        }
                        return Err(error);
                    }

                    self.unifier
                        .unify(
                            &body_result.ty,
                            &declared_return,
                            &ast_helpers::get_span(body),
                        )
                        .map_err(|_| {
                            self.return_type_mismatch_error(
                                name,
                                ret_ann.as_ref().map(|ann| ann.span.clone()),
                                &declared_return,
                                &body_result.ty,
                                ast_helpers::get_span(body),
                            )
                        })?;
                }

                let scheme = TypeScheme::generalize(func_type, &free_vars_before);
                self.env.insert_or_update(name.clone(), scheme);

                Ok(FlowInfo::new(Type::Unit))
            }

            Stmt::TypeAlias { .. } => Ok(FlowInfo::new(Type::Unit)),

            Stmt::If {
                condition,
                then_branch,
                else_branch,
                span,
                ..
            } => {
                let cond_ty = self.infer_expression(condition)?;
                self.unifier
                    .unify(&cond_ty, &Type::Bool, &ast_helpers::get_span(condition))?;

                self.env.enter_scope();
                let then_result = self.check_block_with_flow(then_branch)?;
                self.env.exit_scope()?;

                if let Some(else_branch) = else_branch {
                    self.env.enter_scope();
                    let else_result = self.check_block_with_flow(else_branch)?;
                    self.env.exit_scope()?;

                    self.unifier.unify(&then_result.ty, &else_result.ty, span)?;
                    Ok(FlowInfo {
                        ty: self.unifier.apply(&then_result.ty),
                        always_returns: then_result.always_returns && else_result.always_returns,
                        always_produces_value: then_result.always_produces_value
                            && else_result.always_produces_value,
                    })
                } else {
                    Ok(FlowInfo::new(Type::Unit))
                }
            }

            Stmt::While {
                condition, body, ..
            } => {
                let cond_ty = self.infer_expression(condition)?;
                self.unifier
                    .unify(&cond_ty, &Type::Bool, &ast_helpers::get_span(condition))?;

                self.env.enter_scope();
                self.check_block_with_flow(body)?;
                self.env.exit_scope()?;

                Ok(FlowInfo::new(Type::Unit))
            }

            Stmt::For {
                variable,
                iter,
                body,
                ..
            } => {
                let iter_ty = self.infer_expression(iter)?;

                let elem_ty = if let Type::List(elem) = iter_ty {
                    *elem
                } else {
                    return Err(TypeError::type_mismatch(
                        Type::List(Box::new(Type::var(0))),
                        iter_ty,
                        ast_helpers::get_span(iter),
                    ));
                };

                self.env.enter_scope();
                self.check_pattern(variable, &elem_ty)?;
                self.check_block_with_flow(body)?;
                self.env.exit_scope()?;

                Ok(FlowInfo::new(Type::Unit))
            }

            Stmt::Match {
                scrutinee,
                arms,
                span,
                ..
            } => {
                let scrutinee_ty = self.infer_expression(scrutinee)?;
                let mut arm_results = Vec::new();

                for arm in arms {
                    self.env.enter_scope();
                    self.check_pattern(&arm.pattern, &scrutinee_ty)?;

                    if let Some(guard) = &arm.guard {
                        let guard_ty = self.infer_expression(guard)?;
                        self.unifier.unify(
                            &guard_ty,
                            &Type::Bool,
                            &ast_helpers::get_span(guard),
                        )?;
                    }

                    let arm_result = self.infer_expression_with_flow(&arm.body)?;
                    arm_results.push(arm_result);
                    self.env.exit_scope()?;
                }

                if let Some(first_result) = arm_results.first() {
                    let is_exhaustive = arms
                        .iter()
                        .any(|arm| arm.guard.is_none() && !arm.pattern.is_refutable());
                    for result in &arm_results[1..] {
                        self.unifier.unify(&first_result.ty, &result.ty, span)?;
                    }
                    Ok(FlowInfo {
                        ty: self.unifier.apply(&first_result.ty),
                        always_returns: is_exhaustive
                            && arm_results.iter().all(|result| result.always_returns),
                        always_produces_value: is_exhaustive
                            && arm_results
                                .iter()
                                .all(|result| result.always_produces_value),
                    })
                } else {
                    Ok(FlowInfo::new(Type::Unit))
                }
            }

            Stmt::Return { value, span, .. } => {
                let function = self.function_contexts.last().cloned();
                let value_ty = match value {
                    Some(value) => self.infer_expression(value)?,
                    None => Type::Unit,
                };

                if let Some(function) = function {
                    self.unifier
                        .unify(&value_ty, &function.return_type, span)
                        .map_err(|_| {
                            let expected = self.unifier.apply(&function.return_type);
                            if value.is_none() && expected != Type::Unit && !expected.is_var() {
                                self.missing_return_value_error(&function, expected, span.clone())
                            } else {
                                self.return_type_mismatch_for_function(
                                    &function,
                                    &function.return_type,
                                    &value_ty,
                                    span.clone(),
                                )
                            }
                        })?;

                    Ok(FlowInfo::returning(
                        self.unifier.apply(&function.return_type),
                    ))
                } else {
                    Ok(FlowInfo::returning(Type::Unit))
                }
            }

            Stmt::Break { .. } | Stmt::Continue { .. } => Ok(FlowInfo::new(Type::Unit)),

            Stmt::ExprStmt { expr, .. } => self.infer_expression_with_flow(expr),

            Stmt::Import {
                module, symbols, ..
            } => {
                let register = |env: &mut TypeEnvironment, ctx: &mut TypeContext, name: &str| {
                    let var = ctx.fresh_var();
                    let scheme = crate::ty::TypeScheme::new(
                        vec![var.clone()],
                        Type::Var(crate::types::TypeVarRef::new(var.id())),
                    );
                    env.insert_or_update(name.to_string(), scheme);
                };

                match symbols {
                    Some(syms) => {
                        for sym_name in syms {
                            register(&mut self.env, &mut self.ctx, sym_name);
                        }
                    }
                    None => {
                        let namespace = module.split('/').next_back().unwrap_or(module.as_str());
                        register(&mut self.env, &mut self.ctx, namespace);
                    }
                }

                Ok(FlowInfo::new(Type::Unit))
            }

            Stmt::Class { .. } => Ok(FlowInfo::new(Type::Unit)),
        }
    }

    /// Infer the type of an expression
    fn infer_expression(&mut self, expr: &Expr) -> Result<Type> {
        Ok(self.infer_expression_with_flow(expr)?.ty)
    }

    fn infer_expression_with_flow(&mut self, expr: &Expr) -> Result<FlowInfo> {
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
                Ok(FlowInfo::new(ty))
            }

            Expr::Variable { name, span, .. } => {
                // Look up the variable in the environment
                if let Some(scheme) = self.env.lookup(name) {
                    // Instantiate the type scheme
                    Ok(FlowInfo::new(scheme.instantiate(&mut self.ctx)))
                } else {
                    Err(TypeError::undefined_variable(name.clone(), span.clone()))
                }
            }

            Expr::Binary {
                left,
                right,
                span,
                id: _,
                ..
            } => {
                let left_ty = self.infer_expression(left)?;
                let right_ty = self.infer_expression(right)?;

                // Type check based on operator
                self.unifier.unify(&left_ty, &right_ty, span)?;
                // Numeric operators return the same type as operands
                Ok(FlowInfo::new(left_ty.clone()))
            }

            Expr::Comparison {
                left,
                right,
                span,
                id: _,
                ..
            } => {
                let left_ty = self.infer_expression(left)?;
                let right_ty = self.infer_expression(right)?;

                self.unifier.unify(&left_ty, &right_ty, span)?;
                Ok(FlowInfo::new(Type::Bool))
            }

            Expr::Logical {
                left,
                right,
                span,
                id: _,
                ..
            } => {
                let left_ty = self.infer_expression(left)?;
                let right_ty = self.infer_expression(right)?;

                self.unifier.unify(&left_ty, &Type::Bool, span)?;
                self.unifier.unify(&right_ty, &Type::Bool, span)?;
                Ok(FlowInfo::new(Type::Bool))
            }

            Expr::Unary { expr, id: _, .. } => {
                let expr_ty = self.infer_expression(expr)?;
                Ok(FlowInfo::new(expr_ty))
            }

            Expr::Call {
                callee,
                args,
                span,
                id: _,
            } => {
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

                Ok(FlowInfo::new(return_var))
            }

            Expr::Pipeline {
                stages,
                span,
                id: _,
            } => {
                // Type check each stage
                let mut current_ty = self.infer_expression(&stages[0])?;

                for stage in &stages[1..] {
                    let stage_ty = self.infer_expression(stage)?;

                    // Create a function type: current_ty -> ?
                    let var = self.ctx.fresh_var();
                    let expected_func = Type::Function(
                        vec![current_ty.clone()],
                        Box::new(Type::Var(crate::types::TypeVarRef::new(var.id()))),
                    );

                    self.unifier.unify(&stage_ty, &expected_func, span)?;

                    // Update current type to the return type of the function
                    current_ty = Type::Var(crate::types::TypeVarRef::new(var.id()));
                }

                Ok(FlowInfo::new(current_ty))
            }

            Expr::Lambda { params, body, .. } => {
                // Enter a new scope
                self.env.enter_scope();

                // Create fresh type variables for parameters
                let param_types: Vec<Type> = params
                    .iter()
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

                Ok(FlowInfo::new(Type::Function(
                    param_types,
                    Box::new(body_ty),
                )))
            }

            Expr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                // Type check condition
                let cond_ty = self.infer_expression(condition)?;
                self.unifier
                    .unify(&cond_ty, &Type::Bool, &ast_helpers::get_span(condition))?;

                // Type check branches
                let then_result = self.infer_expression_with_flow(then_branch)?;
                let else_result = self.infer_expression_with_flow(else_branch)?;

                // Unify branch types
                self.unifier.unify(
                    &then_result.ty,
                    &else_result.ty,
                    &ast_helpers::get_span(expr),
                )?;

                Ok(FlowInfo {
                    ty: self.unifier.apply(&then_result.ty),
                    always_returns: then_result.always_returns && else_result.always_returns,
                    always_produces_value: then_result.always_produces_value
                        && else_result.always_produces_value,
                })
            }

            Expr::Block { statements, .. } => {
                self.env.enter_scope();

                let result = self.check_block_with_flow(statements)?;
                self.env.exit_scope()?;

                Ok(result)
            }

            Expr::List { elements, .. } => {
                if elements.is_empty() {
                    // Empty list has a fresh type variable
                    let var = self.ctx.fresh_var();
                    Ok(FlowInfo::new(Type::List(Box::new(Type::Var(
                        crate::types::TypeVarRef::new(var.id()),
                    )))))
                } else {
                    // Infer the type of the first element
                    let elem_ty = self.infer_expression(&elements[0])?;

                    // All elements must have the same type
                    for elem in &elements[1..] {
                        let ty = self.infer_expression(elem)?;
                        self.unifier
                            .unify(&elem_ty, &ty, &ast_helpers::get_span(elem))?;
                    }

                    Ok(FlowInfo::new(Type::List(Box::new(elem_ty))))
                }
            }

            Expr::Map { entries, .. } => {
                if entries.is_empty() {
                    // Empty map has a fresh type variable
                    let var = self.ctx.fresh_var();
                    Ok(FlowInfo::new(Type::Map(Box::new(Type::Var(
                        crate::types::TypeVarRef::new(var.id()),
                    )))))
                } else {
                    // Keys must be strings
                    // Values can be any type, but all must be the same
                    let value_ty = self.infer_expression(&entries[0].1)?;

                    for (key, value) in entries {
                        let key_ty = self.infer_expression(key)?;
                        self.unifier
                            .unify(&key_ty, &Type::String, &ast_helpers::get_span(key))?;

                        let ty = self.infer_expression(value)?;
                        self.unifier
                            .unify(&value_ty, &ty, &ast_helpers::get_span(value))?;
                    }

                    Ok(FlowInfo::new(Type::Map(Box::new(value_ty))))
                }
            }

            Expr::Match {
                scrutinee, arms, ..
            } => {
                // Type check scrutinee
                let scrutinee_ty = self.infer_expression(scrutinee)?;

                // Type check each arm
                let mut arm_results = Vec::new();

                for arm in arms {
                    self.env.enter_scope();

                    // Type check pattern
                    self.check_pattern(&arm.pattern, &scrutinee_ty)?;

                    // Type check guard if present
                    if let Some(guard) = &arm.guard {
                        let guard_ty = self.infer_expression(guard)?;
                        self.unifier.unify(
                            &guard_ty,
                            &Type::Bool,
                            &ast_helpers::get_span(guard),
                        )?;
                    }

                    // Type check body
                    let arm_result = self.infer_expression_with_flow(&arm.body)?;
                    arm_results.push(arm_result);

                    self.env.exit_scope()?;
                }

                // All arms must have the same type
                if let Some(first_result) = arm_results.first() {
                    let is_exhaustive = arms
                        .iter()
                        .any(|arm| arm.guard.is_none() && !arm.pattern.is_refutable());
                    for result in &arm_results[1..] {
                        self.unifier.unify(
                            &first_result.ty,
                            &result.ty,
                            &ast_helpers::get_span(expr),
                        )?;
                    }
                    Ok(FlowInfo {
                        ty: self.unifier.apply(&first_result.ty),
                        always_returns: is_exhaustive
                            && arm_results.iter().all(|result| result.always_returns),
                        always_produces_value: is_exhaustive
                            && arm_results
                                .iter()
                                .all(|result| result.always_produces_value),
                    })
                } else {
                    Ok(FlowInfo::new(Type::Unit))
                }
            }

            Expr::Index { array, index, .. } => {
                // Infer array and index types
                let array_ty = self.infer_expression(array)?;
                let _index_ty = self.infer_expression(index)?;

                // Return the element type: if the array is a known List(T) return T,
                // otherwise produce a fresh type variable (array type is still unknown).
                match array_ty {
                    Type::List(elem_ty) => Ok(FlowInfo::new(*elem_ty)),
                    _ => {
                        let var = self.ctx.fresh_var();
                        Ok(FlowInfo::new(Type::Var(crate::types::TypeVarRef::new(
                            var.id(),
                        ))))
                    }
                }
            }

            Expr::Assign { target, value, .. } => {
                let _target_ty = self.infer_expression(target)?;
                let value_ty = self.infer_expression(value)?;
                Ok(FlowInfo::new(value_ty))
            }

            Expr::MemberAccess { object, .. } => {
                let _obj_ty = self.infer_expression(object)?;
                // Return a fresh type variable since we don't know the member type
                let var = self.ctx.fresh_var();
                Ok(FlowInfo::new(Type::Var(crate::types::TypeVarRef::new(
                    var.id(),
                ))))
            }
        }
    }

    fn check_block_with_flow(&mut self, stmts: &[Stmt]) -> Result<FlowInfo> {
        let mut result = FlowInfo::new(Type::Unit);

        for stmt in stmts {
            let stmt_result = self.check_statement_with_flow(stmt)?;
            if !result.always_returns {
                result = stmt_result;
            }
        }

        Ok(result)
    }

    fn return_type_mismatch_error(
        &self,
        function: &str,
        return_annotation_span: Option<Span>,
        expected: &Type,
        found: &Type,
        span: Span,
    ) -> TypeError {
        let expected = self.unifier.apply(expected);
        let found = self.unifier.apply(found);
        let mut error =
            TypeError::return_type_mismatch(function.to_string(), expected.clone(), found, span);
        if let Some(span) = return_annotation_span {
            error = error.with_context(
                format!(
                    "function '{}' declares return type {}",
                    function,
                    expected.display_name()
                ),
                Some(span),
            );
        }
        error
    }

    fn return_type_mismatch_for_function(
        &self,
        function: &FunctionContext,
        expected: &Type,
        found: &Type,
        span: Span,
    ) -> TypeError {
        self.return_type_mismatch_error(
            &function.name,
            function.return_annotation_span.clone(),
            expected,
            found,
            span,
        )
    }

    fn missing_return_value_error(
        &self,
        function: &FunctionContext,
        expected: Type,
        span: Span,
    ) -> TypeError {
        let mut error =
            TypeError::missing_return_value(function.name.clone(), expected.clone(), span);
        if let Some(span) = function.return_annotation_span.clone() {
            error = error.with_context(
                format!(
                    "function '{}' declares return type {}",
                    function.name,
                    expected.display_name()
                ),
                Some(span),
            );
        }
        error
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
                        return Err(TypeError::arity_mismatch(
                            elem_types.len(),
                            patterns.len(),
                            ast_helpers::get_span_pattern(pattern),
                        ));
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

            Pattern::Struct { .. } => {
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

            Pattern::Constructor { name: _, args, .. } => {
                // Type check constructor arguments
                for arg in args {
                    let var = self.ctx.fresh_var();
                    let arg_ty = Type::Var(crate::types::TypeVarRef::new(var.id()));
                    self.check_pattern(arg, &arg_ty)?;
                }
                Ok(())
            }
        }
    }

    /// Resolve an AST type annotation to a type-checker type
    fn resolve_type_annotation(&mut self, ann: &nevermind_ast::TypeAnnotation) -> Type {
        use nevermind_ast::types::{PrimitiveType as AstPrim, Type as AstType};
        match &ann.kind {
            AstType::Primitive(prim) => match prim {
                AstPrim::Int | AstPrim::Int32 | AstPrim::Int64 => Type::Int,
                AstPrim::UInt | AstPrim::UInt32 | AstPrim::UInt64 => Type::Int,
                AstPrim::Float | AstPrim::Float32 | AstPrim::Float64 => Type::Float,
                AstPrim::Bool => Type::Bool,
                AstPrim::String => Type::String,
                AstPrim::Char => Type::String,
                AstPrim::Unit => Type::Unit,
                AstPrim::Null => Type::Null,
            },
            AstType::Identifier(name) => match name.as_str() {
                "Int" => Type::Int,
                "Float" => Type::Float,
                "Bool" => Type::Bool,
                "String" => Type::String,
                "Unit" | "Void" => Type::Unit,
                _ => Type::User(name.clone()),
            },
            AstType::List(elem) => {
                let elem_ty = self.resolve_type_annotation(elem);
                Type::List(Box::new(elem_ty))
            }
            AstType::Tuple(elems) => {
                let elem_tys: Vec<Type> = elems
                    .iter()
                    .map(|e| self.resolve_type_annotation(e))
                    .collect();
                Type::Tuple(elem_tys)
            }
            AstType::Function {
                params,
                return_type,
            } => {
                let param_tys: Vec<Type> = params
                    .iter()
                    .map(|p| self.resolve_type_annotation(p))
                    .collect();
                let ret_ty = self.resolve_type_annotation(return_type);
                Type::Function(param_tys, Box::new(ret_ty))
            }
            _ => {
                // For unsupported types, use a fresh type variable
                let var = self.ctx.fresh_var();
                Type::Var(crate::types::TypeVarRef::new(var.id()))
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
            Expr::Assign { span, .. } => span.clone(),
            Expr::MemberAccess { span, .. } => span.clone(),
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
            Pattern::Constructor { span, .. } => span.clone(),
        }
    }

    fn get_span_literal(lit: &Literal) -> Span {
        match lit {
            Literal::Integer(_, span)
            | Literal::Float(_, span)
            | Literal::String(_, span)
            | Literal::Char(_, span)
            | Literal::Boolean(_, span)
            | Literal::Null(span) => span.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nevermind_ast::types::{PrimitiveType as AstPrimitiveType, Type as AstType};
    use nevermind_ast::TypeAnnotation;
    use nevermind_common::Span;

    fn int_expr(value: i64) -> Expr {
        Expr::Literal(Literal::Integer(value, Span::dummy()))
    }

    fn bool_expr(value: bool) -> Expr {
        Expr::Literal(Literal::Boolean(value, Span::dummy()))
    }

    fn expr_stmt(expr: Expr) -> Stmt {
        Stmt::ExprStmt {
            id: 99,
            expr,
            span: Span::dummy(),
        }
    }

    fn return_stmt(value: Option<Expr>) -> Stmt {
        Stmt::Return {
            id: 100,
            value,
            span: Span::dummy(),
        }
    }

    fn int_annotation() -> TypeAnnotation {
        TypeAnnotation {
            id: 101,
            span: Span::dummy(),
            kind: AstType::Primitive(AstPrimitiveType::Int),
        }
    }

    fn int_pattern(value: i64) -> Pattern {
        Pattern::Literal {
            value: Literal::Integer(value, Span::dummy()),
            span: Span::dummy(),
        }
    }

    fn wildcard_pattern() -> Pattern {
        Pattern::Wildcard {
            span: Span::dummy(),
        }
    }

    fn match_expr(arms: Vec<(Pattern, Option<Expr>, Expr)>) -> Expr {
        Expr::Match {
            id: 108,
            scrutinee: Box::new(int_expr(1)),
            arms: arms
                .into_iter()
                .map(|(pattern, guard, body)| nevermind_ast::expr::MatchArm {
                    pattern,
                    guard: guard.map(Box::new),
                    body: Box::new(body),
                })
                .collect(),
            span: Span::dummy(),
        }
    }

    fn function_with_body(name: &str, return_type: Option<TypeAnnotation>, body: Expr) -> Stmt {
        Stmt::Function {
            id: 102,
            name: name.to_string(),
            params: vec![],
            return_type,
            body,
            span: Span::dummy(),
        }
    }

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
        checker
            .env
            .insert("x".to_string(), TypeScheme::monomorphic(Type::Int))
            .unwrap();

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

    #[test]
    fn test_declared_return_type_mismatch_reports_error() {
        let mut checker = TypeChecker::new();
        let stmt = function_with_body("foo", Some(int_annotation()), bool_expr(true));

        let err = checker.check(&[stmt]).unwrap_err();

        assert!(matches!(
            err.kind,
            crate::error::TypeErrorKind::ReturnTypeMismatch { .. }
        ));
        assert!(err.message.contains("expected Int, found Bool"));
    }

    #[test]
    fn test_missing_return_value_reports_error() {
        let mut checker = TypeChecker::new();
        let stmt = function_with_body(
            "foo",
            Some(int_annotation()),
            Expr::Block {
                id: 103,
                statements: vec![return_stmt(None)],
                span: Span::dummy(),
            },
        );

        let err = checker.check(&[stmt]).unwrap_err();

        assert!(matches!(
            err.kind,
            crate::error::TypeErrorKind::MissingReturnValue { .. }
        ));
        assert!(err.message.contains("must return Int"));
    }

    #[test]
    fn test_missing_return_path_reports_error() {
        let mut checker = TypeChecker::new();
        let stmt = function_with_body(
            "foo",
            Some(int_annotation()),
            Expr::Block {
                id: 104,
                statements: vec![Stmt::If {
                    id: 105,
                    condition: bool_expr(true),
                    then_branch: vec![return_stmt(Some(int_expr(1)))],
                    else_branch: None,
                    span: Span::dummy(),
                }],
                span: Span::dummy(),
            },
        );

        let err = checker.check(&[stmt]).unwrap_err();

        assert!(matches!(
            err.kind,
            crate::error::TypeErrorKind::MissingReturn { .. }
        ));
        assert!(err
            .message
            .contains("not all paths in function 'foo' return Int"));
    }

    #[test]
    fn test_explicit_return_and_fallthrough_value_can_agree() {
        let mut checker = TypeChecker::new();
        let stmt = function_with_body(
            "foo",
            None,
            Expr::Block {
                id: 106,
                statements: vec![
                    Stmt::If {
                        id: 107,
                        condition: bool_expr(true),
                        then_branch: vec![return_stmt(Some(int_expr(1)))],
                        else_branch: None,
                        span: Span::dummy(),
                    },
                    expr_stmt(int_expr(2)),
                ],
                span: Span::dummy(),
            },
        );

        checker.check(&[stmt]).unwrap();
    }

    #[test]
    fn test_non_exhaustive_if_statement_can_fall_through_to_later_value() {
        let mut checker = TypeChecker::new();
        let stmt = function_with_body(
            "foo",
            Some(int_annotation()),
            Expr::Block {
                id: 108,
                statements: vec![
                    Stmt::If {
                        id: 109,
                        condition: bool_expr(true),
                        then_branch: vec![expr_stmt(int_expr(1))],
                        else_branch: None,
                        span: Span::dummy(),
                    },
                    expr_stmt(int_expr(2)),
                ],
                span: Span::dummy(),
            },
        );

        checker.check(&[stmt]).unwrap();
    }

    #[test]
    fn test_non_exhaustive_match_expression_reports_missing_return() {
        let mut checker = TypeChecker::new();
        let stmt = function_with_body(
            "foo",
            Some(int_annotation()),
            match_expr(vec![(int_pattern(1), None, int_expr(1))]),
        );

        let err = checker.check(&[stmt]).unwrap_err();

        assert!(matches!(
            err.kind,
            crate::error::TypeErrorKind::MissingReturn { .. }
        ));
        assert!(err
            .message
            .contains("not all paths in function 'foo' return Int"));
    }

    #[test]
    fn test_guarded_wildcard_match_expression_reports_missing_return() {
        let mut checker = TypeChecker::new();
        let stmt = function_with_body(
            "foo",
            Some(int_annotation()),
            match_expr(vec![(
                wildcard_pattern(),
                Some(bool_expr(false)),
                int_expr(1),
            )]),
        );

        let err = checker.check(&[stmt]).unwrap_err();

        assert!(matches!(
            err.kind,
            crate::error::TypeErrorKind::MissingReturn { .. }
        ));
        assert!(err
            .message
            .contains("not all paths in function 'foo' return Int"));
    }

    #[test]
    fn test_match_with_wildcard_arm_satisfies_declared_return_type() {
        let mut checker = TypeChecker::new();
        let stmt = function_with_body(
            "foo",
            Some(int_annotation()),
            match_expr(vec![
                (int_pattern(1), None, int_expr(1)),
                (wildcard_pattern(), None, int_expr(2)),
            ]),
        );

        checker.check(&[stmt]).unwrap();
    }

    #[test]
    fn test_non_exhaustive_match_statement_can_fall_through_to_later_value() {
        let mut checker = TypeChecker::new();
        let stmt = function_with_body(
            "foo",
            Some(int_annotation()),
            Expr::Block {
                id: 109,
                statements: vec![
                    Stmt::Match {
                        id: 110,
                        scrutinee: int_expr(1),
                        arms: vec![nevermind_ast::stmt::MatchArm {
                            pattern: int_pattern(1),
                            guard: None,
                            body: int_expr(1),
                        }],
                        span: Span::dummy(),
                    },
                    expr_stmt(int_expr(2)),
                ],
                span: Span::dummy(),
            },
        );

        checker.check(&[stmt]).unwrap();
    }

    #[test]
    fn test_guarded_match_statement_can_fall_through_to_later_value() {
        let mut checker = TypeChecker::new();
        let stmt = function_with_body(
            "foo",
            Some(int_annotation()),
            Expr::Block {
                id: 111,
                statements: vec![
                    Stmt::Match {
                        id: 112,
                        scrutinee: int_expr(1),
                        arms: vec![nevermind_ast::stmt::MatchArm {
                            pattern: wildcard_pattern(),
                            guard: Some(bool_expr(false)),
                            body: int_expr(1),
                        }],
                        span: Span::dummy(),
                    },
                    expr_stmt(int_expr(2)),
                ],
                span: Span::dummy(),
            },
        );

        checker.check(&[stmt]).unwrap();
    }
}
