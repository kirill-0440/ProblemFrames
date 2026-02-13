# Start in 30 Minutes

This guide gives a narrow onboarding path with one concrete outcome.

## 0-5 min: Build

```bash
cargo build --release
```

## 5-10 min: Open sample model and inspect report

```bash
cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --report
```

## 10-15 min: Generate PF evidence

```bash
cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --decomposition-closure
cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --concern-coverage
cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --obligations
```

## 15-20 min: Generate downstream and trace artifacts

```bash
cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --ddd-pim
cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --sysml2-text
cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --traceability-md
```

## 20-25 min: Run one-command gate on canonical system model

```bash
bash ./scripts/run_pf_quality_gate.sh models/system/tool_spec.pf
```

## 25-30 min: Run scripted demo bundle

```bash
bash ./scripts/run_adoption_demo.sh
```

## Expected Outcome

You should have a deterministic artifact bundle in `.ci-artifacts/adoption-demo`.
