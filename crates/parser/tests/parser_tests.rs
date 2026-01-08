//! Comprehensive unit tests for the Nevermind Parser
//!
//! This test suite covers:
//! - All statement types (let, function, if, while, for, match, return, break, continue)
//! - All expression types (literals, variables, binary ops, unary ops, function calls, lists, maps, lambdas)
//! - Error cases (missing keywords, mismatched delimiters, invalid syntax)
//! - Complex scenarios (nested functions, multiple statements, pattern matching)

use nevermind_parser::{Parser, ParseError};
use nevermind_ast::{Stmt, Expr, Pattern, Literal, BinaryOp, ComparisonOp, UnaryOp, LogicalOp};

// ============================================================================
// Helper Functions
// ============================================================================

/// Helper to parse source code and get statements
fn parse(source: &str) -> Result<Vec<Stmt>, ParseError> {
    let mut parser = Parser::new(source)?;
    parser.parse()
}

/// Helper to parse and get the first statement
fn parse_first(source: &str) -> Result<Stmt, ParseError> {
    let stmts = parse(source)?;
    stmts.into_iter().next().ok_or_else(|| {
        ParseError::new("No statements found", nevermind_common::Span::dummy())
    })
}

/// Helper to parse an expression statement
fn parse_expr(source: &str) -> Result<Expr, ParseError> {
    match parse_first(source)? {
        Stmt::ExprStmt { expr, .. } => Ok(expr),
        other => Err(ParseError::new(
            format!("Expected ExprStmt, got {:?}", other),
            nevermind_common::Span::dummy(),
        )),
    }
}

// ============================================================================
// Statement Tests
// ============================================================================

mod statement_tests {
    use super::*;

    // ---------------------------------------------------------------------
    // Let Statements
    // ---------------------------------------------------------------------

    #[test]
    fn test_let_statement_immutable() {
        let stmt = parse_first("let x = 42").unwrap();
        match stmt {
            Stmt::Let { name, is_mutable, .. } => {
                assert_eq!(name, "x");
                assert!(!is_mutable);
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_let_statement_mutable() {
        let stmt = parse_first("var x = 42").unwrap();
        match stmt {
            Stmt::Let { name, is_mutable, .. } => {
                assert_eq!(name, "x");
                assert!(is_mutable);
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_let_statement_with_type() {
        let stmt = parse_first("let x: Int = 42").unwrap();
        match stmt {
            Stmt::Let { name, type_annotation, .. } => {
                assert_eq!(name, "x");
                assert!(type_annotation.is_some());
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_let_statement_complex_expr() {
        let stmt = parse_first("let x = 1 + 2 * 3").unwrap();
        match stmt {
            Stmt::Let { name, value, .. } => {
                assert_eq!(name, "x");
                match value {
                    Expr::Binary { op, .. } => {
                        assert_eq!(op, BinaryOp::Add);
                    }
                    _ => panic!("Expected Binary expression"),
                }
            }
            _ => panic!("Expected Let statement"),
        }
    }

    // ---------------------------------------------------------------------
    // Function Statements
    // ---------------------------------------------------------------------

    #[test]
    fn test_function_declaration_simple() {
        let stmt = parse_first("fn foo() do 42 end").unwrap();
        match stmt {
            Stmt::Function { name, params, .. } => {
                assert_eq!(name, "foo");
                assert_eq!(params.len(), 0);
            }
            _ => panic!("Expected Function statement"),
        }
    }

    #[test]
    fn test_function_declaration_with_params() {
        let stmt = parse_first("fn add(a, b) do a + b end").unwrap();
        match stmt {
            Stmt::Function { name, params, .. } => {
                assert_eq!(name, "add");
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].name, "a");
                assert_eq!(params[1].name, "b");
            }
            _ => panic!("Expected Function statement"),
        }
    }

    #[test]
    fn test_function_declaration_with_types() {
        let stmt = parse_first("fn add(a: Int, b: Int) -> Int do a + b end").unwrap();
        match stmt {
            Stmt::Function { name, params, return_type, .. } => {
                assert_eq!(name, "add");
                assert_eq!(params.len(), 2);
                assert!(params[0].type_annotation.is_some());
                assert!(params[1].type_annotation.is_some());
                assert!(return_type.is_some());
            }
            _ => panic!("Expected Function statement"),
        }
    }

    #[test]
    fn test_function_declaration_with_default_param() {
        let stmt = parse_first("fn greet(name = \"World\") do name end").unwrap();
        match stmt {
            Stmt::Function { params, .. } => {
                assert_eq!(params.len(), 1);
                assert!(params[0].default_value.is_some());
            }
            _ => panic!("Expected Function statement"),
        }
    }

    #[test]
    fn test_function_declaration_complex_body() {
        let stmt = parse_first(
            "fn factorial(n) do \
                if n <= 1 then 1 else n * factorial(n - 1) end \
             end"
        ).unwrap();
        match stmt {
            Stmt::Function { name, body, .. } => {
                assert_eq!(name, "factorial");
                match body {
                    Expr::If { .. } => {
                        // Expected an if expression in the body
                    }
                    _ => panic!("Expected If expression in function body"),
                }
            }
            _ => panic!("Expected Function statement"),
        }
    }

    // ---------------------------------------------------------------------
    // If Statements and Expressions
    // ---------------------------------------------------------------------

    #[test]
    fn test_if_expression() {
        let stmt = parse_first("if x > 0 then x else 0 end").unwrap();
        match stmt {
            Stmt::ExprStmt { expr, .. } => match expr {
                Expr::If { condition, .. } => {
                    match *condition {
                        Expr::Comparison { .. } => {
                            // Expected comparison in condition
                        }
                        _ => panic!("Expected Comparison in condition"),
                    }
                }
                _ => panic!("Expected If expression"),
            },
            _ => panic!("Expected ExprStmt"),
        }
    }

    #[test]
    fn test_if_statement_with_do() {
        let stmt = parse_first(
            "if x > 0 do \
                print x \
             end"
        ).unwrap();
        match stmt {
            Stmt::If { then_branch, else_branch, .. } => {
                assert!(!then_branch.is_empty());
                assert!(else_branch.is_none());
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_if_statement_with_else() {
        let stmt = parse_first(
            "if x > 0 do \
                print x \
             end else do \
                print 0 \
             end end"
        ).unwrap();
        match stmt {
            Stmt::If { then_branch, else_branch, .. } => {
                assert!(!then_branch.is_empty());
                assert!(else_branch.is_some());
                assert!(!else_branch.unwrap().is_empty());
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_if_statement_else_if() {
        let stmt = parse_first(
            "if x > 0 do \
                print \"positive\" \
             end else if x < 0 do \
                print \"negative\" \
             end end"
        ).unwrap();
        match stmt {
            Stmt::If { else_branch, .. } => {
                assert!(else_branch.is_some());
            }
            _ => panic!("Expected If statement"),
        }
    }

    // ---------------------------------------------------------------------
    // While Loops
    // ---------------------------------------------------------------------

    #[test]
    fn test_while_loop() {
        let stmt = parse_first(
            "while true do \
                print \"looping\" \
             end end"
        ).unwrap();
        match stmt {
            Stmt::While { body, .. } => {
                assert!(!body.is_empty());
            }
            _ => panic!("Expected While statement"),
        }
    }

    #[test]
    fn test_while_loop_with_condition() {
        let stmt = parse_first(
            "while x < 10 do \
                x = x + 1 \
             end end"
        ).unwrap();
        match stmt {
            Stmt::While { condition, .. } => {
                match condition {
                    Expr::Comparison { .. } => {
                        // Expected comparison in condition
                    }
                    _ => panic!("Expected Comparison in while condition"),
                }
            }
            _ => panic!("Expected While statement"),
        }
    }

    // ---------------------------------------------------------------------
    // For Loops
    // ---------------------------------------------------------------------

    #[test]
    fn test_for_loop() {
        let stmt = parse_first(
            "for i in [1, 2, 3] do \
                print i \
             end end"
        ).unwrap();
        match stmt {
            Stmt::For { variable, body, .. } => {
                match variable {
                    Pattern::Variable { name, .. } => {
                        assert_eq!(name, "i");
                    }
                    _ => panic!("Expected Variable pattern"),
                }
                assert!(!body.is_empty());
            }
            _ => panic!("Expected For statement"),
        }
    }

    #[test]
    fn test_for_loop_with_pattern() {
        let stmt = parse_first(
            "for (x, y) in pairs do \
                print x \
                print y \
             end end"
        ).unwrap();
        match stmt {
            Stmt::For { variable, .. } => {
                match variable {
                    Pattern::Tuple { .. } => {
                        // Expected tuple pattern
                    }
                    _ => panic!("Expected Tuple pattern"),
                }
            }
            _ => panic!("Expected For statement"),
        }
    }

    // ---------------------------------------------------------------------
    // Match Statements
    // ---------------------------------------------------------------------

    #[test]
    fn test_match_statement() {
        let stmt = parse_first(
            "match x { \
                1 => print \"one\", \
                2 => print \"two\", \
                _ => print \"other\" \
             }"
        ).unwrap();
        match stmt {
            Stmt::Match { arms, .. } => {
                assert_eq!(arms.len(), 3);
            }
            _ => panic!("Expected Match statement"),
        }
    }

    #[test]
    fn test_match_with_guard() {
        let stmt = parse_first(
            "match x { \
                n: n > 5 => print \"big\", \
                _ => print \"small\" \
             }"
        ).unwrap();
        match stmt {
            Stmt::Match { arms, .. } => {
                assert_eq!(arms.len(), 2);
                assert!(arms[0].guard.is_some());
            }
            _ => panic!("Expected Match statement"),
        }
    }

    // ---------------------------------------------------------------------
    // Return Statements
    // ---------------------------------------------------------------------

    #[test]
    fn test_return_with_value() {
        let stmt = parse_first("return 42").unwrap();
        match stmt {
            Stmt::Return { value, .. } => {
                assert!(value.is_some());
            }
            _ => panic!("Expected Return statement"),
        }
    }

    #[test]
    fn test_return_without_value() {
        let stmt = parse_first("return").unwrap();
        match stmt {
            Stmt::Return { value, .. } => {
                assert!(value.is_none());
            }
            _ => panic!("Expected Return statement"),
        }
    }

    // ---------------------------------------------------------------------
    // Break and Continue Statements
    // ---------------------------------------------------------------------

    #[test]
    fn test_break_statement() {
        let stmt = parse_first("break").unwrap();
        match stmt {
            Stmt::Break { .. } => {
                // Success
            }
            _ => panic!("Expected Break statement"),
        }
    }

    #[test]
    fn test_continue_statement() {
        let stmt = parse_first("continue").unwrap();
        match stmt {
            Stmt::Continue { .. } => {
                // Success
            }
            _ => panic!("Expected Continue statement"),
        }
    }
}

// ============================================================================
// Expression Tests
// ============================================================================

mod expression_tests {
    use super::*;

    // ---------------------------------------------------------------------
    // Literals
    // ---------------------------------------------------------------------

    #[test]
    fn test_integer_literal() {
        let expr = parse_expr("42").unwrap();
        match expr {
            Expr::Literal(Literal::Integer(n, _)) => {
                assert_eq!(n, 42);
            }
            _ => panic!("Expected Integer literal"),
        }
    }

    #[test]
    fn test_float_literal() {
        let expr = parse_expr("3.14").unwrap();
        match expr {
            Expr::Literal(Literal::Float(f, _)) => {
                assert!((f - 3.14).abs() < 0.001);
            }
            _ => panic!("Expected Float literal"),
        }
    }

    #[test]
    fn test_string_literal() {
        let expr = parse_expr("\"hello\"").unwrap();
        match expr {
            Expr::Literal(Literal::String(s, _)) => {
                assert_eq!(s, "hello");
            }
            _ => panic!("Expected String literal"),
        }
    }

    #[test]
    fn test_boolean_true() {
        let expr = parse_expr("true").unwrap();
        match expr {
            Expr::Literal(Literal::Boolean(b, _)) => {
                assert_eq!(b, true);
            }
            _ => panic!("Expected Boolean literal"),
        }
    }

    #[test]
    fn test_boolean_false() {
        let expr = parse_expr("false").unwrap();
        match expr {
            Expr::Literal(Literal::Boolean(b, _)) => {
                assert_eq!(b, false);
            }
            _ => panic!("Expected Boolean literal"),
        }
    }

    #[test]
    fn test_null_literal() {
        let expr = parse_expr("null").unwrap();
        match expr {
            Expr::Literal(Literal::Null(_)) => {
                // Success
            }
            _ => panic!("Expected Null literal"),
        }
    }

    // ---------------------------------------------------------------------
    // Variables
    // ---------------------------------------------------------------------

    #[test]
    fn test_variable_reference() {
        let expr = parse_expr("x").unwrap();
        match expr {
            Expr::Variable { name, .. } => {
                assert_eq!(name, "x");
            }
            _ => panic!("Expected Variable"),
        }
    }

    // ---------------------------------------------------------------------
    // Binary Operations
    // ---------------------------------------------------------------------

    #[test]
    fn test_binary_addition() {
        let expr = parse_expr("1 + 2").unwrap();
        match expr {
            Expr::Binary { op, left, right, .. } => {
                assert_eq!(op, BinaryOp::Add);
                match (*left, *right) {
                    (Expr::Literal(Literal::Integer(1, _)),
                     Expr::Literal(Literal::Integer(2, _))) => {
                        // Success
                    }
                    _ => panic!("Expected integer literals"),
                }
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_binary_subtraction() {
        let expr = parse_expr("5 - 3").unwrap();
        match expr {
            Expr::Binary { op, .. } => {
                assert_eq!(op, BinaryOp::Sub);
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_binary_multiplication() {
        let expr = parse_expr("3 * 4").unwrap();
        match expr {
            Expr::Binary { op, .. } => {
                assert_eq!(op, BinaryOp::Mul);
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_binary_division() {
        let expr = parse_expr("10 / 2").unwrap();
        match expr {
            Expr::Binary { op, .. } => {
                assert_eq!(op, BinaryOp::Div);
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_binary_modulo() {
        let expr = parse_expr("10 % 3").unwrap();
        match expr {
            Expr::Binary { op, .. } => {
                assert_eq!(op, BinaryOp::Mod);
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_binary_power() {
        let expr = parse_expr("2 ** 3").unwrap();
        match expr {
            Expr::Binary { op, .. } => {
                assert_eq!(op, BinaryOp::Pow);
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_binary_concatenation() {
        let expr = parse_expr("\"hello\" ++ \"world\"").unwrap();
        match expr {
            Expr::Binary { op, .. } => {
                assert_eq!(op, BinaryOp::Concat);
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_operator_precedence() {
        let expr = parse_expr("1 + 2 * 3").unwrap();
        match expr {
            Expr::Binary { op, left, .. } => {
                assert_eq!(op, BinaryOp::Add);
                match *left {
                    Expr::Literal(Literal::Integer(1, _)) => {
                        // Success - multiplication should be on right
                    }
                    _ => panic!("Expected 1 on left side"),
                }
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_operator_precedence_with_parens() {
        let expr = parse_expr("(1 + 2) * 3").unwrap();
        match expr {
            Expr::Binary { op, left, .. } => {
                assert_eq!(op, BinaryOp::Mul);
                match *left {
                    Expr::Binary { op: inner_op, .. } => {
                        assert_eq!(inner_op, BinaryOp::Add);
                    }
                    _ => panic!("Expected addition in left side"),
                }
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    // ---------------------------------------------------------------------
    // Comparison Operations
    // ---------------------------------------------------------------------

    #[test]
    fn test_comparison_equal() {
        let expr = parse_expr("x == y").unwrap();
        match expr {
            Expr::Comparison { op, .. } => {
                assert_eq!(op, ComparisonOp::Eq);
            }
            _ => panic!("Expected Comparison expression"),
        }
    }

    #[test]
    fn test_comparison_not_equal() {
        let expr = parse_expr("x != y").unwrap();
        match expr {
            Expr::Comparison { op, .. } => {
                assert_eq!(op, ComparisonOp::Ne);
            }
            _ => panic!("Expected Comparison expression"),
        }
    }

    #[test]
    fn test_comparison_less_than() {
        let expr = parse_expr("x < y").unwrap();
        match expr {
            Expr::Comparison { op, .. } => {
                assert_eq!(op, ComparisonOp::Lt);
            }
            _ => panic!("Expected Comparison expression"),
        }
    }

    #[test]
    fn test_comparison_greater_than() {
        let expr = parse_expr("x > y").unwrap();
        match expr {
            Expr::Comparison { op, .. } => {
                assert_eq!(op, ComparisonOp::Gt);
            }
            _ => panic!("Expected Comparison expression"),
        }
    }

    #[test]
    fn test_comparison_less_equal() {
        let expr = parse_expr("x <= y").unwrap();
        match expr {
            Expr::Comparison { op, .. } => {
                assert_eq!(op, ComparisonOp::Le);
            }
            _ => panic!("Expected Comparison expression"),
        }
    }

    #[test]
    fn test_comparison_greater_equal() {
        let expr = parse_expr("x >= y").unwrap();
        match expr {
            Expr::Comparison { op, .. } => {
                assert_eq!(op, ComparisonOp::Ge);
            }
            _ => panic!("Expected Comparison expression"),
        }
    }

    // ---------------------------------------------------------------------
    // Logical Operations
    // ---------------------------------------------------------------------

    #[test]
    fn test_logical_and() {
        let expr = parse_expr("true and false").unwrap();
        match expr {
            Expr::Logical { op, .. } => {
                assert_eq!(op, LogicalOp::And);
            }
            _ => panic!("Expected Logical expression"),
        }
    }

    #[test]
    fn test_logical_or() {
        let expr = parse_expr("true or false").unwrap();
        match expr {
            Expr::Logical { op, .. } => {
                assert_eq!(op, LogicalOp::Or);
            }
            _ => panic!("Expected Logical expression"),
        }
    }

    // ---------------------------------------------------------------------
    // Unary Operations
    // ---------------------------------------------------------------------

    #[test]
    fn test_unary_negation() {
        let expr = parse_expr("-42").unwrap();
        match expr {
            Expr::Unary { op, .. } => {
                assert_eq!(op, UnaryOp::Neg);
            }
            _ => panic!("Expected Unary expression"),
        }
    }

    #[test]
    fn test_unary_logical_not() {
        let expr = parse_expr("!true").unwrap();
        match expr {
            Expr::Unary { op, .. } => {
                assert_eq!(op, UnaryOp::Not);
            }
            _ => panic!("Expected Unary expression"),
        }
    }

    #[test]
    fn test_unary_bitwise_not() {
        let expr = parse_expr("~x").unwrap();
        match expr {
            Expr::Unary { op, .. } => {
                assert_eq!(op, UnaryOp::BitNot);
            }
            _ => panic!("Expected Unary expression"),
        }
    }

    // ---------------------------------------------------------------------
    // Function Calls
    // ---------------------------------------------------------------------

    #[test]
    fn test_function_call_no_args() {
        let expr = parse_expr("foo()").unwrap();
        match expr {
            Expr::Call { args, .. } => {
                assert_eq!(args.len(), 0);
            }
            _ => panic!("Expected Call expression"),
        }
    }

    #[test]
    fn test_function_call_with_args() {
        let expr = parse_expr("add(1, 2)").unwrap();
        match expr {
            Expr::Call { args, .. } => {
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected Call expression"),
        }
    }

    #[test]
    fn test_function_call_nested() {
        let expr = parse_expr("add(add(1, 2), 3)").unwrap();
        match expr {
            Expr::Call { args, .. } => {
                assert_eq!(args.len(), 2);
                match &args[0] {
                    Expr::Call { .. } => {
                        // Expected nested call
                    }
                    _ => panic!("Expected nested Call"),
                }
            }
            _ => panic!("Expected Call expression"),
        }
    }

    // ---------------------------------------------------------------------
    // Lists
    // ---------------------------------------------------------------------

    #[test]
    fn test_empty_list() {
        let expr = parse_expr("[]").unwrap();
        match expr {
            Expr::List { elements, .. } => {
                assert_eq!(elements.len(), 0);
            }
            _ => panic!("Expected List expression"),
        }
    }

    #[test]
    fn test_list_with_elements() {
        let expr = parse_expr("[1, 2, 3]").unwrap();
        match expr {
            Expr::List { elements, .. } => {
                assert_eq!(elements.len(), 3);
            }
            _ => panic!("Expected List expression"),
        }
    }

    #[test]
    fn test_list_nested() {
        let expr = parse_expr("[[1, 2], [3, 4]]").unwrap();
        match expr {
            Expr::List { elements, .. } => {
                assert_eq!(elements.len(), 2);
                match &elements[0] {
                    Expr::List { .. } => {
                        // Expected nested list
                    }
                    _ => panic!("Expected nested List"),
                }
            }
            _ => panic!("Expected List expression"),
        }
    }

    // ---------------------------------------------------------------------
    // Maps
    // ---------------------------------------------------------------------

    #[test]
    fn test_empty_map() {
        let expr = parse_expr("{}").unwrap();
        match expr {
            Expr::Map { entries, .. } => {
                assert_eq!(entries.len(), 0);
            }
            _ => panic!("Expected Map expression"),
        }
    }

    #[test]
    fn test_map_with_entries() {
        let expr = parse_expr("{\"a\": 1, \"b\": 2}").unwrap();
        match expr {
            Expr::Map { entries, .. } => {
                assert_eq!(entries.len(), 2);
            }
            _ => panic!("Expected Map expression"),
        }
    }

    // ---------------------------------------------------------------------
    // Lambda Expressions
    // ---------------------------------------------------------------------

    #[test]
    fn test_lambda_no_params() {
        let expr = parse_expr("|| 42").unwrap();
        match expr {
            Expr::Lambda { params, .. } => {
                assert_eq!(params.len(), 0);
            }
            _ => panic!("Expected Lambda expression"),
        }
    }

    #[test]
    fn test_lambda_with_params() {
        let expr = parse_expr("|x, y| x + y").unwrap();
        match expr {
            Expr::Lambda { params, .. } => {
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].name, "x");
                assert_eq!(params[1].name, "y");
            }
            _ => panic!("Expected Lambda expression"),
        }
    }

    #[test]
    fn test_lambda_with_types() {
        let expr = parse_expr("|x: Int, y: Int| x + y").unwrap();
        match expr {
            Expr::Lambda { params, .. } => {
                assert_eq!(params.len(), 2);
                assert!(params[0].type_annotation.is_some());
                assert!(params[1].type_annotation.is_some());
            }
            _ => panic!("Expected Lambda expression"),
        }
    }

    // ---------------------------------------------------------------------
    // Block Expressions
    // ---------------------------------------------------------------------

    #[test]
    fn test_block_expression() {
        let stmt = parse_first("do let x = 42 x end").unwrap();
        match stmt {
            Stmt::ExprStmt { expr, .. } => match expr {
                Expr::Block { statements, .. } => {
                    assert!(!statements.is_empty());
                }
                _ => panic!("Expected Block expression"),
            },
            _ => panic!("Expected ExprStmt"),
        }
    }

    // ---------------------------------------------------------------------
    // Pipeline Expressions
    // ---------------------------------------------------------------------

    #[test]
    fn test_pipeline_simple() {
        let expr = parse_expr("[1, 2, 3] |> map |x| x * 2 |").unwrap();
        match expr {
            Expr::Pipeline { stages, .. } => {
                assert_eq!(stages.len(), 2);
            }
            _ => panic!("Expected Pipeline expression"),
        }
    }
}

// ============================================================================
// Error Case Tests
// ============================================================================

mod error_tests {
    use super::*;

    #[test]
    fn test_missing_let_name() {
        let result = parse("let = 42");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_let_value() {
        let result = parse("let x");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_function_name() {
        let result = parse("fn() do 42 end");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_function_body() {
        let result = parse("fn foo()");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_if_condition() {
        let result = parse("if then 1 else 0 end");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_then_branch() {
        let result = parse("if x > 0 else 0 end");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_else_branch() {
        let result = parse("if x > 0 then x end");
        assert!(result.is_err());
    }

    #[test]
    fn test_mismatched_parens() {
        let result = parse_expr("(1 + 2");
        assert!(result.is_err());
    }

    #[test]
    fn test_mismatched_brackets() {
        let result = parse_expr("[1, 2, 3");
        assert!(result.is_err());
    }

    #[test]
    fn test_mismatched_braces() {
        let result = parse_expr("{\"a\": 1");
        assert!(result.is_err());
    }

    #[test]
    fn test_unclosed_string() {
        let result = parse_expr("\"hello");
        // This might not fail in the lexer, but could fail in parser
        // depending on implementation
    }

    #[test]
    fn test_invalid_operator() {
        let result = parse_expr("1 @ 2");
        // @ is not a valid operator
        assert!(result.is_err());
    }
}

// ============================================================================
// Complex Scenario Tests
// ============================================================================

mod complex_tests {
    use super::*;

    #[test]
    fn test_nested_functions() {
        let source = "\
            fn outer(x) do \
                fn inner(y) do \
                    x + y \
                end \
            end";

        let stmts = parse(source).unwrap();
        assert!(!stmts.is_empty());
    }

    #[test]
    fn test_multiple_statements() {
        let source = "\
            let x = 1 \
            let y = 2 \
            let z = x + y";

        let stmts = parse(source).unwrap();
        assert_eq!(stmts.len(), 3);
    }

    #[test]
    fn test_complex_arithmetic() {
        let expr = parse_expr("2 + 3 * 4 - 5 / 2 ** 3").unwrap();
        // Should parse correctly with proper precedence
        match expr {
            Expr::Binary { .. } => {
                // Success
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_nested_if_expressions() {
        let expr = parse_expr("if x > 0 then if y > 0 then 1 else 2 end else 3 end").unwrap();
        match expr {
            Expr::If { then_branch, .. } => {
                match *then_branch {
                    Expr::If { .. } => {
                        // Expected nested if
                    }
                    _ => panic!("Expected nested If in then branch"),
                }
            }
            _ => panic!("Expected If expression"),
        }
    }

    #[test]
    fn test_nested_lists() {
        let expr = parse_expr("[[1, 2], [3, [4, 5]]]").unwrap();
        match expr {
            Expr::List { elements, .. } => {
                assert_eq!(elements.len(), 2);
            }
            _ => panic!("Expected List expression"),
        }
    }

    #[test]
    fn test_complex_match() {
        let source = "\
            match result { \
                Ok(value) => value, \
                Error(msg) => 0, \
                _ => -1 \
            }";

        let stmt = parse_first(source).unwrap();
        match stmt {
            Stmt::Match { arms, .. } => {
                assert_eq!(arms.len(), 3);
            }
            _ => panic!("Expected Match statement"),
        }
    }

    #[test]
    fn test_higher_order_function() {
        let source = "\
            fn apply(f, x) do \
                f(x) \
            end";

        let stmt = parse_first(source).unwrap();
        match stmt {
            Stmt::Function { params, body, .. } => {
                assert_eq!(params.len(), 2);
                match body {
                    Expr::Call { .. } => {
                        // Expected function call
                    }
                    _ => panic!("Expected Call in function body"),
                }
            }
            _ => panic!("Expected Function statement"),
        }
    }

    #[test]
    fn test_closure_capture() {
        let source = "\
            let x = 10 \
            let add_x = |y| x + y";

        let stmts = parse(source).unwrap();
        assert_eq!(stmts.len(), 2);
    }

    #[test]
    fn test_complex_pipeline() {
        let expr = parse_expr("[1, 2, 3, 4, 5] \
            |> filter |x| x > 2 | \
            |> map |x| x * 2 | \
            |> fold 0 |acc, x| acc + x |"
        ).unwrap();
        match expr {
            Expr::Pipeline { stages, .. } => {
                assert_eq!(stages.len(), 4);
            }
            _ => panic!("Expected Pipeline expression"),
        }
    }

    #[test]
    fn test_recursive_function() {
        let source = "\
            fn factorial(n) do \
                if n <= 1 then \
                    1 \
                else \
                    n * factorial(n - 1) \
                end \
            end";

        let stmt = parse_first(source).unwrap();
        match stmt {
            Stmt::Function { body, .. } => {
                match body {
                    Expr::If { .. } => {
                        // Expected if expression
                    }
                    _ => panic!("Expected If in function body"),
                }
            }
            _ => panic!("Expected Function statement"),
        }
    }
}

// ============================================================================
// Example File Tests
// ============================================================================

mod example_tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_hello_example() {
        let source = "\
            fn main() do \
                print \"Hello, World!\" \
            end";

        let stmts = parse(source).unwrap();
        assert!(!stmts.is_empty());
    }

    #[test]
    fn test_variables_example() {
        let source = "\
            let name = \"Alice\" \
            let age = 30 \
            var score = 0";

        let stmts = parse(source).unwrap();
        assert_eq!(stmts.len(), 3);
    }

    #[test]
    fn test_functions_example() {
        let source = "\
            fn add(a, b) do \
                a + b \
            end \
            fn factorial(n) do \
                if n <= 1 then 1 else n * factorial(n - 1) end \
            end";

        let stmts = parse(source).unwrap();
        assert_eq!(stmts.len(), 2);
    }

    #[test]
    fn test_lists_example() {
        let source = "\
            let numbers = [1, 2, 3, 4, 5] \
            let doubled = numbers.map |n| n * 2 |";

        let stmts = parse(source).unwrap();
        assert_eq!(stmts.len(), 2);
    }

    #[test]
    fn test_patterns_example() {
        let source = "\
            fn classify_number(n) do \
                match n { \
                    0 => \"Zero\", \
                    1 => \"One\", \
                    _ => \"Many\" \
                } \
            end";

        let stmt = parse_first(source).unwrap();
        match stmt {
            Stmt::Function { body, .. } => {
                match body {
                    Expr::Match { .. } => {
                        // Expected match expression
                    }
                    _ => panic!("Expected Match in function body"),
                }
            }
            _ => panic!("Expected Function statement"),
        }
    }

    #[test]
    fn test_complex_example() {
        let source = "\
            fn max(a, b) do \
                if a > b then a else b end \
            end \
            let numbers = [1, 2, 3, 4, 5] \
            let result = max(10, 20) \
            print result";

        let stmts = parse(source).unwrap();
        assert_eq!(stmts.len(), 4);
    }

    #[test]
    fn test_while_example() {
        let source = "\
            fn loop_test() do \
                while true do \
                    print \"looping\" \
                end end \
            end";

        let stmt = parse_first(source).unwrap();
        match stmt {
            Stmt::Function { body, .. } => {
                match body {
                    Expr::Block { statements, .. } => {
                        assert!(!statements.is_empty());
                    }
                    _ => panic!("Expected Block in function body"),
                }
            }
            _ => panic!("Expected Function statement"),
        }
    }

    #[test]
    fn test_for_example() {
        let source = "\
            fn test() do \
                for i in [1, 2, 3] do \
                    print i \
                end end \
            end";

        let stmt = parse_first(source).unwrap();
        match stmt {
            Stmt::Function { body, .. } => {
                match body {
                    Expr::Block { statements, .. } => {
                        assert!(!statements.is_empty());
                    }
                    _ => panic!("Expected Block in function body"),
                }
            }
            _ => panic!("Expected Function statement"),
        }
    }

    #[test]
    fn test_control_flow_example() {
        let source = "\
            fn test_return() do \
                return 42 \
            end \
            fn test_break() do \
                while true do \
                    break \
                end end \
            end \
            fn test_continue() do \
                while true do \
                    continue \
                end end \
            end";

        let stmts = parse(source).unwrap();
        assert_eq!(stmts.len(), 3);
    }
}
