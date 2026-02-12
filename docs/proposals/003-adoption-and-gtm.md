# Proposal 003: Adoption and GTM Focus

## Status

Draft

## Problem Statement

The product has strong technical foundations, but adoption risk is high if value is not packaged into clear, repeatable workflows for specific teams.

Without a focused ICP and proof-of-value story, the project can become a technically strong but niche tool.

## Goals

1. Define who the first users are and why they adopt.
2. Provide one concrete end-to-end story with measurable value.
3. Reduce time-to-first-value for new users.

## Initial ICP

Primary:

- teams building safety- or correctness-sensitive systems
- architecture/reliability leads who need traceable requirements and design rationale

Secondary:

- teams with heavy interface complexity between software and external domains
- organizations with formal review or compliance pressure

## Value Proposition

Problem Frames provides:

- explicit domain/interface modeling instead of implicit assumptions
- early structural validation before implementation
- editor-integrated feedback via LSP
- exportable artifacts (reports/diagrams) for design reviews

## Product Narrative (Proof-of-Value)

### Scenario: "From Requirement to Review-Ready Artifact"

1. Author `.pf` model for a real subsystem.
2. Receive immediate structural diagnostics in VS Code.
3. Generate diagram/report for architecture review.
4. Gate merges on model validation and CI checks.

Success signal:

- fewer review cycles caused by unclear or conflicting requirements.

## Scope

### In Scope

- ICP definition and one canonical use case.
- Demo repository flow/script for onboarding.
- Lightweight documentation for first 30 minutes.
- Instrumentation for adoption metrics.

### Out of Scope

- Broad horizontal expansion before initial product-market signal.
- Enterprise feature set before repeatable team adoption.

## Rollout Plan

### Phase 1: Positioning and Packaging

- Write one-page product position note.
- Publish "start in 30 minutes" guide.
- Prepare demo based on one realistic domain model.

### Phase 2: Pilot with 1-2 Teams

- Run structured pilot.
- Capture baseline and post-adoption metrics:
  - model authoring time
  - review iteration count
  - defect leakage from requirement misunderstandings

### Phase 3: Iterate and Scale

- Prioritize roadmap based on pilot evidence.
- Expand docs/examples from proven use cases only.

## Metrics

- Time to first successful model validation.
- Number of active modeled projects.
- Review-cycle reduction on pilot projects.
- Ratio of recurring vs one-off users.

## Risks

- Messaging too academic for practitioners.
- Onboarding friction despite strong tooling.
- Premature expansion into low-fit segments.

## Mitigations

- Lead with concrete workflow outcomes, not formalism.
- Keep onboarding path narrow and opinionated.
- Use pilot data to decide roadmap priorities.

## Decision Needed

- Approve this adoption-first GTM approach for next planning cycle.
