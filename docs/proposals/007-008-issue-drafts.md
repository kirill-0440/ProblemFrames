# Proposal 007+008: GitHub Issue Drafts

## Status

Draft

## How to Use

- Copy one section per issue into GitHub.
- Keep task IDs unchanged to preserve traceability to:
  - `docs/proposals/007-008-implementation-task-breakdown.md`
  - `docs/proposals/007-execution-backlog-m1-m3.md`
  - `docs/proposals/008-execution-backlog-m4-m5.md`

---

## T007-M1-01

- Title: `T007-M1-01: Create invariant catalog contract`
- Labels: `roadmap:007`, `milestone:m1`, `area:dsl`, `priority:p1`
- Depends on: none
- Description:
  - Define a machine-readable invariant catalog format.
  - Assign stable `rule_id` values for active validator rules.
  - Add schema/format validation in tests or CI.
- Acceptance Criteria:
  - Catalog exists in repository and validates in CI.
  - Each active validator rule has a unique `rule_id`.

## T007-M1-02

- Title: `T007-M1-02: Build rule-to-tests trace matrix`
- Labels: `roadmap:007`, `milestone:m1`, `area:dsl`, `priority:p1`
- Depends on: `T007-M1-01`
- Description:
  - Map each `rule_id` to valid and invalid fixtures.
  - Add a report/check for missing fixture coverage.
- Acceptance Criteria:
  - Trace matrix committed.
  - CI reports missing rule coverage.

## T007-M1-03

- Title: `T007-M1-03: Close fixture coverage gaps`
- Labels: `roadmap:007`, `milestone:m1`, `area:dsl`, `priority:p2`
- Depends on: `T007-M1-02`
- Description:
  - Add missing fixtures for uncovered invariants.
  - Use deterministic naming convention by `rule_id`.
- Acceptance Criteria:
  - No uncovered rules in coverage report.
  - `cargo test -p pf_dsl` passes.

## T007-M2-01

- Title: `T007-M2-01: Implement relationship graph core`
- Labels: `roadmap:007`, `milestone:m2`, `area:traceability`, `priority:p1`
- Depends on: `T007-M1-01`
- Description:
  - Build graph extraction over validated AST for requirements/domains/interfaces/phenomena/subproblems.
- Acceptance Criteria:
  - Graph module added with unit tests.
  - At least one multi-subproblem fixture validated.

## T007-M2-02

- Title: `T007-M2-02: Add CLI traceability exports`
- Labels: `roadmap:007`, `milestone:m2`, `area:codegen`, `priority:p1`
- Depends on: `T007-M2-01`
- Description:
  - Add `--traceability-md` and `--traceability-csv` modes.
  - Document usage in `crates/pf_dsl/README.md`.
- Acceptance Criteria:
  - CLI outputs are deterministic.
  - Command docs are updated.

## T007-M2-03

- Title: `T007-M2-03: Add impact ranking engine`
- Labels: `roadmap:007`, `milestone:m2`, `area:traceability`, `priority:p1`
- Depends on: `T007-M2-01`
- Description:
  - Compute affected elements from changed seeds (requirements/domains).
  - Produce deterministic rank ordering.
- Acceptance Criteria:
  - Impact output includes ranked results.
  - Tests cover deterministic ordering.

## T007-M2-04

- Title: `T007-M2-04: Add LSP impact navigation`
- Labels: `roadmap:007`, `milestone:m2`, `area:lsp`, `priority:p2`
- Depends on: `T007-M2-01`
- Description:
  - Expose impacted requirements through one LSP action/command.
  - Support import/multi-file cases.
- Acceptance Criteria:
  - Integration test added for cross-file case.
  - No regression in existing LSP tests.

## T007-M3-01

- Title: `T007-M3-01: Select executable obligation class`
- Labels: `roadmap:007`, `milestone:m3`, `area:formal`, `priority:p1`
- Depends on: `T007-M1-01`
- Description:
  - Choose one obligation class for executable formal check.
  - Define pass/fail fixture pattern.
- Acceptance Criteria:
  - Selection rationale documented.
  - Fixture contract documented.

## T007-M3-02

- Title: `T007-M3-02: Implement formal backend execution path`
- Labels: `roadmap:007`, `milestone:m3`, `area:formal`, `priority:p1`
- Depends on: `T007-M3-01`
- Description:
  - Implement executable check path for selected class (Alloy-first).
  - Add pass and fail fixtures.
- Acceptance Criteria:
  - One class is checked end-to-end.
  - Pass/fail fixtures produce expected outcomes.

## T007-M3-03

- Title: `T007-M3-03: Add differential verdict report`
- Labels: `roadmap:007`, `milestone:m3`, `area:formal`, `area:traceability`, `priority:p1`
- Depends on: `T007-M3-02`
- Description:
  - Compare Rust validator verdict and formal backend verdict.
  - Emit mismatch categories and model/obligation IDs.
- Acceptance Criteria:
  - Differential report artifact generated.
  - Mismatch categories are explicit.

## T007-M3-04

- Title: `T007-M3-04: Wire non-blocking formal CI stage`
- Labels: `roadmap:007`, `milestone:m3`, `area:ci`, `area:formal`, `priority:p2`
- Depends on: `T007-M3-03`
- Description:
  - Add non-blocking workflow stage for formal checks.
  - Publish artifacts and rerun guidance docs.
- Acceptance Criteria:
  - CI stage runs and uploads artifacts.
  - Stage remains non-blocking.

## T008-M4A-01

- Title: `T008-M4A-01: Add AST mark metadata for DDD/SysML`
- Labels: `roadmap:008`, `milestone:m4a`, `area:dsl`, `priority:p1`
- Depends on: `T007-M1-01`
- Description:
  - Add optional marks to AST entities without breaking unmarked models.
- Acceptance Criteria:
  - AST supports optional marks.
  - Existing unmarked fixtures keep passing.

## T008-M4A-02

- Title: `T008-M4A-02: Parse mark syntax`
- Labels: `roadmap:008`, `milestone:m4a`, `area:dsl`, `priority:p1`
- Depends on: `T008-M4A-01`
- Description:
  - Extend grammar/parser for mark annotations.
  - Add malformed mark diagnostics.
- Acceptance Criteria:
  - Parser fixtures for valid/invalid marks exist.
  - Diagnostics are source-accurate.

## T008-M4A-03

- Title: `T008-M4A-03: Validate mark consistency`
- Labels: `roadmap:008`, `milestone:m4a`, `area:dsl`, `priority:p1`
- Depends on: `T008-M4A-02`
- Description:
  - Add validator rules for conflicting marks and missing prerequisites.
  - Attach stable rule IDs.
- Acceptance Criteria:
  - New validator rules are test-covered.
  - Invariant catalog references mark rules.

## T008-M4B-01

- Title: `T008-M4B-01: Implement --ddd-pim generator`
- Labels: `roadmap:008`, `milestone:m4b`, `area:codegen`, `priority:p1`
- Depends on: `T008-M4A-03`
- Description:
  - Generate bounded contexts, command/event inventory, aggregate candidates.
- Acceptance Criteria:
  - `--ddd-pim` mode available.
  - Outputs are deterministic in tests.

## T008-M4B-02

- Title: `T008-M4B-02: Implement --sysml2-text generator`
- Labels: `roadmap:008`, `milestone:m4b`, `area:codegen`, `priority:p1`
- Depends on: `T008-M4A-03`
- Description:
  - Map PF structures to textual SysML v2 constructs.
- Acceptance Criteria:
  - `--sysml2-text` mode available.
  - Snapshot tests exist for representative fixtures.

## T008-M4B-03

- Title: `T008-M4B-03: Implement --sysml2-json generator`
- Labels: `roadmap:008`, `milestone:m4b`, `area:codegen`, `priority:p1`
- Depends on: `T008-M4A-03`
- Description:
  - Emit SysML JSON aligned with targeted schema version.
  - Add schema validation hook.
- Acceptance Criteria:
  - `--sysml2-json` mode available.
  - JSON validates against pinned schema in CI/tests.

## T008-M4C-01

- Title: `T008-M4C-01: Add stable source/target IDs`
- Labels: `roadmap:008`, `milestone:m4c`, `area:traceability`, `priority:p1`
- Depends on: `T008-M4B-01`, `T008-M4B-02`, `T008-M4B-03`
- Description:
  - Define stable ID strategy for PF and generated artifacts.
- Acceptance Criteria:
  - ID contract documented and tested.
  - IDs remain deterministic across repeated runs.

## T008-M4C-02

- Title: `T008-M4C-02: Generate trace-map.json`
- Labels: `roadmap:008`, `milestone:m4c`, `area:traceability`, `priority:p1`
- Depends on: `T008-M4C-01`
- Description:
  - Emit mandatory `trace-map.json` with `pf_id -> generated_id`.
  - Enforce full mapping coverage.
- Acceptance Criteria:
  - Trace map produced for each generation run.
  - Coverage check fails when mapping is incomplete.

## T008-M4C-03

- Title: `T008-M4C-03: Publish generation and trace CI artifacts`
- Labels: `roadmap:008`, `milestone:m4c`, `area:ci`, `area:traceability`, `priority:p2`
- Depends on: `T008-M4C-02`
- Description:
  - Upload DDD/SysML outputs and trace bundles in CI.
  - Add trace coverage gate.
- Acceptance Criteria:
  - Artifacts are available per CI run.
  - Trace coverage threshold is enforced.

## T008-M5A-01

- Title: `T008-M5A-01: Scaffold pf_sysml_api crate`
- Labels: `roadmap:008`, `milestone:m5a`, `area:ci`, `area:codegen`, `priority:p2`
- Depends on: `T008-M4C-03`
- Description:
  - Create crate with configuration and minimal push/pull smoke path.
- Acceptance Criteria:
  - Crate builds and exposes one smoke entrypoint.
  - Basic docs/config examples added.

## T008-M5A-02

- Title: `T008-M5A-02: Add non-blocking SysML API smoke job`
- Labels: `roadmap:008`, `milestone:m5a`, `area:ci`, `priority:p2`
- Depends on: `T008-M5A-01`
- Description:
  - Add optional/non-blocking CI stage for API smoke runs.
  - Publish smoke logs as artifacts.
- Acceptance Criteria:
  - Job executes when configured.
  - Failures do not block merge.

## T008-M5A-03

- Title: `T008-M5A-03: Publish API bridge decision memo`
- Labels: `roadmap:008`, `milestone:m5a`, `area:ci`, `area:formal`, `priority:p3`
- Depends on: `T008-M5A-02`
- Description:
  - Summarize reliability, complexity, runtime, maintenance cost.
  - Recommend continue/pause/expand.
- Acceptance Criteria:
  - Decision memo merged in docs.
  - Follow-up decision recorded for next planning cycle.
