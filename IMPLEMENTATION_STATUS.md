# Nevermind Implementation Status

**Version**: 1.0.0  
**Last Updated**: 2026-04-19  
**Status**: 1.0.0 release

## Summary

Nevermind 1.0.0 ships a working Rust compiler that lowers `.nm` programs to Python and exposes real `compile`, `check`, `run`, `fmt`, `lint`, and `repl` commands. This is an intentionally narrow 1.0 release: module behavior, diagnostics, examples, tests, CI, and documentation are aligned to the implementation that actually exists today.

## Verified Commands

The following commands are the current stability baseline and are expected to keep passing together:

```bash
cargo test
cargo run -- fmt --check examples/hello.nm
cargo run -- lint examples/hello.nm
cargo run -- run examples/patterns.nm
cargo run -- lint examples/modules.nm
cargo run -- run examples/modules.nm
cargo clippy -- -D warnings
```

## Implemented Surface

### Compiler Pipeline

- Lexer
- Parser
- Name resolution
- Type checking
- MIR lowering
- Python code generation

### Language Features In Regular Use

- `let` and `var`
- function definitions and calls
- lists, indexing, and indexed mutation
- `if`, `while`, and `for`
- `match` with guards
- `break`, `continue`, and `return`
- lambdas
- pipeline operator `|>`
- explicit function return type checking

### Tooling

- `compile` writes Python output
- `run` compiles and executes with Python
- `check` runs parse, resolve, and type checks without codegen
- `fmt` enforces the current source style
- `lint` runs semantic checks plus style warnings
- `repl` keeps definitions across inputs and resolves imports from the current working directory

## Module System Status

The current module system is intentionally narrow:

- Local `.nm` modules are resolved relative to the importing file.
- Imported names from local modules must come from explicit top-level `export` declarations.
- If no local `.nm` file exists, the import is treated as an external Python import.
- `run` and the REPL recursively precompile local module dependencies before execution.
- Nested local module imports now generate package-qualified Python imports so resolve/check/compile/run all agree on the same module target.

This is the supported behavior today. Re-exports, package boundaries, and a broader package system are not part of the current shipped surface.

## Release References

- Stable boundary: [docs/STABLE_BOUNDARY.md](./docs/STABLE_BOUNDARY.md)
- Runtime contract: [docs/RUNTIME_CONTRACT.md](./docs/RUNTIME_CONTRACT.md)
- Release checklist: [docs/RELEASE_CHECKLIST_1_0.md](./docs/RELEASE_CHECKLIST_1_0.md)

## Current Limitations

The following are not implemented or not ready to be advertised as supported features:

- package manager
- debugger
- async runtime
- concurrency primitives
- generics or traits
- broad `Result` / `Option` standard-library ergonomics
- stable re-export and package-boundary semantics

The standard library is still small, and diagnostics are still being tightened around real edge cases.

## Current 1.0 Focus

Post-1.0 maintenance should stay focused on:

- keeping module resolution, checking, code generation, and runtime behavior consistent
- improving return-type and module diagnostics where real mismatches still appear
- ensuring examples, README claims, tests, and CI only describe verified behavior
- adding targeted regression tests around real user-facing failures
