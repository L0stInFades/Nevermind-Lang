# Nevermind Compiler Architecture

## Overview

The Nevermind compiler transforms Nevermind source code into executable programs through a multi-stage pipeline. The architecture prioritizes:

1. **Simplicity**: Each stage has a single responsibility
2. **Debuggability**: Rich error messages with source locations
3. **Extensibility**: Easy to add new passes or optimizations
4. **Performance**: Fast compilation despite complex analysis

---

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Compilation Pipeline                     │
└─────────────────────────────────────────────────────────────┘
    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────┐
    │  Source  │───▶│   AST    │───▶│   HIR    │───▶│ MIR  │
    │  Code    │    │ (Typed)  │    │ (High IR)│    │      │
    └──────────┘    └──────────┘    └──────────┘    └──────┘
          │              │              │              │
          │              ▼              ▼              ▼
    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────┐
    │  Lexer   │    │  Parser  │    │  Name    │    │ Type │
    │          │    │          │    │ Resolver │    │ Check│
    └──────────┘    └──────────┘    └──────────┘    └──────┘
                                                       │
                                                       ▼
                                                 ┌──────────┐
                                                 │   LIR    │
                                                 │ (Low IR) │
                                                 └──────────┘
                                                       │
                                    ┌──────────────────┼──────────────────┐
                                    ▼                  ▼                  ▼
                              ┌──────────┐      ┌──────────┐      ┌──────────┐
                              │  Python  │      │  LLVM    │      │   WASM   │
                              │Bytecode │      │  IRGen   │      │  Gen     │
                              └──────────┘      └──────────┘      └──────────┘
```

---

## Stage 1: Lexical Analysis (Lexer)

### Responsibilities
- Convert source text into tokens
- Track locations (line, column)
- Manage whitespace and indentation
- Handle comments (line and block)
- String interpolation lexing

### Token Types

```nevermind
# Internal representation (in Rust)
enum Token {
  // Keywords
  Let, Var, Fn, Return,
  If, Then, Else, Elif,
  For, While, Forever, In,
  Match, Case, Try, Catch, Finally,
  Class, Extends, Implements, Trait,
  Type, Where, Use, From, Import,
  Async, Await, Parallel, Sync,

  // Literals
  Identifier(String),
  StringLiteral(String),
  CharLiteral(char),
  IntegerLiteral(i64),
  FloatLiteral(f64),
  BooleanLiteral(bool),

  // Operators
  Plus, Minus, Star, Slash, Percent,
  PlusPlus, MinusMinus,
  Equal, EqualEqual, BangEqual,
  Less, Greater, LessEqual, GreaterEqual,
  And, Or, Not, Bang,
  Pipe, Backtick,

  // Delimiters
  LParen, RParen,
  LBrace, RBrace,
  LBracket, RBracket,
  Comma, Colon, Semicolon,
  Dot, DotDot, DotDotDot,
  Arrow, FatArrow,
  Question, At, Dollar,

  // Structural
  Do, End,
  Indent(u32),  // Dedent handled specially
  Newline,

  // EOF
  EOF,
}
```

### Indentation Handling

Nevermind uses **significant indentation** (like Python). The lexer tracks indentation levels:

```rust
struct Lexer {
    input: Vec<char>,
    position: usize,
    indent_stack: Vec<u32>,
    at_start_of_line: bool,
}

impl Lexer {
    fn handle_indentation(&mut self) -> Option<Token> {
        if !self.at_start_of_line {
            return None;
        }

        let current_indent = self.count_whitespace();
        let top_indent = *self.indent_stack.last().unwrap_or(&0);

        if current_indent > top_indent {
            // Increase indentation
            self.indent_stack.push(current_indent);
            Some(Token::Indent(current_indent))
        } else if current_indent < top_indent {
            // Decrease indentation
            self.indent_stack.pop();
            Some(Token::Dedent)
        } else {
            // Same indentation
            None
        }
    }
}
```

### Error Recovery

```rust
enum LexerError {
    UnexpectedChar(char, SourceLocation),
    UnterminatedString(SourceLocation),
    InvalidEscapeSequence(String, SourceLocation),
    InvalidNumberFormat(String, SourceLocation),
}

impl Lexer {
    fn recover(&mut self) -> Token {
        // Skip to next synchronizing point
        // (newline, semicolon, or delimiter)
        while let Some(ch) = self.peek() {
            if ch == '\n' || ch == ';' || matches!(ch, '}' | ')' | ']') {
                break;
            }
            self.advance();
        }
        self.next_token()
    }
}
```

---

## Stage 2: Parsing

### Parser Design

**Recursive descent parser** with:
- Prat parsing ( Pratt parsing ) for expressions
- Error recovery with synchronization points
- Operator precedence climbing
- AST construction with location tracking

### Grammar Implementation

```rust
struct Parser {
    tokens: Vec<Token>,
    current: usize,
    errors: Vec<ParseError>,
}

impl Parser {
    // Entry point
    fn parse(&mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();

        while !self.check(TokenType::EOF) {
            statements.push(self.parse_statement()?);
        }

        Ok(Program { statements })
    }

    // Statements
    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        match self.peek_type() {
            TokenType::Let => self.parse_let_statement(),
            TokenType::Var => self.parse_var_statement(),
            TokenType::Fn => self.parse_fn_statement(),
            TokenType::If => self.parse_if_statement(),
            TokenType::For => self.parse_for_statement(),
            TokenType::While => self.parse_while_statement(),
            TokenType::Match => self.parse_match_statement(),
            TokenType::Try => self.parse_try_statement(),
            TokenType::Do => self.parse_block_statement(),
            _ => self.parse_expr_statement(),
        }
    }

    // Let statement: let name: Type = expr
    fn parse_let_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.previous().location;

        self.consume(TokenType::Let, "Expected 'let'")?;

        let name = self.consume_identifier()?;

        let type_annotation = if self.match_(TokenType::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.consume(TokenType::Equal, "Expected '=' after let binding")?;

        let value = self.parse_expression()?;

        Ok(Stmt::Let {
            name,
            type_annotation,
            value,
            location: start.to(self.previous().location),
        })
    }

    // Expressions (Pratt parsing)
    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_expression_bp(0)
    }

    fn parse_expression_bp(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        // Parse left side
        let mut lhs = self.parse_prefix()?;

        loop {
            let op = match self.peek() {
                Some(token) => token.clone(),
                None => return Ok(lhs),
            };

            let (l_bp, r_bp) = self.get_binding_power(&op);
            if l_bp < min_bp {
                break;
            }

            self.advance();

            lhs = self.parse_infix(lhs, op, r_bp)?;
        }

        Ok(lhs)
    }

    // Prefix expressions
    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        match self.peek_type() {
            TokenType::Identifier => {
                let name = self.advance().unwrap();
                Ok(Expr::Variable(name.lexeme.clone(), name.location))
            }
            TokenType::IntegerLiteral => {
                let token = self.advance().unwrap();
                Ok(Expr::IntegerLiteral(
                    token.lexeme.parse().unwrap(),
                    token.location,
                ))
            }
            TokenType::StringLiteral => {
                let token = self.advance().unwrap();
                Ok(Expr::StringLiteral(
                    token.lexeme.clone(),
                    token.location,
                ))
            }
            TokenType::LParen => self.parse_grouping(),
            TokenType::If => self.parse_if_expression(),
            TokenType::Pipe => self.parse_lambda(),
            TokenType::Not | TokenType::Minus => self.parse_unary(),
            _ => Err(self.error("Expected expression")),
        }
    }

    // Infix expressions
    fn parse_infix(&mut self, lhs: Expr, op: Token, r_bp: u8) -> Result<Expr, ParseError> {
        let rhs = self.parse_expression_bp(r_bp)?;

        Ok(match op.token_type {
            TokenType::Plus => Expr::Binary(Box::new(lhs), BinaryOp::Add, Box::new(rhs)),
            TokenType::Minus => Expr::Binary(Box::new(lhs), BinaryOp::Sub, Box::new(rhs)),
            TokenType::Star => Expr::Binary(Box::new(lhs), BinaryOp::Mul, Box::new(rhs)),
            TokenType::Slash => Expr::Binary(Box::new(lhs), BinaryOp::Div, Box::new(rhs)),
            TokenType::EqualEqual => Expr::Binary(Box::new(lhs), BinaryOp::Eq, Box::new(rhs)),
            TokenType::And => Expr::Logical(Box::new(lhs), LogicalOp::And, Box::new(rhs)),
            TokenType::Or => Expr::Logical(Box::new(lhs), LogicalOp::Or, Box::new(rhs)),
            TokenType::Pipe => Expr::Pipeline(Box::new(lhs), Box::new(rhs)),
            _ => return Err(self.error(&format!("Unknown operator: {:?}", op))),
        })
    }
}
```

### Binding Powers (Precedence)

```rust
fn get_binding_power(&self, token: &Token) -> (u8, u8) {
    match token.token_type {
        // Assignment (lowest)
        TokenType::Equal => (2, 1),

        // Logical or
        TokenType::Or => (4, 5),

        // Logical and
        TokenType::And => (6, 7),

        // Comparison
        TokenType::EqualEqual | TokenType::BangEqual |
        TokenType::Less | TokenType::Greater |
        TokenType::LessEqual | TokenType::GreaterEqual => (8, 9),

        // Additive
        TokenType::Plus | TokenType::Minus => (10, 11),

        // Multiplicative
        TokenType::Star | TokenType::Slash | TokenType::Percent => (12, 13),

        // Unary (highest)
        _ => (0, 0),
    }
}
```

### Error Recovery

```rust
impl Parser {
    fn synchronize(&mut self) {
        while !self.check(TokenType::EOF) {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek_type() {
                TokenType::Fn | TokenType::Let | TokenType::Var |
                TokenType::If | TokenType::For | TokenType::While |
                TokenType::Class | TokenType::Trait => return,
                _ => self.advance(),
            }
        }
    }

    fn error(&mut self, message: &str) -> ParseError {
        let token = self.peek().cloned().unwrap_or(Token {
            token_type: TokenType::EOF,
            lexeme: String::new(),
            location: SourceLocation::eof(),
        });

        ParseError {
            message: message.to_string(),
            location: token.location,
        }
    }
}
```

---

## Stage 3: Name Resolution

### Responsibilities
- Resolve all identifiers to their declarations
- Build symbol table
- Detect undefined variables
- Handle shadowing
- Resolve imports and module paths

### Symbol Table

```rust
struct SymbolTable {
    scopes: Vec<Scope>,
}

struct Scope {
    symbols: HashMap<String, Symbol>,
    parent: Option<usize>,  // Index of parent scope
}

struct Symbol {
    name: String,
    kind: SymbolKind,
    declaration: NodeId,
    type_id: Option<TypeId>,
}

enum SymbolKind {
    Variable,
    Function,
    Parameter,
    Type,
    Module,
    Trait,
    Impl,
}

impl SymbolTable {
    fn push_scope(&mut self) {
        self.scopes.push(Scope {
            symbols: HashMap::new(),
            parent: self.scopes.len().checked_sub(1),
        });
    }

    fn pop_scope(&mut self) -> Option<Scope> {
        self.scopes.pop()
    }

    fn insert(&mut self, name: String, symbol: Symbol) -> Result<(), ResolutionError> {
        let scope = self.scopes.last_mut().ok_or(ResolutionError::NoScope)?;
        if scope.symbols.contains_key(&name) {
            return Err(ResolutionError::DuplicateSymbol(name));
        }
        scope.symbols.insert(name, symbol);
        Ok(())
    }

    fn lookup(&self, name: &str) -> Option<&Symbol> {
        let mut current = self.scopes.len().checked_sub(1)?;

        loop {
            let scope = &self.scopes[current];
            if let Some(symbol) = scope.symbols.get(name) {
                return Some(symbol);
            }

            current = scope.parent?;
        }
    }
}
```

### Resolution Pass

```rust
struct NameResolver {
    symbols: SymbolTable,
    ast: Ast,
    errors: Vec<ResolutionError>,
}

impl NameResolver {
    fn resolve(&mut self, program: &mut Program) -> Result<(), ResolutionError> {
        // First pass: declarations
        self.resolve_declarations(program)?;

        // Second pass: bodies
        self.resolve_bodies(program)?;

        Ok(())
    }

    fn resolve_declarations(&mut self, program: &Program) -> Result<(), ResolutionError> {
        for statement in &program.statements {
            self.resolve_statement_decl(statement)?;
        }
        Ok(())
    }

    fn resolve_statement_decl(&mut self, stmt: &Stmt) -> Result<(), ResolutionError> {
        match stmt {
            Stmt::Let { name, type_annotation, .. } => {
                let symbol = Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Variable,
                    declaration: stmt.id(),
                    type_id: None,  // Will be filled by type checker
                };
                self.symbols.insert(name.clone(), symbol)?;
            }
            Stmt::Fn { name, parameters, .. } => {
                self.symbols.push_scope();

                for param in parameters {
                    let symbol = Symbol {
                        name: param.name.clone(),
                        kind: SymbolKind::Parameter,
                        declaration: param.id,
                        type_id: None,
                    };
                    self.symbols.insert(param.name.clone(), symbol)?;
                }

                self.symbols.pop_scope();
            }
            _ => {}
        }
        Ok(())
    }

    fn resolve_expression(&mut self, expr: &Expr) -> Result<(), ResolutionError> {
        match expr {
            Expr::Variable(name, _) => {
                let symbol = self.symbols.lookup(name)
                    .ok_or_else(|| ResolutionError::UndefinedVariable(name.clone()))?;

                // Attach symbol ID to expression
                expr.symbol_id = Some(symbol.id);
            }
            Expr::Binary(lhs, _, rhs) => {
                self.resolve_expression(lhs)?;
                self.resolve_expression(rhs)?;
            }
            Expr::Call(callee, args) => {
                self.resolve_expression(callee)?;
                for arg in args {
                    self.resolve_expression(arg)?;
                }
            }
            Expr::Lambda(params, body) => {
                self.symbols.push_scope();

                for param in params {
                    let symbol = Symbol {
                        name: param.name.clone(),
                        kind: SymbolKind::Parameter,
                        declaration: param.id,
                        type_id: None,
                    };
                    self.symbols.insert(param.name.clone(), symbol)?;
                }

                self.resolve_expression(body)?;
                self.symbols.pop_scope();
            }
            _ => {}
        }
        Ok(())
    }
}
```

---

## Stage 4: Type Checking

### Type Checker Design

```rust
struct TypeChecker {
    symbols: SymbolTable,
    types: TypeStorage,
    constraints: Vec<TypeConstraint>,
    inference: TypeInference,
}

impl TypeChecker {
    fn check(&mut self, program: &Program) -> Result<(), TypeError> {
        // Generate constraints
        self.generate_constraints(program)?;

        // Solve constraints
        self.solve_constraints()?;

        // Apply solution to AST
        self.apply_solution(program)?;

        Ok(())
    }

    fn generate_constraints(&mut self, program: &Program) -> Result<(), TypeError> {
        for statement in &program.statements {
            self.check_statement(statement)?;
        }
        Ok(())
    }

    fn check_statement(&mut self, stmt: &Stmt) -> Result<Ty, TypeError> {
        match stmt {
            Stmt::Let { name, type_annotation, value, .. } => {
                let value_ty = self.check_expression(value)?;

                if let Some(annot_ty) = type_annotation {
                    let annot_ty = self.resolve_type(annot_ty)?;
                    self.unify(&value_ty, &annot_ty)?;
                }

                self.symbols.set_type(name, value_ty)?;
                Ok(value_ty)
            }
            Stmt::Fn { name, parameters, return_type, body, .. } => {
                self.symbols.push_scope();

                let mut param_tys = Vec::new();
                for param in parameters {
                    let param_ty = self.resolve_type(&param.type_annotation)?;
                    self.symbols.set_type(&param.name, param_ty.clone())?;
                    param_tys.push(param_ty);
                }

                let body_ty = self.check_block(body)?;

                if let Some(ret_ty) = return_type {
                    let ret_ty = self.resolve_type(ret_ty)?;
                    self.unify(&body_ty, &ret_ty)?;
                }

                self.symbols.pop_scope();

                let fn_ty = Type::Function(param_tys, Box::new(body_ty));
                self.symbols.set_type(name, fn_ty)?;

                Ok(fn_ty)
            }
            _ => Ok(Type::Unit),
        }
    }

    fn check_expression(&mut self, expr: &Expr) -> Result<Ty, TypeError> {
        match expr {
            Expr::IntegerLiteral(_, _) => Ok(Type::Int),
            Expr::StringLiteral(_, _) => Ok(Type::String),
            Expr::BooleanLiteral(_, _) => Ok(Type::Bool),
            Expr::Variable(name, _) => {
                let symbol = self.symbols.lookup(name)
                    .ok_or_else(|| TypeError::UndefinedVariable(name.clone()))?;

                Ok(symbol.type_id.clone().unwrap_or(Type::Unknown))
            }
            Expr::Binary(lhs, op, rhs) => {
                let lhs_ty = self.check_expression(lhs)?;
                let rhs_ty = self.check_expression(rhs)?;

                match op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                        self.unify(&lhs_ty, &rhs_ty)?;
                        Ok(lhs_ty)
                    }
                    BinaryOp::Eq | BinaryOp::Ne => {
                        self.unify(&lhs_ty, &rhs_ty)?;
                        Ok(Type::Bool)
                    }
                    _ => Err(TypeError::InvalidBinaryOp(op.clone(), lhs_ty, rhs_ty)),
                }
            }
            Expr::If(condition, then_branch, else_branch) => {
                let cond_ty = self.check_expression(condition)?;
                self.unify(&cond_ty, &Type::Bool)?;

                let then_ty = self.check_block(then_branch)?;
                let else_ty = self.check_block(else_branch)?;

                self.unify(&then_ty, &else_ty)?;
                Ok(then_ty)
            }
            Expr::Lambda(params, body) => {
                self.symbols.push_scope();

                let mut param_tys = Vec::new();
                for param in params {
                    let param_ty = self.inference.new_var();
                    self.symbols.set_type(&param.name, param_ty.clone())?;
                    param_tys.push(param_ty);
                }

                let body_ty = self.check_expression(body)?;
                self.symbols.pop_scope();

                Ok(Type::Function(param_tys, Box::new(body_ty)))
            }
            Expr::Call(callee, args) => {
                let callee_ty = self.check_expression(callee)?;

                let mut arg_tys = Vec::new();
                for arg in args {
                    arg_tys.push(self.check_expression(arg)?);
                }

                let return_ty = self.inference.new_var();
                let expected_fn_ty = Type::Function(arg_tys, Box::new(return_ty.clone()));

                self.unify(&callee_ty, &expected_fn_ty)?;
                Ok(return_ty)
            }
            _ => Ok(Type::Unknown),
        }
    }
}
```

### Constraint Solving

```rust
struct TypeInference {
    var_counter: usize,
    substitution: HashMap<TypeVar, Type>,
}

impl TypeInference {
    fn new_var(&mut self) -> Type {
        let var = TypeVar(self.var_counter);
        self.var_counter += 1;
        Type::Var(var)
    }

    fn unify(&mut self, ty1: &Type, ty2: &Type) -> Result<(), TypeError> {
        let ty1 = self.apply(ty1);
        let ty2 = self.apply(ty2);

        match (ty1, ty2) {
            (Type::Var(var), ty) | (ty, Type::Var(var)) => {
                if let Some(existing) = self.substitution.get(&var) {
                    self.unify(existing, &ty)?;
                } else {
                    self.substitution.insert(var, ty);
                }
                Ok(())
            }
            (Type::Function(params1, ret1), Type::Function(params2, ret2)) => {
                if params1.len() != params2.len() {
                    return Err(TypeError::ArityMismatch(params1.len(), params2.len()));
                }

                for (p1, p2) in params1.iter().zip(params2.iter()) {
                    self.unify(p1, p2)?;
                }

                self.unify(ret1, ret2)?;
                Ok(())
            }
            (Type::Int, Type::Int) |
            (Type::Bool, Type::Bool) |
            (Type::String, Type::String) => Ok(()),
            (ty1, ty2) => Err(TypeError::TypeMismatch(ty1, ty2)),
        }
    }

    fn apply(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(var) => {
                if let Some(subst) = self.substitution.get(var) {
                    self.apply(subst)
                } else {
                    ty.clone()
                }
            }
            Type::Function(params, ret) => {
                Type::Function(
                    params.iter().map(|p| self.apply(p)).collect(),
                    Box::new(self.apply(ret)),
                )
            }
            _ => ty.clone(),
        }
    }
}
```

---

## Stage 5: High-Level IR (HIR)

### HIR Design

HIR is a **typed, desugared** representation:
- All syntactic sugar removed
- Types attached to all nodes
- Control flow normalized
- Pattern matching compiled to decision trees

```rust
enum HirExpr {
    // Literals
    Literal(Literal, Ty),

    // Variables
    Variable(SymbolId, Ty),

    // Functions
    Function(Vec<Parameter>, HirBlock, Ty),

    // Calls
    Call(Box<HirExpr>, Vec<HirExpr>, Ty),

    // Operations
    Binary(BinaryOp, Box<HirExpr>, Box<HirExpr>, Ty),
    Unary(UnaryOp, Box<HirExpr>, Ty),

    // Control flow
    If(Box<HirExpr>, HirBlock, HirBlock, Ty),
    Loop(HirBlock),
    Break,
    Continue,

    // Structural
    Block(HirBlock),
    Let(SymbolId, Box<HirExpr>, HirBlock),

    // Data structures
    List(Vec<HirExpr>, Ty),
    Struct(Vec<(String, HirExpr)>, Ty),

    // Pattern matching (desugared to if-else chain)
    Match(Box<HirExpr>, Vec<MatchArm>, Ty),
}

struct MatchArm {
    pattern: Pattern,
    guard: Option<HirExpr>,
    body: HirExpr,
}

enum Pattern {
    Wildcard,
    Literal(Literal),
    Variable(SymbolId),
    Constructor(SymbolId, Vec<Pattern>),
}
```

### HIR Passes

```rust
struct HirLowering {
    types: TypeStorage,
    symbols: SymbolTable,
}

impl HirLowering {
    fn lower(&mut self, program: Program) -> HirProgram {
        HirProgram {
            functions: program.functions.into_iter().map(|f| self.lower_function(f)).collect(),
        }
    }

    fn lower_function(&mut self, func: Function) -> HirFunction {
        HirFunction {
            name: func.name,
            parameters: func.parameters,
            body: self.lower_block(func.body),
            return_type: func.return_type,
        }
    }

    fn lower_expression(&mut self, expr: Expr) -> HirExpr {
        match expr {
            Expr::IntegerLiteral(value, loc) => {
                let ty = expr.ty.clone();
                HirExpr::Literal(Literal::Int(value), ty)
            }
            Expr::Variable(name, _) => {
                let symbol_id = self.symbols.lookup(&name).unwrap().id;
                let ty = expr.ty.clone();
                HirExpr::Variable(symbol_id, ty)
            }
            Expr::If(cond, then_branch, else_branch, _) => {
                let ty = expr.ty.clone();
                HirExpr::If(
                    Box::new(self.lower_expression(*cond)),
                    self.lower_block(then_branch),
                    self.lower_block(else_branch),
                    ty,
                )
            }
            Expr::Match(expr, arms, _) => {
                let ty = expr.ty.clone();
                HirExpr::Match(
                    Box::new(self.lower_expression(*expr)),
                    arms.into_iter().map(|arm| self.lower_match_arm(arm)).collect(),
                    ty,
                )
            }
            _ => todo!(),
        }
    }

    fn lower_match_arm(&mut self, arm: MatchArm) -> HirMatchArm {
        HirMatchArm {
            pattern: self.lower_pattern(arm.pattern),
            guard: arm.guard.map(|g| self.lower_expression(g)),
            body: self.lower_expression(arm.body),
        }
    }

    fn lower_pattern(&mut self, pattern: Pattern) -> HirPattern {
        match pattern {
            Pattern::Wildcard => HirPattern::Wildcard,
            Pattern::Variable(name) => {
                let symbol_id = self.symbols.lookup(&name).unwrap().id;
                HirPattern::Variable(symbol_id)
            }
            Pattern::Constructor(name, fields) => {
                let symbol_id = self.symbols.lookup(&name).unwrap().id;
                HirPattern::Constructor(
                    symbol_id,
                    fields.into_iter().map(|f| self.lower_pattern(f)).collect(),
                )
            }
            _ => todo!(),
        }
    }
}
```

---

## Stage 6: Mid-Level IR (MIR)

### MIR Design

MIR is **control-flow graph (CFG)** based:
- Basic blocks
- SSA (Static Single Assignment) form
- Explicit memory operations
- No nested expressions (flattened)

```rust
struct MirFunction {
    name: SymbolId,
    parameters: Vec<Parameter>,
    basic_blocks: Vec<BasicBlock>,
    locals: Vec<Local>,
}

struct BasicBlock {
    id: BasicBlockId,
    statements: Vec<Statement>,
    terminator: Terminator,
}

enum Statement {
    Assign(Place, Rvalue),
    StorageLive(Local),
    StorageDead(Local),
    Drop(Place),
    Assert(AssertKind, Operand),
}

enum Terminator {
    Goto(BasicBlockId),
    Switch(Int, Vec<(BasicBlockId, BasicBlockId)>),
    Return,
    Panic,
}

enum Rvalue {
    Use(Operand),
    BinaryOp(BinOp, Operand, Operand),
    UnaryOp(UnOp, Operand),
    Ref(Place),
    Len(Place),
}

enum Operand {
    Copy(Place),
    Move(Place),
    Constant(Constant),
}
```

### MIR Construction

```rust
struct MirBuilder {
    basic_blocks: Vec<BasicBlock>,
    current_block: Option<BasicBlockId>,
    locals: Vec<Local>,
}

impl MirBuilder {
    fn build(&mut self, hir: HirFunction) -> MirFunction {
        let entry = self.create_block();
        self.start_block(entry);

        for param in &hir.parameters {
            let local = self.new_local(param.ty.clone());
            self.storage_live(local);
            self.assign(local.clone(), Operand::Move(Place::Parameter(param.id)));
        }

        let expr = self.lower_expression(hir.body);
        self.ret(expr);

        MirFunction {
            name: hir.name,
            parameters: hir.parameters,
            basic_blocks: self.basic_blocks,
            locals: self.locals,
        }
    }

    fn lower_expression(&mut self, expr: HirExpr) -> Operand {
        match expr {
            HirExpr::Literal(lit, ty) => {
                Operand::Constant(Constant::Literal(lit))
            }
            HirExpr::Variable(sym, ty) => {
                let local = self.find_local(sym);
                Operand::Copy(Place::Local(local))
            }
            HirExpr::Binary(op, left, right, ty) => {
                let left = self.lower_expression(*left);
                let right = self.lower_expression(*right);

                let temp = self.new_local(ty);
                self.storage_live(temp);
                self.assign(temp.clone(), Rvalue::BinaryOp(op, left, right));

                Operand::Move(Place::Local(temp))
            }
            HirExpr::If(cond, then_block, else_block, ty) => {
                let cond = self.lower_expression(*cond);

                let then_bb = self.create_block();
                let else_bb = self.create_block();
                let join_bb = self.create_block();

                // Switch on condition
                self.switch_int(cond, then_bb, else_bb);

                // Then branch
                self.start_block(then_bb);
                let then_val = self.lower_block(then_block);
                let then_temp = self.new_local(ty.clone());
                self.storage_live(then_temp);
                self.assign(then_temp.clone(), then_val);
                self.goto(join_bb);

                // Else branch
                self.start_block(else_bb);
                let else_val = self.lower_block(else_block);
                let else_temp = self.new_local(ty.clone());
                self.storage_live(else_temp);
                self.assign(else_temp.clone(), else_val);
                self.goto(join_bb);

                // Join block
                self.start_block(join_bb);
                Operand::Move(Place::Local(then_temp))  // TODO: phi node
            }
            _ => todo!(),
        }
    }
}
```

---

## Stage 7: Low-Level IR (LIR) & Code Generation

### Python Bytecode Backend

```rust
struct PythonCodegen {
    output: Vec<u8>,
    constants: Vec<PyObject>,
    names: Vec<String>,
}

impl PythonCodegen {
    fn generate(&mut self, mir: MirProgram) -> Vec<u8> {
        let mut code = Vec::new();

        for function in &mir.functions {
            code.extend(self.generate_function(function));
        }

        code
    }

    fn generate_function(&mut self, func: &MirFunction) -> Vec<u8> {
        let mut code = Vec::new();

        for bb in &func.basic_blocks {
            for stmt in &bb.statements {
                match stmt {
                    Statement::Assign(place, rvalue) => {
                        code.extend(self.emit_assignment(place, rvalue));
                    }
                    Statement::StorageLive(local) => {
                        // Python doesn't need explicit allocation
                    }
                    _ => {}
                }
            }

            code.extend(self.emit_terminator(&bb.terminator));
        }

        code
    }

    fn emit_assignment(&mut self, place: &Place, rvalue: &Rvalue) -> Vec<u8> {
        match rvalue {
            Rvalue::Use(operand) => {
                let source = self.emit_operand(operand);
                let target = self.emit_place(place);

                // LOAD source
                code.push(LOAD_FAST);
                code.extend(&source);

                // STORE target
                code.push(STORE_FAST);
                code.extend(&target);
            }
            Rvalue::BinaryOp(op, left, right) => {
                let left = self.emit_operand(left);
                let right = self.emit_operand(right);

                // LOAD left
                code.push(LOAD_FAST);
                code.extend(&left);

                // LOAD right
                code.push(LOAD_FAST);
                code.extend(&right);

                // BINARY_OPERATION
                code.push(self.binop_to_bytecode(op));

                // STORE target
                code.push(STORE_FAST);
                code.extend(&self.emit_place(place));
            }
            _ => {}
        }

        code
    }
}
```

---

## Summary

The Nevermind compiler pipeline:

1. **Lexer**: Source → Tokens
2. **Parser**: Tokens → AST (untyped)
3. **Name Resolution**: Resolve symbols
4. **Type Checker**: Infer and check types
5. **HIR Lowering**: AST → HIR (typed, desugared)
6. **MIR Construction**: HIR → MIR (CFG-based)
7. **LIR & Codegen**: MIR → Python bytecode / LLVM IR / WASM

Each stage is **isolated**, **testable**, and **debuggable**, with rich error reporting throughout.

---

*Compiler Architecture Specification v1.0*
