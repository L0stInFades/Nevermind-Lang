# Nevermind

Nevermind 1.0.0 is a deliberately small programming language release that compiles to Python. Its contract is narrow on purpose: the CLI, examples, tests, documentation, and generated Python are all aligned to the same real implementation.

## What Works Today

- `compile`, `check`, `run`, `fmt`, `lint`, and `repl` are real CLI commands.
- The compiler pipeline is live end-to-end: lexer -> parser -> name resolution -> type checking -> MIR lowering -> Python code generation.
- The language supports variables, functions, lists, indexing, mutation, `if`, `while`, `for`, `match`, lambdas, and the pipeline operator.
- Local modules use an explicit top-level `export` boundary.
- Local `.nm` imports resolve relative to the importing file.
- If no local `.nm` file exists, imports are passed through as external Python imports.
- `run` and the REPL both precompile local module dependencies before execution.

## Quick Example

```nevermind
fn main() do
  print "Hello, World!"
end
```

```bash
cargo run -- run examples/hello.nm
```

## Module Example

```nevermind
from "mathutils" import square
use "mathutils"

fn main() do
  print square(5)
  print mathutils.abs_val(-7)
end
```

Current module semantics are intentionally small and explicit:

- Importable names from local `.nm` files must be exported.
- Resolution happens relative to the importing file's directory.
- Nested local modules are compiled with package-qualified Python imports so `resolve`, `check`, `compile`, and `run` agree.

See [examples/modules.nm](./examples/modules.nm) for the checked example that the CLI runs today.

## Quick Start

```bash
cargo test
cargo run -- fmt --check examples/hello.nm
cargo run -- lint examples/hello.nm
cargo run -- run examples/patterns.nm
cargo run -- lint examples/modules.nm
cargo run -- run examples/modules.nm
cargo clippy -- -D warnings
```

You can also compile a file without running it:

```bash
cargo run -- compile examples/hello.nm
```

## Current Scope

Nevermind 1.0.0 does not claim or ship the following:

- a package manager
- a debugger
- an async runtime or concurrency primitives
- generics or traits
- a broad `Result` / `Option`-style standard-library surface
- a full package or re-export system

Those areas may be designed later, but they are not part of the 1.0.0 support surface.

## Release Boundary

- The frozen `1.0` scope is defined in [docs/STABLE_BOUNDARY.md](./docs/STABLE_BOUNDARY.md).
- The actual compile-and-run behavior is defined in [docs/RUNTIME_CONTRACT.md](./docs/RUNTIME_CONTRACT.md).
- The release gate lives in [docs/RELEASE_CHECKLIST_1_0.md](./docs/RELEASE_CHECKLIST_1_0.md).

## Repository Guide

- [examples/](./examples/) contains runnable programs kept in sync with the CLI.
- [IMPLEMENTATION_STATUS.md](./IMPLEMENTATION_STATUS.md) describes the current verified implementation status.
- [tests/](./tests/) contains end-to-end compile and runtime regression tests.
- [docs/STABLE_BOUNDARY.md](./docs/STABLE_BOUNDARY.md) defines the `1.0` support surface.
- [docs/RUNTIME_CONTRACT.md](./docs/RUNTIME_CONTRACT.md) documents emitted artifacts and Python execution behavior.
- [docs/RELEASE_CHECKLIST_1_0.md](./docs/RELEASE_CHECKLIST_1_0.md) is the `1.0` release gate.

## Status

Version `1.0.0` is a deliberately scoped release focused on:

- tightening local module semantics
- improving diagnostics and regression coverage
- keeping docs and examples honest about what really works today
