# Nevermind Toolchain

## Overview

The Nevermind toolchain provides a complete development experience:

```
┌────────────────────────────────────────────────────────────┐
│                    Nevermind Toolchain                      │
└────────────────────────────────────────────────────────────┘

         ┌──────────┐  ┌──────────┐  ┌──────────┐
         │   REPL   │  │ Debugger │  │  Docs    │
         └──────────┘  └──────────┘  └──────────┘

         ┌──────────┐  ┌──────────┐  ┌──────────┐
         │  Format  │  │  Linter  │  │ Package  │
         └──────────┘  └──────────┘  └──────────┘

         ┌──────────────────────────────────────────┐
         │              Compiler & Runtime          │
         └──────────────────────────────────────────┘
```

---

## 1. REPL (Read-Eval-Print Loop)

### Design

The Nevermind REPL provides:
- **Immediate feedback**: Type errors as you type
- **Rich output**: Pretty-printed values with types
- **History**: Persistent command history
- **Multiline input**: Indentation-aware
- **Magic commands**: Special commands for meta-operations
- **IDE integration**: Support for Jupyter, nREPL, etc.

### REPL Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Input     │────▶│   Parser    │────▶│   Compiler  │
│  ( stdin )  │     │ (partial)   │     │  (increment)│
└─────────────┘     └─────────────┘     └─────────────┘
                                              │
                                              ▼
┌─────────────┐     ┌─────────────◀─────────────┐
│   Output    │◀────│    Runtime   │             │
│ ( stdout )  │     │   (eval)    │             │
└─────────────┘     └─────────────┴─────────────┘
```

### Implementation

```rust
struct Repl {
    compiler: Compiler,
    runtime: Runtime,
    history: History,
    context: ReplContext,
}

struct ReplContext {
    // Previous definitions
    definitions: HashMap<String, Definition>,

    // Imported modules
    imports: HashSet<ModuleId>,

    // Type information
    types: TypeContext,
}

impl Repl {
    fn new() -> Self {
        Repl {
            compiler: Compiler::new(),
            runtime: Runtime::new(),
            history: History::load(),
            context: ReplContext::new(),
        }
    }

    fn run(&mut self) {
        println!("Nevermind REPL v1.0");
        println!("Type 'help' for help, 'exit' to exit\n");

        let mut input = String::new();
        let mut indent_level = 0;

        loop {
            // Show prompt
            let prompt = if indent_level == 0 {
                ">>> "
            } else {
                &"  ".repeat(indent_level as usize)
            };

            print!("{}", prompt);
            io::stdout().flush().unwrap();

            // Read line
            io::stdin().read_line(&mut input).unwrap();

            // Check for magic commands
            if self.handle_magic_command(&input) {
                input.clear();
                continue;
            }

            // Parse incrementally
            match self.compiler.parse_line(&input, &self.context) {
                ParseResult::Complete(ast) => {
                    // Evaluate
                    match self.eval(ast, &mut self.context) {
                        Ok(result) => {
                            println!("{:?}", result);
                        }
                        Err(err) => {
                            eprintln!("Error: {}", err);
                        }
                    }

                    input.clear();
                    indent_level = 0;
                }
                ParseResult::Incomplete(new_indent) => {
                    // Need more input
                    indent_level = new_indent;
                    input.push('\n');
                }
                ParseResult::Error(err) => {
                    eprintln!("Parse error: {}", err);
                    input.clear();
                    indent_level = 0;
                }
            }
        }
    }

    fn handle_magic_command(&mut self, input: &str) -> bool {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();

        match parts.first() {
            Some(&":exit") | Some(&":quit") => {
                std::process::exit(0);
            }
            Some(&":help") => {
                self.print_help();
                true
            }
            Some(&":type") => {
                if parts.len() > 1 {
                    let expr = parts[1..].join(" ");
                    if let Ok(ty) = self.compiler.infer_type(&expr, &self.context) {
                        println!("{} : {}", expr, ty);
                    }
                }
                true
            }
            Some(&":load") => {
                if parts.len() > 1 {
                    self.load_file(parts[1]);
                }
                true
            }
            Some(&":reset") => {
                self.context = ReplContext::new();
                println!("Context reset");
                true
            }
            Some(&":history") => {
                for (i, cmd) in self.history.iter().enumerate() {
                    println!("{}  {}", i, cmd);
                }
                true
            }
            Some(&":doc") => {
                if parts.len() > 1 {
                    self.show_documentation(parts[1]);
                }
                true
            }
            _ => false,
        }
    }

    fn eval(&mut self, ast: Ast, context: &mut ReplContext) -> Result<Value, RuntimeError> {
        // Type check
        self.compiler.type_check(&ast, context)?;

        // Compile to bytecode
        let bytecode = self.compiler.compile(ast)?;

        // Execute
        let result = self.runtime.eval(bytecode, context)?;

        // Update context with new definitions
        context.extract_definitions(&result);

        Ok(result)
    }

    fn print_help(&self) {
        println!("Nevermind REPL Commands:");
        println!("  :exit, :quit    Exit the REPL");
        println!("  :help           Show this help message");
        println!("  :type <expr>    Show type of expression");
        println!("  :load <file>    Load and execute a file");
        println!("  :reset          Clear all definitions");
        println!("  :history        Show command history");
        println!("  :doc <name>     Show documentation");
        println!();
    }
}
```

### Multi-line Input Handling

```rust
enum ParseResult {
    Complete(Ast),
    Incomplete(u32),  // Indent level needed
    Error(ParseError),
}

impl Compiler {
    fn parse_line(&mut self, line: &str, context: &ReplContext) -> ParseResult {
        let full_input = if let Some(partial) = &self.partial_input {
            format!("{}\n{}", partial, line)
        } else {
            line.to_string()
        };

        match self.parse(&full_input) {
            Ok(ast) => {
                self.partial_input = None;
                ParseResult::Complete(ast)
            }
            Err(ParseError::UnexpectedEOF) => {
                // Need more input
                let indent = self.count_indentation(&full_input);
                self.partial_input = Some(full_input);
                ParseResult::Incomplete(indent)
            }
            Err(err) => ParseResult::Error(err),
        }
    }

    fn count_indentation(&self, input: &str) -> u32 {
        input.lines().last()
            .map(|line| line.chars().take_while(|c| c.is_whitespace()).count() as u32)
            .unwrap_or(0)
    }
}
```

---

## 2. Debugger

### Design

The Nevermind debugger provides:
- **Breakpoints**: Line and conditional breakpoints
- **Stepping**: Step into, over, out
- **Inspection**: Examine variables, stack frames
- **Evaluation**: Eval expressions in context
- **Profiling**: Performance profiling
- **Time travel**: Reverse debugging (experimental)

### Debugger Architecture

```rust
struct Debugger {
    runtime: Runtime,
    breakpoints: HashMap<SourceLocation, Breakpoint>,
    call_stack: Vec<StackFrame>,
    state: DebuggerState,
}

enum DebuggerState {
    Running,
    Paused(SourceLocation),
    Stepping(StepMode),
    Finished,
}

enum StepMode {
    StepOver,
    StepInto,
    StepOut,
}

struct Breakpoint {
    location: SourceLocation,
    condition: Option<Expr>,
    hit_count: usize,
    enabled: bool,
}

struct StackFrame {
    function: SymbolId,
    location: SourceLocation,
    locals: HashMap<String, Value>,
}

impl Debugger {
    fn new(runtime: Runtime) -> Self {
        Debugger {
            runtime,
            breakpoints: HashMap::new(),
            call_stack: Vec::new(),
            state: DebuggerState::Running,
        }
    }

    fn set_breakpoint(&mut self, location: SourceLocation, condition: Option<Expr>) {
        self.breakpoints.insert(location, Breakpoint {
            location,
            condition,
            hit_count: 0,
            enabled: true,
        });
    }

    fn clear_breakpoint(&mut self, location: &SourceLocation) {
        self.breakpoints.remove(location);
    }

    fn step(&mut self, mode: StepMode) {
        self.state = DebuggerState::Stepping(mode);
        self.runtime.resume();
    }

    fn continue_(&mut self) {
        self.state = DebuggerState::Running;
        self.runtime.resume();
    }

    fn get_stack_trace(&self) -> &[StackFrame] {
        &self.call_stack
    }

    fn get_locals(&self, frame_index: usize) -> Option<&HashMap<String, Value>> {
        self.call_stack.get(frame_index).map(|frame| &frame.locals)
    }

    fn eval_in_context(&self, expr: &str, frame_index: usize) -> Result<Value, EvalError> {
        let frame = self.call_stack.get(frame_index)
            .ok_or(EvalError::InvalidFrame)?;

        let context = self.build_eval_context(frame);
        self.runtime.eval_expr(expr, &context)
    }

    fn on_breakpoint_hit(&mut self, location: &SourceLocation) {
        if let Some(bp) = self.breakpoints.get(location) {
            if !bp.enabled {
                return;
            }

            if let Some(condition) = &bp.condition {
                if let Ok(result) = self.eval_in_context(&condition.to_string(), 0) {
                    if !result.as_bool() {
                        return;
                    }
                }
            }

            self.state = DebuggerState::Paused(*location);
            self.runtime.pause();
        }
    }
}
```

### Debug Protocol (DAP)

Nevermind implements the **Debug Adapter Protocol** for IDE integration:

```rust
struct DebugAdapter {
    debugger: Debugger,
    messages: Vec<ProtocolMessage>,
}

impl DebugAdapter {
    fn handle_message(&mut self, msg: ProtocolMessage) -> Option<ProtocolMessage> {
        match msg.command {
            "initialize" => Some(self.initialize()),
            "setBreakpoints" => Some(self.set_breakpoints(msg.arguments)),
            "continue" => Some(self.continue_()),
            "next" => Some(self.next()),
            "stepIn" => Some(self.step_in()),
            "stepOut" => Some(self.step_out()),
            "stackTrace" => Some(self.stack_trace()),
            "scopes" => Some(self.scopes()),
            "variables" => Some(self.variables()),
            "evaluate" => Some(self.evaluate(msg.arguments)),
            _ => None,
        }
    }

    fn initialize(&self) -> ProtocolMessage {
        json!({
            "type": "response",
            "request_seq": 0,
            "success": true,
            "body": {
                "capabilities": {
                    "supportsConfigurationDoneRequest": true,
                    "supportsConditionalBreakpoints": true,
                    "supportsEvaluateForHovers": true,
                    "supportsStepBack": false,
                    "supportsRestartFrame": false,
                }
            }
        })
    }

    fn stack_trace(&self) -> ProtocolMessage {
        let frames: Vec<_> = self.debugger.get_stack_trace()
            .iter()
            .enumerate()
            .map(|(i, frame)| json!({
                "id": i,
                "name": frame.function.name(),
                "line": frame.location.line,
                "column": frame.location.column,
            }))
            .collect();

        json!({
            "type": "response",
            "body": {
                "stackFrames": frames,
            }
        })
    }

    fn evaluate(&self, args: serde_json::Value) -> ProtocolMessage {
        let expr = args["expression"].as_str().unwrap();
        let frame_id = args["frameId"].as_u64().unwrap() as usize;

        match self.debugger.eval_in_context(expr, frame_id) {
            Ok(value) => json!({
                "type": "response",
                "body": {
                    "result": format!("{:?}", value),
                    "type": value.type_name(),
                }
            }),
            Err(err) => json!({
                "type": "response",
                "success": false,
                "body": {
                    "result": format!("Error: {}", err),
                }
            }),
        }
    }
}
```

---

## 3. Formatter

### Design

The Nevermind formatter (`nvm fmt`) provides:
- **Opinionated formatting**: One true style (no configuration)
- **Idiomatic code**: Enforces community standards
- **Safe**: Preserves program semantics
- **Fast**: Incremental formatting

### Formatting Rules

```nevermind
# Indentation: 2 spaces (never tabs)
fn add(a: Int, b: Int) -> Int
  do
    a + b
  end
end

# Line length: 100 characters (soft limit)
let very_long_name = some_function_with_many_arguments(arg1, arg2, arg3, arg4)

# Alignment: align similar constructs
let name       = "Alice"
let age        = 30
let occupation = "Engineer"

# Blank lines: 1 blank line between top-level definitions
fn foo()
  do
    ...
  end
end

fn bar()
  do
    ...
  end
end

# Spaces around operators (no spaces for unary)
let result = a + b * c
let negated = -x

# Spaces after commas
fn func(a: Int, b: Int, c: Int)
  do
    ...
  end
end

# Trailing commas in multi-line lists
let items = [
  item1,
  item2,
  item3,
]

# No trailing commas in single-line
let items = [item1, item2, item3]

# Pipeline: one operator per line
let result = data
  |> transform
  |> filter
  |> sort
```

### Implementation

```rust
struct Formatter {
    config: FormatConfig,
}

struct FormatConfig {
    indent_size: u32,
    max_line_length: usize,
    trailing_comma: bool,
}

impl Formatter {
    fn format(&self, source: &str) -> Result<String, FormatError> {
        let mut tokens = Lexer::new(source).collect()?;
        let mut ast = Parser::new(tokens).parse()?;

        self.format_ast(&mut ast)?;

        Ok(self.ast_to_string(&ast))
    }

    fn format_ast(&self, ast: &mut Ast) -> Result<(), FormatError> {
        // Normalize spacing
        self.normalize_spacing(ast)?;

        // Break long lines
        self.break_long_lines(ast)?;

        // Sort imports
        self.sort_imports(ast)?;

        // Remove unused imports
        self.remove_unused_imports(ast)?;

        Ok(())
    }

    fn normalize_spacing(&self, ast: &mut Ast) -> Result<(), FormatError> {
        // Ensure consistent spacing around operators
        for expr in &mut ast.expressions {
            match expr {
                Expr::Binary(left, op, right) => {
                    // Add spaces around binary operators
                    op.spacing = Spacing::Wide;
                }
                Expr::Unary(op, expr) => {
                    // No spaces after unary operators
                    op.spacing = Spacing::Tight;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn break_long_lines(&self, ast: &mut Ast) -> Result<(), FormatError> {
        // Break lines that exceed max_line_length
        for expr in &mut ast.expressions {
            let line_length = self.calculate_line_length(expr)?;

            if line_length > self.config.max_line_length {
                self.break_expression(expr)?;
            }
        }

        Ok(())
    }

    fn break_expression(&self, expr: &mut Expr) -> Result<(), FormatError> {
        match expr {
            Expr::Call(callee, args) => {
                // Break arguments onto multiple lines
                for arg in args {
                    arg.needs_line_break = true;
                }
            }
            Expr::Pipeline(exprs) => {
                // Each pipeline stage on its own line
                for expr in exprs {
                    expr.needs_line_break = true;
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn ast_to_string(&self, ast: &Ast) -> String {
        let mut output = String::new();

        for (i, expr) in ast.expressions.iter().enumerate() {
            if i > 0 {
                output.push_str("\n\n");
            }

            self.write_expression(&mut output, expr, 0);
        }

        output
    }

    fn write_expression(&self, output: &mut String, expr: &Expr, indent: u32) {
        match expr {
            Expr::Let { name, value, .. } => {
                output.push_str(&"  ".repeat(indent as usize));
                output.push_str(&format!("let {} = ", name));

                if value.needs_line_break {
                    output.push_str("\n");
                    self.write_expression(output, value, indent + 1);
                } else {
                    self.write_expression(output, value, 0);
                }
            }
            Expr::Binary(left, op, right) => {
                self.write_expression(output, left, 0);
                output.push_str(&format!(" {} ", op));
                self.write_expression(output, right, 0);
            }
            _ => {}
        }
    }
}
```

---

## 4. Linter

### Design

The Nevermind linter (`nvm lint`) provides:
- **Static analysis**: Find bugs without running
- **Best practices**: Enforce idiomatic code
- **Performance hints**: Identify inefficiencies
- **Security checks**: Detect vulnerabilities
- **Code smell**: Identify maintainability issues

### Lint Rules

```rust
enum LintRule {
    // Code style
    UnusedVariable,
    UnusedImport,
    DeadCode,
    InconsistentNaming,
    TooManyArguments,
    TooLongFunction,

    // Correctness
    TypeMismatch,
    UnreachableCode,
    PatternMatchExhaustiveness,
    NullPointerDereference,
    DivisionByZero,

    // Performance
    InefficientConcatenation,
    UnnecessaryAllocation,
    LoopInefficiency,
    RedundantComputation,

    // Security
    HardcodedPassword,
    SQLInjection,
    XSS,
    PathTraversal,

    // Best practices
    Shadowing,
    MutableGlobal,
    PanicInFunction,
    RecursiveWithoutBaseCase,
}

struct LintMessage {
    rule: LintRule,
    severity: Severity,
    location: SourceLocation,
    message: String,
    suggestion: Option<String>,
}

enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}
```

### Implementation

```rust
struct Linter {
    rules: Vec<Box<dyn LintRule>>,
}

impl Linter {
    fn new() -> Self {
        Linter {
            rules: vec![
                Box::new(UnusedVariableRule),
                Box::new(UnreachableCodeRule),
                Box::new(InefficientConcatenationRule),
                Box::new(HardcodedPasswordRule),
                // ... more rules
            ],
        }
    }

    fn lint(&self, ast: &Ast) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for rule in &self.rules {
            messages.extend(rule.check(ast));
        }

        messages.sort_by_key(|m| m.location);
        messages
    }
}

trait LintRule {
    fn check(&self, ast: &Ast) -> Vec<LintMessage>;
}

// Example: Unreachable code detection
struct UnreachableCodeRule;

impl LintRule for UnreachableCodeRule {
    fn check(&self, ast: &Ast) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for expr in &ast.expressions {
            self.check_expression(expr, &mut messages);
        }

        messages
    }
}

impl UnreachableCodeRule {
    fn check_expression(&self, expr: &Expr, messages: &mut Vec<LintMessage>) {
        match expr {
            Expr::If(condition, then_branch, else_branch) => {
                if self.is_always_true(condition) {
                    // Else branch is unreachable
                    messages.push(LintMessage {
                        rule: LintRule::UnreachableCode,
                        severity: Severity::Warning,
                        location: else_branch.location,
                        message: "Unreachable code: else branch will never execute".to_string(),
                        suggestion: Some("Remove the else branch or fix the condition".to_string()),
                    });
                }

                self.check_expression(then_branch, messages);
                self.check_expression(else_branch, messages);
            }
            Expr::Match(expr, arms) => {
                for arm in arms {
                    self.check_expression(&arm.body, messages);
                }
            }
            _ => {}
        }
    }

    fn is_always_true(&self, expr: &Expr) -> bool {
        match expr {
            Expr::BooleanLiteral(true) => true,
            _ => false,
        }
    }
}

// Example: Inefficient string concatenation
struct InefficientConcatenationRule;

impl LintRule for InefficientConcatenationRule {
    fn check(&self, ast: &Ast) -> Vec<LintMessage> {
        let mut messages = Vec::new();

        for expr in &ast.expressions {
            self.check_expression(expr, &mut messages);
        }

        messages
    }
}

impl InefficientConcatenationRule {
    fn check_expression(&self, expr: &Expr, messages: &mut Vec<LintMessage>) {
        match expr {
            Expr::Binary(_, BinaryOp::Add, _) => {
                if self.is_string_concatenation(expr) {
                    messages.push(LintMessage {
                        rule: LintRule::InefficientConcatenation,
                        severity: Severity::Info,
                        location: expr.location,
                        message: "String concatenation in loop may be inefficient".to_string(),
                        suggestion: Some("Use StringBuilder or join() instead".to_string()),
                    });
                }
            }
            _ => {}
        }
    }

    fn is_string_concatenation(&self, expr: &Expr) -> bool {
        // Check if this is a string concatenation
        // (implementation omitted)
        false
    }
}
```

---

## 5. Package Manager

### Design

The Nevermind package manager (`nvm pkg`) provides:
- **Dependency management**: Semantic versioning
- **Package publishing**: Central package registry
- **Virtual environments**: Isolated dependencies
- **Lock files**: Reproducible builds
- **Workspaces**: Monorepo support

### Package Manifest

```toml
# nvm.toml
[package]
name = "my-app"
version = "1.0.0"
description = "My awesome application"
authors = ["Alice <alice@example.com>"]
license = "MIT"

[dependencies]
nevermind-http = "^1.2.0"
nevermind-json = "~2.0.0"

[dev-dependencies]
nevermind-test = "^1.0.0"

[workspace]
members = [
  "packages/core",
  "packages/utils",
]
```

### Implementation

```rust
struct PackageManager {
    registry: PackageRegistry,
    cache: PackageCache,
}

impl PackageManager {
    fn install(&mut self, package: &str) -> Result<(), PackageError> {
        // Resolve version
        let version = self.registry.resolve(package, None)?;

        // Download package
        let tarball = self.registry.download(package, &version)?;

        // Extract to cache
        self.cache.extract(package, &version, tarball)?;

        // Update lock file
        self.update_lock_file()?;

        Ok(())
    }

    fn publish(&self, package_path: &Path) -> Result<(), PackageError> {
        // Read package manifest
        let manifest = self.read_manifest(package_path)?;

        // Validate package
        self.validate_package(&manifest)?;

        // Build package
        self.build_package(package_path)?;

        // Upload to registry
        let tarball = self.create_tarball(package_path)?;
        self.registry.upload(&manifest.name, &manifest.version, tarball)?;

        Ok(())
    }

    fn update(&mut self) -> Result<(), PackageError> {
        // Read lock file
        let lock = self.read_lock_file()?;

        // Check for updates
        let updates = self.registry.check_updates(&lock)?;

        // Install updates
        for (package, version) in updates {
            println!("Updating {} to {}", package, version);
            self.install(&format!("{}@{}", package, version))?;
        }

        Ok(())
    }
}

struct PackageRegistry {
    url: String,
    client: HttpClient,
}

impl PackageRegistry {
    fn resolve(&self, package: &str, constraint: Option<VersionConstraint>) -> Result<Version, PackageError> {
        let url = format!("{}/package/{}", self.url, package);
        let response = self.client.get(&url)?;

        let metadata: PackageMetadata = serde_json::from_str(&response.body)?;

        match constraint {
            Some(constraint) => {
                let versions = metadata.versions.into_iter()
                    .filter(|v| constraint.satisfies(v))
                    .collect();

                Ok(versions.into_iter().max().unwrap())
            }
            None => Ok(metadata.versions.into_iter().max().unwrap()),
        }
    }

    fn download(&self, package: &str, version: &Version) -> Result<Vec<u8>, PackageError> {
        let url = format!("{}/download/{}/{}", self.url, package, version);
        let response = self.client.get(&url)?;

        Ok(response.body.into_bytes())
    }

    fn upload(&self, package: &str, version: &Version, tarball: Vec<u8>) -> Result<(), PackageError> {
        let url = format!("{}/publish", self.url);

        let mut form = FormData::new();
        form.add_file("tarball", tarball);

        let response = self.client.post(&url, form)?;

        if response.status != 200 {
            return Err(PackageError::PublishFailed(response.body));
        }

        Ok(())
    }

    fn check_updates(&self, lock: &LockFile) -> Result<Vec<(String, Version)>, PackageError> {
        let mut updates = Vec::new();

        for (package, current_version) in &lock.packages {
            let latest = self.resolve(package, None)?;

            if latest > *current_version {
                updates.push((package.clone(), latest));
            }
        }

        Ok(updates)
    }
}
```

---

## Summary

The Nevermind toolchain provides:

1. **REPL**: Interactive development environment
2. **Debugger**: Full-featured debugging with DAP support
3. **Formatter**: Opinionated code formatting
4. **Linter**: Static analysis and best practices
5. **Package Manager**: Dependency and workspace management

All tools are:
- **Fast**: Optimized for performance
- **Usable**: Rich output and error messages
- **Integratable**: Support for IDEs and editors
- **Consistent**: Uniform UX across all tools

---

*Toolchain Specification v1.0*
