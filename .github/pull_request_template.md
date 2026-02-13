## What Changed

- 

## Why

- 

## Checklist

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `npm run compile --prefix editors/code`
- [ ] If `.pf` models or semantics changed: `bash ./scripts/run_pf_quality_gate.sh <model.pf> [...]`
- [ ] If `.pf` models or semantics changed: attached or updated PF artifacts (`report`, `decomposition-closure`, `obligations`, `alloy`, `traceability`)
- [ ] If `.pf` models or semantics changed: no open decomposition closure items, or exception is documented
- [ ] `cargo audit --file Cargo.lock` and `npm audit --prefix editors/code --audit-level=high` (recommended)
- [ ] Updated docs if commands, workflows, or UX changed
