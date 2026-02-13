# System Model

Canonical PF model for the ProblemFrames toolchain.

Use this model as the source of truth when validating methodology execution and
roadmap alignment.

Quick checks:

```bash
cargo run -p pf_dsl -- models/system/tool_spec.pf --report
cargo run -p pf_dsl -- models/system/tool_spec.pf --obligations
cargo run -p pf_dsl -- models/system/tool_spec.pf --alloy > system_model.als
```
