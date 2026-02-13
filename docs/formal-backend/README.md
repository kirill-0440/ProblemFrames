# Formal Backend Artifacts

This directory stores snapshots and summaries from the non-blocking Alloy backend translator stage.

Generate artifacts locally:

```bash
bash ./scripts/run_formal_backend_check.sh
```

In CI, artifacts are generated into `.ci-artifacts/formal-backend` and uploaded as `formal-backend`.

Related governance notes:

- `adequacy-obligation-selection.md`: selected M7 obligation class and fixture bindings.
- `adequacy-go-no-go-criteria.md`: criteria for future promotion of adequacy checks to blocking.
- `sysml-api-spike-go-no-go.md`: M5a runtime/reliability/maintenance memo and recommendation.
- `lean-differential-policy.md`: non-blocking Lean differential policy and promotion criteria.

## Lean W/S/R closure mode

`pf_dsl --lean-model` now emits machine-checked closure theorems for
`correctnessArgument` entries when all three linked assertion sets use
`@LeanAtom` assertions and requirement assertions mirror specification assertions.

This keeps the theorem closure explicit and non-placeholder while preserving
the existing natural-language (`@LTL`) argumentation track.
