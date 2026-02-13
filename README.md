# ProblemFrames

ProblemFrames is a Rust-based toolchain for modeling and validating software problems using Jackson Problem Frames.
It is designed as a production-oriented pipeline: model -> validate -> analyze -> generate artifacts -> integrate in editor/CI.

## Current Capabilities

### PF-DSL (core modeling language)

Implemented in `crates/pf_dsl`:

- PF v2 grammar with imports, domain `kind/role`, interfaces, and typed phenomena with `controlledBy`
- requirements with core frame types:
  - `RequiredBehavior`
  - `CommandedBehavior`
  - `InformationDisplay`
  - `SimpleWorkpieces`
  - `Transformation`
- subproblem decomposition (`machine`, participants, requirement scope)
- machine-checkable assertion blocks and correctness arguments:
  - `worldProperties` (`W`)
  - `specification` (`S`)
  - `requirementAssertions` (`R`)
  - `correctnessArgument` (`S and W entail R`)

### Validation and Semantics

Strict PF validation runs by default:

- role/kind consistency and single-machine constraints
- interface/phenomenon integrity checks
- controller consistency (`controlledBy`)
- frame-fit checks for the five core frames
- subproblem boundary checks
- correctness-argument reference/scope checks
- source-aware diagnostics across imported files

Guides:

- `docs/pf-mode-guide.md`
- `docs/migration-v2.md`
- `metamodel/README.md`

### CLI Outputs

Available modes:

```bash
pf_dsl <file.pf> [--dot | --dot-context | --dot-problem | --dot-decomposition | --report | --gen-rust | --obligations | --alloy | --lean-model | --lean-coverage-json | --formal-closure-map-tsv | --requirements-tsv | --correctness-arguments-tsv | --traceability-md | --traceability-csv | --decomposition-closure | --concern-coverage | --wrspm-report | --wrspm-json | --ddd-pim | --sysml2-text | --sysml2-json | --trace-map-json] [--impact=requirement:<name>,domain:<name>] [--impact-hops=<n>]
```

Artifact generation currently includes:

- DOT diagram exports (`--dot`, `--dot-context`, `--dot-problem`, `--dot-decomposition`)
- structured model report (`--report`)
- decomposition closure report (`--decomposition-closure`)
- proof-obligation markdown (`--obligations`)
- Alloy model export (`--alloy`)
- Lean model export for research track (`--lean-model`)
- Lean formal coverage export (`--lean-coverage-json`)
- Requirement-to-correctness-argument closure map export (`--formal-closure-map-tsv`)
- Requirement inventory export (`--requirements-tsv`)
- Correctness-argument inventory export (`--correctness-arguments-tsv`)
- traceability markdown/CSV exports (`--traceability-md`, `--traceability-csv`)
- WRSPM bridge report (`--wrspm-report`)
- WRSPM bridge JSON (`--wrspm-json`)
- Rust code skeleton generation (`--gen-rust`)

### LSP and VS Code

Implemented in `crates/pf_lsp` and `editors/code`:

- diagnostics on open/change with unsaved-buffer support
- go-to-definition across files/imports
- completion aligned with PF language tokens
- VS Code extension packaging and release artifacts

## Quick Start

### Prerequisites

- Rust (stable)
- Node.js (for VS Code extension)

### Build

```bash
cargo build --release
```

### Run the CLI on sample model

```bash
cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --report
cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --obligations
cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --alloy > model.als
cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --lean-model > model.lean
cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --lean-coverage-json
cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --decomposition-closure
cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --wrspm-report
cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --traceability-md --impact=domain:Controller --impact-hops=2
cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --dot > model.dot
cargo run -p pf_dsl -- models/system/tool_spec.pf --report
cargo run -p pf_dsl -- models/system/tool_spec.pf --obligations
```

### Install VS Code extension

```bash
./scripts/install_extension.sh
```

## Repository Layout

- `crates/pf_dsl`: AST, parser, resolver, validator, generators
- `crates/pf_lsp`: language server
- `editors/code`: VS Code extension
- `models`: repository-level PF models (including the canonical system model)
- `docs/proposals`: product and engineering roadmap proposals
- `docs/adoption`: onboarding, positioning, and pilot evidence assets
- `metamodel`: machine-readable invariant catalog and rule-to-test contract
- `theory`: Lean research-track project scaffold
- `docs/runbooks`: operational playbooks (release rollback, supply chain, triage)
- `scripts`: local automation for reports, obligations, metrics, and smoke checks

## Engineering and Security Baseline

### CI-equivalent quality checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo install cargo-llvm-cov --locked
cargo llvm-cov --workspace --all-features --fail-under-lines 54
npm run compile --prefix editors/code
```

### Security checks

```bash
cargo install cargo-audit --locked
cargo audit --file Cargo.lock
npm audit --prefix editors/code --audit-level=high
```

Security workflows and policies are defined in:

- `.github/workflows/security-audit.yml`
- `.github/workflows/codeql.yml`
- `.github/workflows/dependency-review.yml`
- `.github/dependabot.yml`

## Release and Operations

Release pipeline (`.github/workflows/release-artifacts.yml`) publishes:

- `pf_lsp` binaries (Linux/macOS)
- platform-specific VSIX packages
- checksums (`SHA256SUMS.txt`)
- SBOM and provenance bundles

Operational docs:

- `docs/support-matrix.md`
- `docs/runbooks/release-rollback.md`
- `docs/runbooks/supply-chain-verification.md`
- `docs/runbooks/weekly-triage.md`

## Roadmap

The roadmap is maintained as proposal documents in `docs/proposals`.

- `docs/proposals/005-v0.2.0-scope-and-exit-criteria.md`
  - current release window scope (`v0.2.0`), reliability, dogfooding loop, validator/LSP quality
- `docs/proposals/006-pf-dsl-machine-checkable-semantics-plan.md`
  - semantic hardening baseline (`W/S/R`, invariants, formal backend staging)
- `docs/proposals/007-paper-aligned-roadmap-adaptation.md`
  - paper-driven prioritization (traceability, executable obligations, design bridge)
- `docs/proposals/007-execution-backlog-m1-m3.md`
  - execution-ready backlog for near-term milestones
- `metamodel/invariant-catalog.json`
  - authoritative M1 invariant contract (`rule_id -> validator mapping -> tests`)
- `docs/proposals/008-pf-ddd-sysmlv2-integration.md`
  - PF -> DDD/SysML v2 integration track (`CIM -> PIM -> PSM`)
- `docs/proposals/008-execution-backlog-m4-m5.md`
  - execution-ready backlog for marks, generators, trace contract, API spike
- `docs/proposals/009-pf-canonical-retro-addendum.md`
  - canonical PF alignment addendum (explicit PF views, decomposition closure, concern coverage gate)
- `docs/proposals/010-pf-wrspm-contract-bridge.md`
  - WRSPM (`W/R/S/P/M`) operational bridge from PF artifacts to explicit contract evidence
- `docs/proposals/010-execution-backlog-m6-m7.md`
  - execution-ready backlog for WRSPM bridge coverage and executable adequacy evidence

## Dogfooding and Reporting

Generate internal artifacts from dogfooding PF models:

```bash
bash ./scripts/generate_dogfooding_reports.sh
bash ./scripts/generate_obligation_reports.sh
DOGFOODING_TRIAGE_MODE=all ./scripts/generate_dogfooding_triage_report.sh
bash ./scripts/run_adoption_demo.sh
bash ./scripts/generate_pilot_evidence_report.sh
```

Canonical model of this toolchain:

```bash
cargo run -p pf_dsl -- models/system/tool_spec.pf --report
bash ./scripts/run_pf_quality_gate.sh models/system/tool_spec.pf
bash ./scripts/check_system_model.sh
bash ./scripts/run_lean_formal_check.sh --model models/system/tool_spec.pf --min-formalized-args 2
bash ./scripts/run_lean_differential_check.sh --model models/system/tool_spec.pf
bash ./scripts/check_requirement_formal_closure.sh --model models/system/tool_spec.pf --lean-coverage-json .ci-artifacts/system-model/tool_spec.lean-coverage.json
bash ./scripts/generate_formal_gap_report.sh --model models/system/tool_spec.pf --closure-rows-tsv .ci-artifacts/system-model/tool_spec.formal-closure.rows.tsv --traceability-csv .ci-artifacts/system-model/tool_spec.traceability.csv
```

Generate engineering metrics:

```bash
GH_TOKEN=$(gh auth token) bash ./scripts/generate_engineering_metrics_report.sh
```

## Contributing and History

- Contribution guide: `CONTRIBUTING.md`
- Changelog: `CHANGELOG.md`
