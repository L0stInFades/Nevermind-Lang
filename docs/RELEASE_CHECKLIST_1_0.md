# 1.0 Release Checklist

This checklist is the release gate that was used to change Nevermind from pre-1.0 stabilization to `1.0.0`.

## Implementation Gate

- [x] The stable boundary in `docs/STABLE_BOUNDARY.md` still matches the real implementation.
- [x] The runtime contract in `docs/RUNTIME_CONTRACT.md` still matches the real CLI behavior.
- [x] No root-level documentation advertises features outside the stable boundary.
- [x] Module resolution, checking, code generation, and runtime behavior agree for local modules.
- [x] Return-type diagnostics still catch non-exhaustive `if` / `match` paths that can fall through.

## Verification Gate

- [x] `cargo test`
- [x] `cargo run -- fmt --check examples/hello.nm`
- [x] `cargo run -- lint examples/hello.nm`
- [x] `cargo run -- run examples/patterns.nm`
- [x] `cargo run -- lint examples/modules.nm`
- [x] `cargo run -- run examples/modules.nm`
- [x] `cargo clippy -- -D warnings`

## Review Gate

- [x] README, implementation status, examples, and CLI behavior are consistent.
- [x] High-risk module regression tests cover nested `from`, nested `use`, and relative-local precedence.
- [x] Generated example artifacts do not create noisy tracked changes.
- [x] Release version and release messaging only changed after every gate above was green.

## Release Rule

Every checkbox above was intentionally confirmed before renaming the project state to `1.0.0`.
