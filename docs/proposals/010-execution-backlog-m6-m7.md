# Proposal 010 Execution Backlog (M6-M7)

## Status

Draft

## Scope

Execution backlog for milestones from `docs/proposals/010-pf-wrspm-contract-bridge.md`:

- M6: WRSPM Bridge and Coverage Contract
- M7: Executable Adequacy Evidence

## Ownership Model

- `DSL Maintainer`: WRSPM projection model, parser/validator invariants, fixtures
- `Tooling Maintainer`: CLI/report generation and quality gate integration
- `Formal Track Owner`: executable obligation class and differential verdict logic
- `Platform Maintainer`: CI artifact publication and non-blocking governance

## Backlog Items

### M6 - WRSPM Bridge and Coverage Contract

#### R010-M6-01 - Add WRSPM Projection Model

- Proposed issue title: `R010 M6: Add WRSPM projection model from validated PF AST`
- Owner: `DSL Maintainer`
- Priority: `P1`
- Definition of Done:
- [x] Add deterministic WRSPM projection structures (`W/R/S/P/M`) derived from validated AST.
- [x] Add projection tests for at least one multi-subproblem model.
- [x] Ensure unresolved `eh/sh` and detailed `P/M` sections are explicitly represented.

#### R010-M6-02 - Add WRSPM CLI Reports

- Proposed issue title: `R010 M6: Add --wrspm-report (and optional --wrspm-json)`
- Owner: `Tooling Maintainer`
- Priority: `P1`
- Definition of Done:
- [x] Add CLI mode `--wrspm-report` with deterministic Markdown output.
- [x] Add optional machine-readable `--wrspm-json` output for CI consumption.
- [x] Document command usage in `README.md` and `crates/pf_dsl/README.md`.

#### R010-M6-03 - Enforce WRSPM Vocabulary Discipline for Specification

- Proposed issue title: `R010 M6: Add validator rules for S vocabulary on shared interface phenomena`
- Owner: `DSL Maintainer`
- Priority: `P1`
- Definition of Done:
- [x] Add validator rules ensuring `S` references only shared interface vocabulary representation.
- [x] Add valid/invalid fixtures including one intentional violation case.
- [x] Register rule IDs and test mapping in metamodel contract files.

#### R010-M6-04 - Concern Coverage Report and Gate Wiring

- Proposed issue title: `R010 M6: Extend concern coverage baseline with WRSPM contract mapping`
- Owner: `Tooling Maintainer`
- Priority: `P1`
- Definition of Done:
- [x] Reuse `R007-M3-04` concern coverage output as baseline input (no duplicate engine).
- [x] Extend output with WRSPM contract links (`W/R/S` plus unresolved `P/M` placeholders).
- [x] Integrate extended concern coverage in `scripts/run_pf_quality_gate.sh` with controlled exception path.

### M7 - Executable Adequacy Evidence

#### R010-M7-01 - Select Executable Obligation Class

- Proposed issue title: `R010 M7: Select first executable adequacy-oriented obligation class`
- Owner: `Formal Track Owner`
- Priority: `P1`
- Definition of Done:
- [x] Write short rationale memo for selected obligation class and assumptions.
- [x] Tie selection explicitly to `R007-M3-01` outputs.
- [x] Define expected-pass and expected-fail fixture templates.

#### R010-M7-02 - Implement Formal Check Path

- Proposed issue title: `R010 M7: Implement executable pass/fail check path for selected obligation`
- Owner: `Formal Track Owner`
- Priority: `P1`
- Definition of Done:
- [x] Implement executable check path (Alloy-first unless decision memo changes).
- [x] Add one passing and one failing fixture with deterministic verdict.
- [x] Publish check output as machine-readable artifact.

#### R010-M7-03 - Add Differential Verdict Report

- Proposed issue title: `R010 M7: Add rust-vs-formal differential verdict report`
- Owner: `Formal Track Owner`
- Priority: `P1`
- Definition of Done:
- [x] Generate differential report per model/obligation with mismatch categories.
- [x] Include clear triage keys (model ID, obligation ID, verdict pair).
- [x] Keep report generation non-blocking in initial rollout.

#### R010-M7-04 - CI Artifact and Governance Wiring

- Proposed issue title: `R010 M7: Publish WRSPM and adequacy artifacts in non-blocking CI stage`
- Owner: `Platform Maintainer`
- Priority: `P2`
- Definition of Done:
- [x] Upload WRSPM and adequacy artifacts with stable naming in CI.
- [x] Add runbook update describing rerun and triage procedure.
- [x] Record go/no-go checkpoint criteria for future blocking gate decision.

#### R010-M7-05 - Formal Closure Referential Integrity

- Proposed issue title: `R010 M7: Enforce requirement formal.argument references to declared correctness arguments`
- Owner: `DSL Maintainer`
- Priority: `P1`
- Definition of Done:
- [x] Extend requirement-mark validation to reject `@formal.argument("...")` references to undefined correctness arguments.
- [x] Keep canonical formal-closure mapping model-derived from requirement marks.
- [x] Add valid/invalid tests for the referential integrity contract.

#### R010-M7-06 - LeanAtom Subset Entailment Mode

- Proposed issue title: `R010 M7: Formalize LeanAtom subset entailment when requirement projection is covered by S/W`
- Owner: `Formal Track Owner`
- Priority: `P1`
- Definition of Done:
- [x] Extend Lean coverage formalization decision with subset entailment mode (in addition to mirror mode).
- [x] Emit closed-coverage and entailment theorems for subset-covered requirement projections.
- [x] Publish mode in coverage JSON and cover with deterministic tests.

## Sequencing and Dependencies

- `R010-M6-04` depends on closure of `R007-M3-04` concern coverage baseline.
- `R010-M7-01` and `R010-M7-02` depend on `R007-M3-01` executable obligation baseline.
- M7 starts only after at least one M6 model artifact is published in CI.

## Tracking Recommendation

- Labels:
  - `roadmap:010`
  - `milestone:m6`, `milestone:m7`
  - `owner:dsl`, `owner:tooling`, `owner:formal`, `owner:platform`
- Require one explicit DRI (GitHub handle) for each backlog item before status changes to `in_progress`.
