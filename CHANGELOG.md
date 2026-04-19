# Changelog

All notable changes to the Nevermind programming language will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-04-19

### Added
- `docs/STABLE_BOUNDARY.md` to freeze the supported 1.0 surface.
- `docs/RUNTIME_CONTRACT.md` to document the real compile-and-run contract.
- `docs/RELEASE_CHECKLIST_1_0.md` as the 1.0 release gate record.
- Runtime regression coverage for nested local `from`, nested local `use`, and same-name relative module precedence.
- Type-checker regression coverage for non-exhaustive `if` / `match` statements that fall through to later values.

### Changed
- Root crate version, CLI `--version`, and REPL banner now converge on `1.0.0`.
- README and implementation status now describe Nevermind as an intentionally scoped `1.0.0` release instead of a pre-1.0 stabilization build.

### Fixed
- Nested local namespace imports now preserve the Nevermind module binding name in generated Python (`import pkg.bar as bar`).
- Nested local imports continue to resolve relative to the importing module at runtime, matching compile-time resolution.

## [0.5.0] - 2026-04-15

### Added

#### Algorithm & Data Structure Examples (this session)
- **`binary_search.nm`** — 折半查找 (Binary Search)
  - Iterative + recursive variants, 14 tests
  - Covers: empty array, single element, first/middle/last, not found (below/above/between), negatives, large array, all-same
- **`divide_conquer.nm`** — 分治算法 (Divide and Conquer)
  - `dc_max` / `dc_min` / `dc_minmax` — extremum via divide and conquer
  - `dc_sum` — divide-and-conquer summation
  - `power` — fast exponentiation (binary exponentiation)
  - `dc_count_positive` — D&C counting
  - `dc_max_subarray` — maximum subarray with cross-midpoint merge
  - 14 tests, all passing
- **`merge_sort.nm`** — 归并排序 (Merge Sort)
  - `sublist` helper (manual slicing via while loop), `merge` (using `take_left` flag), `merge_sort` (recursive top-down)
  - 14 tests including stability verification
- **`quick_sort.nm`** — 快速排序 (Quick Sort)
  - Functional variant (filter-based partitioning + list concatenation)
  - Three pivot strategies: middle, first-element, last-element
  - In-place variant: `partition` + `quick_sort_inplace`
  - 15 tests
- **`generalized_list.nm`** — 广义表 (Generalized List)
  - Flat integer encoding: positive int = atom, `0` = open sublist, `-1` = close sublist
  - Operations: `gl_length`, `gl_depth`, `gl_head_is_atom`, `gl_head_atom`, `gl_tail`, `gl_equal`, `gl_copy`, `gl_atom_count`, `gl_to_str`
  - 15 tests including depth-4 nesting, chain tail, equality, copy
  - Avoids heterogeneous-list type restriction by encoding structure as `List[Int]`

#### Earlier Algorithm Examples (undocumented in v0.4.0)
- **`bubble_sort.nm`** — 冒泡排序, in-place using IndexAssign
- **`insertion_sort.nm`** — 直接插入排序 (10 tests)
- **`shell_sort.nm`** — 希尔排序 (10 tests)
- **`radix_sort.nm`** — 基数排序 LSD (10 tests)
- **`multi_key_radix_sort.nm`** — 多关键字基数排序 (8 tests, 3-level priority)

#### Earlier Data Structure Examples (undocumented in v0.4.0)
- **`linked_list.nm`** — nil/cons linked list with `car`, `cdr`, `list_append`, `list_concat`, `list_map`
- **`seq_list.nm`** — 顺序表 sequential list with insert/delete/search
- **`seq_stack.nm`** — 顺序栈 array-backed stack with push/pop/peek
- **`circ_queue.nm`** — 循环队列 circular queue with modular wrap-around
- **`sparse_poly.nm`** — 稀疏多项式 sparse polynomial with `poly_add`, `poly_eval`
- **`sparse_matrix.nm`** — 稀疏矩阵 sparse matrix with transpose and matrix-vector multiply

#### Module Examples (undocumented)
- **`greet.nm`**, **`mathutils.nm`**, **`modules.nm`** — module system demonstration
- **GitHub Actions CI workflow** — runs `cargo test` plus CLI smoke checks for `fmt`, `lint`, and `run examples/patterns.nm`

### Changed
- Root crate version, CLI `--version`, and REPL banner now converge on `0.5.0`
- Implementation status docs now describe 0.5.0 as a pre-1.0 stabilization release instead of leaving 0.4.0 and 0.5.0 mixed together

### Fixed
- **Integer division** — `/` now correctly generates Python floor division `//` (ensures int/int → int semantics)
- **`IndexAssign`** (`arr[i] = v`) — added to MIR `MirStmt`, `lowering.rs`, and Python codegen; enables in-place mutation of list elements in all sorting algorithms
- **While loop parser** — fixed a double-`end` consumption bug that caused `while` bodies to be parsed incorrectly
- **Match codegen** — fixed incorrect Python `match`/`case` generation
- **`print` with keyword arguments** — fixed generated Python output
- **REPL multi-line tracking** — fixed continuation detection for `do...end` blocks
- **For-loop lowering** — corrected variable binding in generated Python
- **Type inference** — improved inference for several edge cases

### Language Gotchas Documented
- Variables must **not** start with `not`, `or`, `and` — the lexer greedily matches these as operator keywords (e.g., `not_found` → token `not` + identifier `_found`)
- A list literal `[...]` appearing on its own line immediately after an `if...then...else...end` expression is parsed as postfix indexing of that expression; work around by assigning to a variable first: `var tmp = [a, b]` then `tmp`
- String type annotation is `String`, not `Str`
- `else` in `do`-form (`if cond do ... end else do ... end end`) requires an extra trailing `end`; prefer expression-form `if cond then a else b end` for simple branches

---

## [0.4.0] - 2026-02-08

### Added
- **Interactive REPL** - `nevermind repl` now runs the full compilation pipeline interactively
  - Persistent definitions: `fn`, `let`, `var` declarations are remembered across inputs
  - Expression evaluation: non-definition input is compiled, executed, and output is displayed
  - Multi-line support: `do...end`, `match...end` blocks auto-detect continuation (`... ` prompt)
  - REPL commands: `:help`, `:clear` (reset definitions), `:defs` (show stored definitions)
  - Silent compilation via `compile_source_silent()` (lex → parse → resolve → typecheck → MIR → codegen)
  - Temp-file Python execution with cross-platform interpreter discovery
  - Auto-strips `if __name__ == "__main__"` guard from generated code
- **Complete MIR/Codegen pipeline for runtime support** - End-to-end compilation now works for all examples
- **MIR statement control flow types** - `If`, `While`, `For`, `Return`, `Break`, `Continue`, `Match` variants in `MirStmt`
- **MIR match arm and pattern types** - `MirMatchArm`, `MirPattern::Constructor` for pattern matching
- **Python codegen for all statement types** - If/elif/else, while, for, return, break, continue, match/case
- **Function body lowering** - Smart detection of `print` calls to avoid spurious `return print(...)` wrapping
- **Auto `main()` entry point** - Generated Python includes `if __name__ == "__main__": main()` when `main` function exists
- **17 end-to-end compilation tests** in `tests/compile_tests.rs`
- **Cross-platform Python discovery** - `nevermind run` tries `python`, `python3`, and `py` (Windows launcher)

### Changed
- Bumped version to 0.4.0
- Example files updated to use single `end` (removed double `end` from parser grammar change)
- `MirStmt` enum expanded from 3 to 10 variants
- `MirExprStmt` expanded with `If`, `While`, `For`, `Break`, `Continue` for nested control flow

### Fixed
- Example files (`hello.nm`, `functions.nm`, `variables.nm`, `lists.nm`, `patterns.nm`, `simple_fn.nm`) fixed for current parser grammar
- `nevermind run` now works on Windows by trying `py` launcher
- Name resolver duplicate definition test fixed for built-in shadowing

### Verified
- `nevermind run examples/hello.nm` produces "Hello, World!"
- `nevermind run examples/functions.nm` produces correct factorial(5)=120, fibonacci(10)=55
- `nevermind run examples/math.nm` produces 30
- `nevermind run examples/variables.nm` produces correct output
- 296 tests passing across all workspace crates

## [0.3.1] - 2026-01-13

### Added
- Lexer boundary tests for tab indentation rejection and chained dedent emission.
- End-to-end typing edge cases in `tests/edge_cases.rs` covering mixed lists, map key constraints, and pipeline stage typing.

### Changed
- Parser allows an optional trailing `end` after function bodies and keeps `|>` pipelines distinct from `|...|` lambdas.
- Documentation reorganized: legacy reports moved to `docs/archive/`, with `DOCUMENTATION_INDEX.md` and `docs/ARCHIVE.md` providing a clean map and historical summary.

### Fixed
- `!` is now recognized as the logical `not` operator in the lexer.

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

### 1.0.0 - "Converged Python Backend" Release (CURRENT)
- [x] Stable boundary frozen and documented
- [x] Runtime contract documented
- [x] Release checklist fully green
- [x] Nested local module runtime behavior aligned with compile-time semantics
- [x] Root release messaging and implementation status aligned to the shipped surface

### 0.5.0 - "Tooling & Convergence" Release
- [x] Complete MIR/Codegen pipeline for all control flow
- [x] End-to-end compilation and execution
- [x] Built-in functions (print, len, input, range, str, int)
- [x] Working `fmt` / `lint` CLI commands
- [x] Basic CI smoke checks
- [x] 296 tests passing

### Upcoming (1.1.0+)
- [ ] Standard library expansion
- [ ] Improved error recovery
- [ ] REPL enhancements (tab completion, command history, persistent Python session)

### Future
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
