# Proposal 010: PF + WRSPM Contract Bridge (W/R/S/P/M Operationalization)

## Status

Draft

## Planning Window

- Start: February 13, 2026
- Target design freeze: July 31, 2026
- Target first delivery checkpoint: November 30, 2026

## Problem Statement

The repository now enforces strong PF structure (domains/interfaces/frame fit/decomposition) and generates obligation artifacts, but the WRSPM contract is still implicit.

Current risk:

- correctness arguments can remain structurally valid while semantic contract coverage is unclear;
- interface vocabulary discipline for `S` is not enforced as a first-class rule set;
- formal evidence can drift from requirements semantics if `W/R/S` links are not tracked explicitly as one contract.

## Decision Summary

1. Keep PF-DSL validated AST as the only semantic source of truth.
2. Add an explicit WRSPM bridge layer derived from validated PF models.
3. Introduce WRSPM-oriented contract checks (especially interface vocabulary discipline for `S`).
4. Add concern coverage and adequacy reporting as release-quality evidence.
5. Keep executable formal checks non-blocking until mismatch rate and runtime are stable.

## WRSPM Mapping Policy

Derived mapping from PF artifacts:

- `W`: `worldProperties` assertion sets plus domain assumptions.
- `R`: `requirementAssertions` sets plus requirement-level world effects.
- `S`: `specification` sets and machine-side interface commitments.
- `P`: program realization metadata (initially report-level placeholder).
- `M`: platform/runtime realization metadata (initially report-level placeholder).

Derived phenomenon partitions for interface-visible behavior:

- `ev`: environment-controlled shared phenomena visible to machine.
- `sv`: machine-controlled shared phenomena visible to environment.
- `eh`/`sh`: non-shared/internal phenomena are explicitly marked as out-of-scope in phase 1.

## Scope

### In Scope

- WRSPM bridge report generation from validated PF AST.
- rule set for `W/R/S` linkage completeness and vocabulary discipline.
- concern coverage report (`requirement -> correctnessArgument -> W/S/R sets`).
- one executable adequacy-oriented obligation class (Alloy-first unless formal decision changes).
- CI artifact publication for WRSPM bridge outputs.

### Out of Scope

- replacing PF validator with a full theorem prover;
- full semantic model of `P` and `M` in phase 1;
- mandatory external model repository/API integration;
- blocking CI on formal mismatch before stabilization criteria are met.

## Workstreams

### WS1: WRSPM Bridge Model

- add internal WRSPM projection model derived from PF AST;
- deterministic IDs linking WRSPM entities back to PF entities;
- explicit unresolved sections for artifacts not yet modeled (`eh/sh`, detailed `P/M`).

### WS2: Reporting and CLI

- add CLI mode `--wrspm-report` (Markdown);
- optional machine-readable variant `--wrspm-json` for CI diffing;
- include coverage summaries and unresolved contract entries.

### WS3: Validation Contract Extension

- add new validator rules for WRSPM bridge consistency;
- enforce `S` vocabulary checks against interface shared phenomena;
- register all new rules in metamodel catalog and rule-test matrix.

### WS4: Executable Obligation Pilot

- select one obligation class that can be checked end-to-end;
- produce expected-pass and expected-fail fixtures;
- generate differential report between Rust structural verdict and formal backend result.

### WS5: Developer Workflow Integration

- include WRSPM report in PF quality gate and PR checklist for PF-semantic changes;
- publish dogfooding WRSPM artifacts in CI;
- keep stage non-blocking until metrics justify gating policy escalation.

## Milestones and Exit Criteria

### M6: WRSPM Bridge and Coverage Contract

Deliverables:

- `--wrspm-report` (and optionally `--wrspm-json`);
- WRSPM mapping and coverage sections for `W/R/S/P/M`;
- validator and metamodel rule updates for WRSPM consistency;
- fixture suite for valid and invalid bridge cases.

Exit criteria:

- at least one dogfooding model publishes WRSPM report artifact in CI;
- every WRSPM bridge rule has one valid and one invalid fixture;
- unresolved parts are explicit and deterministic in report output.

### M7: Executable Adequacy Evidence

Deliverables:

- one executable obligation class with pass/fail fixtures;
- differential report (`rust_verdict` vs `formal_verdict`) with mismatch categories;
- non-blocking CI publication for adequacy evidence.

Exit criteria:

- at least one reproducible pass and one reproducible fail run in CI artifacts;
- mismatch report is deterministic and triage-ready;
- go/no-go memo defines criteria for moving from non-blocking to blocking.

## Risks

- over-modeling `P/M` too early and slowing adoption;
- false confidence from textual reports without executable checks;
- increased CI noise and triage burden.

## Mitigations

- phase `P/M` as explicit placeholders first, not hard constraints;
- tie report outputs to fixture-backed rule checks;
- keep formal stage non-blocking until stability thresholds are observed.

## Success Metrics

- WRSPM report generated for all dogfooding models in CI artifacts;
- 100% WRSPM bridge rules tracked in `metamodel/invariant-catalog.json`;
- at least one obligation class checked end-to-end with pass/fail evidence;
- decreasing unresolved/mismatch counts across two planning cycles.

## Dependencies

- Proposal `007` and backlog `007-execution-backlog-m1-m3.md` (`R007-M3-01`, `R007-M3-04`).
- Proposal `009` for canonical PF quality constraints.
- Proposal `008` for downstream PIM generation and transformation trace contracts.

## Decision Needed

Approve this proposal as the WRSPM contract layer that turns current PF semantics into explicit, testable `W/R/S/P/M` evidence before broader integration milestones.
