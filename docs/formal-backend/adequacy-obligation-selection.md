# Adequacy Obligation Selection (M7)

- Date: 2026-02-13
- Selected class ID: `ADEQ-CONCERN-COVERAGE-001`
- Selected class name: `concern_coverage_contract`
- Scope anchor: `R007-M3-01`, `R010-M7-01`, `R010-M7-02`

## Rationale

The first executable adequacy-oriented class is the concern coverage contract:

1. The model must provide requirement-to-subproblem mapping and at least one correctness argument (`rust_verdict` via `--concern-coverage`).
2. The formal backend artifact must expose at least one obligation predicate (`formal_verdict` proxy via `--alloy` output containing `pred Obl_`).

This preserves alignment with existing PF quality artifacts while keeping the execution path deterministic and CI-friendly.

## Fixture Templates

- Expected pass fixture: `models/dogfooding/adequacy/pass.pf`
- Expected fail fixture: `models/dogfooding/adequacy/fail.pf`

## Differential Contract

`scripts/run_adequacy_evidence.sh` publishes:

- `adequacy-differential.md`
- `adequacy-evidence.json`
- `adequacy.status` (`PASS`/`OPEN`)

The rollout is non-blocking by default; `--enforce-pass` turns it into a blocking gate.
