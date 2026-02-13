# Proposal 011: Execution Hardening Contracts

Status: adopted
Owner: toolchain
Scope: implementation hardening for adequacy evidence and model-first governance enforcement.

## Problem Statement

Current quality signals pass, but two checks are weaker than the requirement intent:

1. adequacy differential formal verdicts can be derived from predicate-shape heuristics instead of solver verdicts;
2. model-first governance is asserted structurally but not enforced over the actual git diff boundary.

## Goals

- make adequacy evidence verdicts solver-backed and expectation-driven;
- enforce model-first governance on changed artifacts, not only on static anchors.

## Contract Requirements

- `R011-H1-SolverBackedAdequacyEvidence`
- `R011-H2-DiffBasedModelFirstGate`

## Delivery

1. Update `run_adequacy_evidence.sh` to use Alloy solver checks with per-fixture SAT/UNSAT expectations.
2. Extend Codex self-model contract with git diff analysis:
   implementation changes under code/tooling paths require canonical model changes under `models/system/*.pf`.
3. Wire evidence in `implementation_trace.tsv`.

## Exit Criteria

- canonical system model quality gate remains PASS;
- implementation trace policy remains PASS;
- both `R011-*` requirements show executable evidence checks in trace report.
