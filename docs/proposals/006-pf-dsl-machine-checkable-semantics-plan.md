# Proposal 006: PF DSL Machine-Checkable Semantics Plan

## Status

Draft

## Planning Window

- Start: February 13, 2026
- Target completion: June 30, 2026

## Problem Statement

Current PF DSL is strong on parsing, validation, LSP, and delivery mechanics, but still mostly operates as a structural modeling tool.

To unlock formal confidence and better requirement quality, the project needs a staged move from:

- notation-level modeling (domains/interfaces/requirements)
- to semantics-aware modeling (`W`, `S`, `R`, proof obligations, frame-fit checks)

without destabilizing the current Rust production path.

## Goals

1. Introduce a PF metamodel that encodes control, domain role/kind, and subproblem decomposition explicitly.
2. Add strict static semantics for core PF invariants as first-class validator rules.
3. Represent `W`/`S`/`R` in DSL and generate machine-checkable proof obligations (`S and W entail R`).
4. Keep Rust validator and LSP as default production path while formal backend remains additive.
5. Deliver this as incremental milestones with explicit migration and cutover controls.

## Non-Goals

- Replacing Rust validator with theorem proving in this planning window.
- Building a complete theorem proving platform for all frame classes.
- Long-term dual maintenance of legacy and new DSL syntax.

## Compatibility Policy

This plan uses a controlled breaking change for DSL syntax and semantics.

- Backward compatibility with legacy `.pf` syntax is not required.
- The legacy parser/grammar is removed at cutover.
- Existing models must migrate to the new syntax before release cut.
- Migration tooling and clear diagnostics are mandatory release artifacts.

## Proposed Metamodel Extensions

### Domain Model

- Domain `kind`: `biddable | causal | lexical`
- Domain `role`: `given | designed | machine`
- Explicit machine-domain marker with single-machine invariant per problem/subproblem.

### Interface and Phenomena

- `interface` connects two or more domains.
- `phenomenon` is typed (`event | state | symbolic | causal`) and has mandatory `controlledBy`.
- `controlledBy` must reference a domain connected by the same interface.

### Requirements and References

- `requirement` supports references to domains/phenomena.
- `constrains` references are explicit and mandatory where frame semantics require them.
- Strict PF mode forbids requirement references to machine-domain.

### Subproblems and Frames

- `subproblem` entity with scoped participants and requirements.
- `frame instance` with `frameType` and fit-mapping of concrete domains to frame roles.
- Frame-fit checks as validator rules.

### Assertions and Correctness

- Assertion containers for:
  - world properties (`W`)
  - machine specification (`S`)
  - formalized requirement clauses (`R`)
- Correctness argument section:
  - `prove S and W entail R`
- Proof obligations generated even when backend checker is not available.

## Workstreams

### WS1: AST and Parser Evolution

- Extend AST with domain role/kind, `controlledBy`, subproblems, assertions, and obligations.
- Replace legacy syntax with the v2 grammar and parser.
- Add migration diagnostics and conversion tooling for repository models.

### WS2: Static Semantics and Validator Hardening

- Implement PF invariants as deterministic validator rules:
  - one machine per scope
  - machine not `given`
  - lexical not machine
  - biddable constrained restrictions
  - interface multiplicities
  - phenomenon controller consistency
  - strict-mode requirement-to-machine constraints
- Add rule severity levels: `error`, `warning`, `info`.

### WS3: Multi-File Source Mapping and LSP Accuracy

- Attach source file identity to semantic diagnostics and references.
- Fix imported-file range attribution in diagnostics and go-to-definition.
- Add precise parse-error range mapping from parser errors.

### WS4: Frame-Fit and Decomposition

- Implement frame-fit checks for:
  - RequiredBehavior
  - CommandedBehavior
  - InformationDisplay
  - SimpleWorkpieces
  - Transformation
- Add decomposition-aware checks at subproblem boundaries.

### WS5: W/S/R and Proof Obligation Pipeline

- Add DSL blocks for `W`, `S`, `R` assertions.
- Generate explicit obligations (`obl_*.json` or `obl_*.md`) from models.
- Validate structural consistency of obligations before external solving.

### WS6: First Formal Backend (Non-Blocking)

- Start with one backend (recommended: Alloy or TLA+).
- Build translator for a minimal, deterministic subset.
- Run backend checks in CI as non-blocking informational stage.

### WS7: Tooling, Docs, and Adoption

- Update diagrams export to include controller semantics and frame instances.
- Publish user guide for strict PF mode vs engineering mode.
- Add migration guide from current DSL to extended DSL.

## Milestones and Exit Criteria

### M1 (Weeks 1-2): Metamodel v2 Skeleton

Deliverables:

- AST/parser support for new entities and fields.
- Legacy syntax removed; v2 grammar is the only accepted syntax.

Exit criteria:

- New syntax fixtures pass parser tests.
- Repository dogfooding and sample models are migrated and parse in v2 format.

### M2 (Weeks 3-4): Core Invariants in Validator

Deliverables:

- Core PF invariant checks implemented with clear diagnostics.
- Strict-mode toggle and rule severities.

Exit criteria:

- At least 15 new semantic tests cover invariant failures and valid cases.
- CI remains green with no flaky increase.

### M3 (Weeks 5-6): LSP and Source Precision

Deliverables:

- File-aware diagnostics and definition navigation for imports.
- Accurate parser error ranges in LSP.

Exit criteria:

- Multi-file integration tests prove correct URI+range attribution.
- No known false-positive cross-file definition jumps.

### M4 (Weeks 7-8): Frames and Subproblems

Deliverables:

- Subproblem model support.
- Frame-fit checks for all five core frame classes.

Exit criteria:

- Dogfooding models can express at least two decomposed subproblems.
- Frame-fit checks detect at least one intentionally invalid fixture per frame.

### M5 (Weeks 9-10): W/S/R and Obligations

Deliverables:

- Assertion blocks and obligation generator.
- Obligation artifacts published by CI for sample models.

Exit criteria:

- For each dogfooding model, at least one obligation artifact is produced.
- Obligation schema validation has zero errors in CI.

### M6 (Weeks 11-12): Formal Backend Spike

Deliverables:

- One backend translator operational on scoped subset.
- One end-to-end checked property from a real model.

Exit criteria:

- CI runs backend check in non-blocking mode.
- Decision memo created: continue, pause, or expand backend.

## Backlog Breakdown (Execution-Ready)

### Epic A: Language and AST

- A1: Add domain `kind` and `role` enums to AST.
- A2: Add `controlledBy` to phenomenon model.
- A3: Add subproblem/frame/assertion nodes.
- A4: Introduce migration diagnostics and syntax conversion script.

### Epic B: Validation

- B1: Implement one-machine and role consistency checks.
- B2: Implement interface/phenomenon multiplicity checks.
- B3: Implement controller-in-interface membership checks.
- B4: Implement strict PF requirement-reference constraints.
- B5: Implement frame-fit rule engine.

### Epic C: LSP and Diagnostics

- C1: Carry `source_path` through semantic error model.
- C2: Publish diagnostics against owning document URI.
- C3: Parse-error location mapping to LSP ranges.
- C4: Multi-file definition navigation safety fixes.

### Epic D: Semantics and Formalization

- D1: Add `W`/`S`/`R` assertion blocks to DSL.
- D2: Define obligation schema and generator.
- D3: Implement first backend translator for scoped subset.
- D4: Add non-blocking CI checks and artifact upload.

### Epic E: Documentation and Operations

- E1: Strict PF mode guide and engineering mode guide.
- E2: Migration guide with before/after examples.
- E3: Update proposal and release documentation.
- E4: Add dogfooding models that exercise new semantics.

## Risks

- Scope creep from mixing DSL evolution and formal methods.
- Usability regression if strict rules are always-on.
- CI cost growth from extra semantic/formal checks.
- Migration friction during syntax cutover.

## Mitigations

- Keep formal backend non-blocking until model quality stabilizes.
- Support two modes: strict PF and engineering-friendly.
- Add path-based CI execution and incremental gates.
- Provide migration tooling, staged cutover branch policy, and pre-release model migration sweep.

## Success Metrics

- 90% of semantic diagnostics include precise file+range metadata.
- Zero known false-positive cross-file go-to-definition incidents.
- At least 25 new tests covering PF invariants and frame-fit semantics.
- At least one model with generated obligations and one checked formal property.
- No increase in weekly flaky-rate baseline above agreed threshold.
- 100% repository `.pf` models migrated to v2 syntax before release cut.

## Decision Needed

Approve this plan as the execution baseline for PF DSL semantic maturity work through June 2026.
