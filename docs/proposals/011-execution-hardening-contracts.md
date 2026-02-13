# Proposal 011: Execution Hardening Contracts

Status: adopted
Owner: toolchain
Scope: implementation hardening for adequacy evidence and model-first governance enforcement.

## Problem Statement

Current quality signals pass, but three checks are weaker than the requirement intent:

1. adequacy differential formal verdicts can be derived from predicate-shape heuristics instead of solver verdicts;
2. adequacy expectations can be too coarse (fixture-level wildcard) and miss per-command obligation coverage gaps;
3. model-first governance is asserted structurally but not enforced over the actual git diff boundary.

## Goals

- make adequacy evidence verdicts solver-backed and expectation-driven;
- enforce command-level adequacy coverage with required expectation rules;
- enforce model-first governance on changed artifacts, not only on static anchors.

## Contract Requirements

- `R011-H1-SolverBackedAdequacyEvidence`
- `R011-H3-CommandLevelAdequacyCoverage`
- `R011-H2-DiffBasedModelFirstGate`

## Delivery

1. Update `run_adequacy_evidence.sh` to use Alloy solver checks with command-level expectation manifests and required-rule coverage extraction.
2. Extend `run_alloy_solver_check.sh` to validate required expectation-rule coverage (missing required command checks produce OPEN).
3. Enforce fail-closed adequacy in canonical gates (`check_system_model.sh`) and in formal-track blocking mode (`run_pf_quality_gate.sh --enforce-formal-track`).
4. Extend Codex self-model contract with git diff analysis:
   implementation changes under code/tooling paths require canonical model changes under `models/system/*.pf`.
5. Wire evidence in `implementation_trace.tsv`.

## Exit Criteria

- canonical system model quality gate remains PASS;
- implementation trace policy remains PASS;
- adequacy status is enforced as PASS in canonical checks (no accepted `OPEN`);
- command-level required expectation coverage is visible in solver/adequacy artifacts;
- all `R011-*` requirements show executable evidence checks in trace report.
