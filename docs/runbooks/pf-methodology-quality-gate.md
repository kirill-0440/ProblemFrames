# PF Methodology Quality Gate

This runbook makes the current PF methodology operational for day-to-day engineering work.

## Scope

Use this gate for any PR that changes one or more PF models (`*.pf`) or model semantics.

## Methodology (Required Order)

1. Model the problem world first (domains, interfaces, requirements), then adjust implementation artifacts.
2. Enforce frame fit and strict semantic validation through `pf_dsl`.
3. Verify decomposition closure (no uncovered requirements, no orphan subproblems, no boundary mismatches).
4. Generate correctness evidence (`obligations`, `alloy`) from the validated model.
5. Generate traceability artifacts (relationship matrix + optional impact analysis).
6. Generate WRSPM bridge artifacts (`W/R/S/P/M` projection) for contract review.

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
- `wrspm.md`
- `wrspm.json`

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

`--impact` and `--impact-hops` are forwarded to `pf_dsl --traceability-*`
for impact-aware traceability artifacts. The decomposition closure verdict rule
remains unchanged.

## CI Alignment

CI publishes equivalent evidence through dogfooding artifacts:

- `dogfooding-reports`
- `dogfooding-obligations`
- `system-model` (includes decomposition closure and WRSPM outputs)
- `formal-backend`

For agent-assisted model execution, run:

```bash
bash ./scripts/check_codex_self_model_contract.sh
```
