<!-- Generated from roadmap_q1.pf. Do not edit manually. -->

# Problem Report: ProblemFramesRoadmapQ1

## 1. Domains
- **Maintainer** (Biddable)
- **EngineeringTeam** (Biddable)
- **RoadmapBoard** (Machine)
- **CI** (Machine)
- **Repository** (Lexical)
- **ProposalSet** (Lexical)
- **Metrics** (Causal)
- **Users** (Biddable)
- **ReleaseArtifacts** (Lexical)

## 2. Intefaces
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
- **Reference**: RoadmapBoard

### FastFeedbackLoop
- **Frame**: RequiredBehavior
- **Constraint**: Engineering team gets rapid CI check outcomes
- **Constrains**: EngineeringTeam
- **Reference**: CI


