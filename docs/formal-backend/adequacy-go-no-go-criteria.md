# Adequacy Gate Go/No-Go Criteria (M7)

- Date: 2026-02-13
- Scope: proposal `010`, milestone `M7`
- Baseline artifacts:
  - `adequacy-differential.md`
  - `adequacy-evidence.json`
  - `adequacy.status`

## Checkpoint Criteria for Blocking Promotion

Promote adequacy checks from non-blocking to blocking only when all criteria below hold:

1. **Stability window**
   - At least 2 consecutive weeks of CI runs with `adequacy.status=PASS`.

2. **Mismatch trend**
   - `mismatches=0` for selected obligation class across the last 10 main-branch runs.

3. **Fixture health**
   - Both canonical fixtures (`pass.pf`, `fail.pf`) keep deterministic verdicts and command-level required-rule coverage behavior.

4. **Operational recoverability**
   - Triage and rerun procedure is documented and validated by at least one dry incident drill.

5. **Ownership**
   - A named DRI is assigned for formal-backend regressions and gate policy updates.

## Decision Rule

- If all criteria are satisfied: set `--enforce-pass` in CI quality gate stage.
- If any criterion is not satisfied: keep non-blocking mode and track deltas in weekly triage.
