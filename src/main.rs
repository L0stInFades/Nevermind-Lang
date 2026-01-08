//! Nevermind CLI - Command-line interface for the Nevermind language

use std::path::PathBuf;
use std::fs;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "nevermind")]
#[command(about = "The Nevermind Programming Language", long_about = None)]
#[command(version = "0.1.0")]
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

        /// Emit Python bytecode
        #[arg(long)]
        python: bool,

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
        Commands::Compile { input, output, python, parse_only } => {
            compile(input, output, python, parse_only)
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
    python: bool,
    parse_only: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Compiling: {:?}", input);

    // Read the source file
    let source = fs::read_to_string(&input)?;

    // Lex the source
    let mut lexer = nevermind_lexer::Lexer::new(&source);
    let tokens = lexer.tokenize()?;

    println!("  Tokenized {} tokens", tokens.len());

    if parse_only {
        // Just show the tokens
        for token in &tokens {
            println!("    {:?}", token.kind);
        }
        return Ok(());
    }

    // Parse the AST
    let mut parser = nevermind_parser::Parser::from_tokens(tokens);
    let statements = parser.parse()?;

    println!("  Parsed {} statements", statements.len());

    // TODO: Type checking
    // TODO: Code generation

    Ok(())
}

/// Run a Nevermind file
fn run(input: PathBuf, args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running: {:?}", input);
    println!("Args: {:?}", args);

    // For now, just compile it
    compile(input, None, false, false)?;

    // TODO: Execute the compiled code

    Ok(())
}

/// Start the REPL
fn repl() -> Result<(), Box<dyn std::error::Error>> {
    println!("Nevermind REPL v0.1.0");
    println!("Type 'exit' or Ctrl-D to exit\n");

    let mut input = String::new();
    let mut indent_level = 0;

    loop {
        // Show prompt
        let prompt = if indent_level == 0 {
            ">>> "
        } else {
            &"  ".repeat(indent_level)
        };

        print!("{}", prompt);
        use std::io::Write;
        std::io::stdout().flush()?;

        // Read line
        use std::io::BufRead;
        let stdin = std::io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line)?;

        if line.is_empty() {
            // EOF
            println!("\nGoodbye!");
            break;
        }

        let line = line.trim();

        // Check for exit command
        if line == "exit" || line == "quit" {
            println!("Goodbye!");
            break;
        }

        // For now, just echo the input
        println!("  Input: {}", line);

        // TODO: Actually evaluate the input
    }

    Ok(())
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

    // TODO: Type checking
    println!("  ⚠ Type checking not yet implemented");

    Ok(())
}

/// Format Nevermind files
fn fmt(inputs: Vec<PathBuf>, write: bool, check: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("Formatting: {:?}", inputs);

    for input in inputs {
        let source = fs::read_to_string(&input)?;

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
