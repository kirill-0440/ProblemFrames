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
cargo install cargo-llvm-cov --locked
cargo llvm-cov --workspace --all-features --fail-under-lines 54
npm run compile --prefix editors/code
```

## Security Baseline

- Dependabot tracks updates for Cargo, npm, and GitHub Actions via `.github/dependabot.yml`.
- Dependabot policy uses automatic rebases and grouped editor dependency updates to reduce PR conflicts.
- `.github/workflows/security-audit.yml` runs Rust and npm dependency audits on schedule and on demand.
- `.github/workflows/codeql.yml` runs static security analysis for Rust and TypeScript.
- `.github/workflows/dependency-review.yml` gates pull requests on high-severity dependency risks.

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
  - `pf_lsp` binaries for Linux/macOS
  - Platform-specific VSIX packages (`linux-x64`, `darwin-x64`)
  - `SHA256SUMS.txt` with checksums for all release files
  - `SBOM.spdx.json` and provenance bundles (`sha256-*.jsonl`, `trusted_root.jsonl`)
- On tag push (`v*`), workflow also creates a GitHub Release and attaches all generated assets.
- On tag builds, VSIX package version is automatically aligned with the git tag (e.g. `v0.1.0` -> `0.1.0`).
- Release workflow includes smoke checks for `pf_lsp` startup, VSIX contents, and release bundle completeness.
- Platform support policy is documented in `docs/support-matrix.md`.
- Rollback procedure is documented in `docs/runbooks/release-rollback.md`.
- Supply-chain verification procedure is documented in `docs/runbooks/supply-chain-verification.md`.
- See `.github/workflows/release-artifacts.yml`.

## Engineering Metrics and Triage

- `scripts/generate_engineering_metrics_report.sh` produces weekly engineering health metrics:
  - lead time for change
  - change failure rate (proxy)
  - mean time to recovery (proxy)
  - flaky test rate (proxy)
- `.github/workflows/weekly-engineering-triage.yml` generates a weekly metrics artifact and opens a triage issue on schedule.
- Weekly triage process is documented in `docs/runbooks/weekly-triage.md`.

Generate metrics locally:

```bash
GH_TOKEN=$(gh auth token) bash ./scripts/generate_engineering_metrics_report.sh
```

## Changelog

Project history is tracked in `CHANGELOG.md`.
