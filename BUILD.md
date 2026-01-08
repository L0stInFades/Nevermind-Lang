# Building Nevermind

## Prerequisites

### Install Rust

Nevermind is written in Rust. You need to install Rust first:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Or visit [rustup.rs](https://rustup.rs/) for detailed instructions.

### Verify Installation

```bash
rustc --version
cargo --version
```

## Building the Project

```bash
# Clone the repository
git clone https://github.com/nevermind-lang/nevermind.git
cd nevermind

# Build the project
cargo build --workspace

# Run tests
cargo test --workspace

# Build with optimizations
cargo build --release --workspace
```

## Development

```bash
# Check code without building
cargo check --workspace

# Format code
cargo fmt --all

# Run linter
cargo clippy --workspace

# Run specific crate tests
cargo test -p nevermind-lexer
cargo test -p nevermind-parser
```

## IDE Setup

### VS Code

Install the following extensions:
- rust-analyzer
- CodeLLDB
- Even Better TOML
- Error Lens

### IntelliJ IDEA / CLion

Install the Rust plugin.

## Project Structure

```
nevermind/
├── crates/
│   ├── common/      # Shared types and utilities
│   ├── ast/         # AST definitions
│   ├── lexer/       # Lexer (tokenizer)
│   └── parser/      # Parser
├── src/             # Main binary (CLI)
├── tests/           # Integration tests
├── examples/        # Example programs
└── Cargo.toml       # Workspace configuration
```

## Troubleshooting

### "error: linker `link.exe` not found" (Windows)

You need to install Microsoft C++ Build Tools:
- Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/
- Install "Desktop development with C++"

### "error: failed to compile"

Try:
```bash
cargo clean
cargo build --workspace
```

### Out of memory during compilation

If you encounter memory issues:
```bash
# Limit parallel jobs
export CARGO_BUILD_JOBS=2
cargo build --workspace
```
