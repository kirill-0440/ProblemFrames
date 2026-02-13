# Proposal 007 Execution Backlog (M1-M3)

## Status

Draft

## Scope

Execution backlog for near-term milestones from `docs/proposals/007-paper-aligned-roadmap-adaptation.md`:

- M1: Metamodel Contract as Versioned Spec
- M2: Traceability Graph and Impact Analysis
- M3: End-to-End Formal Check for a Real Obligation

## Ownership Model

- `DSL Maintainer`: parser/AST/validator and fixture coverage
- `Tooling Maintainer`: CLI/reporting/export and CI artifacts
- `LSP Maintainer`: editor-side impact navigation and UX
- `Formal Track Owner`: backend check and differential verification

## Backlog Items

### M1 - Metamodel Contract

#### R007-M1-01 - Invariant Catalog (Machine-Readable)

- Proposed issue title: `R007 M1: Add invariant catalog with stable rule IDs`
- Owner: `DSL Maintainer`
- Priority: `P1`
- Definition of Done:
- [ ] Add catalog file with stable rule IDs, severity, rationale, and references to validator paths.
- [ ] Cover all active validator rules (duplicates, role/kind rules, frame-fit, subproblem, assertion/correctness).
- [ ] Add schema-level sanity check in tests or CI script for catalog format.
- [ ] Link catalog from `docs/proposals/007-paper-aligned-roadmap-adaptation.md`.

#### R007-M1-02 - Rule-to-Test Trace Matrix

- Proposed issue title: `R007 M1: Build validator rule to fixture trace matrix`
- Owner: `DSL Maintainer`
- Priority: `P1`
- Definition of Done:
- [ ] Add matrix file mapping `rule_id -> valid fixtures -> invalid fixtures`.
- [ ] Ensure each rule has at least one pass and one fail fixture.
- [ ] Add CI check that reports missing fixture coverage for any rule.

#### R007-M1-03 - Fixture Gap Closure

- Proposed issue title: `R007 M1: Close fixture coverage gaps for strict PF invariants`
- Owner: `DSL Maintainer`
- Priority: `P2`
- Definition of Done:
- [ ] Add missing fixtures for uncovered rules from the matrix.
- [ ] Keep fixture names deterministic and grouped by rule ID.
- [ ] Verify `cargo test -p pf_dsl` stays green.

### M2 - Traceability and Impact

#### R007-M2-01 - Traceability Graph Builder

- Proposed issue title: `R007 M2: Implement AST relationship graph builder`
- Owner: `Tooling Maintainer`
- Priority: `P1`
- Definition of Done:
- [ ] Add relationship graph extraction for requirement/domain/interface/phenomenon/subproblem links.
- [ ] Add unit tests for graph construction on at least one multi-subproblem model.
- [ ] Reuse resolved/validated AST as the only source of truth.

#### R007-M2-02 - CLI Export for Matrix + Impact

- Proposed issue title: `R007 M2: Add traceability markdown and csv exports`
- Owner: `Tooling Maintainer`
- Priority: `P1`
- Definition of Done:
- [ ] Add CLI modes `--traceability-md` and `--traceability-csv`.
- [ ] Emit impact report for changed requirement/domain input list.
- [ ] Document command usage in `crates/pf_dsl/README.md`.

#### R007-M2-03 - LSP/UX Hook for Impact Navigation

- Proposed issue title: `R007 M2: Surface impacted requirements in editor workflow`
- Owner: `LSP Maintainer`
- Priority: `P2`
- Definition of Done:
- [ ] Add one command or code action to query impacted requirements for selected symbol.
- [ ] Add integration test for cross-file model with imports.
- [ ] Ensure diagnostics/navigation performance is not regressed in current tests.

### M3 - Executable Obligation Check

#### R007-M3-01 - Promote One Obligation Class to Executable Check

- Proposed issue title: `R007 M3: Make one obligation class formally executable`
- Owner: `Formal Track Owner`
- Priority: `P1`
- Definition of Done:
- [ ] Select one obligation class and document selection rationale.
- [ ] Implement executable check path (Alloy-first unless decision memo changes).
- [ ] Provide one expected-pass and one expected-fail model fixture.

#### R007-M3-02 - Differential Rust vs Formal Verdict Report

- Proposed issue title: `R007 M3: Add differential verdict report for formal checks`
- Owner: `Formal Track Owner`
- Priority: `P1`
- Definition of Done:
- [ ] Generate report comparing Rust validator verdict and formal backend verdict.
- [ ] Include mismatch categorization and model/obligation identifiers.
- [ ] Publish report as CI artifact in non-blocking stage.

#### R007-M3-03 - CI Integration and Operational Guardrails

- Proposed issue title: `R007 M3: Wire formal check into non-blocking CI stage`
- Owner: `Tooling Maintainer`
- Priority: `P2`
- Definition of Done:
- [ ] Add workflow job with explicit non-blocking behavior and artifact upload.
- [ ] Document rerun procedure in `docs/formal-backend/README.md`.
- [ ] Record go/no-go checkpoint criteria after two weeks of CI runs.

## Sequencing and Dependencies

- M1 items must close before M2 and M3 are considered release-candidate complete.
- M2 and M3 can execute in parallel after `R007-M1-01` and `R007-M1-02` are merged.
- M3 remains non-blocking until mismatch rate and runtime are acceptable at triage.

## Tracking Recommendation

- Create one GitHub project view keyed by labels:
  - `roadmap:007`
  - `milestone:m1`, `milestone:m2`, `milestone:m3`
  - `owner:dsl`, `owner:tooling`, `owner:lsp`, `owner:formal`
