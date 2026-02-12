# Contributing

Thanks for contributing to Problem Frames Toolchain.

## Development Setup

Prerequisites:
- Rust (stable toolchain)
- Node.js 20+ (for VS Code extension)

Install dependencies:

```bash
npm ci --prefix editors/code
```

## Required Checks

Run all checks before opening a PR:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
npm run compile --prefix editors/code
```

## Building Artifacts

Build VSIX package (includes Linux `pf_lsp` binary):

```bash
./scripts/build_vsix.sh
```

Install extension locally:

```bash
./scripts/install_extension.sh
```

## Release Process

- Update `CHANGELOG.md` under `[Unreleased]`.
- Create a version tag in the form `vX.Y.Z`.
- Push the tag; CI publishes release artifacts from `.github/workflows/release-artifacts.yml`.

## Pull Requests

- Keep PR scope focused.
- Add or update tests for behavior changes.
- Update docs when changing UX, commands, or workflows.

## Language Token Source of Truth

- Keep DSL token lists in `crates/pf_dsl/src/language.rs`.
- `pf_lsp` completions and VS Code syntax must stay aligned with this file.
- Sync is validated by `crates/pf_lsp/tests/language_sync.rs`.
