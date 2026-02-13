# Problem Frames DSL Tool

A Rust-based CLI tool for defining and visualizing Problem Frames.

## Usage

1.  **Define your problem** in a `.pf` file (e.g., `crates/pf_dsl/sample.pf`).
2.  **Run the tool** to generate a DOT file:
    ```bash
    cargo run -p pf_dsl -- crates/pf_dsl/sample.pf --dot > output.dot
    ```
3.  **Generate an image** (requires Graphviz):
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
```
