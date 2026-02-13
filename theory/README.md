# Lean Theory Track

This directory contains the non-blocking Lean research track assets.

- `lakefile.lean` and `lean-toolchain` define the Lean project boundary.
- `ProblemFrames.lean` is a minimal shared theory module.
- generated model artifacts are emitted by `pf_dsl --lean-model`.

Run the local non-blocking smoke check:

```bash
bash ./scripts/run_lean_formal_check.sh --model models/system/tool_spec.pf
```
