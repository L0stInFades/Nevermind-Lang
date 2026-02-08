//! Nevermind CLI - Command-line interface for the Nevermind language

use std::path::PathBuf;
use std::fs;
use std::io::{self, BufRead, Write};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "nevermind")]
#[command(about = "The Nevermind Programming Language", long_about = None)]
#[command(version = "0.4.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a Nevermind file
    Compile {
        /// Input file
        input: PathBuf,

        /// Output file (default: input with .py extension)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Parse only (don't compile)
        #[arg(long)]
        parse_only: bool,
    },

    /// Run a Nevermind file
    Run {
        /// Input file
        input: PathBuf,

        /// Arguments to pass to the program
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Start the REPL
    Repl,

    /// Check a file for errors (without compiling)
    Check {
        /// Input file
        input: PathBuf,
    },

    /// Format a Nevermind file
    Fmt {
        /// Input file(s)
        inputs: Vec<PathBuf>,

        /// Write to file instead of stdout
        #[arg(short, long)]
        write: bool,

        /// Check if files are formatted without modifying them
        #[arg(long)]
        check: bool,
    },

    /// Lint a Nevermind file
    Lint {
        /// Input file(s)
        inputs: Vec<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Compile { input, output, parse_only } => {
            compile(input, output, parse_only)
        }
        Commands::Run { input, args } => run(input, args),
        Commands::Repl => repl(),
        Commands::Check { input } => check(input),
        Commands::Fmt { inputs, write, check } => fmt(inputs, write, check),
        Commands::Lint { inputs } => lint(inputs),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Compile a Nevermind file
fn compile(
    input: PathBuf,
    output: Option<PathBuf>,
    parse_only: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Compiling: {:?}", input);

    // Read the source file
    let source = fs::read_to_string(&input)?;

    // Lex the source
    let mut lexer = nevermind_lexer::Lexer::new(&source);
    let tokens = lexer.tokenize()?;

    println!("  ✓ Lexical analysis passed ({} tokens)", tokens.len());

    // Parse the AST
    let mut parser = nevermind_parser::Parser::from_tokens(tokens);
    let statements = parser.parse()?;

    println!("  ✓ Syntax analysis passed ({} statements)", statements.len());

    if parse_only {
        // Just show AST
        for (i, stmt) in statements.iter().enumerate() {
            println!("    [{}] {:?}", i, stmt);
        }
        return Ok(());
    }

    // Name resolution
    let mut resolver = nevermind_name_resolver::NameResolver::new();
    let name_scope = match resolver.resolve(&statements) {
        Ok(scope) => scope,
        Err(errors) => {
            eprintln!("  Name resolution errors: {}", errors.len());
            for error in &errors {
                eprintln!("    - {}: {}", error.span, error.message);
            }
            return Err(format!("Name resolution failed with {} errors", errors.len()).into());
        }
    };

    let _ = name_scope; // Suppress unused warning

    println!("  ✓ Name resolution passed");

    // Type checking
    let mut checker = nevermind_type_checker::TypeChecker::new();
    checker.check(&statements)?;

    println!("  ✓ Type checking passed");

    // Lower to MIR
    let mir_program = nevermind_mir::lower_program(&statements)?;

    println!("  ✓ MIR lowering passed");

    // Code generation
    let python_code = nevermind_codegen::generate(&mir_program)?;

    println!("  ✓ Code generation passed");

    // Determine output file
    let output = output.unwrap_or_else(|| {
        let mut out = input.clone();
        out.set_extension("py");
        out
    });

    // Write output
    fs::write(&output, python_code)?;

    println!("  ✓ Output written to: {:?}", output);

    Ok(())
}

/// Run a Nevermind file
fn run(input: PathBuf, args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running: {:?}", input);
    println!("Args: {:?}", args);

    // Compile to Python
    let py_output = {
        let mut out = input.clone();
        out.set_extension("py");
        out
    };

    compile(input.clone(), Some(py_output.clone()), false)?;

    // Run with Python
    println!("\nExecuting with Python...");

    // Try python interpreters in order: python3, python, py (Windows launcher)
    let python_cmds = if cfg!(windows) {
        vec!["python", "python3", "py"]
    } else {
        vec!["python3", "python"]
    };

    let mut last_err = None;
    let mut status = None;
    for python_cmd in &python_cmds {
        let result = std::process::Command::new(python_cmd)
            .arg(&py_output)
            .args(&args)
            .spawn();
        match result {
            Ok(mut child) => {
                status = Some(child.wait()?);
                last_err = None;
                break;
            }
            Err(e) => {
                last_err = Some(e);
                continue;
            }
        }
    }

    if let Some(e) = last_err {
        return Err(format!("Could not find Python interpreter. Tried: {}. Error: {}",
            python_cmds.join(", "), e).into());
    }

    let status = status.unwrap();

    if !status.success() {
        return Err(format!("Python execution failed with status: {}", status).into());
    }

    Ok(())
}

/// Start the REPL
fn repl() -> Result<(), Box<dyn std::error::Error>> {
    println!("Nevermind REPL v0.4.0");
    println!("Type :help for help, exit or Ctrl-D to quit\n");

    let mut definitions: Vec<String> = Vec::new();
    let mut input_buffer = String::new();

    let stdin = io::stdin();

    loop {
        // Show prompt
        let prompt = if input_buffer.is_empty() { ">>> " } else { "... " };
        print!("{}", prompt);
        io::stdout().flush()?;

        // Read line
        let mut line = String::new();
        let bytes_read = stdin.lock().read_line(&mut line)?;

        if bytes_read == 0 {
            // EOF
            println!("\nGoodbye!");
            break;
        }

        let trimmed = line.trim();

        // Handle exit at top level
        if input_buffer.is_empty() && (trimmed == "exit" || trimmed == "quit") {
            println!("Goodbye!");
            break;
        }

        // Handle REPL commands at top level
        if input_buffer.is_empty() && trimmed.starts_with(':') {
            match trimmed {
                ":help" => {
                    println!("Commands:");
                    println!("  :help   Show this help message");
                    println!("  :clear  Clear all definitions");
                    println!("  :defs   Show current definitions");
                    println!("  exit    Exit the REPL");
                }
                ":clear" => {
                    definitions.clear();
                    println!("Definitions cleared.");
                }
                ":defs" => {
                    if definitions.is_empty() {
                        println!("No definitions.");
                    } else {
                        for def in &definitions {
                            println!("{}", def);
                        }
                    }
                }
                _ => {
                    eprintln!("Unknown command: {}. Type :help for help.", trimmed);
                }
            }
            continue;
        }

        // Skip empty lines at top level
        if input_buffer.is_empty() && trimmed.is_empty() {
            continue;
        }

        // Append line to buffer
        if !input_buffer.is_empty() {
            input_buffer.push('\n');
        }
        input_buffer.push_str(&line.trim_end_matches('\n').trim_end_matches('\r'));

        // Check if we need more input
        if needs_more_input(&input_buffer) {
            continue;
        }

        // Extract complete input and clear buffer
        let complete_input = input_buffer.trim().to_string();
        input_buffer.clear();

        if complete_input.is_empty() {
            continue;
        }

        // Classify: definition or expression
        let is_def = complete_input.starts_with("fn ")
            || complete_input.starts_with("let ")
            || complete_input.starts_with("var ");

        if is_def {
            // Validate by compiling all defs + new def
            let mut source = String::new();
            for def in &definitions {
                source.push_str(def);
                source.push('\n');
            }
            source.push_str(&complete_input);
            source.push('\n');
            // Add a dummy main so codegen succeeds
            source.push_str("fn main() do 0 end\n");

            match compile_source_silent(&source) {
                Ok(_) => {
                    definitions.push(complete_input);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        } else {
            // Expression: compile all defs + expression, execute
            let mut source = String::new();
            for def in &definitions {
                source.push_str(def);
                source.push('\n');
            }
            source.push_str(&complete_input);
            source.push('\n');

            match compile_source_silent(&source) {
                Ok(python_code) => {
                    let code = strip_main_guard(&python_code);
                    match execute_python_code(&code) {
                        Ok(output) => {
                            let output = output.trim_end();
                            if !output.is_empty() {
                                println!("{}", output);
                            }
                        }
                        Err(e) => {
                            eprintln!("Runtime error: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }
    }

    Ok(())
}

/// Compile source code silently, returning the generated Python code.
fn compile_source_silent(source: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Lex
    let mut lexer = nevermind_lexer::Lexer::new(source);
    let tokens = lexer.tokenize()?;

    // Parse
    let mut parser = nevermind_parser::Parser::from_tokens(tokens);
    let statements = parser.parse()?;

    // Name resolution
    let mut resolver = nevermind_name_resolver::NameResolver::new();
    if let Err(errors) = resolver.resolve(&statements) {
        let msgs: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
        return Err(msgs.join("; ").into());
    }

    // Type checking
    let mut checker = nevermind_type_checker::TypeChecker::new();
    checker.check(&statements)?;

    // Lower to MIR
    let mir_program = nevermind_mir::lower_program(&statements)?;

    // Code generation
    let python_code = nevermind_codegen::generate(&mir_program)?;

    Ok(python_code)
}

/// Execute Python code and return its stdout output.
fn execute_python_code(code: &str) -> Result<String, Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("nevermind_repl.py");
    fs::write(&temp_file, code)?;

    let python_cmds = if cfg!(windows) {
        vec!["python", "python3", "py"]
    } else {
        vec!["python3", "python"]
    };

    let mut last_err = None;
    for python_cmd in &python_cmds {
        let result = std::process::Command::new(python_cmd)
            .arg(&temp_file)
            .output();
        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                if !output.status.success() {
                    return Err(stderr.into());
                }
                return Ok(stdout);
            }
            Err(e) => {
                last_err = Some(e);
                continue;
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

/// Check if the input buffer needs more lines (multi-line input).
fn needs_more_input(input: &str) -> bool {
    let mut depth: i32 = 0;

    for line in input.lines() {
        let trimmed = line.trim();
        // Count block openers
        if trimmed.contains(" do") || trimmed.starts_with("do") {
            depth += 1;
        }
        if trimmed.starts_with("match ") || trimmed == "match" {
            depth += 1;
        }
        // Count block closers
        if trimmed == "end" || trimmed.ends_with(" end") {
            depth -= 1;
        }
    }

    if depth > 0 {
        return true;
    }

    // Check for fn signature without body (no "do" on same line)
    let first_line = input.lines().next().unwrap_or("").trim();
    if first_line.starts_with("fn ") && !first_line.contains(" do") && !first_line.contains(" end") {
        // Single-line fn without do/end needs more input
        let line_count = input.lines().count();
        if line_count == 1 {
            return true;
        }
    }

    false
}

/// Strip the `if __name__ == "__main__": main()` guard from generated Python.
fn strip_main_guard(python: &str) -> String {
    let mut lines: Vec<&str> = Vec::new();
    let mut skip_next = false;

    for line in python.lines() {
        if skip_next {
            // Skip indented line after __name__ guard (e.g., "    main()")
            if line.starts_with(' ') || line.starts_with('\t') {
                skip_next = false;
                continue;
            }
            skip_next = false;
        }
        if line.contains("if __name__") && line.contains("__main__") {
            skip_next = true;
            continue;
        }
        lines.push(line);
    }

    lines.join("\n")
}

/// Check a file for errors
fn check(input: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("Checking: {:?}", input);

    let source = fs::read_to_string(&input)?;

    // Lex
    let mut lexer = nevermind_lexer::Lexer::new(&source);
    let tokens = lexer.tokenize()?;

    println!("  ✓ Lexical analysis passed");

    // Parse
    let mut parser = nevermind_parser::Parser::from_tokens(tokens);
    let statements = parser.parse()?;

    println!("  ✓ Syntax analysis passed");
    println!("  ✓ Parsed {} statements", statements.len());

    // Name resolution
    let mut resolver = nevermind_name_resolver::NameResolver::new();
    let name_scope = match resolver.resolve(&statements) {
        Ok(scope) => scope,
        Err(errors) => {
            eprintln!("  Name resolution errors: {}", errors.len());
            for error in &errors {
                eprintln!("    - {}: {}", error.span, error.message);
            }
            return Err(format!("Name resolution failed with {} errors", errors.len()).into());
        }
    };

    let _ = name_scope; // Suppress unused warning

    println!("  ✓ Name resolution passed");

    // Type checking
    let mut checker = nevermind_type_checker::TypeChecker::new();
    checker.check(&statements)?;

    println!("  ✓ Type checking passed");

    println!("\n  No errors found!");

    Ok(())
}

/// Format Nevermind files
fn fmt(inputs: Vec<PathBuf>, write: bool, check: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("Formatting: {:?}", inputs);

    for input in inputs {
        let _source = fs::read_to_string(&input)?;

        // TODO: Implement formatting
        println!("  Formatting: {:?}", input);
        println!("  ⚠ Formatter not yet implemented");

        if write {
            // Write formatted code back
        }

        if check {
            // Check if formatting is needed
            println!("  ✓ {:?}", input);
        }
    }

    Ok(())
}

/// Lint Nevermind files
fn lint(inputs: Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Linting: {:?}", inputs);

    for input in inputs {
        println!("  Linting: {:?}", input);
        println!("  ⚠ Linter not yet implemented");
    }

    Ok(())
}