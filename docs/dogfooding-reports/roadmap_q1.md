<!-- Generated from roadmap_q1.pf. Do not edit manually. -->

# Problem Report: ProblemFramesRoadmapQ1

## 1. Domains
- **Maintainer** (Biddable/Given)
- **EngineeringTeam** (Biddable/Given)
- **RoadmapBoard** (Causal/Machine)
- **CI** (Causal/Given)
- **Repository** (Lexical/Given)
- **ProposalSet** (Lexical/Given)
- **Metrics** (Causal/Given)
- **Users** (Biddable/Given)
- **ReleaseArtifacts** (Lexical/Given)

## 2. Interfaces
- **Interface**: Maintainer-RoadmapBoard
  - [Command] PrioritizeMilestone (Maintainer -> RoadmapBoard)
  - [Event] MilestoneCommitted (RoadmapBoard -> Maintainer)
- **Interface**: Team-Proposals
  - [Event] DraftProposal (EngineeringTeam -> ProposalSet)
  - [Value] ProposalSnapshot (ProposalSet -> RoadmapBoard)
- **Interface**: Repository-CI
  - [Value] RepositoryState (Repository -> CI)
  - [Event] CheckResult (CI -> EngineeringTeam)
- **Interface**: Metrics-RoadmapBoard
  - [Value] QualitySignals (Metrics -> RoadmapBoard)
- **Interface**: Users-RoadmapBoard
  - [Event] ReportPainPoint (Users -> RoadmapBoard)
  - [Event] PublishRoadmapUpdate (RoadmapBoard -> Users)
- **Interface**: RoadmapBoard-ReleaseArtifacts
  - [Event] PlanRelease (RoadmapBoard -> ReleaseArtifacts)
  - [Value] ReleaseDigest (ReleaseArtifacts -> Maintainer)

## 3. Requirements
### RoadmapPrioritization
- **Frame**: CommandedBehavior
- **Constraint**: Prioritization decisions originate from maintainer intent
- **Constrains**: Repository
- **Reference**: Maintainer

### QualityDrivenPlanning
- **Frame**: RequiredBehavior
- **Constraint**: Roadmap uses quality signals when selecting milestones
- **Constrains**: Metrics

### FastFeedbackLoop
- **Frame**: RequiredBehavior
- **Constraint**: Engineering team gets rapid CI check outcomes
- **Constrains**: Metrics
- **Reference**: CI

## 4. Subproblems
### PlanningControl
- **Machine**: RoadmapBoard
- **Participants**: RoadmapBoard, Maintainer, Repository
- **Requirements**: RoadmapPrioritization

### QualityAndFeedback
- **Machine**: RoadmapBoard
- **Participants**: RoadmapBoard, Metrics, CI
- **Requirements**: QualityDrivenPlanning, FastFeedbackLoop

## 5. Decomposition Closure
### Requirement Coverage
| Requirement | Covered By | Status |
| --- | --- | --- |
| FastFeedbackLoop | QualityAndFeedback | covered |
| QualityDrivenPlanning | QualityAndFeedback | covered |
| RoadmapPrioritization | PlanningControl | covered |

### Uncovered Requirements
- None.

### Orphan Subproblems
- None.

### Boundary Mismatches
- None.


