# System Model

Canonical PF model for the ProblemFrames toolchain.

Use this model as the source of truth when validating methodology execution and
roadmap alignment.

## Layout

- `tool_spec.pf`: entrypoint with model composition via imports.
- `domains.pf`: domain catalog.
- `interfaces.pf`: shared phenomena and control boundaries.
- `requirements.pf`: requirement set and frame typing.
  - includes `@formal.argument("...")` marks for requirement-to-correctness-argument closure mapping with referential integrity checks.
  - includes `@mda.layer("CIM"|"PIM"|"PSM")` marks for explicit MDA-layer grouping of requirements.
- `subproblems.pf`: decomposition into delivery-sized concerns.
- `arguments.pf`: W/S/R assertion sets and correctness argument.
- `implementation_trace.tsv`: requirement-to-implementation evidence map (`implemented/partial/planned`).
- `implementation_trace_policy.env`: staged policy thresholds for implementation-trace gating.
- `adequacy_selection.env`: selected M7 adequacy obligation class and pass/fail fixture bindings.
- `adequacy_expectations.tsv`: command-level adequacy expectation manifest with required-rule coverage.
- `alloy_expectations.tsv`: SAT/UNSAT expectation contract for Alloy command verdicts.
- `roadmap_alignment.md`: mapping from system-model requirement IDs to proposal/backlog items (`001` through `011`).

## Development Workflow

1. Start with model-first changes in `requirements.pf`, `subproblems.pf`, and related alignment assets.
2. Update implementation artifacts only after the self-model change is explicit.
3. Keep all `*.pf` files under `models/` (contract `R009-A7-ModelDirectoryPFContainment`).
4. Run the quality gate script.
5. Use generated artifacts in PR review to confirm model consistency and implementation status (`implemented/partial/planned`).

Quick checks:

```bash
cargo run -p pf_dsl -- models/system/tool_spec.pf --report
cargo run -p pf_dsl -- models/system/tool_spec.pf --obligations
cargo run -p pf_dsl -- models/system/tool_spec.pf --decomposition-closure
cargo run -p pf_dsl -- models/system/tool_spec.pf --concern-coverage
cargo run -p pf_dsl -- models/system/tool_spec.pf --wrspm-report
cargo run -p pf_dsl -- models/system/tool_spec.pf --lean-model
cargo run -p pf_dsl -- models/system/tool_spec.pf --lean-coverage-json
cargo run -p pf_dsl -- models/system/tool_spec.pf --formal-closure-map-tsv
cargo run -p pf_dsl -- models/system/tool_spec.pf --requirements-tsv
# optional: view grouped by MDA layer
cargo run -q -p pf_dsl -- models/system/tool_spec.pf --requirements-tsv | awk -F'|' '$1 !~ /^#/ {print $3 \"|\" $1 \"|\" $2}' | LC_ALL=C sort
cargo run -p pf_dsl -- models/system/tool_spec.pf --correctness-arguments-tsv
cargo run -p pf_dsl -- models/system/tool_spec.pf --ddd-pim
cargo run -p pf_dsl -- models/system/tool_spec.pf --sysml2-text
cargo run -p pf_dsl -- models/system/tool_spec.pf --sysml2-json
cargo run -p pf_dsl -- models/system/tool_spec.pf --trace-map-json
cargo run -p pf_dsl -- models/system/tool_spec.pf --traceability-md --impact=requirement:R009-A4-OneCommandPFQualityGate --impact-hops=2
cargo run -p pf_dsl -- models/system/tool_spec.pf --alloy > system_model.als
bash ./scripts/run_alloy_solver_check.sh --model models/system/tool_spec.pf --alloy-file system_model.als --expectations models/system/alloy_expectations.tsv
bash ./scripts/run_adequacy_evidence.sh
bash ./scripts/run_lean_formal_check.sh --model models/system/tool_spec.pf --min-formalized-args 2
bash ./scripts/run_lean_differential_check.sh --model models/system/tool_spec.pf
bash ./scripts/check_requirement_formal_closure.sh --model models/system/tool_spec.pf --lean-coverage-json .ci-artifacts/system-model/tool_spec.lean-coverage.json
bash ./scripts/generate_formal_gap_report.sh --model models/system/tool_spec.pf --closure-rows-tsv .ci-artifacts/system-model/tool_spec.formal-closure.rows.tsv --traceability-csv .ci-artifacts/system-model/tool_spec.traceability.csv
bash ./scripts/run_sysml_api_smoke.sh
bash ./scripts/check_model_implementation_trace.sh models/system/tool_spec.pf
bash ./scripts/check_model_implementation_trace.sh --policy models/system/implementation_trace_policy.env --enforce-policy models/system/tool_spec.pf
bash ./scripts/run_pf_quality_gate.sh models/system/tool_spec.pf
bash ./scripts/check_system_model.sh
bash ./scripts/check_codex_self_model_contract.sh
```
