# Proposal 008: PF + DDD + SysML v2 Integration (CIM -> PIM -> PSM)

## Status

Draft

## Planning Window

- Planning start: February 13, 2026
- Target design freeze: June 30, 2026
- Target first delivery checkpoint: October 31, 2026

## Execution Preconditions

- `R007-M3-04` (frame concern coverage baseline) is closed.
- `010` M6 contract outputs are available for reuse in transformation trace quality checks.

## Problem Statement

`PF-DSL` is already strong as a CIM-grade problem/context model and now needs a controlled path to PIM/PSM artifacts without losing semantic integrity.

Current risk:

- design and implementation artifacts can drift from validated PF models
- traceability becomes post-factum documentation instead of a generated artifact
- teams may duplicate intent across PF, architecture docs, and code skeletons

## Decision Summary

1. Keep PF-DSL as the only source of truth for CIM.
2. Add explicit bridge marks for DDD and SysML v2 in PF models.
3. Generate PIM artifacts (DDD + SysML) from validated AST only.
4. Publish mandatory transformation trace artifacts in CI.
5. Defer bidirectional sync and API publishing until file-based generators are stable.

## Layering Model

- CIM: PF-DSL (`problem`, domains, interfaces, requirements, subproblems, `W/S/R` assertions)
- PIM (software): DDD candidates (bounded contexts, aggregates, commands/events, app-service skeletons)
- PIM (systems): SysML v2 model package (requirements/context/interfaces/behavior skeleton)
- PSM: platform-specific service/deployment realizations and generated implementation stubs

## Scope

### In Scope

- PF marks for DDD and SysML mapping guidance
- validator rules for mark consistency and conflict detection
- new generators:
  - `--ddd-pim`
  - `--sysml2-text`
  - `--sysml2-json`
- trace artifacts:
  - `trace-map.json` (`pf_id -> ddd_id/sysml_id`)
  - impact-compatible relationship exports
- staged adoption with CI artifact publication

### Out of Scope

- replacing PF semantic validation with SysML tooling
- full automatic inference of DDD tactical design as final truth
- bidirectional model synchronization in initial rollout
- mandatory external SysML repository/API integration in first phase

## PF Marking Strategy

Add optional bridge marks to PF declarations:

- DDD:
  - `@ddd.bounded_context("...")`
  - `@ddd.aggregate_root`
  - `@ddd.entity`
  - `@ddd.value_object`
  - `@ddd.domain_event`
- SysML:
  - `@sysml.requirement`
  - `@sysml.block`
  - `@sysml.port`
  - `@sysml.signal`

Rules:

- marks are optional and must not weaken strict PF semantics
- marks are advisory for transformation precision, not an alternate source of truth
- conflicting marks are validator errors with rule IDs

## Workstreams

### WS1: Language and Validator Extension

- AST support for optional `marks` metadata
- parser support for mark syntax
- validator checks for incompatible mark combinations and missing prerequisites

### WS2: DDD-PIM Generator

- bounded-context candidate extraction from domain boundaries and marks
- command/event inventory from interface phenomena
- aggregate/app-service candidate skeleton output
- deterministic serialization for reviewable diffs

### WS3: SysML v2 Generator

- textual SysML v2 output (`.sysml`)
- JSON model output aligned with current SysML JSON schema expectations
- mapping from PF requirements/interfaces to SysML requirement/interface constructs

### WS4: Transformation Trace and Impact

- stable element IDs for PF source entities
- mandatory `trace-map.json` for each generated artifact set
- compatibility with existing traceability/impact outputs from proposal `007`

### WS5: Optional API Bridge (After Generator Stability)

- dedicated crate `pf_sysml_api` for publishing/reading model packages via Systems Modeling API
- API bridge enabled only after two stable release cycles of file-based generation

## Milestones and Exit Criteria

### M4a: Marks and Validation Contract

Deliverables:

- mark syntax and AST support
- mark consistency validator rules with stable rule IDs
- fixtures for valid/invalid mark combinations

Exit criteria:

- at least 12 new tests for mark parsing/validation
- no regression in existing strict PF fixtures

### M4b: File-Based PIM Generation

Deliverables:

- `--ddd-pim` output package
- `--sysml2-text` and `--sysml2-json` outputs
- deterministic output checks in CI

Exit criteria:

- at least one dogfooding model generates all three outputs
- generation is deterministic across repeated CI runs

### M4c: Trace Contract and CI Publication

Deliverables:

- `trace-map.json` generation for DDD and SysML outputs
- CI artifact publication with trace and impact bundles
- mismatch checks for missing source-target mapping coverage

Exit criteria:

- 100% generated DDD/SysML elements have PF trace references
- CI fails if trace coverage drops below threshold

### M5a: Controlled API Bridge Spike

Deliverables:

- `pf_sysml_api` crate scaffold and one push/pull smoke path
- non-blocking CI job for API integration smoke check
- short decision memo: continue, pause, or expand

Exit criteria:

- one reproducible API smoke run against configured environment
- go/no-go decision recorded in proposal update

## Risks

- model zoo effect (multiple diverging truths)
- low-confidence auto-inferred DDD outputs
- schema/version drift for SysML JSON representations
- CI cost increase from multi-target generation

## Mitigations

- enforce PF-as-source policy and mandatory trace artifacts
- label DDD outputs as candidate models for review
- pin schema targets per release and publish compatibility matrix
- keep API bridge and heavy checks non-blocking at first

## Success Metrics

- 100% of generated DDD/SysML elements mapped back to PF IDs
- at least one production-like dogfooding model reviewed through PF -> DDD/SysML flow
- zero known cases where generated artifacts bypass PF validation
- stable generation diffs (no unexplained nondeterminism) for two consecutive release cycles

## Dependencies

- Proposal `007` (traceability and design bridge foundation)
- Proposal `009` (PF canonical alignment addendum for concern/decomposition gates)
- Proposal `006` (validator semantics baseline)
- Backlog prerequisite: `R007-M3-04` from `007-execution-backlog-m1-m3.md`
- Proposal `010` outputs are consumed as quality inputs, but `008` is not a dependency of `010`.

## Decision Needed

Approve this proposal as the integration track for extending PF-DSL from CIM authority to DDD/SysML v2 PIM outputs with traceable transformation contracts.
