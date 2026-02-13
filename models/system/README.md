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
- `roadmap_alignment.md`: mapping from system-model requirement IDs to proposal/backlog items (`005` through `010`).

## Development Workflow

1. Update the relevant module for the feature/change.
2. Run the quality gate script.
3. Use generated artifacts in PR review to confirm model consistency.

Quick checks:

```bash
cargo run -p pf_dsl -- models/system/tool_spec.pf --report
cargo run -p pf_dsl -- models/system/tool_spec.pf --obligations
cargo run -p pf_dsl -- models/system/tool_spec.pf --decomposition-closure
cargo run -p pf_dsl -- models/system/tool_spec.pf --wrspm-report
cargo run -p pf_dsl -- models/system/tool_spec.pf --traceability-md --impact=requirement:R009-A4-OneCommandPFQualityGate --impact-hops=2
cargo run -p pf_dsl -- models/system/tool_spec.pf --alloy > system_model.als
bash ./scripts/run_pf_quality_gate.sh models/system/tool_spec.pf
bash ./scripts/check_system_model.sh
bash ./scripts/check_codex_self_model_contract.sh
```
