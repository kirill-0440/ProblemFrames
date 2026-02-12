# Proposal 001: Product Maturity Roadmap (90 Days)

## Status

Draft

## Problem Statement

Problem Frames already has a strong engineering core (DSL, validator, LSP, CI/release automation), but it is not yet operated as a mature, predictable product lifecycle.

Current gap:

- quality controls are present but not fully governed as strict release gates
- release process exists but lacks explicit operational policy (support matrix, rollback rules)
- security posture is good but not yet fully supply-chain hardened
- product metrics are not yet formalized for decision making

## Goals

1. Make `main` reliably releasable.
2. Reduce CI cost/noise while keeping strict quality standards.
3. Establish auditable release and security practices.
4. Introduce measurable engineering/product health metrics.

## Non-Goals

- Major DSL redesign.
- Replacing the Rust validator with a theorem prover.
- Building a full enterprise governance platform in this phase.

## Scope

### Track A: Quality Gates

- Enforce required checks for merge to `main`:
  - `CI`
  - `CodeQL`
  - policy checks for dependency updates
- Add test coverage reporting with a minimum threshold.
- Add workflow concurrency cancellation for stale runs.

### Track B: Release Maturity

- Freeze support policy for platforms (explicit Windows policy: supported or temporarily excluded).
- Add release smoke checks:
  - `pf_lsp` starts and responds to initialization
  - VSIX installs and activates on supported targets
- Add rollback runbook for failed release and post-release incident.

### Track C: Security Baseline

- Pin third-party GitHub Actions to commit SHA where feasible.
- Add dependency review check on pull requests.
- Generate and publish SBOM and provenance artifacts with releases.

### Track D: Engineering Operations

- Define and track baseline metrics:
  - lead time for change
  - change failure rate
  - mean time to recovery
  - flaky test rate
- Establish a weekly triage routine for:
  - dependency backlog
  - CI regressions
  - security alerts

## Delivery Plan

### Milestone 1 (Weeks 1-3): Deterministic Merge Gates

- Required status checks fully configured.
- Coverage threshold enabled.
- Stale workflow cancellation enabled.

Exit criteria:

- No direct merge to `main` without all required checks.
- Average CI runtime reduced on stale-PR churn.

### Milestone 2 (Weeks 4-6): Release Reliability

- Support matrix documented and approved.
- Release smoke checks running on tag pipeline.
- Rollback playbook published.

Exit criteria:

- One full dry-run release with green smoke checks.

### Milestone 3 (Weeks 7-9): Security and Supply Chain Hardening

- Critical workflows pin actions by SHA.
- Dependency review and SBOM/provenance in release flow.

Exit criteria:

- Release artifacts include security metadata and remain reproducible.

### Milestone 4 (Weeks 10-12): Operational Baseline

- Dashboards or reports for core metrics are available.
- Weekly triage cadence in place and documented.

Exit criteria:

- Two consecutive weeks of metric-based triage with clear actions.

## Risks

- Gate fatigue from stricter checks.
- CI duration regressions.
- Team overhead from new process.

## Mitigations

- Start with narrow required checks and expand after stability.
- Use path-based execution and concurrency cancellation aggressively.
- Keep runbooks short and actionable.

## Success Metrics

- `main` red rate below 5%.
- Median PR cycle time does not regress by more than 10%.
- Change failure rate trend improves month-over-month.
- Zero high-severity unresolved supply-chain issues at release cut.

## Decision Needed

- Approve this as the default roadmap for the next 90 days.
