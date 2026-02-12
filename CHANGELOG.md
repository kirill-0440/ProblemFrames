# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project aims to follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- CI quality gates for formatting, linting, tests, and VS Code extension build.
- Release artifact workflow for `pf_lsp` binaries and platform-specific VSIX packages.
- Automated GitHub Release publishing with attached artifacts on version tags (`v*`).
- LSP integration tests for diagnostics and go-to-definition behavior.
- Contribution guide and pull request checklist template.
- Dependabot configuration for Cargo, npm, and GitHub Actions dependency updates.
- Scheduled/manual security audit workflow for Rust and npm dependencies.
- CodeQL static analysis workflow for Rust and TypeScript code.

### Changed
- LSP now uses in-memory document state for definition and diagnostics flow.
- Position mapping in LSP updated to UTF-16 semantics.
- Validator now reports missing required requirement fields and unsupported frames.
- DOT export now preserves all domain-pair connections from interface phenomena.
- `pf_lsp` now supports `lsp-types` 0.97 `Uri` API for path/URI handling.
- VS Code extension toolchain updated (`typescript` 5.9, `vscode-languageclient` 9.x, `@types/node` 25.x).
- CI and release workflows now use Rust dependency caching and stricter default `contents: read` permissions.
- VSIX release packaging now derives extension version from the git release tag.
- CodeQL workflow uses `github/codeql-action@v4` to stay ahead of v3 deprecation.
- GitHub Release now includes `SHA256SUMS.txt` with checksums for all published assets.
- GitHub workflows now use `actions/checkout@v6` and `actions/setup-node@v6`.
- Release workflow now uses `actions/upload-artifact@v6` and `actions/download-artifact@v7`.
- Dependabot now uses explicit auto-rebase policy and grouped editor dependency updates.
- CodeQL now runs language-specific jobs only when matching source areas changed on push/PR.

### Fixed
- VS Code extension now resolves bundled `pf_lsp` binary more robustly across platforms.
- Build/install scripts now copy platform-specific binary names (`pf_lsp`/`pf_lsp.exe`).
- VS Code extension compile no longer fails on modern Node type definitions (`skipLibCheck` in `tsconfig`).
