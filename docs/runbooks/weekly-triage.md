# Weekly Engineering Triage Runbook

This runbook establishes a weekly triage cadence for engineering health signals.

## Cadence

- Frequency: weekly (Monday, 09:00 UTC)
- Trigger: `.github/workflows/weekly-engineering-triage.yml`
- Output:
  - artifact `engineering-metrics` (`engineering-metrics.md`, `engineering-metrics.json`, `dogfooding-triage.md`)
  - scheduled issue `Weekly engineering triage YYYY-MM-DD`

## Inputs

1. Engineering metrics report from the workflow artifact.
2. Open dependency updates (`label:dependencies` PRs).
3. Recent CI failures/regressions and flaky-test candidates.
4. Security workflows and alerts:
   - CodeQL
   - dependency-review
   - security audit
5. Dogfooding triage backlog (`dogfooding-triage.md`).
   - owner/SLA rules are configured in `.github/dogfooding-triage-owners.tsv`

## Triage Agenda

1. Review metric deltas:
   - lead time for change
   - change failure rate (proxy)
   - mean time to recovery (proxy)
   - flaky test rate (proxy)
2. Review blocked or risky dependency updates.
3. Review failed or unstable CI checks and identify owners.
4. Review unresolved security findings and determine remediation plan.
5. Capture follow-up actions with:
   - owner
   - due date
   - success signal
6. Confirm the top dogfooding actions are either accepted (owner/date assigned) or explicitly deferred.

## Exit Criteria (for each weekly session)

- At least one prioritized follow-up action (or an explicit “no action needed” decision) is recorded.
- Owners and due dates are assigned for all accepted actions.
- Status of previous week actions is updated.

## Useful Commands

```bash
# Latest metrics workflow runs
gh run list --workflow "Weekly Engineering Triage" --limit 10

# Open dependency PR backlog
gh pr list --search "is:pr is:open label:dependencies" --limit 100

# Recent CI runs on main
gh run list --workflow "CI" --branch main --event push --limit 30

# Recent security audit runs
gh run list --workflow "Security Audit" --limit 20
```
