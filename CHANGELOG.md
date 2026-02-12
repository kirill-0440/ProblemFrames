# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project aims to follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Engineering metrics report generator script (`scripts/generate_engineering_metrics_report.sh`) with weekly baseline snapshot (lead time, change failure proxy, MTTR proxy, flaky-rate proxy).
- Weekly engineering triage workflow (`.github/workflows/weekly-engineering-triage.yml`) to publish metrics artifacts and open scheduled triage issues.
- Weekly engineering triage runbook (`docs/runbooks/weekly-triage.md`).
- Dependency review workflow for pull requests (`.github/workflows/dependency-review.yml`) with high-severity gate.
- Release pipeline now generates SBOM (`SBOM.spdx.json`) and provenance bundles (`sha256-*.jsonl`, `trusted_root.jsonl`).
- Supply-chain verification runbook (`docs/runbooks/supply-chain-verification.md`) for checksum + provenance validation.
- CI quality gates for formatting, linting, tests, and VS Code extension build.
- Release artifact workflow for `pf_lsp` binaries and platform-specific VSIX packages.
- Automated GitHub Release publishing with attached artifacts on version tags (`v*`).
- LSP integration tests for diagnostics and go-to-definition behavior.
- Contribution guide and pull request checklist template.
- Dependabot configuration for Cargo, npm, and GitHub Actions dependency updates.
- Scheduled/manual security audit workflow for Rust and npm dependencies.
- CodeQL static analysis workflow for Rust and TypeScript code.
- Dependabot policy workflow to enforce expected metadata on Dependabot PRs.
- Dogfooding roadmap model (`crates/pf_dsl/dogfooding/roadmap_q1.pf`) for planning system evolution in PF DSL.
- Script to generate Markdown reports from dogfooding PF models (`scripts/generate_dogfooding_reports.sh`).
- Platform support matrix document (`docs/support-matrix.md`).
- Release rollback runbook (`docs/runbooks/release-rollback.md`).
- VS Code extension packaging ignore rules (`editors/code/.vscodeignore`) and local extension license file (`editors/code/LICENSE`).

### Changed
- Critical GitHub Actions workflows now pin action refs to immutable commit SHAs.
- CI now installs `cargo-llvm-cov` via `cargo install --locked` instead of a floating action ref.
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
- CI now enforces a Rust line-coverage gate via `cargo llvm-cov` (`--fail-under-lines 54`).
- CodeQL now uses workflow-level concurrency cancellation for stale branch runs.
- CI now validates all dogfooding `.pf` models to keep self-models parseable and semantically valid.
- CI now uploads generated dogfooding Markdown reports as build artifacts.
- Release workflow now includes artifact smoke checks (binary startup, VSIX contents, and release bundle integrity).
- Windows release artifacts are temporarily paused; Linux/macOS remain supported targets.
- Support matrix now includes explicit Windows re-enable criteria and smoke-plan checklist.

### Fixed
- Release bundle smoke-check now matches VSIX filenames that include target plus version suffix.
- VS Code extension now resolves bundled `pf_lsp` binary more robustly across platforms.
- Build/install scripts now copy platform-specific binary names (`pf_lsp`/`pf_lsp.exe`).
- VS Code extension compile no longer fails on modern Node type definitions (`skipLibCheck` in `tsconfig`).
