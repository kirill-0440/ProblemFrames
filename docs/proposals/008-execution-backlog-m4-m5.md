# Proposal 008 Execution Backlog (M4a-M5a)

## Status

Draft

## Scope

Execution backlog for milestones from `docs/proposals/008-pf-ddd-sysmlv2-integration.md`:

- M4a: Marks and Validation Contract
- M4b: File-Based PIM Generation
- M4c: Trace Contract and CI Publication
- M5a: Controlled API Bridge Spike

## Ownership Model

- `DSL Maintainer`: AST/parser/validator/fixtures
- `Codegen Maintainer`: DDD and SysML generators
- `Traceability Maintainer`: trace-map and impact integration
- `Platform Maintainer`: CI and release artifact wiring

## Backlog Items

### M4a - Marks and Validation Contract

#### R008-M4A-01 - Add AST and Parser Support for Marks

- Proposed issue title: `R008 M4a: Add mark syntax and AST support for DDD/SysML`
- Owner: `DSL Maintainer`
- Priority: `P1`
- Definition of Done:
- [x] Introduce optional marks metadata in AST nodes.
- [x] Extend parser grammar for annotation/mark syntax.
- [x] Add parser fixtures for valid and malformed marks.

#### R008-M4A-02 - Add Mark Consistency Validation

- Proposed issue title: `R008 M4a: Implement mark consistency validator rules`
- Owner: `DSL Maintainer`
- Priority: `P1`
- Definition of Done:
- [x] Add validator rule IDs for conflicting/missing mark prerequisites.
- [x] Add invalid fixtures for each new rule.
- [x] Keep strict PF semantics unchanged for unmarked models.

#### R008-M4A-03 - Document Marking Guide

- Proposed issue title: `R008 M4a: Publish PF marking guide for DDD and SysML`
- Owner: `DSL Maintainer`
- Priority: `P2`
- Definition of Done:
- [x] Add concise guide with examples and anti-patterns.
- [x] Link guide from `crates/pf_dsl/README.md` and proposals index docs.

### M4b - File-Based PIM Generation

#### R008-M4B-01 - Implement DDD-PIM Generator

- Proposed issue title: `R008 M4b: Add --ddd-pim generator output`
- Owner: `Codegen Maintainer`
- Priority: `P1`
- Definition of Done:
- [x] Add CLI mode `--ddd-pim`.
- [x] Generate bounded-context map, command/event inventory, and aggregate candidates.
- [x] Add deterministic output tests.

#### R008-M4B-02 - Implement SysML v2 Text Generator

- Proposed issue title: `R008 M4b: Add --sysml2-text generator output`
- Owner: `Codegen Maintainer`
- Priority: `P1`
- Definition of Done:
- [x] Add CLI mode `--sysml2-text`.
- [x] Map PF requirements/interfaces to SysML textual elements.
- [x] Add fixture-based output snapshots.

#### R008-M4B-03 - Implement SysML v2 JSON Generator

- Proposed issue title: `R008 M4b: Add --sysml2-json generator output`
- Owner: `Codegen Maintainer`
- Priority: `P1`
- Definition of Done:
- [x] Add CLI mode `--sysml2-json`.
- [x] Align output with targeted SysML JSON schema version.
- [x] Add schema validation job or test hook.

### M4c - Trace Contract and CI Publication

#### R008-M4C-01 - Add Source-to-Target Trace Map

- Proposed issue title: `R008 M4c: Generate trace-map.json for DDD/SysML outputs`
- Owner: `Traceability Maintainer`
- Priority: `P1`
- Definition of Done:
- [x] Generate stable `trace-map.json` containing PF IDs and generated target IDs.
- [x] Ensure all generated elements are mapped.
- [x] Fail CI on unmapped generated elements.

#### R008-M4C-02 - Integrate with Existing Impact Exports

- Proposed issue title: `R008 M4c: Connect DDD/SysML traces with impact analysis`
- Owner: `Traceability Maintainer`
- Priority: `P2`
- Definition of Done:
- [x] Extend impact report to include generated DDD/SysML targets.
- [x] Add one cross-model fixture demonstrating change propagation.

#### R008-M4C-03 - CI Artifact Publication

- Proposed issue title: `R008 M4c: Publish DDD/SysML bundles and trace artifacts`
- Owner: `Platform Maintainer`
- Priority: `P2`
- Definition of Done:
- [x] Upload generation outputs and trace bundles in CI artifacts.
- [x] Add artifact naming/versioning convention.

### M5a - Controlled API Bridge Spike

#### R008-M5A-01 - Create `pf_sysml_api` Crate Skeleton

- Proposed issue title: `R008 M5a: Scaffold pf_sysml_api crate`
- Owner: `Platform Maintainer`
- Priority: `P2`
- Definition of Done:
- [x] Add crate with basic client abstractions and config wiring.
- [x] Include smoke command for push/pull test path.

#### R008-M5A-02 - Non-Blocking API Smoke Job

- Proposed issue title: `R008 M5a: Add non-blocking CI smoke job for SysML API bridge`
- Owner: `Platform Maintainer`
- Priority: `P2`
- Definition of Done:
- [x] Add non-blocking workflow job with explicit env gating.
- [x] Publish smoke logs as CI artifacts.

#### R008-M5A-03 - Decision Memo After Spike

- Proposed issue title: `R008 M5a: Record go/no-go memo for API bridge`
- Owner: `Platform Maintainer`
- Priority: `P3`
- Definition of Done:
- [x] Document runtime, reliability, and maintenance observations.
- [x] Recommend continue/pause/expand decision for next planning cycle.

## Sequencing and Dependencies

- Complete M4a before enabling M4b outputs by default.
- M4c starts after first stable outputs from M4b.
- M5a starts only after two successful CI cycles with M4b/M4c artifacts.
- Start execution only after `R007-M3-04` is closed (canonical concern coverage baseline).
- Reuse `010` M6 contract outputs when available; this is recommended for quality but not a hard blocker.

## Tracking Recommendation

- Labels:
  - `roadmap:008`
  - `milestone:m4a`, `milestone:m4b`, `milestone:m4c`, `milestone:m5a`
  - `owner:dsl`, `owner:codegen`, `owner:traceability`, `owner:platform`
- Require one explicit DRI (GitHub handle) for each backlog item before status changes to `in_progress`.
