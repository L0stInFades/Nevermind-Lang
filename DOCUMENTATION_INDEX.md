# Documentation Map

> Start here for navigation, onboarding, and historical context.

## Quick Path for New Contributors
- Project overview & language tour: `README.md`
- Build & run: `BUILD.md`, `QUICKSTART.md`
- Contributing & workflow: `CONTRIBUTING.md`, `DEVELOPER_HANDOFF.md`

## Core Specifications
- Language design: `DESIGN_SPEC.md`
- Type system: `TYPE_SYSTEM_DESIGN.md`
- Standard library: `STANDARD_LIBRARY.md`
- Compiler architecture: `COMPILER_ARCHITECTURE.md`
- Runtime system: `RUNTIME_DESIGN.md`
- Tooling: `TOOLCHAIN.md`

## Status & Planning
- Current implementation status: `IMPLEMENTATION_STATUS.md`
- Roadmap: `ROADMAP.md`
- Change history & highlights: `CHANGELOG.md`

## Module References
- Name resolver: `crates/name-resolver/README.md`, `crates/name-resolver/IMPLEMENTATION_SUMMARY.md`
- Type checker: `crates/type-checker/README.md`

## Learning & Examples
- Examples: `examples/` (e.g., `examples/brainfuck_simple.nm`, `examples/functions.nm`)
- Tests as executable documentation:
  - Lexer coverage: `crates/lexer/tests/lexer_tests.rs` (now includes tab-indentation rejection and multi-dedent handling)
  - Parser coverage: `crates/parser/tests/parser_tests.rs`
  - End-to-end typing edge cases: `tests/edge_cases.rs`

## Recent Updates (Feb 2026 - v0.4.0)
- Complete MIR/codegen pipeline: all control flow (if/while/for/match/return/break/continue) compiles to Python.
- Built-in function support in name resolver and type checker (print, len, range, input, str, int, etc.).
- Recursive function support via type annotation pre-declaration.
- 17 end-to-end compilation tests in `tests/compile_tests.rs`.
- `nevermind run examples/hello.nm` produces "Hello, World!".
- 296 tests, 100% pass rate.
- Cross-platform Python discovery (python/python3/py).

## Archives
- High-signal summaries of older reports: `docs/ARCHIVE.md`
- Full historical reports moved to `docs/archive/` (daily log, parser fix progress, demo report, etc.).
