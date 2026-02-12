# Proposal 002: Formal Verification Track (Lean-First, Parallel to Production)

## Status

Draft

## Problem Statement

The project needs stronger assurance for semantic correctness of Problem Frames models, but production workflows (Rust parser/validator + LSP) must remain fast and stable.

We need formal verification without destabilizing editor latency or delivery speed.

## Decision Summary

- Keep Rust validator as production source of truth in the near term.
- Add formal methods as a separate research/verification track.
- Use Lean 4 as the primary formal target first.
- Keep K Framework optional for specific operational-semantics experiments.

## Why Lean-First

- Strong environment for theorem proving and machine-checked proofs.
- Better fit for long-lived proof artifacts than ad hoc symbolic checks.
- Existing proposal in `docs/proposals/004-lean-integration-proposal.md` can be refined and executed incrementally.

## Why Not Replace Rust Validator Now

- LSP validation is on interactive path and must stay low-latency.
- Current validator already encodes meaningful frame/domain/causality constraints.
- Full migration would duplicate semantics and increase delivery risk.

## Scope

### In Scope

- Generate typed Lean model from PF AST (offline).
- Build a small core theory for domains/interfaces/phenomena/requirements.
- Check generated model + proofs in dedicated CI job.
- Differential checks against Rust validator outcomes for baseline consistency.

### Out of Scope

- Blocking regular PR merge on Lean proof completion in initial rollout.
- Embedding raw Lean in `.pf` syntax in early phases.
- Rewriting LSP diagnostics to depend on Lean.

## Plan

### Phase 0: Semantic Contract

- Write explicit semantic contract for:
  - phenomena directionality
  - causality assumptions
  - frame-level constraints
- Define which properties are expected to be provable.

Deliverable:

- Spec document and reviewed assumptions.

### Phase 1: Lean Runtime + Transpiler

- Introduce `theory/` with Lean project and core definitions.
- Implement Rust `LeanEmitter` for deterministic codegen.
- Add CLI output mode to generate `.lean` model.

Deliverable:

- `sample.pf` compiles to Lean model and builds with `lake`.

### Phase 2: Baseline Proofs

- Add a minimal set of proofs/sanity lemmas:
  - all referenced domains are declared
  - interface endpoints are well-formed
  - selected frame invariants
- Compare Lean and Rust verdicts on curated fixtures.

Deliverable:

- Differential test report and mismatch triage.

### Phase 3: Controlled Adoption

- Run Lean CI in non-blocking mode first.
- After stability period, decide whether to make selected checks required.

Deliverable:

- Go/no-go decision memo with cost/benefit.

## Risks

- Semantics drift between Rust and Lean.
- Proof maintenance overhead.
- Team ramp-up cost on theorem proving.

## Mitigations

- Differential tests as mandatory guardrail.
- Keep proof surface small and high-value first.
- Document proof patterns and provide templates.

## Success Criteria

- Lean pipeline remains isolated and reproducible.
- At least one real model property proven end-to-end.
- No regression in LSP responsiveness or core CI throughput.

## Decision Needed

- Approve this track as research-first, parallel to production validator.
