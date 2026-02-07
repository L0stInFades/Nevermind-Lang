//! MIR lowering - convert typed AST to MIR

use super::{MirExpr, MirExprStmt, MirBlock, MirStmt, BinOp, UnaryOp, Literal, NodeId, Param};
use nevermind_ast::{Expr, Stmt, op::{BinaryOp, ComparisonOp, LogicalOp}};
use nevermind_type_checker::Type;

/// Error during MIR lowering
#[derive(Debug, thiserror::Error)]
pub enum LoweringError {
    #[error("Unsupported AST node: {0}")]
    UnsupportedNode(String),

    #[error("Missing type information")]
    MissingType,
}

pub type Result<T> = std::result::Result<T, LoweringError>;

/// Lower a typed AST statement to MIR
pub fn lower_statement(stmt: &Stmt) -> Result<MirStmt> {
    match stmt {
        Stmt::Let {
            name, value, is_mutable: _, type_annotation, ..
        } => {
            let mir_value = lower_expression(value)?;
            let mir_type = type_annotation
                .as_ref()
                .and_then(|t| resolve_type_annotation(t))
                .unwrap_or_else(|| mir_value.get_type().clone());

            Ok(MirStmt::Let {
                name: name.clone(),
                value: mir_value,
                ty: mir_type,
                id: fresh_node_id(),
            })
        }

        Stmt::Function {
            name,
            params,
            body,
            return_type: _,
            ..
        } => {
            let mir_params = params
                .iter()
                .map(|p| {
                    Ok(Param {
                        name: p.name.clone(),
                        ty: p.type_annotation
                            .as_ref()
                            .and_then(|t| resolve_type_annotation(t))
                            .unwrap_or(Type::Unit),
                        id: p.id,
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            let mut mir_statements = Vec::new();
            let mir_expr = lower_expression(body)?;

            let mir_block = MirBlock {
                statements: mir_statements,
                expr: Some(Box::new(mir_expr)),
            };

            Ok(MirStmt::Function {
                name: name.clone(),
                params: mir_params,
                body: mir_block,
                return_type: Type::Unit, // TODO: Infer return type
                id: fresh_node_id(),
            })
        }

        Stmt::ExprStmt { expr, .. } => {
            let mir_expr = lower_expression(expr)?;
            Ok(MirStmt::Expr(mir_expr))
        }

        Stmt::If { condition, then_branch, else_branch, .. } => {
            let mir_cond = lower_expression(&condition)?;
            let mut then_stmts = Vec::new();
            for s in then_branch {
                then_stmts.push(lower_expr_stmt(s)?);
            }
            let else_stmts = if let Some(else_b) = else_branch {
                let mut stmts = Vec::new();
                for s in else_b {
                    stmts.push(lower_expr_stmt(s)?);
                }
                Some(stmts)
            } else {
                None
            };
            Ok(MirStmt::Expr(MirExpr::Block {
                statements: vec![MirExprStmt::Expr(mir_cond)],
                expr: None,
                ty: Type::Unit,
                id: fresh_node_id(),
            }))
        }

        Stmt::While { condition, body, .. } => {
            let mir_cond = lower_expression(&condition)?;
            let mut mir_body = Vec::new();
            for s in body {
                mir_body.push(lower_expr_stmt(s)?);
            }
            Ok(MirStmt::Expr(MirExpr::Block {
                statements: mir_body,
                expr: None,
                ty: Type::Unit,
                id: fresh_node_id(),
            }))
        }

        Stmt::For { variable, iter, body, .. } => {
            let mir_iter = lower_expression(&iter)?;
            let mut mir_body = Vec::new();
            for s in body {
                mir_body.push(lower_expr_stmt(s)?);
            }
            Ok(MirStmt::Expr(MirExpr::Block {
                statements: mir_body,
                expr: None,
                ty: Type::Unit,
                id: fresh_node_id(),
            }))
        }

        Stmt::Return { value, .. } => {
            let mir_value = if let Some(v) = value {
                Some(Box::new(lower_expression(v)?))
            } else {
                None
            };
            Ok(MirStmt::Expr(MirExpr::Literal {
                value: Literal::Null,
                ty: Type::Unit,
                id: fresh_node_id(),
            }))
        }

        Stmt::Break { .. } | Stmt::Continue { .. } => {
            Ok(MirStmt::Expr(MirExpr::Literal {
                value: Literal::Null,
                ty: Type::Unit,
                id: fresh_node_id(),
            }))
        }

        Stmt::Match { scrutinee, arms, .. } => {
            let mir_scrutinee = lower_expression(&scrutinee)?;
            Ok(MirStmt::Expr(mir_scrutinee))
        }

        _ => Err(LoweringError::UnsupportedNode(format!("{:?}", stmt))),
    }
}

/// Lower a typed AST expression to MIR
pub fn lower_expression(expr: &Expr) -> Result<MirExpr> {
    match expr {
        Expr::Literal(literal) => lower_literal(literal),

        Expr::Variable { name, id, .. } => {
            Ok(MirExpr::Variable {
                name: name.clone(),
                ty: Type::Unit, // TODO: Get type from type checker
                id: *id,
            })
        }

        Expr::Binary {
            left,
            op,
            right,
            id,
            ..
        } => {
            let mir_left = Box::new(lower_expression(left)?);
            let mir_right = Box::new(lower_expression(right)?);
            let mir_op = map_binary_op(op);

            Ok(MirExpr::Binary {
                op: mir_op,
                left: mir_left,
                right: mir_right,
                ty: Type::Unit,
                id: *id,
            })
        }

        Expr::Comparison {
            left,
            op,
            right,
            id,
            ..
        } => {
            let mir_left = Box::new(lower_expression(left)?);
            let mir_right = Box::new(lower_expression(right)?);
            let mir_op = map_comparison_op(op);

            Ok(MirExpr::Binary {
                op: mir_op,
                left: mir_left,
                right: mir_right,
                ty: Type::Bool,
                id: *id,
            })
        }

        Expr::Logical {
            left,
            op,
            right,
            id,
            ..
        } => {
            let mir_left = Box::new(lower_expression(left)?);
            let mir_right = Box::new(lower_expression(right)?);
            let mir_op = map_logical_op(op);

            Ok(MirExpr::Binary {
                op: mir_op,
                left: mir_left,
                right: mir_right,
                ty: Type::Bool,
                id: *id,
            })
        }

        Expr::Unary {
            op,
            expr,
            id,
            ..
        } => {
            let mir_operand = Box::new(lower_expression(expr)?);
            let mir_op = map_unary_op(op);

            Ok(MirExpr::Unary {
                op: mir_op,
                operand: mir_operand,
                ty: Type::Unit,
                id: *id,
            })
        }

        Expr::Call {
            callee, args, id, ..
        } => {
            let mir_callee = Box::new(lower_expression(callee)?);
            let mir_args = args
                .iter()
                .map(lower_expression)
                .collect::<Result<Vec<_>>>()?;

            Ok(MirExpr::Call {
                callee: mir_callee,
                args: mir_args,
                ty: Type::Unit,
                id: *id,
            })
        }

        Expr::Block { statements, id, .. } => {
            let mut mir_statements = Vec::new();
            for stmt in statements {
                mir_statements.push(lower_expr_stmt(stmt)?);
            }

            Ok(MirExpr::Block {
                statements: mir_statements,
                expr: None,
                ty: Type::Unit,
                id: *id,
            })
        }

        Expr::List { elements, id, .. } => {
            let mir_elements = elements
                .iter()
                .map(lower_expression)
                .collect::<Result<Vec<_>>>()?;

            Ok(MirExpr::List {
                elements: mir_elements,
                ty: Type::List(Box::new(Type::Int)), // TODO: Infer element type
                id: *id,
            })
        }

        Expr::If { condition, then_branch, else_branch, id, .. } => {
            let mir_condition = Box::new(lower_expression(condition)?);
            let mir_then = Box::new(lower_expression(then_branch)?);
            let mir_else = Box::new(lower_expression(else_branch)?);

            Ok(MirExpr::If {
                condition: mir_condition,
                then_branch: mir_then,
                else_branch: mir_else,
                ty: Type::Unit, // TODO: Infer result type
                id: *id,
            })
        }

        Expr::Index { array, index, id, .. } => {
            let mir_array = Box::new(lower_expression(array)?);
            let mir_index = Box::new(lower_expression(index)?);

            Ok(MirExpr::Index {
                array: mir_array,
                index: mir_index,
                ty: Type::Int, // TODO: Infer element type
                id: *id,
            })
        }

        Expr::Assign { target, value, id, .. } => {
            let mir_value = lower_expression(value)?;
            // For simple variable assignment, extract target name
            if let Expr::Variable { name, .. } = target.as_ref() {
                Ok(MirExpr::Block {
                    statements: vec![MirExprStmt::Assign {
                        target: name.clone(),
                        value: mir_value,
                        id: *id,
                    }],
                    expr: None,
                    ty: Type::Unit,
                    id: fresh_node_id(),
                })
            } else {
                Ok(mir_value)
            }
        }

        Expr::MemberAccess { object, member, id, .. } => {
            let mir_obj = lower_expression(object)?;
            // Represent as dotted name for now
            if let MirExpr::Variable { name, ty, .. } = &mir_obj {
                Ok(MirExpr::Variable {
                    name: format!("{}.{}", name, member),
                    ty: ty.clone(),
                    id: *id,
                })
            } else {
                Ok(MirExpr::Variable {
                    name: member.clone(),
                    ty: Type::Unit,
                    id: *id,
                })
            }
        }

        Expr::Pipeline { stages, id, .. } => {
            // Lower pipeline as nested function calls
            if stages.is_empty() {
                return Ok(MirExpr::Literal { value: Literal::Null, ty: Type::Unit, id: fresh_node_id() });
            }
            let mut result = lower_expression(&stages[0])?;
            for stage in &stages[1..] {
                let func = lower_expression(stage)?;
                result = MirExpr::Call {
                    callee: Box::new(func),
                    args: vec![result],
                    ty: Type::Unit,
                    id: fresh_node_id(),
                };
            }
            Ok(result)
        }

        Expr::Lambda { params, body, id, .. } => {
            let mir_params = params.iter().map(|p| {
                Param {
                    name: p.name.clone(),
                    ty: p.type_annotation.as_ref()
                        .and_then(|t| resolve_type_annotation(t))
                        .unwrap_or(Type::Unit),
                    id: p.id,
                }
            }).collect::<Vec<_>>();
            let mir_body = lower_expression(body)?;
            // Represent lambda as a block for now
            Ok(mir_body)
        }

        Expr::Map { entries, id, .. } => {
            // Lower map as a list of key-value pairs for now
            let mut elements = Vec::new();
            for (key, value) in entries {
                elements.push(lower_expression(key)?);
                elements.push(lower_expression(value)?);
            }
            Ok(MirExpr::List {
                elements,
                ty: Type::Unit,
                id: *id,
            })
        }

        Expr::Match { scrutinee, arms, id, .. } => {
            // Lower match as the scrutinee for now
            lower_expression(scrutinee)
        }

        _ => Err(LoweringError::UnsupportedNode(format!("{:?}", expr))),
    }
}

/// Lower a literal value
fn lower_literal(literal: &nevermind_ast::expr::Literal) -> Result<MirExpr> {
    Ok(match literal {
        nevermind_ast::expr::Literal::Integer(value, _) => {
            MirExpr::Literal {
                value: Literal::Int(*value),
                ty: Type::Int,
                id: fresh_node_id(),
            }
        }
        nevermind_ast::expr::Literal::Float(value, _) => {
            MirExpr::Literal {
                value: Literal::Float(*value),
                ty: Type::Float,
                id: fresh_node_id(),
            }
        }
        nevermind_ast::expr::Literal::String(value, _) => {
            MirExpr::Literal {
                value: Literal::String(value.clone()),
                ty: Type::String,
                id: fresh_node_id(),
            }
        }
        nevermind_ast::expr::Literal::Boolean(value, _) => {
            MirExpr::Literal {
                value: Literal::Bool(*value),
                ty: Type::Bool,
                id: fresh_node_id(),
            }
        }
        nevermind_ast::expr::Literal::Null(_) => {
            MirExpr::Literal {
                value: Literal::Null,
                ty: Type::Null,
                id: fresh_node_id(),
            }
        }
        nevermind_ast::expr::Literal::Char(_, _) => {
            // Treat char as integer for now
            MirExpr::Literal {
                value: Literal::Int(0),
                ty: Type::Int,
                id: fresh_node_id(),
            }
        }
    })
}

/// Lower a statement inside an expression block
pub fn lower_expr_stmt(stmt: &Stmt) -> Result<MirExprStmt> {
    match stmt {
        Stmt::Let { name, value, type_annotation, .. } => {
            let mir_value = lower_expression(value)?;
            let ty = type_annotation
                .as_ref()
                .and_then(|t| resolve_type_annotation(t))
                .unwrap_or_else(|| mir_value.get_type().clone());

            Ok(MirExprStmt::Let {
                name: name.clone(),
                value: mir_value,
                ty,
                id: fresh_node_id(),
            })
        }

        Stmt::ExprStmt { expr, .. } => {
            let mir_expr = lower_expression(expr)?;
            Ok(MirExprStmt::Expr(mir_expr))
        }

        Stmt::Function { name, params, body, .. } => {
            let mir_body = lower_expression(body)?;
            Ok(MirExprStmt::Expr(mir_body))
        }

        Stmt::If { condition, then_branch, else_branch, .. } => {
            let mir_cond = lower_expression(&condition)?;
            Ok(MirExprStmt::Expr(mir_cond))
        }

        Stmt::While { condition, body, .. } => {
            let mir_cond = lower_expression(&condition)?;
            Ok(MirExprStmt::Expr(mir_cond))
        }

        Stmt::For { iter, .. } => {
            let mir_iter = lower_expression(&iter)?;
            Ok(MirExprStmt::Expr(mir_iter))
        }

        Stmt::Return { value, .. } => {
            let mir_value = value.as_ref().map(|v| lower_expression(v)).transpose()?;
            Ok(MirExprStmt::Return {
                value: mir_value.map(Box::new),
                id: fresh_node_id(),
            })
        }

        Stmt::Break { .. } | Stmt::Continue { .. } => {
            Ok(MirExprStmt::Expr(MirExpr::Literal {
                value: Literal::Null,
                ty: Type::Unit,
                id: fresh_node_id(),
            }))
        }

        Stmt::Match { scrutinee, .. } => {
            let mir_scrutinee = lower_expression(&scrutinee)?;
            Ok(MirExprStmt::Expr(mir_scrutinee))
        }

        _ => Err(LoweringError::UnsupportedNode(format!("{:?}", stmt))),
    }
}

/// Resolve a type annotation to a Type
fn resolve_type_annotation(ty: &nevermind_ast::TypeAnnotation) -> Option<Type> {
    match &ty.kind {
        nevermind_ast::types::Type::Primitive(prim) => match prim {
            nevermind_ast::types::PrimitiveType::Int => Some(Type::Int),
            nevermind_ast::types::PrimitiveType::Float => Some(Type::Float),
            nevermind_ast::types::PrimitiveType::String => Some(Type::String),
            nevermind_ast::types::PrimitiveType::Bool => Some(Type::Bool),
            _ => Some(Type::Unit),
        },
        nevermind_ast::types::Type::Identifier(name) => Some(Type::User(name.clone())),
        _ => Some(Type::Unit),
    }
}

/// Map an AST binary operator to MIR binary operator
fn map_binary_op(op: &nevermind_ast::op::BinaryOp) -> BinOp {
    match op {
        nevermind_ast::op::BinaryOp::Add => BinOp::Add,
        nevermind_ast::op::BinaryOp::Sub => BinOp::Sub,
        nevermind_ast::op::BinaryOp::Mul => BinOp::Mul,
        nevermind_ast::op::BinaryOp::Div => BinOp::Div,
        nevermind_ast::op::BinaryOp::Mod => BinOp::Mod,
        nevermind_ast::op::BinaryOp::Pow => BinOp::Pow,
        _ => BinOp::Add, // Fallback
    }
}

/// Map an AST comparison operator to MIR binary operator
fn map_comparison_op(op: &nevermind_ast::op::ComparisonOp) -> BinOp {
    match op {
        nevermind_ast::op::ComparisonOp::Eq => BinOp::Eq,
        nevermind_ast::op::ComparisonOp::Ne => BinOp::Ne,
        nevermind_ast::op::ComparisonOp::Lt => BinOp::Lt,
        nevermind_ast::op::ComparisonOp::Le => BinOp::Le,
        nevermind_ast::op::ComparisonOp::Gt => BinOp::Gt,
        nevermind_ast::op::ComparisonOp::Ge => BinOp::Ge,
    }
}

/// Map an AST unary operator to MIR unary operator
fn map_unary_op(op: &nevermind_ast::op::UnaryOp) -> UnaryOp {
    match op {
        nevermind_ast::op::UnaryOp::Neg => UnaryOp::Neg,
        nevermind_ast::op::UnaryOp::Not => UnaryOp::Not,
        _ => UnaryOp::Neg,
    }
}

/// Map an AST logical operator to MIR binary operator
fn map_logical_op(op: &nevermind_ast::op::LogicalOp) -> BinOp {
    match op {
        nevermind_ast::op::LogicalOp::And => BinOp::And,
        nevermind_ast::op::LogicalOp::Or => BinOp::Or,
    }
}


/// Generate a fresh node ID
fn fresh_node_id() -> NodeId {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(10000);
    COUNTER.fetch_add(1, Ordering::SeqCst) as NodeId
}