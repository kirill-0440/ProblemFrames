# Proposal 009: PF Canonical Alignment Addendum (for 007/008)

## Status

Draft

## Date

February 13, 2026

## Why This Addendum Exists

This addendum records a retrospective check of the active roadmap against canonical PF expectations:

- problem-first framing (before solution choices)
- explicit world/machine boundaries and domain-centric modeling
- explicit frame concern (requirement + domain properties + machine specification)
- decomposition and recomposition discipline

It updates execution priorities in proposals `007` and `008` to reduce drift from PF method intent.

## Confirmed Alignment (Current Strengths)

- PF-DSL keeps world-facing model elements as first-class (`domains`, `interfaces`, `requirements`, `subproblems`).
- Strict validator enforces core structural/frame-fit constraints with machine-checkable diagnostics.
- Metamodel contract is now explicit and versioned (`metamodel/invariant-catalog.json` + automated consistency tests).
- Traceability and impact exports exist from validated AST (`--traceability-md`, `--traceability-csv`).
- PF remains CIM authority for follow-on DDD/SysML integration track.

## Gaps Identified by Retro

1. Frame concern is not yet enforced as a coverage gate.
   - `W/S/R` and correctness arguments exist, but uncovered requirements can still pass semantic validation.
2. Diagram/report outputs do not yet separate context/problem/decomposition views explicitly.
3. Decomposition closure is validated structurally but not reported as a dedicated operational artifact.
4. "Avoid solution-first drift" is present as principle but not yet encoded as a repeatable review gate.

## Decisions

1. Extend `007` with corrective execution items before advancing design-bridge work:
   - explicit context/problem/decomposition views
   - decomposition closure report
   - frame concern coverage gate and report
2. Keep `008` dependency on `007` foundation, but require this addendum alignment to be reflected in acceptance checks.
3. Keep formal backend path non-blocking until explicit promotion thresholds are met:
   - mismatch rate <= 5% across at least 20 CI runs
   - p95 runtime <= 180 seconds
   - artifact publication success rate >= 99%
   - no unresolved `P0` mismatch older than 7 days

## Added Acceptance Constraints

- No roadmap stage can be marked "PF complete" if requirements are not mapped to correctness arguments (or explicitly deferred with rationale).
- CI artifacts must include at least one decomposition/concern coverage report on dogfooding models.
- PF generated artifacts must continue to consume resolved+validated AST only.

## Non-Goals

- Replacing the Rust validator as interactive source of truth.
- Introducing mandatory heavyweight modeling stack migration.
- Blocking delivery on full theorem-prover integration in this addendum scope.
