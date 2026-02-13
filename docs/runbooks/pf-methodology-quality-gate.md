# PF Methodology Quality Gate

This runbook makes the current PF methodology operational for day-to-day engineering work.

## Scope

Use this gate for any PR that changes one or more PF models (`*.pf`) or model semantics.

## Methodology (Required Order)

1. Model the problem world first (domains, interfaces, requirements), then adjust implementation artifacts.
2. Enforce frame fit and strict semantic validation through `pf_dsl`.
3. Verify decomposition closure (no uncovered requirements, no orphan subproblems, no boundary mismatches).
4. Verify frame concern coverage (`requirement -> correctness argument`) with explicit uncovered/deferred entries.
5. Generate correctness evidence (`obligations`, `alloy`) from the validated model.
6. Generate PIM artifacts (`ddd-pim`, `sysml2-text`, `sysml2-json`) from the same validated source model.
7. Generate source-to-target trace map and enforce coverage (`trace-map.json` status must be `PASS`).
8. Generate traceability artifacts (relationship matrix + impact analysis with generated DDD/SysML targets).
9. Generate adequacy differential evidence (`rust_verdict` vs `formal_verdict` proxy) for selected obligation class.
10. Generate implementation trace evidence (`implemented/partial/planned`) against model requirements.
11. Generate WRSPM bridge artifacts (`W/R/S/P/M` projection) for contract review.
12. Generate Lean research-track artifacts (`--lean-model`, `--lean-coverage-json`, non-blocking Lean smoke, differential report).
13. For the canonical system model, run `check_system_model.sh` to generate per-requirement formal closure report (requirement-to-correctness-argument mapping + formalized status).

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
- `concern-coverage.md`
- `ddd-pim.md`
- `sysml2.txt`
- `sysml2.json`
- `trace-map.json`
- `model.als`
- `traceability.md`
- `traceability.csv`
- `adequacy-differential.md`
- `adequacy-evidence.json`
- `implementation-trace.md`
- `implementation-trace.policy.status`
- `lean-model.lean`
- `lean-coverage.json`
- `lean-check.json`
- `lean-differential.md`
- `lean-differential.json`
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

If a PR intentionally carries open concern coverage items (for staged argumentation), run:

```bash
bash ./scripts/run_pf_quality_gate.sh --allow-open-concern-coverage <model.pf>
```

If you want implementation trace to act as a blocking gate (instead of informative mode), run:

```bash
bash ./scripts/run_pf_quality_gate.sh --enforce-implementation-trace <model.pf>
```

If you want staged policy enforcement (instead of strict all-PASS), run:

```bash
bash ./scripts/run_pf_quality_gate.sh \
  --implementation-policy models/system/implementation_trace_policy.env \
  --enforce-implementation-policy \
  <model.pf>
```

Document the reason in PR "Why" and list closure/coverage debt explicitly.

`--impact` and `--impact-hops` are forwarded to `pf_dsl --traceability-*`
for impact-aware traceability artifacts. Decomposition closure and concern
coverage verdict rules remain unchanged. Implementation trace is informative by
default and blocking only when explicitly enforced.

To require a minimum formalized Lean correctness-argument floor in the quality gate:

```bash
bash ./scripts/run_pf_quality_gate.sh \
  --min-lean-formalized-args 2 \
  <model.pf>
```

## CI Alignment

CI publishes equivalent evidence through dogfooding artifacts:

- `dogfooding-reports`
- `dogfooding-obligations`
- `system-model` (includes decomposition closure, concern coverage, PIM artifacts, trace-map coverage, adequacy evidence, implementation trace, and WRSPM outputs)
- `formal-backend`
- `sysml-api-smoke` (non-blocking, env-gated smoke JSON/log bundle)

SysML API smoke is gated via `PF_SYSML_API_SMOKE_ENABLED=1` in CI environment.

For agent-assisted model execution, run:

```bash
bash ./scripts/check_codex_self_model_contract.sh
```
