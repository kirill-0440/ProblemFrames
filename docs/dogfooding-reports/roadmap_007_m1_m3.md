<!-- Generated from roadmap_007_m1_m3.pf. Do not edit manually. -->

# Problem Report: PaperAlignedRoadmapM1M3

## 1. Domains
- **Maintainer** (Biddable/Given)
- **EngineeringTeam** (Biddable/Given)
- **RoadmapEngine** (Causal/Machine)
- **ValidatorSpec** (Lexical/Given)
- **TraceabilityStore** (Lexical/Given)
- **CI** (Causal/Given)

## 2. Intefaces
- **Interface**: Maintainer-RoadmapEngine
  - [Command] PrioritizeM1M3 (Maintainer -> RoadmapEngine)
  - [Event] PublishExecutionPlan (RoadmapEngine -> Maintainer)
- **Interface**: Team-RoadmapEngine
  - [Event] ProposeFixtureGap (EngineeringTeam -> RoadmapEngine)
  - [Event] ShareCoverageStatus (RoadmapEngine -> EngineeringTeam)
- **Interface**: RoadmapEngine-ValidatorSpec
  - [Value] ExportInvariantCatalog (RoadmapEngine -> ValidatorSpec)
  - [Value] ImportCoverageMatrix (ValidatorSpec -> RoadmapEngine)
- **Interface**: RoadmapEngine-TraceabilityStore
  - [Value] PersistRelationshipMatrix (RoadmapEngine -> TraceabilityStore)
  - [Value] LoadImpactView (TraceabilityStore -> RoadmapEngine)
- **Interface**: RoadmapEngine-CI
  - [Event] TriggerFormalCheck (RoadmapEngine -> CI)
  - [Event] PublishFormalVerdict (CI -> RoadmapEngine)

## 3. Requirements
### M1InvariantCatalogContract
- **Frame**: SimpleWorkpieces
- **Constraint**: Rule catalog and fixture matrix stay synchronized with validator behavior
- **Constrains**: ValidatorSpec
- **Reference**: EngineeringTeam

### M2TraceabilityImpactLoop
- **Frame**: Transformation
- **Constraint**: Relationship graph updates produce deterministic impact ranking outputs
- **Constrains**: TraceabilityStore

### M3ExecutableObligationCheck
- **Frame**: RequiredBehavior
- **Constraint**: At least one obligation class is executed in CI with reproducible pass and fail evidence
- **Constrains**: CI

## 4. Subproblems
### MetamodelContract
- **Machine**: RoadmapEngine
- **Participants**: RoadmapEngine, EngineeringTeam, ValidatorSpec
- **Requirements**: M1InvariantCatalogContract

### TraceabilityImpact
- **Machine**: RoadmapEngine
- **Participants**: RoadmapEngine, TraceabilityStore
- **Requirements**: M2TraceabilityImpactLoop

### FormalCheckPath
- **Machine**: RoadmapEngine
- **Participants**: RoadmapEngine, CI
- **Requirements**: M3ExecutableObligationCheck


