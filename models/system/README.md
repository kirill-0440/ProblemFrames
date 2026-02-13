# System Model

Canonical PF model for the ProblemFrames toolchain.

Use this model as the source of truth when validating methodology execution and
roadmap alignment.

## Layout

- `tool_spec.pf`: entrypoint with model composition via imports.
- `domains.pf`: domain catalog.
- `interfaces.pf`: shared phenomena and control boundaries.
- `requirements.pf`: requirement set and frame typing.
- `subproblems.pf`: decomposition into delivery-sized concerns.
- `arguments.pf`: W/S/R assertion sets and correctness argument.
- `implementation_trace.tsv`: requirement-to-implementation evidence map (`implemented/partial/planned`).
- `implementation_trace_policy.env`: staged policy thresholds for implementation-trace gating.
- `formal_closure_map.tsv`: requirement-to-correctness-argument map used for per-requirement formal closure checks.
- `adequacy_selection.env`: selected M7 adequacy obligation class and pass/fail fixture bindings.
- `roadmap_alignment.md`: mapping from system-model requirement IDs to proposal/backlog items (`001` through `010`).

## Development Workflow

1. Start with model-first changes in `requirements.pf`, `subproblems.pf`, and related alignment assets.
2. Update implementation artifacts only after the self-model change is explicit.
3. Run the quality gate script.
4. Use generated artifacts in PR review to confirm model consistency and implementation status (`implemented/partial/planned`).

Quick checks:

```bash
cargo run -p pf_dsl -- models/system/tool_spec.pf --report
cargo run -p pf_dsl -- models/system/tool_spec.pf --obligations
cargo run -p pf_dsl -- models/system/tool_spec.pf --decomposition-closure
cargo run -p pf_dsl -- models/system/tool_spec.pf --concern-coverage
cargo run -p pf_dsl -- models/system/tool_spec.pf --wrspm-report
cargo run -p pf_dsl -- models/system/tool_spec.pf --lean-model
cargo run -p pf_dsl -- models/system/tool_spec.pf --lean-coverage-json
cargo run -p pf_dsl -- models/system/tool_spec.pf --ddd-pim
cargo run -p pf_dsl -- models/system/tool_spec.pf --sysml2-text
cargo run -p pf_dsl -- models/system/tool_spec.pf --sysml2-json
cargo run -p pf_dsl -- models/system/tool_spec.pf --trace-map-json
cargo run -p pf_dsl -- models/system/tool_spec.pf --traceability-md --impact=requirement:R009-A4-OneCommandPFQualityGate --impact-hops=2
cargo run -p pf_dsl -- models/system/tool_spec.pf --alloy > system_model.als
bash ./scripts/run_adequacy_evidence.sh
bash ./scripts/run_lean_formal_check.sh --model models/system/tool_spec.pf --min-formalized-args 2
bash ./scripts/run_lean_differential_check.sh --model models/system/tool_spec.pf
bash ./scripts/check_requirement_formal_closure.sh --requirements-file models/system/requirements.pf --arguments-file models/system/arguments.pf --map-file models/system/formal_closure_map.tsv --lean-coverage-json .ci-artifacts/system-model/tool_spec.lean-coverage.json
bash ./scripts/run_sysml_api_smoke.sh
bash ./scripts/check_model_implementation_trace.sh models/system/tool_spec.pf
bash ./scripts/check_model_implementation_trace.sh --policy models/system/implementation_trace_policy.env --enforce-policy models/system/tool_spec.pf
bash ./scripts/run_pf_quality_gate.sh models/system/tool_spec.pf
bash ./scripts/check_system_model.sh
bash ./scripts/check_codex_self_model_contract.sh
```
