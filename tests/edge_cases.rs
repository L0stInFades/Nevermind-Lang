use nevermind_name_resolver::NameResolver;
use nevermind_parser::Parser;
use nevermind_type_checker::{Type, TypeChecker, TypeError, TypeErrorKind};

fn parse_and_resolve(source: &str) -> Vec<nevermind_ast::Stmt> {
    let mut parser = Parser::new(source).expect("failed to create parser");
    let stmts = parser.parse().expect("failed to parse source");

    let mut resolver = NameResolver::new();
    if let Err(errors) = resolver.resolve(&stmts) {
        panic!("name resolution failed: {:?}", errors);
    }

    stmts
}

fn type_check_program(source: &str) -> Result<Type, TypeError> {
    let stmts = parse_and_resolve(source);
    let mut checker = TypeChecker::new();
    checker.check(&stmts)
}

#[test]
fn mixed_list_rejected_by_type_checker() {
    let result = type_check_program("let mixed = [1, \"two\"]");
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(err.kind, TypeErrorKind::TypeMismatch { .. }));
    }
}

#[test]
fn map_requires_string_keys() {
    let result = type_check_program("let mapping = {1: \"value\"}");
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(err.kind, TypeErrorKind::TypeMismatch { .. }));
    }
}

#[test]
fn pipeline_stage_must_be_function() {
    let result = type_check_program("let value = 10 |> 5");
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(err.kind, TypeErrorKind::TypeMismatch { .. }));
    }
}

#[test]
fn pipeline_respects_generic_function_shape() {
    let source = "fn identity(x) do x end\nlet numbers = [1, 2, 3]\nlet piped = numbers |> identity\npiped";
    let ty = type_check_program(source).expect("expected pipeline to type check");
    assert!(!matches!(ty, Type::Unit));
}
