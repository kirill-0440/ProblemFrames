# System Model to Proposal Alignment

This map links canonical system-model requirements to roadmap/proposal items in
`docs/proposals`.

| System requirement ID | Proposal source | Alignment target |
| --- | --- | --- |
| `R005-G1-ReleaseReliabilityContract` | `005-v0.2.0-scope-and-exit-criteria.md` | Product Goal 1 (merge-time reliability contract) |
| `R005-G2-DogfoodingPlanningLoop` | `005-v0.2.0-scope-and-exit-criteria.md` | Product Goal 2 (dogfooding-driven planning loop) |
| `R005-G3-EditorFeedbackQuality` | `005-v0.2.0-scope-and-exit-criteria.md` | Product Goal 3 (validator/LSP quality tightening) |
| `R007-M1-MetamodelContract` | `007-execution-backlog-m1-m3.md` | M1 (`R007-M1-01`, `R007-M1-02`) |
| `R007-M2-TraceabilityImpactExports` | `007-execution-backlog-m1-m3.md` | M2 (`R007-M2-01`, `R007-M2-02`) |
| `R009-A1-ExplicitPFViews` | `007-execution-backlog-m1-m3.md`, `009-pf-canonical-retro-addendum.md` | M2 explicit view separation (`R007-M2-04`) |
| `R009-A2-DecompositionClosureArtifact` | `007-execution-backlog-m1-m3.md`, `009-pf-canonical-retro-addendum.md` | M2 decomposition closure reporting (`R007-M2-05`) |
| `R007-M3-ExecutableObligationCheck` | `007-execution-backlog-m1-m3.md` | M3 (`R007-M3-01`, `R007-M3-03`) |
| `R009-A3-FrameConcernCoverageGate` | `007-execution-backlog-m1-m3.md`, `009-pf-canonical-retro-addendum.md` | M3 frame concern coverage gate (`R007-M3-04`) |
| `R009-A4-OneCommandPFQualityGate` | `009-pf-canonical-retro-addendum.md`, `010-execution-backlog-m6-m7.md` | one-command gate operationalization (`scripts/run_pf_quality_gate.sh`) |
| `R009-A5-AgentAssistedModelExecution` | `009-pf-canonical-retro-addendum.md` | agent-assisted execution under the same PF quality gate contract |
| `R008-M4A-MarkValidationContract` | `008-execution-backlog-m4-m5.md` | M4a (`R008-M4A-01`, `R008-M4A-02`) |
| `R008-M4B-FileBasedPIMGeneration` | `008-execution-backlog-m4-m5.md` | M4b (`R008-M4B-01`, `R008-M4B-03`) |
| `R008-M4C-TraceCoverageContract` | `008-execution-backlog-m4-m5.md` | M4c (`R008-M4C-01`, `R008-M4C-02`) |
| `R008-M5A-ControlledAPIBridgeSpike` | `008-execution-backlog-m4-m5.md` | M5a (`R008-M5A-01`, `R008-M5A-02`) |
| `R010-M6-WRSPMBridgeCoverage` | `010-pf-wrspm-contract-bridge.md`, `010-execution-backlog-m6-m7.md` | M6 WRSPM bridge projection and reports (`R010-M6-01`, `R010-M6-02`) |
| `R010-M6-SVocabularyDiscipline` | `010-pf-wrspm-contract-bridge.md`, `010-execution-backlog-m6-m7.md` | M6 vocabulary discipline and coverage gate wiring (`R010-M6-03`, `R010-M6-04`) |
| `R010-M6-WRSPMReportModes` | `010-execution-backlog-m6-m7.md` | M6 CLI report modes (`--wrspm-report`, `--wrspm-json`) |
| `R010-M7-ObligationSelectionControl` | `010-execution-backlog-m6-m7.md` | M7 obligation class selection (`R010-M7-01`) |
| `R010-M7-ExecutableAdequacyEvidence` | `010-pf-wrspm-contract-bridge.md`, `010-execution-backlog-m6-m7.md` | M7 executable adequacy check path (`R010-M7-02`) |
| `R010-M7-DifferentialVerdictArtifacts` | `010-pf-wrspm-contract-bridge.md`, `010-execution-backlog-m6-m7.md` | M7 differential verdict and CI publication (`R010-M7-03`, `R010-M7-04`) |

## Usage

When implementing a roadmap task:

1. Update matching requirement/subproblem entries in `models/system/*.pf`.
2. Run `bash ./scripts/run_pf_quality_gate.sh models/system/tool_spec.pf`.
3. Include generated system-model artifacts in PR review.
