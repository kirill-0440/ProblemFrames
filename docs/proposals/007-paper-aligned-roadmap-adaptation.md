# Proposal 007: Paper-Aligned Roadmap Adaptation (PF DSL -> Product Value)

## Status

Draft

## Planning Window

- Start: February 13, 2026
- Target review checkpoint: May 31, 2026
- Target completion checkpoint: September 30, 2026

## Why This Proposal Exists

`006-pf-dsl-machine-checkable-semantics-plan.md` already defines the semantics hardening track.
This proposal adapts that track using the papers in `docs/papers` and the current implementation baseline, so execution focuses on the highest-value gaps instead of re-planning already delivered work.

Canonical PF retro constraints are captured in `009-pf-canonical-retro-addendum.md`.

## Input Papers (Repository Set)

- `docs/papers/A_Formal_Metamodel_for_Problem_Frames.pdf`
- `docs/papers/re02.hall.pdf`
- `docs/papers/3476-9113-1-PB.pdf`
- `docs/papers/Enhancing_Software_Development_Efficiency_An_Autom.pdf`
- `docs/papers/Twer07-souza.pdf`
- `docs/papers/How-do-you-frame-ill-defined-problems-A-study-on-creative-logics-in-action.pdf`
- `docs/papers/JLAMP2021.pdf`

## Current Baseline (Already in Repo)

The repository already covers a substantial part of PF semantics and tooling:

- metamodel-level fields in AST (domain kind/role, controlledBy, subproblems, assertion sets, correctness arguments)
- strict validator invariants and frame-fit checks for core frame classes
- decomposition validation for subproblems
- `W/S/R` representation and obligation rendering (`--obligations`)
- Alloy artifact generation (`--alloy`) as first formal backend output
- source-aware semantic diagnostics for imports in LSP
- migration and strict-mode docs (`docs/migration-v2.md`, `docs/pf-mode-guide.md`)

## Adaptation Principles

1. Keep Rust validator and LSP as production path.
2. Treat formal backends as additive evidence, not merge blockers (until proven stable).
3. Convert paper ideas into shippable, testable artifacts (schema, report, command, fixture, CI signal).
4. Prioritize value visible to users: impact analysis, traceability, design handoff, and rationale capture.

## Adapted Milestones

### M1: Metamodel Contract as Versioned Spec

Paper anchor: Formal metamodel + invariants.

Deliverables:

- explicit invariant catalog (stable rule IDs, severity, rationale, fixture links)
- validator-to-spec trace table (`rule_id -> code path -> tests`)
- canonical fixture suite with `valid` and `invalid` examples per invariant
- repository anchor:
  - `metamodel/invariant-catalog.json`
  - `metamodel/rule-test-matrix.tsv`
  - `metamodel/README.md`

Exit criteria:

- every semantic rule in validator is listed in one machine-readable catalog file
- each catalog rule is covered by at least one failing fixture and one passing fixture

### M2: Traceability Graph and Impact Analysis

Paper anchor: PF-oriented traceability matrix.

Deliverables:

- relationship graph extraction from AST:
  - requirement <-> domain
  - requirement <-> interface/phenomenon
  - subproblem <-> requirement/domain
- report/export commands:
  - `--traceability-md`
  - `--traceability-csv`
- impact ranking from changed requirement/domain to affected elements

Exit criteria:

- dogfooding model generates relationship matrix and impact report in CI artifacts
- one LSP command (or CLI helper) shows impacted requirements for a selected element

### M3: End-to-End Formal Check for a Real Obligation

Paper anchor: machine-checkable assurance over PF models.

Deliverables:

- promote one obligation class from comment-only artifact to executable formal check
- keep Alloy first (or Lean if track switches), but ensure one real pass/fail case is checked in CI
- differential note between Rust verdict and formal backend result

Exit criteria:

- at least one repository model has a checked obligation with reproducible result
- CI publishes formal check artifact and mismatch summary (non-blocking)

### M4: PF -> UML Bridge (Pragmatic Codegen)

Paper anchor: PF2UML transformation.

Deliverables:

- add one design-export backend (`PlantUML` or `Mermaid`) with deterministic output
- cover at least:
  - context-level interactions
  - requirement-to-design mapping comments/metadata
- optional semantic annotations for richer export without making them mandatory for base modeling

Exit criteria:

- one dogfooding PF model exports to design diagram artifact in CI
- trace link from PF elements to generated diagram elements is preserved in report output

### M5: Architectural Services + Pattern Packs

Paper anchor: architecture as world context + PF with patterns.

Deliverables:

- machine-domain extension for declaring architectural dependencies/services
- pattern library templates (importable PF snippets) for recurring frame situations
- recommendation rules that map frame/domain combinations to candidate pattern templates

Exit criteria:

- at least three pattern templates validated by parser and validator fixtures
- one dogfooding model demonstrates machine service assumptions referenced in correctness arguments

### M6: Rationale Layer for Ill-Defined Framing

Paper anchor: creative framing logics for ill-defined problems.

Deliverables:

- optional blocks for:
  - assumptions
  - alternatives
  - rationale
  - open questions
- report section that flags unresolved assumptions/questions

Exit criteria:

- rationale elements are optional and do not break strict PF semantic checks
- review report highlights unresolved framing decisions before release cut

### Optional Track: Process-Model Correctness Backend

Paper anchor: BPMN well-structuredness/safeness/soundness.

Deliverables:

- only if workflow export is introduced:
  - process artifact export
  - correctness checks in non-blocking CI stage

Exit criteria:

- explicit go/no-go memo confirms whether process-model track should continue

## Sequencing Against Existing Proposals

- `005-v0.2.0-scope-and-exit-criteria.md` remains the release gate for near-term scope.
- `006-pf-dsl-machine-checkable-semantics-plan.md` remains the semantic execution backbone.
- `007-execution-backlog-m1-m3.md` is the execution-ready backlog for near-term milestone delivery.
- `009-pf-canonical-retro-addendum.md` captures retrospective PF-method alignment constraints for execution quality gates.
- `008-pf-ddd-sysmlv2-integration.md` defines the follow-on integration track for DDD/SysML v2 after M1-M3 foundation.
- this proposal defines the paper-aligned prioritization order on top of `005/006`:
  - near-term: M1, M2, M3
  - medium-term: M4, M5
  - exploratory: M6 + optional process-model track

## Out of Scope

- replacing Rust validator on the interactive LSP path
- broad syntax redesign without migration policy
- heavyweight framework migration (for example, full Eclipse/EMF stack adoption)

## Risks and Mitigations

- Risk: roadmap duplicates existing proposals.
  - Mitigation: this plan is additive; `005/006` remain normative for scope gates.
- Risk: traceability and UML exports drift from core semantics.
  - Mitigation: generation must consume validated AST only; no parallel parser path.
- Risk: formal checks add CI noise.
  - Mitigation: keep formal stage non-blocking until stable mismatch rate is proven low.

## Success Metrics

- invariant catalog coverage reaches 100% of active validator rules
- at least one CI artifact includes actionable impact analysis for dogfooding model changes
- at least one obligation is checked end-to-end by a formal backend
- at least one PF model exports to a design artifact with trace links
- rationale report detects unresolved assumptions/questions before triage

## Decision Needed

Approve this proposal as the paper-aligned prioritization layer for roadmap execution after `v0.2.0` gating.
