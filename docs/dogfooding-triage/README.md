# Dogfooding Triage Backlog

This directory stores generated dogfooding triage backlogs.

Use the generator script:

```bash
DOGFOODING_TRIAGE_MODE=all ./scripts/generate_dogfooding_triage_report.sh
```

- Default output path for the script is this directory.
- `DOGFOODING_TRIAGE_MODE` defaults to `all`.
- CI workflow writes the triage backlog into the engineering metrics artifact (`.ci-artifacts/metrics/dogfooding-triage.md`) with mode set explicitly.
