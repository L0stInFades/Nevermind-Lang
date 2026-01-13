# Name Resolver for Nevermind

This crate provides name resolution functionality for the Nevermind compiler.

## Overview

The name resolver is responsible for:
- Symbol declaration and resolution
- Scope management (global, function, block, loop scopes)
- Detection of undefined variables
- Detection of duplicate definitions
- Support for variable shadowing
- Validation of control flow (return, break, continue)

## Architecture

### Core Components

1. **Symbol** (`symbol.rs`)
   - Represents a named entity (variable, function, parameter, type)
   - Contains name, kind, span, and optional type information

2. **Scope** (`scope.rs`)
   - Represents a lexical scope with parent-child relationships
   - Manages symbols declared in that scope
   - Supports shadowing through parent scope lookups

3. **SymbolTable** (`symbol_table.rs`)
   - Manages a stack of scopes
   - Tracks loop and function depth
   - Provides scope navigation (enter/exit)

4. **NameResolver** (`resolver.rs`)
   - Main resolver that walks the AST
   - Resolves all symbol references
   - Validates control flow statements

5. **NameError** (`error.rs`)
   - Error types for name resolution failures
   - Rich error reporting with source locations

## Usage Example

```rust
use nevermind_name_resolver::{NameResolver, Symbol, SymbolKind};
use nevermind_ast::{Stmt, Expr};
use nevermind_common::Span;

// Create a resolver
let mut resolver = NameResolver::new();

// Your AST statements
let stmts = vec![
    // let x = 42
    Stmt::Let {
        id: 1,
        is_mutable: false,
        name: "x".to_string(),
        type_annotation: None,
        value: Expr::Literal(Literal::Integer(42, Span::dummy())),
        span: Span::dummy(),
    },
];

// Resolve names
match resolver.resolve(&stmts) {
    Ok(()) => println!("Name resolution successful"),
    Err(errors) => {
        for error in errors {
            eprintln!("Error: {}", error);
        }
    }
}
```

## Error Handling

The resolver detects and reports:

### Undefined Variables
```rust
// Error: Cannot find value 'x' in this scope
let y = x  // x not defined
```

### Duplicate Definitions
```rust
// Error: Name 'x' is already defined in this scope
let x = 1
let x = 2  // duplicate
```

### Invalid Control Flow
```rust
// Error: return statement can only be used inside a function
return 42

// Error: break statement can only be used inside a loop
break

// Error: continue statement can only be used inside a loop
continue
```

## Variable Shadowing

The resolver allows shadowing in nested scopes:

```rust
let x = 1  // outer scope
{
    let x = 2  // inner scope, shadows outer x
    // x refers to the inner x
}
// x refers to the outer x again
```

## Scope Types

1. **Global Scope** - Level 0, contains top-level declarations
2. **Function Scope** - Created for each function, contains parameters
3. **Block Scope** - Created for blocks in expressions and statements
4. **Loop Scope** - Special scope for loop bodies (tracks loop depth)

## Integration with Parser

The name resolver works with the AST produced by the parser:

```rust
use nevermind_parser::Parser;
use nevermind_name_resolver::NameResolver;

let source = "let x = 42";
let mut parser = Parser::new(source);
let stmts = parser.parse()?;

let mut resolver = NameResolver::new();
resolver.resolve(&stmts)?;
```

## Testing

Run tests with:
```bash
cargo test -p nevermind-name-resolver
```

Run all tests with:
```bash
cargo test
```

## Future Enhancements

- Import resolution for modules
- Type annotation resolution
- Function overload resolution
- Generic type parameter resolution
- Macro name resolution
