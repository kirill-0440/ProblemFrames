# SysML API Spike Go/No-Go Memo (M5a)

- Date: 2026-02-13
- Scope: proposal `008`, milestone `M5a`
- Inputs:
  - `scripts/run_sysml_api_smoke.sh`
  - CI job `sysml-api-smoke` in `.github/workflows/ci.yml`
  - artifact bundle `sysml-api-smoke` (`smoke.json`, `smoke.log`)

## Runtime Observations

1. Smoke path is deterministic in dry-run mode (`pf_sysml_api smoke --dry-run`).
2. Endpoint wiring is explicit through `PF_SYSML_API_ENDPOINT`.
3. CI gate can skip safely when `PF_SYSML_API_SMOKE_ENABLED != 1`, still publishing evidence.

## Reliability Observations

1. Job is non-blocking (`continue-on-error: true`) and cannot block release flow.
2. Every run publishes machine-readable outcome (`smoke.json`) and execution log (`smoke.log`).
3. Gated mode prevents accidental external calls in forks or unprepared environments.

## Maintenance Observations

1. Integration surface is small and isolated in `crates/pf_sysml_api`.
2. Operational control is centralized in one script and one CI job.
3. Rollback is low-risk: disable `PF_SYSML_API_SMOKE_ENABLED` to return to no-op evidence runs.

## Recommendation

- Decision: **continue (controlled expansion)**.
- Next planning cycle:
  1. Keep smoke non-blocking for two CI cycles with endpoint enabled in a protected environment.
  2. Add one live probe mode behind a second explicit flag.
  3. Reassess promotion to blocking only after stable endpoint SLIs are captured.

