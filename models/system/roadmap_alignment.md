# System Model to Proposal Alignment

This map links canonical system-model requirements to roadmap/proposal items in
`docs/proposals`.

| System requirement ID | Proposal source | Alignment target |
| --- | --- | --- |
| `R001-A1-DeterministicMergeGateSet` | `001-product-maturity-roadmap.md` | Track A: deterministic merge gates (CI/CodeQL/dependency review + concurrency) |
| `R001-A2-ReleaseSmokeRollbackReadiness` | `001-product-maturity-roadmap.md` | Track B: release smoke checks + rollback policy |
| `R001-A3-SupplyChainEvidenceBundle` | `001-product-maturity-roadmap.md` | Track C: SBOM/provenance/checksum supply-chain baseline |
| `R001-A4-WeeklyOpsMetricsCadence` | `001-product-maturity-roadmap.md` | Track D: engineering metrics and weekly triage cadence |
| `R002-F1-ProductionValidatorPrimaryPath` | `002-formal-verification-track.md` | Research-track isolation; Rust validator remains primary production path |
| `R002-F2-AlloyFirstExecutableBaseline` | `002-formal-verification-track.md` | Alloy-first executable formal baseline in parallel non-blocking mode |
| `R002-F3-FormalDifferentialTriaging` | `002-formal-verification-track.md` | Differential verdict reporting and mismatch triage discipline |
| `R003-G1-ICPUseCasePackaging` | `003-adoption-and-gtm.md` | ICP definition and proof-of-value packaging |
| `R003-G2-TimeToFirstValueWorkflow` | `003-adoption-and-gtm.md` | narrow onboarding path / time-to-first-value workflow |
| `R003-G3-PilotFeedbackOperationalLoop` | `003-adoption-and-gtm.md` | pilot feedback loop and roadmap-driven iteration |
| `R004-L1-LeanTrackSeparation` | `004-lean-integration-proposal.md` | Lean as parallel research track (separate lifecycle, non-blocking) |
| `R004-L2-LeanEmitterDeterminism` | `004-lean-integration-proposal.md` | robust deterministic Lean transpilation contract |
| `R004-L3-LeanBuildFeedbackLoop` | `004-lean-integration-proposal.md` | dedicated Lean check path (`lake`/toolchain execution) |
| `R004-L4-LeanDifferentialContract` | `004-lean-integration-proposal.md`, `002-formal-verification-track.md` | Lean-vs-Rust differential governance before any blocking adoption |
| `R005-G1-ReleaseReliabilityContract` | `005-v0.2.0-scope-and-exit-criteria.md` | Product Goal 1 (merge-time reliability contract) |
| `R005-G2-DogfoodingPlanningLoop` | `005-v0.2.0-scope-and-exit-criteria.md` | Product Goal 2 (dogfooding-driven planning loop) |
| `R005-G3-EditorFeedbackQuality` | `005-v0.2.0-scope-and-exit-criteria.md` | Product Goal 3 (validator/LSP quality tightening) |
| `R006-S1-MetamodelV2AsContract` | `006-pf-dsl-machine-checkable-semantics-plan.md` | WS1/WS2 contract backbone (metamodel + invariant catalog) |
| `R006-S2-StrictSemanticInvariantSet` | `006-pf-dsl-machine-checkable-semantics-plan.md` | WS2 strict static semantic checks |
| `R006-S3-ImportAwareDiagnosticPrecision` | `006-pf-dsl-machine-checkable-semantics-plan.md` | WS3 multi-file source mapping and diagnostic precision |
| `R006-S4-FrameFitDecompositionDiscipline` | `006-pf-dsl-machine-checkable-semantics-plan.md` | WS4 frame-fit and decomposition checks |
| `R006-S5-WSRObligationArtifactPipeline` | `006-pf-dsl-machine-checkable-semantics-plan.md` | WS5 W/S/R obligations and artifacts |
| `R006-S6-NonBlockingFormalBackendStage` | `006-pf-dsl-machine-checkable-semantics-plan.md` | WS6 first formal backend in non-blocking CI mode |
| `R007-M1-MetamodelContract` | `007-execution-backlog-m1-m3.md` | M1 (`R007-M1-01`, `R007-M1-02`, `R007-M1-03`) |
| `R007-M2-TraceabilityImpactExports` | `007-execution-backlog-m1-m3.md` | M2 (`R007-M2-01`, `R007-M2-02`, `R007-M2-03`) |
| `R009-A1-ExplicitPFViews` | `007-execution-backlog-m1-m3.md`, `009-pf-canonical-retro-addendum.md` | M2 explicit view separation (`R007-M2-04`) |
| `R009-A2-DecompositionClosureArtifact` | `007-execution-backlog-m1-m3.md`, `009-pf-canonical-retro-addendum.md` | M2 decomposition closure reporting (`R007-M2-05`) |
| `R007-M3-ExecutableObligationCheck` | `007-execution-backlog-m1-m3.md` | M3 (`R007-M3-01`, `R007-M3-03`) |
| `R009-A3-FrameConcernCoverageGate` | `007-execution-backlog-m1-m3.md`, `009-pf-canonical-retro-addendum.md` | M3 frame concern coverage gate (`R007-M3-04`) |
| `R009-A4-OneCommandPFQualityGate` | `009-pf-canonical-retro-addendum.md`, `010-execution-backlog-m6-m7.md` | one-command PF quality gate operationalization (`scripts/run_pf_quality_gate.sh`) |
| `R009-A5-AgentAssistedModelExecution` | `009-pf-canonical-retro-addendum.md` | agent-assisted execution under the same PF quality gate contract |
| `R009-A6-ModelFirstChangeControl` | `009-pf-canonical-retro-addendum.md` | model-first change governance: canonical self-model update precedes implementation updates |
| `R009-A7-ModelDirectoryPFContainment` | `009-pf-canonical-retro-addendum.md` | repository model governance: all `.pf` files are stored under `models/` and enforced by codex self-model contract |
| `R008-M4A-MarkValidationContract` | `008-execution-backlog-m4-m5.md` | M4a (`R008-M4A-01`, `R008-M4A-02`, `R008-M4A-03`) |
| `R008-M4B-FileBasedPIMGeneration` | `008-execution-backlog-m4-m5.md` | M4b (`R008-M4B-01`, `R008-M4B-02`, `R008-M4B-03`) |
| `R008-M4C-TraceCoverageContract` | `008-execution-backlog-m4-m5.md` | M4c (`R008-M4C-01`, `R008-M4C-02`, `R008-M4C-03`) |
| `R008-M5A-ControlledAPIBridgeSpike` | `008-execution-backlog-m4-m5.md` | M5a (`R008-M5A-01`, `R008-M5A-02`, `R008-M5A-03`) |
| `R010-M6-WRSPMBridgeCoverage` | `010-pf-wrspm-contract-bridge.md`, `010-execution-backlog-m6-m7.md` | M6 WRSPM bridge projection and reports (`R010-M6-01`, `R010-M6-02`) |
| `R010-M6-SVocabularyDiscipline` | `010-pf-wrspm-contract-bridge.md`, `010-execution-backlog-m6-m7.md` | M6 vocabulary discipline and coverage gate wiring (`R010-M6-03`, `R010-M6-04`) |
| `R010-M6-WRSPMReportModes` | `010-execution-backlog-m6-m7.md` | M6 CLI report modes (`--wrspm-report`, `--wrspm-json`) |
| `R010-M7-ObligationSelectionControl` | `010-execution-backlog-m6-m7.md` | M7 obligation class selection (`R010-M7-01`) |
| `R010-M7-ExecutableAdequacyEvidence` | `010-pf-wrspm-contract-bridge.md`, `010-execution-backlog-m6-m7.md` | M7 executable adequacy check path (`R010-M7-02`) |
| `R010-M7-DifferentialVerdictArtifacts` | `010-pf-wrspm-contract-bridge.md`, `010-execution-backlog-m6-m7.md` | M7 differential verdict and CI publication (`R010-M7-03`, `R010-M7-04`) |
| `R010-M7-FormalClosureCoverageReport` | `010-pf-wrspm-contract-bridge.md`, `010-execution-backlog-m6-m7.md`, `004-lean-integration-proposal.md` | M7 Lean closure coverage projection (`formalized/skipped` with reasons plus mirror/subset entailment mode per correctness argument, `R010-M7-06`) |
| `R010-M7-MinFormalClosureFloor` | `010-execution-backlog-m6-m7.md`, `004-lean-integration-proposal.md` | M7 canonical floor for full formalized Lean correctness arguments in system gate |
| `R010-M7-RequirementFormalClosureTrace` | `010-execution-backlog-m6-m7.md`, `004-lean-integration-proposal.md` | M7 per-requirement formal closure report mapped to declared correctness arguments from requirement marks with gate enforcement (`R010-M7-05`) |
| `R010-M7-FormalGapReport` | `010-execution-backlog-m6-m7.md` | M7 formal gap report (`requirement -> frame -> subproblem`) in quality gate artifacts (`R010-M7-07`) |
| `R010-M7-FormalTrackPolicySwitch` | `010-execution-backlog-m6-m7.md` | M7 formal-track policy switch (`non-blocking` default, explicit blocking mode) (`R010-M7-08`) |

## Usage

When implementing a roadmap task:

1. Update matching requirement/subproblem entries in `models/system/*.pf`.
2. Run `bash ./scripts/run_pf_quality_gate.sh models/system/tool_spec.pf`.
3. Run `bash ./scripts/run_adequacy_evidence.sh` to capture differential adequacy evidence for the selected obligation class.
4. Run `bash ./scripts/check_model_implementation_trace.sh --policy models/system/implementation_trace_policy.env --enforce-policy models/system/tool_spec.pf` to capture implementation status and staged policy compliance.
5. Include generated system-model artifacts in PR review.
