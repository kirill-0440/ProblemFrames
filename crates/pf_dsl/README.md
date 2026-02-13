# Problem Frames DSL Tool

A Rust-based CLI tool for defining and visualizing Problem Frames.

## Usage

1.  **Define your problem** in a `.pf` file (e.g., `crates/pf_dsl/sample.pf`).
2.  **Run the tool** to generate a DOT file:
    ```bash
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --dot > output.dot
    ```
3.  **Generate a planning report**:
    ```bash
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --report
    ```
4.  **Generate proof obligations**:
    ```bash
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --obligations
    ```
5.  **Generate concern coverage report**:
    ```bash
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --concern-coverage
    ```
6.  **Generate Alloy backend artifact**:
    ```bash
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --alloy > model.als
    ```
7.  **Generate Lean research-track model**:
    ```bash
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --lean-model > model.lean
    ```
8.  **Generate Lean formal coverage summary**:
    ```bash
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --lean-coverage-json
    ```
9.  **Generate requirement formal-closure map (from requirement marks)**:
    ```bash
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --formal-closure-map-tsv
    ```
10.  **Generate model inventories (requirements and correctness arguments)**:
    ```bash
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --requirements-tsv
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --correctness-arguments-tsv
    ```
11.  **Generate PIM outputs**:
    ```bash
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --ddd-pim
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --sysml2-text
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --sysml2-json
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --trace-map-json
    ```
12.  **Generate an image** (requires Graphviz):
    ```bash
    dot -Tpng output.dot -o output.png
    ```

## Syntax Example

```pf
problem: SluiceGateControl

domain Gate kind causal role given
domain Operator kind biddable role given
domain Controller kind causal role machine

interface "Operator-Controller" connects Operator, Controller {
    shared: {
        phenomenon OpenCommand : event [Operator -> Controller] controlledBy Operator
    }
}

requirement "SafeOperation" {
    frame: CommandedBehavior
    constraint: "..."
    constrains: Gate
    reference: Operator
}

worldProperties W_base {
    assert "gate hardware responds to commands" @LTL
}

specification S_controller {
    assert "operator command [[Operator-Controller.OpenCommand]] eventually triggers control output" @LTL
}

requirementAssertions R_safe {
    assert "gate remains in safe operating envelope" @LTL
}

correctnessArgument A1 {
    prove S_controller and W_base entail R_safe
}
```

For WRSPM-oriented specification discipline, interface vocabulary references in
`specification` assertions can be marked as `[[Interface.Phenomenon]]`. The
validator rejects unknown references.

## Related Guides

- `docs/pf-mode-guide.md`
- `docs/migration-v2.md`
- `docs/runbooks/pf-marks-ddd-sysml-guide.md`

## Import Collision Policy

Imports are resolved into one merged model for validation and tooling.

- Top-level names must be unique across the merged scope for:
  - `domain`
  - `interface`
  - `requirement`
  - `subproblem`
  - assertion sets (`worldProperties`, `specification`, `requirementAssertions`)
  - `correctnessArgument`
- Name collisions are treated as validation errors. There is no implicit override by import order.
- Diagnostics for collisions are attributed to the later declaration (the duplicate occurrence).
