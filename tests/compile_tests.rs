//! End-to-end compilation tests
//!
//! These tests verify that .nm source files compile through the entire pipeline:
//! Lexer -> Parser -> Name Resolution -> Type Checking -> MIR Lowering -> Python Codegen

/// Helper: compile a source string through the full pipeline and return generated Python
fn compile_to_python(source: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Lex
    let mut lexer = nevermind_lexer::Lexer::new(source);
    let tokens = lexer.tokenize()?;

    // Parse
    let mut parser = nevermind_parser::Parser::from_tokens(tokens);
    let stmts = parser.parse()?;

    // Name resolution
    let mut resolver = nevermind_name_resolver::NameResolver::new();
    match resolver.resolve(&stmts) {
        Ok(_) => {}
        Err(errors) => {
            let msg = errors.iter()
                .map(|e| format!("{}: {}", e.span, e.message))
                .collect::<Vec<_>>()
                .join("; ");
            return Err(format!("Name resolution failed: {}", msg).into());
        }
    }

    // Type checking
    let mut checker = nevermind_type_checker::TypeChecker::new();
    checker.check(&stmts)?;

    // MIR lowering
    let mir_program = nevermind_mir::lower_program(&stmts)?;

    // Code generation
    let python_code = nevermind_codegen::generate(&mir_program)?;

    Ok(python_code)
}

#[test]
fn test_hello_world() {
    let source = r#"
fn main() do
  print "Hello, World!"
end
"#;
    let python = compile_to_python(source).expect("compilation failed");
    assert!(python.contains("def main():"));
    assert!(python.contains("print(\"Hello, World!\")"));
    assert!(python.contains("if __name__"));
}

#[test]
fn test_simple_arithmetic() {
    let source = r#"
let x = 10
let y = 20
let z = x + y
print z
"#;
    let python = compile_to_python(source).expect("compilation failed");
    assert!(python.contains("x = 10"));
    assert!(python.contains("y = 20"));
    assert!(python.contains("z = (x + y)"));
    assert!(python.contains("print(z)"));
}

#[test]
fn test_function_definition() {
    let source = r#"
fn add(a, b) do
  a + b
end

let result = add(5, 3)
print result
"#;
    let python = compile_to_python(source).expect("compilation failed");
    assert!(python.contains("def add(a, b):"));
    assert!(python.contains("return (a + b)"));
    assert!(python.contains("result = add(5, 3)"));
}

#[test]
fn test_recursive_factorial() {
    let source = r#"
fn factorial(n: Int) -> Int do
  if n <= 1 then 1 else n * factorial(n - 1) end
end
"#;
    let python = compile_to_python(source).expect("compilation failed");
    assert!(python.contains("def factorial(n):"));
    assert!(python.contains("factorial"));
}

#[test]
fn test_variables_and_mutation() {
    let source = r#"
fn main() do
  let name = "Alice"
  var score = 0
  score = score + 1
  print name
  print score
end
"#;
    let python = compile_to_python(source).expect("compilation failed");
    assert!(python.contains("name = \"Alice\""));
    assert!(python.contains("score = 0"));
    assert!(python.contains("score = (score + 1)"));
}

#[test]
fn test_list_literal() {
    let source = r#"
let numbers = [1, 2, 3, 4, 5]
print numbers
"#;
    let python = compile_to_python(source).expect("compilation failed");
    assert!(python.contains("[1, 2, 3, 4, 5]"));
}

#[test]
fn test_if_expression() {
    let source = r#"
fn larger(a: Int, b: Int) -> Int do
  if a > b then a else b end
end
"#;
    let python = compile_to_python(source).expect("compilation failed");
    assert!(python.contains("def larger(a, b):"));
    // If expression generates ternary
    assert!(python.contains("if"));
}

#[test]
fn test_multiple_functions() {
    let source = r#"
fn add(a: Int, b: Int) -> Int do
  a + b
end

fn mul(a: Int, b: Int) -> Int do
  a * b
end

fn main() do
  print add(2, 3)
  print mul(4, 5)
end
"#;
    let python = compile_to_python(source).expect("compilation failed");
    assert!(python.contains("def add(a, b):"));
    assert!(python.contains("def mul(a, b):"));
    assert!(python.contains("def main():"));
}

#[test]
fn test_comparison_operators() {
    let source = r#"
let a = 10
let b = 20
let eq = a == b
let ne = a != b
let lt = a < b
let gt = a > b
"#;
    let python = compile_to_python(source).expect("compilation failed");
    assert!(python.contains("(a == b)"));
    assert!(python.contains("(a != b)"));
    assert!(python.contains("(a < b)"));
    assert!(python.contains("(a > b)"));
}

#[test]
fn test_boolean_literals() {
    let source = r#"
let yes = true
let no = false
"#;
    let python = compile_to_python(source).expect("compilation failed");
    assert!(python.contains("True"));
    assert!(python.contains("False"));
}

#[test]
fn test_string_operations() {
    let source = r#"
let greeting = "Hello"
let name = "World"
print greeting
print name
"#;
    let python = compile_to_python(source).expect("compilation failed");
    assert!(python.contains("\"Hello\""));
    assert!(python.contains("\"World\""));
}

#[test]
fn test_generated_python_has_header() {
    let source = "let x = 42\n";
    let python = compile_to_python(source).expect("compilation failed");
    assert!(python.starts_with("# Generated by Nevermind compiler"));
}

#[test]
fn test_main_function_gets_entry_point() {
    let source = r#"
fn main() do
  print 42
end
"#;
    let python = compile_to_python(source).expect("compilation failed");
    assert!(python.contains("if __name__ == \"__main__\":"));
    assert!(python.contains("main()"));
}

#[test]
fn test_no_main_no_entry_point() {
    let source = r#"
fn helper(x: Int) -> Int do
  x + 1
end
"#;
    let python = compile_to_python(source).expect("compilation failed");
    assert!(!python.contains("if __name__"));
}

#[test]
fn test_nested_arithmetic() {
    let source = r#"
let result = (1 + 2) * (3 + 4)
"#;
    let python = compile_to_python(source).expect("compilation failed");
    assert!(python.contains("result"));
    assert!(python.contains("+"));
    assert!(python.contains("*"));
}

#[test]
fn test_array_indexing() {
    let source = r#"
let arr = [10, 20, 30]
let first = arr[0]
"#;
    let python = compile_to_python(source).expect("compilation failed");
    assert!(python.contains("[10, 20, 30]"));
    assert!(python.contains("arr[0]"));
}

#[test]
fn test_print_in_function_no_return() {
    // print calls in functions should NOT generate 'return print(...)'
    // they should just be standalone statements
    let source = r#"
fn greet() do
  print "hi"
  print "bye"
end
"#;
    let python = compile_to_python(source).expect("compilation failed");
    assert!(python.contains("def greet():"));
    assert!(python.contains("print(\"hi\")"));
    assert!(python.contains("print(\"bye\")"));
}
