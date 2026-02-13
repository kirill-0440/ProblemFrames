# Canonical Use Case: Requirement to Review-Ready Artifact

## Scenario

A team models a subsystem with non-trivial operator/device interactions.
Goal: produce one review package with structural validation, obligations, and impact traceability.

## Workflow

1. Model domains, interfaces, and requirements in `.pf`.
2. Validate in editor with LSP diagnostics.
3. Run PF quality gate locally or in CI.
4. Publish generated artifacts to architecture review.

## Required Artifacts

- model report
- decomposition closure report
- concern coverage report
- obligations and formal backend artifacts
- traceability matrix and impact view
- WRSPM contract projection

## Review Questions This Use Case Answers

- Are the requirements structurally valid under PF constraints?
- Which requirements are still uncovered by correctness arguments?
- What is impacted if a requirement/domain changes?
- Is there an explicit semantic contract from requirement statements to produced evidence?
