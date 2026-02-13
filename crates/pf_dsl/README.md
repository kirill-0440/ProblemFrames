# Problem Frames DSL Tool

A Rust-based CLI tool for defining and visualizing Problem Frames.

## Usage

1.  **Define your problem** in a `.pf` file (e.g., `crates/pf_dsl/sample.pf`).
2.  **Run the tool** to generate a DOT file:
    ```bash
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --dot > output.dot
    ```
    View-specific exports:
    ```bash
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --dot-context > context.dot
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --dot-problem > problem.dot
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --dot-decomposition > decomposition.dot
    ```
3.  **Generate a planning report**:
    ```bash
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --report
    ```
4.  **Generate proof obligations**:
    ```bash
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --obligations
    ```
5.  **Generate decomposition closure report**:
    ```bash
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --decomposition-closure
    ```
6.  **Generate Alloy backend artifact**:
    ```bash
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --alloy > model.als
    ```
7.  **Generate traceability report with optional impact seeds**:
    ```bash
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --traceability-md --impact=requirement:SafeOperation,domain:Controller
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --traceability-csv --impact=domain:Controller --impact-hops=2
    ```
8.  **Generate an image** (requires Graphviz):
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
    assert "operator command eventually triggers control output" @LTL
}

requirementAssertions R_safe {
    assert "gate remains in safe operating envelope" @LTL
}

correctnessArgument A1 {
    prove S_controller and W_base entail R_safe
}
```

## Related Guides

- `docs/pf-mode-guide.md`
- `docs/migration-v2.md`

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
