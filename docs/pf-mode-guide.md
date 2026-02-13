# PF Modeling Guide (Current Runtime Behavior)

This guide explains the practical modeling posture enforced by the current validator/runtime.

## Current Mode

Today, the toolchain behaves as **strict PF by default**:

- requirements cannot reference machine domains;
- constrained domains cannot be biddable;
- one machine domain per problem scope;
- frame-fit checks are enforced for all five core frame types;
- subproblem decomposition boundaries are validated.

There is no runtime mode toggle yet. If a model passes validation, it already conforms to strict checks.

## Engineering-Friendly Modeling Tactics

When teams need to model implementation details without breaking strict semantics:

1. Keep requirement statements world-oriented; move machine details into `specification` assertion sets.
2. Introduce explicit causal domains for external systems instead of referencing machine internals.
3. Use `subproblem` declarations to isolate feature slices and make boundary mismatches visible early.
4. Track assumptions in `worldProperties` and correctness links in `correctnessArgument`.

## Quick Checklist

Before opening a PR for a PF model:

- `cargo run -p pf_dsl -- <model.pf> --report`
- `cargo run -p pf_dsl -- <model.pf> --obligations`
- `cargo run -p pf_dsl -- <model.pf> --alloy`

If all three commands succeed, the model is structurally valid, has obligations, and produces formal backend artifacts.
