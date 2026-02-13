# PF Metamodel Contract

This directory defines the machine-checkable contract for PF-DSL semantic invariants (roadmap `007`, milestone `M1`).

## Files

- `invariant-catalog.json`: canonical invariant catalog with stable rule IDs, severity, rationale, validator mapping, and test evidence.
- `rule-test-matrix.tsv`: compact trace matrix (`rule_id -> validator error variant -> valid/invalid tests`).

## Source of Truth

`invariant-catalog.json` is the source of truth.

`rule-test-matrix.tsv` is a human-readable projection and must stay aligned with the catalog.

## Enforcement

Consistency is enforced by tests in `crates/pf_dsl/src/metamodel_contract_tests.rs`:

- catalog syntax and schema sanity
- coverage parity with `ValidationError` variants in `crates/pf_dsl/src/validator.rs`
- test reference integrity against `crates/pf_dsl/src/validator_tests.rs`
- matrix/catalog synchronization
