# Problem Frames Toolchain

A comprehensive toolchain for Jackson's Problem Frames methodology, built in Rust.

## Structure
-   `crates/pf_dsl`: Core library (AST, Parser, Validator, CodeGen).
-   `crates/pf_lsp`: Language Server Protocol implementation.
-   `editors/code`: VS Code Extension.

## Quality Gates

Run the same checks as CI:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
npm run compile --prefix editors/code
```

## Security Baseline

- Dependabot tracks updates for Cargo, npm, and GitHub Actions via `.github/dependabot.yml`.
- `.github/workflows/security-audit.yml` runs Rust and npm dependency audits on schedule and on demand.

Run security checks locally:

```bash
cargo install cargo-audit --locked
cargo audit --file Cargo.lock
npm audit --prefix editors/code --audit-level=high
```

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
cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --dot > output.dot
```
Generate Rust code:
```bash
cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --gen-rust > output.rs
```

## Problem Frames DSL
The DSL allows you to define Domains, Interfaces, and Requirements.
See `crates/pf_dsl/sample.pf` for a complete example.

## Release Artifacts

- CI on tag push (`v*`) or manual trigger publishes:
  - `pf_lsp` binaries for Linux/macOS/Windows
  - Platform-specific VSIX packages (`linux-x64`, `darwin-x64`, `win32-x64`)
- On tag push (`v*`), workflow also creates a GitHub Release and attaches all generated assets.
- On tag builds, VSIX package version is automatically aligned with the git tag (e.g. `v0.1.0` -> `0.1.0`).
- See `.github/workflows/release-artifacts.yml`.

## Changelog

Project history is tracked in `CHANGELOG.md`.
