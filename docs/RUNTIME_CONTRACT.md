# Runtime Contract

This document describes the runtime behavior that the current Nevermind CLI actually guarantees.

## Backend

- Nevermind currently compiles to Python only.
- `run` is a compile-then-execute workflow, not a native runtime.

## `compile`

- `compile path/to/file.nm` reads the source file, runs the full compiler pipeline, and writes `path/to/file.py` unless `-o` is provided.
- When local `.nm` modules are imported, the compiler recursively writes sibling `.py` files for those dependencies before finishing the root compile.

## `run`

- `run path/to/file.nm` first performs the same compilation behavior as `compile`.
- The root module is emitted as `path/to/file.py`.
- Local imported modules are emitted as sibling `.py` files next to their `.nm` sources.
- The CLI then executes Python against the emitted root `.py` file.

## Python Interpreter Selection

- On macOS and Linux, the CLI tries `python3` and then `python`.
- On Windows, the CLI tries `python`, `python3`, and then `py`.
- If none are available, `run` fails with an interpreter-not-found error.

## Module Resolution Contract

- Local `.nm` imports are resolved relative to the importing file's directory.
- Local modules require explicit top-level `export` declarations for imported names.
- If no local `.nm` file exists for an import target, the import is preserved as an external Python import.
- Nested local imports are compiled into package-qualified Python imports so runtime behavior matches compile-time resolution.

## REPL Contract

- The REPL uses the current working directory as its module-resolution base directory.
- `use` and `from` statements persist across inputs like other definitions.
- Before executing a snippet, the REPL precompiles local `.nm` dependencies from that working directory.

## Output Artifacts

- Running or compiling a program may leave generated `.py` files alongside source `.nm` files.
- The repository ignores `examples/*.py` so routine verification does not leave noisy tracked artifacts.
