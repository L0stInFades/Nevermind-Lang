# Stable Boundary For 1.0

This document defines the implementation surface that Nevermind can honestly freeze for a `1.0.0` release.

## Included In The Stable Boundary

### CLI Commands

- `compile`
- `check`
- `run`
- `fmt`
- `lint`
- `repl`

### Compiler Pipeline

- lexing `.nm` source
- parsing to AST
- name resolution
- type checking
- MIR lowering
- Python code generation

### Language Surface

- `let` and `var`
- function definitions and calls
- lists, indexing, and indexed mutation
- `if`, `while`, and `for`
- `match` with guards
- `break`, `continue`, and `return`
- lambdas
- pipeline operator `|>`
- explicit function return annotations and corresponding diagnostics

### Module Semantics

- local `.nm` module resolution is relative to the importing file
- local imports are gated by explicit top-level `export`
- if no local `.nm` file exists, the import falls back to an external Python import
- `run` and `repl` precompile local `.nm` dependencies before Python execution
- nested local imports keep the same target across resolve, check, compile, and run

## Explicitly Outside 1.0

The following are not part of the `1.0.0` promise and must not be advertised as shipped features:

- package manager
- debugger
- async runtime
- concurrency primitives
- generics or traits
- broad `Result` / `Option` standard-library ergonomics
- re-exports and a larger package-boundary system
- non-Python backends

## Release Rule

If a feature is not listed under "Included In The Stable Boundary", it is not part of the release promise for `1.0.0`.
