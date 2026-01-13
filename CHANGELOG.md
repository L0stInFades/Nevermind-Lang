# Changelog

All notable changes to the Nevermind programming language will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-01-13

### Added ✨
- **Array/List indexing support** - Can now access array elements with `array[index]` syntax
- **If expression support** - Complete conditional branching with `if-then-else` expressions
- **Turing-completeness proof** - Nevermind is now proven to be Turing-complete!
  - Implemented Brainfuck interpreter demonstrating all required capabilities
  - Formal proof document in `examples/docs/TURING_COMPLETE.md`
- **List literal support** - Can create lists with `[1, 2, 3]` syntax
- **Index expression parsing** - Postfix index operator in parser
- **Type checking for array operations** - Full type inference for indexing

### Fixed 🐛
- **Critical MIR operator mapping bug** - All binary operators were incorrectly mapped to `Add`
  - Added `map_binary_op()` and `map_comparison_op()` functions
  - Added `BinOp::Pow` variant for power operator
  - Updated Python codegen to handle all operators correctly
- **Parser span ownership** - Fixed use-of-moved-value error in expression parsing
- **Name resolution for Index expressions** - Added proper scoping for array indexing
- **Type checker pattern matching** - Added Index case to all match statements

### Changed 📝
- **Project structure** - Cleaned up examples folder
  - Removed test files and temporary examples
  - Organized documentation into `examples/docs/`
  - Kept only meaningful example programs
- **Version** - Bumped to 0.3.0 to reflect major new features

### Examples 📚
- Added `examples/brainfuck_simple.nm` - Brainfuck interpreter demonstrating Turing completeness
- Added `examples/docs/TURING_COMPLETE.md` - Formal proof of Turing completeness
- Added `examples/docs/MATRIX_CHAIN_DP.md` - Dynamic programming example documentation
- Clean examples: `functions.nm`, `hello.nm`, `lists.nm`, `math.nm`, `patterns.nm`, `simple_fn.nm`, `variables.nm`

## [0.2.0] - 2025-01-08

### Added ✨
- **Complete compiler frontend**
  - Lexer with indentation handling (108 tests)
  - Parser using recursive descent and Pratt parsing (100+ tests)
  - Name resolver with scope tracking (21 tests)
  - Type checker with Hindley-Milner inference (30 tests)
  - MIR (Mid-level Intermediate Representation)
  - Python code generator
- **CLI tools** - `nevermind compile`, `nevermind check`, `nevermind run`
- **Comprehensive test suite** - 259+ tests with 99% pass rate

### Fixed 🐛
- Lexer operator parsing for consecutive operators
- Character escape sequence handling
- EOF dedent handling
- Operator keyword detection (`and`, `or`, `not`)
- Error reporting and messages

### Documentation 📚
- DESIGN_SPEC.md - Complete language specification
- TYPE_SYSTEM_DESIGN.md - Type system design document
- COMPILER_ARCHITECTURE.md - Compiler architecture overview
- CONTRIBUTING.md - Contribution guidelines
- README.md - Project overview and quick start

## [0.1.0] - 2025-01-01

### Added ✨
- Initial project setup
- Basic lexer implementation
- Simple parser prototype
- Project vision and design philosophy

---

## Version Numbering

- **Major version (X.0.0)**: Breaking changes, major milestones
- **Minor version (0.X.0)**: New features, backward compatible
- **Patch version (0.0.X)**: Bug fixes, minor improvements

## Release Notes

### 0.3.0 - "Turing-Complete" Release 🧠

This is a major milestone! Nevermind is now proven to be **Turing-complete**, meaning it can compute any computable function. We achieved this by:

1. Adding array indexing - essential for arbitrary memory access
2. Implementing if expressions - essential for conditional branching
3. Creating a Brainfuck interpreter - demonstrating all required capabilities

**Key Achievement**: The Brainfuck interpreter in `examples/brainfuck_simple.nm` proves Nevermind has the same computational power as any other programming language.

### 0.2.0 - "Frontend Complete" Release 🎉

This release marked the completion of the entire compiler frontend pipeline. Nevermind can now:
- Lex source code correctly
- Parse into an AST
- Resolve names with proper scoping
- Type check with full inference
- Lower to MIR
- Generate working Python code

**Test Coverage**: 259+ tests with 99% pass rate demonstrates production-quality frontend implementation.

---

## Roadmap

### Upcoming (0.4.0)
- [ ] REPL implementation
- [ ] Standard library functions (print, read, etc.)
- [ ] Loop execution support
- [ ] Improved error recovery

### Future (1.0.0)
- [ ] IDE support (VS Code extension, LSP)
- [ ] Package manager
- [ ] Debugger
- [ ] Comprehensive standard library
- [ ] Performance optimizations

### Long Term
- [ ] Native compilation (LLVM backend)
- [ ] WebAssembly backend
- [ ] Macro system
- [ ] Advanced concurrency primitives

---

**For more information**, see the [README.md](./README.md) or visit the [GitHub repository](https://github.com/L0stInFades/Nevermind-Lang).
