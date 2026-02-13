# Proposal 007+008: Integrated Task Breakdown

## Status

Draft

## Purpose

This document translates proposals `007` and `008` into an execution-level task plan that can be copied into GitHub issues/sprints with minimal rework.

Primary source proposals:

- `docs/proposals/007-paper-aligned-roadmap-adaptation.md`
- `docs/proposals/007-execution-backlog-m1-m3.md`
- `docs/proposals/008-pf-ddd-sysmlv2-integration.md`
- `docs/proposals/008-execution-backlog-m4-m5.md`
- `docs/proposals/007-008-issue-drafts.md`

## Execution Window

- Start: February 16, 2026
- End (planned): July 10, 2026

## Milestone-to-Sprint Mapping

- Sprint 1 (February 16, 2026 -> February 27, 2026): M1
- Sprint 2 (March 2, 2026 -> March 13, 2026): M2 (part 1)
- Sprint 3 (March 16, 2026 -> March 27, 2026): M2 (part 2)
- Sprint 4 (March 30, 2026 -> April 10, 2026): M3 (part 1)
- Sprint 5 (April 13, 2026 -> April 24, 2026): M3 (part 2)
- Sprint 6 (May 4, 2026 -> May 15, 2026): M4a
- Sprint 7 (May 18, 2026 -> May 29, 2026): M4b (part 1)
- Sprint 8 (June 1, 2026 -> June 12, 2026): M4b (part 2)
- Sprint 9 (June 15, 2026 -> June 26, 2026): M4c
- Sprint 10 (June 29, 2026 -> July 10, 2026): M5a

## Task IDs and Backlog

### Sprint 1 - M1 Metamodel Contract

#### T007-M1-01 - Create Invariant Catalog Contract

- Scope:
  - define machine-readable invariant catalog format
  - assign stable `rule_id` for active validator rules
- Deliverables:
  - catalog file in repo docs/spec area
  - catalog format validation in CI/tests
- Depends on: none
- Maps to: `R007-M1-01`

#### T007-M1-02 - Build Rule-to-Tests Trace Matrix

- Scope:
  - map each `rule_id` to passing/failing fixtures
  - add gap report command/check
- Deliverables:
  - trace matrix artifact
  - CI step that reports missing coverage
- Depends on: `T007-M1-01`
- Maps to: `R007-M1-02`

#### T007-M1-03 - Close Fixture Gaps

- Scope:
  - add missing fixtures for uncovered rules
  - align naming to deterministic rule-based convention
- Deliverables:
  - expanded fixture set
  - green `cargo test -p pf_dsl`
- Depends on: `T007-M1-02`
- Maps to: `R007-M1-03`

### Sprint 2-3 - M2 Traceability and Impact

#### T007-M2-01 - Implement Relationship Graph Core

- Scope:
  - extract requirement/domain/interface/phenomenon/subproblem links from validated AST
  - provide reusable in-memory graph structure
- Deliverables:
  - graph module + unit tests
- Depends on: `T007-M1-01`
- Maps to: `R007-M2-01`

#### T007-M2-02 - Add CLI Trace Exports

- Scope:
  - add `--traceability-md` and `--traceability-csv`
  - render graph data into deterministic markdown/csv outputs
- Deliverables:
  - CLI flags in `pf_dsl`
  - docs update in `crates/pf_dsl/README.md`
- Depends on: `T007-M2-01`
- Maps to: `R007-M2-02`

#### T007-M2-03 - Add Impact Ranking Engine

- Scope:
  - derive affected elements from changed requirement/domain seeds
  - include rank/scoring strategy and deterministic ordering
- Deliverables:
  - impact report output + tests
- Depends on: `T007-M2-01`
- Maps to: `R007-M2-02`, `R008-M4C-02` (future integration)

#### T007-M2-04 - LSP Impact Navigation

- Scope:
  - expose impacted requirements through one LSP action/command
  - support multi-file/import scenarios
- Deliverables:
  - LSP integration test coverage
- Depends on: `T007-M2-01`
- Maps to: `R007-M2-03`

### Sprint 4-5 - M3 Executable Obligation Check

#### T007-M3-01 - Select and Specify Executable Obligation Class

- Scope:
  - select one obligation pattern
  - define pass/fail criteria and fixture structure
- Deliverables:
  - short decision note in docs/formal-backend area
- Depends on: `T007-M1-01`
- Maps to: `R007-M3-01`

#### T007-M3-02 - Implement Backend Execution Path

- Scope:
  - translate selected obligation to executable backend checks (Alloy-first)
  - include pass/fail fixtures
- Deliverables:
  - executable formal check for one obligation class
- Depends on: `T007-M3-01`
- Maps to: `R007-M3-01`

#### T007-M3-03 - Differential Verdict Reporting

- Scope:
  - compare Rust validator verdict vs backend verdict
  - classify mismatches and emit machine-readable report
- Deliverables:
  - differential report artifact
- Depends on: `T007-M3-02`
- Maps to: `R007-M3-02`

#### T007-M3-04 - Non-Blocking CI Integration

- Scope:
  - wire backend job in non-blocking mode
  - publish artifacts and add rerun guidance
- Deliverables:
  - workflow update + docs update
- Depends on: `T007-M3-03`
- Maps to: `R007-M3-03`

### Sprint 6 - M4a Marks and Validation Contract

#### T008-M4A-01 - Add AST Mark Metadata

- Scope:
  - add optional marks to relevant AST entities
  - maintain backward compatibility for unmarked models
- Deliverables:
  - AST updates + tests
- Depends on: `T007-M1-01`
- Maps to: `R008-M4A-01`

#### T008-M4A-02 - Parse Mark Syntax

- Scope:
  - grammar/parser support for mark annotations
  - malformed mark diagnostics
- Deliverables:
  - parser fixtures and tests
- Depends on: `T008-M4A-01`
- Maps to: `R008-M4A-01`

#### T008-M4A-03 - Validate Mark Consistency

- Scope:
  - validator rules for incompatible marks and missing prerequisites
  - stable rule IDs in invariant catalog
- Deliverables:
  - rule implementations + invalid fixtures
- Depends on: `T008-M4A-02`
- Maps to: `R008-M4A-02`

### Sprint 7-8 - M4b DDD/SysML Generators

#### T008-M4B-01 - Implement `--ddd-pim`

- Scope:
  - output bounded context candidates, command/event inventory, aggregate candidates
- Deliverables:
  - deterministic ddd package output + tests
- Depends on: `T008-M4A-03`
- Maps to: `R008-M4B-01`

#### T008-M4B-02 - Implement `--sysml2-text`

- Scope:
  - map PF requirements/interfaces to textual SysML v2 constructs
- Deliverables:
  - `.sysml` output snapshots + tests
- Depends on: `T008-M4A-03`
- Maps to: `R008-M4B-02`

#### T008-M4B-03 - Implement `--sysml2-json`

- Scope:
  - generate JSON output aligned to targeted SysML JSON schema version
- Deliverables:
  - JSON output + schema validation hook
- Depends on: `T008-M4A-03`
- Maps to: `R008-M4B-03`

### Sprint 9 - M4c Trace Contract and CI Publication

#### T008-M4C-01 - Add Stable Source/Target IDs

- Scope:
  - assign stable PF IDs and generated target IDs
  - provide deterministic ID strategy
- Deliverables:
  - ID contract docs + tests
- Depends on: `T008-M4B-01`, `T008-M4B-02`, `T008-M4B-03`
- Maps to: `R008-M4C-01`

#### T008-M4C-02 - Generate `trace-map.json`

- Scope:
  - emit mandatory mapping `pf_id -> generated_id`
  - enforce full coverage for generated elements
- Deliverables:
  - trace map artifact + coverage check
- Depends on: `T008-M4C-01`
- Maps to: `R008-M4C-01`

#### T008-M4C-03 - CI Artifact Publication and Gates

- Scope:
  - publish DDD/SysML/trace artifacts
  - fail when trace coverage drops below threshold
- Deliverables:
  - CI workflow updates
- Depends on: `T008-M4C-02`
- Maps to: `R008-M4C-03`

### Sprint 10 - M5a API Bridge Spike

#### T008-M5A-01 - Scaffold `pf_sysml_api`

- Scope:
  - create crate and config wiring for API client
  - add one smoke command path
- Deliverables:
  - new crate + minimal command/test
- Depends on: `T008-M4C-03`
- Maps to: `R008-M5A-01`

#### T008-M5A-02 - Non-Blocking API Smoke in CI

- Scope:
  - add optional workflow stage for API smoke checks
- Deliverables:
  - non-blocking job + artifact logs
- Depends on: `T008-M5A-01`
- Maps to: `R008-M5A-02`

#### T008-M5A-03 - Decision Memo

- Scope:
  - summarize reliability, complexity, runtime, and maintenance cost
  - recommend continue/pause/expand
- Deliverables:
  - decision memo in docs/proposals or docs/formal-backend area
- Depends on: `T008-M5A-02`
- Maps to: `R008-M5A-03`

## Release Gates

- Gate A (end of Sprint 1): invariant contract and trace matrix baseline merged
- Gate B (end of Sprint 5): one executable obligation class in CI (non-blocking)
- Gate C (end of Sprint 8): DDD and SysML file generators available and deterministic
- Gate D (end of Sprint 9): full transformation trace coverage enforced
- Gate E (end of Sprint 10): API bridge go/no-go decision recorded

## Suggested GitHub Labels

- `roadmap:007`
- `roadmap:008`
- `milestone:m1`
- `milestone:m2`
- `milestone:m3`
- `milestone:m4a`
- `milestone:m4b`
- `milestone:m4c`
- `milestone:m5a`
- `area:dsl`
- `area:codegen`
- `area:lsp`
- `area:traceability`
- `area:formal`
- `area:ci`
