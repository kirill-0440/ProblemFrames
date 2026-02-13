# Lean Differential Promotion Policy

This note defines promotion criteria for the Lean research-track differential check.

## Current Mode

- Non-blocking.
- Output produced by `scripts/run_lean_differential_check.sh`.
- Lean verdict may be `SKIPPED` when `lake` is unavailable or gate is disabled.

## Promotion Preconditions

Promote from non-blocking to blocking only when all conditions hold for two consecutive weeks:

1. Lean smoke artifact publication success rate >= 99%.
2. Differential status `PASS` on at least 20 consecutive runs for selected canonical models.
3. No unresolved `P0` mismatch older than 7 days.
4. Lean check runtime p95 <= 180 seconds.

## Operational Commands

```bash
bash ./scripts/run_lean_formal_check.sh --model models/system/tool_spec.pf
bash ./scripts/run_lean_differential_check.sh --model models/system/tool_spec.pf
```

Keep this policy aligned with `docs/proposals/002-formal-verification-track.md` and
`docs/proposals/004-lean-integration-proposal.md`.
