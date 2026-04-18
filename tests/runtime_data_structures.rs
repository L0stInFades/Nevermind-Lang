//! Runtime integration tests for non-trivial data structure encodings.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Helper: compile a source string through the full pipeline and return generated Python.
fn compile_to_python(source: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut lexer = nevermind_lexer::Lexer::new(source);
    let tokens = lexer.tokenize()?;

    let mut parser = nevermind_parser::Parser::from_tokens(tokens);
    let stmts = parser.parse()?;

    let mut resolver = nevermind_name_resolver::NameResolver::new();
    match resolver.resolve(&stmts) {
        Ok(_) => {}
        Err(errors) => {
            let msg = errors
                .iter()
                .map(|e| format!("{}: {}", e.span, e.message))
                .collect::<Vec<_>>()
                .join("; ");
            return Err(format!("Name resolution failed: {}", msg).into());
        }
    }

    let mut checker = nevermind_type_checker::TypeChecker::new();
    checker.check(&stmts)?;

    let mir_program = nevermind_mir::lower_program(&stmts)?;
    let python_code = nevermind_codegen::generate(&mir_program)?;

    Ok(python_code)
}

/// Helper: execute generated Python and capture stdout.
fn run_python(source: &str) -> Result<String, Box<dyn std::error::Error>> {
    let python_code = compile_to_python(source)?;
    let unique = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let path = std::env::temp_dir().join(format!(
        "nevermind_runtime_data_structures_{}_{}.py",
        std::process::id(),
        unique
    ));

    fs::write(&path, python_code)?;

    let result = run_python_file(&path);
    let _ = fs::remove_file(&path);
    result
}

fn run_python_file(path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let python_cmds = if cfg!(windows) {
        vec!["python", "python3", "py"]
    } else {
        vec!["python3", "python"]
    };

    let mut last_err = None;

    for python_cmd in &python_cmds {
        match Command::new(python_cmd).arg(path).output() {
            Ok(output) => {
                if output.status.success() {
                    return Ok(String::from_utf8_lossy(&output.stdout).to_string());
                }

                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                return Err(format!("Python execution failed: {}", stderr).into());
            }
            Err(err) => {
                last_err = Some(err);
            }
        }
    }

    Err(format!(
        "Could not find Python interpreter. Tried: {}. Error: {}",
        python_cmds.join(", "),
        last_err.unwrap()
    )
    .into())
}

#[test]
fn tree_and_forest_can_be_implemented_with_flat_list_encoding() {
    let source = r#"
fn tree_size_at(data: List[Int], idx: Int) -> List[Int] do
  let child_count = data[idx + 1]
  var size = 1
  var next = idx + 2
  var i = 0
  while i < child_count do
    let child = tree_size_at(data, next)
    size = size + child[0]
    next = child[1]
    i = i + 1
  end
  [size, next]
end

fn tree_height_at(data: List[Int], idx: Int) -> List[Int] do
  let child_count = data[idx + 1]
  var best_child_height = 0
  var next = idx + 2
  var i = 0
  while i < child_count do
    let child = tree_height_at(data, next)
    if child[0] > best_child_height do
      best_child_height = child[0]
    end
    next = child[1]
    i = i + 1
  end
  [best_child_height + 1, next]
end

fn tree_size(data: List[Int]) -> Int do
  let result = tree_size_at(data, 0)
  result[0]
end

fn tree_height(data: List[Int]) -> Int do
  let result = tree_height_at(data, 0)
  result[0]
end

fn forest_root_count(data: List[Int]) -> Int do
  var roots = 0
  var next = 0
  while next < len(data) do
    let tree = tree_size_at(data, next)
    roots = roots + 1
    next = tree[1]
  end
  roots
end

fn forest_size(data: List[Int]) -> Int do
  var total = 0
  var next = 0
  while next < len(data) do
    let tree = tree_size_at(data, next)
    total = total + tree[0]
    next = tree[1]
  end
  total
end

fn forest_height(data: List[Int]) -> Int do
  var best = 0
  var next = 0
  while next < len(data) do
    let tree = tree_height_at(data, next)
    if tree[0] > best do
      best = tree[0]
    end
    next = tree[1]
  end
  best
end

fn main() do
  let tree = [1, 2, 2, 0, 3, 1, 4, 0]
  let forest = [1, 2, 2, 0, 3, 1, 4, 0, 5, 2, 6, 0, 7, 0]
  print "TREE_SIZE=" + str(tree_size(tree))
  print "TREE_HEIGHT=" + str(tree_height(tree))
  print "FOREST_ROOTS=" + str(forest_root_count(forest))
  print "FOREST_SIZE=" + str(forest_size(forest))
  print "FOREST_HEIGHT=" + str(forest_height(forest))
end
"#;

    let output = run_python(source).expect("tree/forest program should compile and run");
    let lines: Vec<&str> = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    assert_eq!(
        lines,
        vec![
            "TREE_SIZE=4",
            "TREE_HEIGHT=3",
            "FOREST_ROOTS=2",
            "FOREST_SIZE=7",
            "FOREST_HEIGHT=3",
        ]
    );
}
