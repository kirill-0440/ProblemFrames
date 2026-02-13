# PF DSL Migration Guide (Legacy -> v2)

This document describes how to migrate legacy PF models to the current v2 syntax.

## 1) Automatic Baseline Conversion

Use the migration script for a first-pass transformation:

```bash
bash ./scripts/migrate_pf_to_v2.sh <input.pf> <output.pf>
```

The script updates:

- legacy `domain Name [Type]` to `domain Name kind ... role ...`;
- legacy interface phenomenon entries to `phenomenon ... controlledBy ...`;
- legacy interface shape to `interface "... " connects ... { shared: { ... } }`.

## 2) Required Manual Follow-Up

After automated conversion, manually add:

- frame-complete requirement fields (`constrains`, `reference` as required by frame type);
- `worldProperties`, `specification`, `requirementAssertions`;
- at least one `correctnessArgument` (`prove S and W entail R`);
- `subproblem` blocks for decomposition where relevant.

## 3) Validate and Iterate

Run:

```bash
cargo run -p pf_dsl -- <output.pf> --report
cargo run -p pf_dsl -- <output.pf> --obligations
cargo run -p pf_dsl -- <output.pf> --alloy
```

Address all validation errors before moving the model into dogfooding or CI fixtures.

## 4) Common Migration Issues

- Requirement references machine domain: move that concern into `specification`.
- Missing machine in subproblem: add `machine:` and include it in `participants:`.
- Frame mismatch: adjust domain roles/kinds or reclassify the frame.
- Empty assertion sets: add at least one `assert "..."` per referenced set.
