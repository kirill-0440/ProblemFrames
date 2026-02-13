# PF Methodology Quality Gate

This runbook makes the current PF methodology operational for day-to-day engineering work.

## Scope

Use this gate for any PR that changes one or more PF models (`*.pf`) or model semantics.

## Methodology (Required Order)

1. Model the problem world first (domains, interfaces, requirements), then adjust implementation artifacts.
2. Enforce frame fit and strict semantic validation through `pf_dsl`.
3. Verify decomposition closure (no uncovered requirements, no orphan subproblems, no boundary mismatches).
4. Generate correctness evidence (`obligations`, `alloy`) from the validated model.
5. Generate traceability artifacts for impact analysis.

## One-command Gate

```bash
bash ./scripts/run_pf_quality_gate.sh <model.pf> [more models...]
```

Default output location:

- `.ci-artifacts/pf-quality-gate/<model-name>/`

Generated artifacts per model:

- `report.md`
- `decomposition-closure.md`
- `obligations.md`
- `model.als`
- `traceability.md`
- `traceability.csv`

## Impact-aware Gate (Optional)

```bash
bash ./scripts/run_pf_quality_gate.sh \
  --impact requirement:SafeOperation,domain:Controller \
  --impact-hops 2 \
  crates/pf_dsl/sample.pf
```

## Controlled Exceptions

If a PR intentionally carries open decomposition closure items (for exploratory or staged work), run:

```bash
bash ./scripts/run_pf_quality_gate.sh --allow-open-closure <model.pf>
```

Document the reason in PR "Why" and list closure debt explicitly.

## CI Alignment

CI publishes equivalent evidence through dogfooding artifacts:

- `dogfooding-reports`
- `dogfooding-decomposition`
- `dogfooding-obligations`
- `formal-backend`
