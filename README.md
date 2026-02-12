# Problem Frames Toolchain

A comprehensive toolchain for Jackson's Problem Frames methodology, built in Rust.

## Structure
-   `crates/pf_dsl`: Core library (AST, Parser, Validator, CodeGen).
-   `crates/pf_lsp`: Language Server Protocol implementation.
-   `editors/code`: VS Code Extension.

## Getting Started

### Prerequisites
-   Rust (stable)
-   Node.js (for VS Code extension)

### Installation
1.  **Build the Toolchain**:
    ```bash
    cargo build --release
    ```

2.  **Install VS Code Extension**:
    ```bash
    ./scripts/install_extension.sh
    ```

## Usage
### CLI
Generate a diagram:
```bash
cargo run -p pf_dsl -- sample.pf --dot > output.dot
```
Generate Rust code:
```bash
cargo run -p pf_dsl -- sample.pf --gen-rust > output.rs
```

## Problem Frames DSL
The DSL allows you to define Domains, Interfaces, and Requirements.
See `examples/` for more.
