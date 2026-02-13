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
cargo install cargo-llvm-cov --locked
cargo llvm-cov --workspace --all-features --fail-under-lines 54
npm run compile --prefix editors/code
```

## PF Methodology Gate (for `.pf` changes)

If your PR changes PF models or PF semantics, run the methodology gate:

```bash
bash ./scripts/run_pf_quality_gate.sh <model.pf> [more models...]
```

This enforces the current PF-first workflow in executable form:

- structural report generation;
- decomposition closure check (uncovered/orphan/boundary);
- obligations and formal backend artifact generation;
- traceability/impact artifacts.

For intentional exploratory work with open decomposition items, use:

```bash
bash ./scripts/run_pf_quality_gate.sh --allow-open-closure <model.pf>
```

Document any such exception in the PR.

Optional but recommended security checks:

```bash
cargo install cargo-audit --locked
cargo audit --file Cargo.lock
npm audit --prefix editors/code --audit-level=high
```

Optional engineering metrics snapshot:

```bash
GH_TOKEN=$(gh auth token) bash ./scripts/generate_engineering_metrics_report.sh
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

Smoke test scripts (without heavy build/install side effects):

```bash
bash ./scripts/smoke_test_scripts.sh
```

Generate dogfooding reports from PF models:

```bash
bash ./scripts/generate_dogfooding_reports.sh
```

## Release Process

- Update `CHANGELOG.md` under `[Unreleased]`.
- Create a version tag in the form `vX.Y.Z`.
- Push the tag; CI publishes release artifacts and creates a GitHub Release from `.github/workflows/release-artifacts.yml`.
- VSIX artifact version is derived from the pushed tag during release workflow.
- Release job also publishes `SHA256SUMS.txt`, `SBOM.spdx.json`, and provenance bundles for integrity verification.
- Release workflow smoke-checks supported artifacts (binary startup, VSIX contents, release bundle integrity).
- Security audit workflow (`.github/workflows/security-audit.yml`) runs weekly and can be triggered manually.
- CodeQL workflow (`.github/workflows/codeql.yml`) runs static analysis on pushes/PRs to `main`.
- Dependency review workflow (`.github/workflows/dependency-review.yml`) runs on PRs and fails on high-severity dependency issues.
- Weekly engineering triage workflow (`.github/workflows/weekly-engineering-triage.yml`) publishes metrics artifacts and creates a scheduled triage issue.
- Supported platform matrix: `docs/support-matrix.md`.
- Rollback procedure for broken releases: `docs/runbooks/release-rollback.md`.
- Supply-chain verification runbook: `docs/runbooks/supply-chain-verification.md`.
- Weekly triage runbook: `docs/runbooks/weekly-triage.md`.

## Dependabot Merge/Rebase Policy

- Preferred merge method for Dependabot PRs: `squash`.
- Rebase policy: keep Dependabot branches rebased on latest `main` (`rebase-strategy: auto` in `.github/dependabot.yml`).
- CI policy check (`Dependabot Policy`) enforces expected Dependabot PR metadata (target branch, branch naming, labels).
- Batch handling:
  - Merge PRs with green CI/CodeQL and no behavioral regressions.
  - For API-breaking dependency updates, push compatibility fixes to the Dependabot branch, then merge.
  - If duplicate PRs exist for the same dependency, merge the primary PR and close duplicates as superseded.

## Pull Requests

- Keep PR scope focused.
- Add or update tests for behavior changes.
- Update docs when changing UX, commands, or workflows.
- Include PF quality-gate evidence for PF model/semantics changes.

PF methodology runbook: `docs/runbooks/pf-methodology-quality-gate.md`.

## Language Token Source of Truth

- Keep DSL token lists in `crates/pf_dsl/src/language.rs`.
- `pf_lsp` completions and VS Code syntax must stay aligned with this file.
- Sync is validated by `crates/pf_lsp/tests/language_sync.rs`.
