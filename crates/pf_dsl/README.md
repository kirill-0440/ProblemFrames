# Problem Frames DSL Tool

A Rust-based CLI tool for defining and visualizing Problem Frames.

## Usage

1.  **Define your problem** in a `.pf` file (e.g., `sample.pf`).
2.  **Run the tool** to generate a DOT file:
    ```bash
    cargo run -- release -- sample.pf > output.dot
    ```
3.  **Generate an image** (requires Graphviz):
    ```bash
    dot -Tpng output.dot -o output.png
    ```

## Syntax Example

```pf
problem: SluiceGateControl

domain Controller [Machine]
domain Gate      [Causal]
domain Operator  [Biddable]

interface "Operator-Controller" {
    shared: {
        event OpenCommand [Operator -> Controller]
    }
}

requirement "SafeOperation" {
    frame: CommandedBehavior
    constraint: "..."
    constrains: Gate
    reference: Operator
}
```
