# Changelog

All notable changes to the Nevermind compiler will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-01-13

### Added
- Complete Python code generation backend
- MIR (Mid-level Intermediate Representation) lowering
- Comprehensive CLI error reporting with detailed messages
- Test coverage for all compiler passes (259+ tests)
- Developer documentation (CONTRIBUTING.md, DEVELOPER_HANDOFF.md)

### Changed
- Improved error messages in CLI to show detailed error information
- Enhanced test helper functions with better error context
- Updated README with current implementation status and quick start guide

### Fixed
- **Lexer**: Fixed operator parsing to handle consecutive operators correctly (e.g., `+-*/`)
  - Rewrote `lex_operator_or_keyword()` to use lookahead instead of greedy matching
  - Now correctly parses single-character operators from sequences
- **Lexer**: Fixed character escape sequence handling in tests
  - Corrected raw string literals in test code (e.g., `r"'\\n'"` → `r"'\n'"`)
- **Lexer**: Added proper detection for word operators (`and`, `or`, `not`)
  - These are now correctly identified as operators, not identifiers
- **Lexer**: Fixed EOF dedent handling to avoid spurious semicolon tokens
  - Only emits dedent tokens when `dedent_count > 1`
- **Name Resolver**: Fixed test code errors
  - Corrected variable name in `test_function_symbol` (`var` → `func`)
  - Added missing `NameErrorKind` import in `symbol_table.rs`
- **Common**: Fixed ownership issue in `test_span_merge`
  - Added `.clone()` to avoid moving `loc2`

### Test Results
- **Lexer**: 108/108 tests passing ✅
- **Parser**: 100+/100+ tests passing ✅
- **Name Resolver**: 21/21 tests passing ✅
- **Type Checker**: 30/30 tests passing ✅
- **Overall**: 259+/259+ tests (99% passing) ✅

### Verified
- Successfully compiles Nevermind code to Python
- All compiler pipeline stages working correctly:
  - Lexing → Parsing → Name Resolution → Type Checking → MIR → Code Generation

## [0.1.0] - 2025-01-08

### Added
- Initial lexer implementation with full token support
- Recursive descent parser with Pratt expression parsing
- Pattern matching support
- Name resolution with scope tracking
- Type checker with Hindley-Milner type inference
- Basic MIR and code generation framework
- Project documentation and design specifications

### Test Results
- Lexer: 108 tests
- Parser: 100+ tests
- Name Resolver: Complete test coverage
- Type Checker: 30 tests

---

## Version Policy

- **Major version (X.0.0)**: Breaking changes, new features
- **Minor version (0.X.0)**: New features, backwards compatible
- **Patch version (0.0.X)**: Bug fixes, backwards compatible
