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
    @formal.argument("A_roadmap_alignment")
    @mda.layer("PIM")
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
- `@formal.argument("...")` (value required; must reference a declared `correctnessArgument` and binds requirement to formal closure reports)
- `@mda.layer("CIM"|"PIM"|"PSM")` (value required; used to classify requirements by MDA layer)

## Validation Rules

- `ddd.aggregate_root` and `ddd.value_object` are mutually exclusive.
- `ddd.aggregate_root` and `ddd.value_object` require `ddd.bounded_context`.
- Unsupported mark names are rejected.
- Duplicate marks on the same element are rejected.
- `formal.argument` references to undefined correctness arguments are rejected.
- `mda.layer` accepts only `CIM`, `PIM`, or `PSM`.
- Unmarked models keep existing strict PF behavior.

## Anti-patterns

- Adding values where mark arity forbids it, e.g. `@sysml.requirement("x")`.
- Omitting values where required, e.g. `@formal.argument`.
- Using unsupported `mda.layer` values, e.g. `@mda.layer("M3")`.
- Using requirement marks on domains or domain-only marks on requirements.
- Assuming marks replace PF constraints; they only refine generation targets.
