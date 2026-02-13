# PF Marks Guide (DDD + SysML v2)

This guide defines how to annotate PF models with optional DDD/SysML marks
without weakening strict PF semantics.

## Syntax

Use `marks:` blocks in `domain` and `requirement` declarations.

```pf
domain Payments kind causal role given marks: {
  @ddd.bounded_context("Payments")
  @ddd.aggregate_root
  @sysml.block
}

requirement "R1" {
  frame: InformationDisplay
  reference: Operator
  constrains: Dashboard
  marks: {
    @sysml.requirement
    @ddd.application_service("ShowDashboard")
  }
}
```

## Supported Domain Marks

- `@ddd.bounded_context("...")` (value required)
- `@ddd.aggregate_root`
- `@ddd.value_object`
- `@ddd.external_system`
- `@sysml.block`
- `@sysml.port`
- `@sysml.signal`

## Supported Requirement Marks

- `@sysml.requirement`
- `@ddd.application_service("...")` (value required)

## Validation Rules

- `ddd.aggregate_root` and `ddd.value_object` are mutually exclusive.
- `ddd.aggregate_root` and `ddd.value_object` require `ddd.bounded_context`.
- Unsupported mark names are rejected.
- Duplicate marks on the same element are rejected.
- Unmarked models keep existing strict PF behavior.

## Anti-patterns

- Adding values where mark arity forbids it, e.g. `@sysml.requirement("x")`.
- Using requirement marks on domains or domain-only marks on requirements.
- Assuming marks replace PF constraints; they only refine generation targets.
