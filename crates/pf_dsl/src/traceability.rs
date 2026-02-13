use crate::ast::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TraceEntity {
    Requirement(String),
    Domain(String),
    Interface(String),
    Phenomenon { interface: String, name: String },
    Subproblem(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TraceRelation {
    RequirementConstrainsDomain,
    RequirementReferencesDomain,
    RequirementTouchesInterface,
    RequirementTouchesPhenomenon,
    InterfaceConnectsDomain,
    InterfaceHasPhenomenon,
    PhenomenonFromDomain,
    PhenomenonToDomain,
    PhenomenonControlledByDomain,
    SubproblemMachineDomain,
    SubproblemParticipantDomain,
    SubproblemIncludesRequirement,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TraceEdge {
    pub from: TraceEntity,
    pub to: TraceEntity,
    pub relation: TraceRelation,
}

#[derive(Debug, Clone, Default)]
pub struct TraceabilityGraph {
    nodes: BTreeSet<TraceEntity>,
    edges: BTreeSet<TraceEdge>,
    adjacency: BTreeMap<TraceEntity, BTreeSet<TraceEntity>>,
}

impl TraceabilityGraph {
    pub fn nodes(&self) -> &BTreeSet<TraceEntity> {
        &self.nodes
    }

    pub fn edges(&self) -> &BTreeSet<TraceEdge> {
        &self.edges
    }

    pub fn neighbors(&self, entity: &TraceEntity) -> BTreeSet<TraceEntity> {
        self.adjacency.get(entity).cloned().unwrap_or_default()
    }

    pub fn impacted_requirements(&self, seed: &TraceEntity) -> BTreeSet<String> {
        self.impacted_requirements_within_hops(seed, 2)
    }

    pub fn impacted_requirements_within_hops(
        &self,
        seed: &TraceEntity,
        max_hops: usize,
    ) -> BTreeSet<String> {
        if !self.nodes.contains(seed) {
            return BTreeSet::new();
        }

        let mut queue = VecDeque::new();
        let mut visited = BTreeSet::new();

        visited.insert(seed.clone());
        queue.push_back((seed.clone(), 0_usize));

        while let Some((current, depth)) = queue.pop_front() {
            if depth >= max_hops {
                continue;
            }
            for neighbor in self.neighbors(&current) {
                if visited.insert(neighbor.clone()) {
                    queue.push_back((neighbor, depth + 1));
                }
            }
        }

        visited
            .into_iter()
            .filter_map(|entity| match entity {
                TraceEntity::Requirement(name) => Some(name),
                _ => None,
            })
            .collect()
    }

    fn insert_node(&mut self, node: TraceEntity) {
        self.nodes.insert(node);
    }

    fn insert_edge(&mut self, from: TraceEntity, to: TraceEntity, relation: TraceRelation) {
        self.insert_node(from.clone());
        self.insert_node(to.clone());

        self.edges.insert(TraceEdge {
            from: from.clone(),
            to: to.clone(),
            relation,
        });

        self.adjacency
            .entry(from.clone())
            .or_default()
            .insert(to.clone());
        self.adjacency.entry(to).or_default().insert(from);
    }
}

fn phenomenon_id(interface_name: &str, phenomenon_name: &str) -> TraceEntity {
    TraceEntity::Phenomenon {
        interface: interface_name.to_string(),
        name: phenomenon_name.to_string(),
    }
}

fn collect_interface_indexes(problem: &Problem) -> BTreeMap<String, BTreeSet<String>> {
    let mut domain_to_interfaces: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for interface in &problem.interfaces {
        for domain_ref in &interface.connects {
            domain_to_interfaces
                .entry(domain_ref.name.clone())
                .or_default()
                .insert(interface.name.clone());
        }
    }

    domain_to_interfaces
}

fn collect_phenomenon_indexes(
    problem: &Problem,
) -> (
    BTreeMap<String, BTreeSet<TraceEntity>>,
    BTreeMap<String, BTreeSet<TraceEntity>>,
) {
    let mut domain_to_phenomena: BTreeMap<String, BTreeSet<TraceEntity>> = BTreeMap::new();
    let mut name_to_phenomena: BTreeMap<String, BTreeSet<TraceEntity>> = BTreeMap::new();

    for interface in &problem.interfaces {
        for phenomenon in &interface.shared_phenomena {
            let phenomenon_entity = phenomenon_id(&interface.name, &phenomenon.name);
            name_to_phenomena
                .entry(phenomenon.name.clone())
                .or_default()
                .insert(phenomenon_entity.clone());

            for domain_name in [
                phenomenon.from.name.as_str(),
                phenomenon.to.name.as_str(),
                phenomenon.controlled_by.name.as_str(),
            ] {
                domain_to_phenomena
                    .entry(domain_name.to_string())
                    .or_default()
                    .insert(phenomenon_entity.clone());
            }
        }
    }

    (domain_to_phenomena, name_to_phenomena)
}

pub fn build_traceability_graph(problem: &Problem) -> TraceabilityGraph {
    let domain_to_interfaces = collect_interface_indexes(problem);
    let (domain_to_phenomena, name_to_phenomena) = collect_phenomenon_indexes(problem);

    let mut graph = TraceabilityGraph::default();

    for domain in &problem.domains {
        graph.insert_node(TraceEntity::Domain(domain.name.clone()));
    }

    for interface in &problem.interfaces {
        let interface_entity = TraceEntity::Interface(interface.name.clone());
        graph.insert_node(interface_entity.clone());

        for connected_domain in &interface.connects {
            graph.insert_edge(
                interface_entity.clone(),
                TraceEntity::Domain(connected_domain.name.clone()),
                TraceRelation::InterfaceConnectsDomain,
            );
        }

        for phenomenon in &interface.shared_phenomena {
            let phenomenon_entity = phenomenon_id(&interface.name, &phenomenon.name);
            graph.insert_edge(
                interface_entity.clone(),
                phenomenon_entity.clone(),
                TraceRelation::InterfaceHasPhenomenon,
            );

            graph.insert_edge(
                phenomenon_entity.clone(),
                TraceEntity::Domain(phenomenon.from.name.clone()),
                TraceRelation::PhenomenonFromDomain,
            );
            graph.insert_edge(
                phenomenon_entity.clone(),
                TraceEntity::Domain(phenomenon.to.name.clone()),
                TraceRelation::PhenomenonToDomain,
            );
            graph.insert_edge(
                phenomenon_entity,
                TraceEntity::Domain(phenomenon.controlled_by.name.clone()),
                TraceRelation::PhenomenonControlledByDomain,
            );
        }
    }

    for requirement in &problem.requirements {
        let requirement_entity = TraceEntity::Requirement(requirement.name.clone());
        graph.insert_node(requirement_entity.clone());

        let mut linked_domains = BTreeSet::new();

        if let Some(constrains) = &requirement.constrains {
            linked_domains.insert(constrains.name.clone());
            graph.insert_edge(
                requirement_entity.clone(),
                TraceEntity::Domain(constrains.name.clone()),
                TraceRelation::RequirementConstrainsDomain,
            );
        }

        if let Some(reference) = &requirement.reference {
            linked_domains.insert(reference.name.clone());
            graph.insert_edge(
                requirement_entity.clone(),
                TraceEntity::Domain(reference.name.clone()),
                TraceRelation::RequirementReferencesDomain,
            );
        }

        for domain_name in linked_domains {
            if let Some(interfaces) = domain_to_interfaces.get(&domain_name) {
                for interface_name in interfaces {
                    graph.insert_edge(
                        requirement_entity.clone(),
                        TraceEntity::Interface(interface_name.clone()),
                        TraceRelation::RequirementTouchesInterface,
                    );
                }
            }

            if let Some(phenomena) = domain_to_phenomena.get(&domain_name) {
                for phenomenon_entity in phenomena {
                    graph.insert_edge(
                        requirement_entity.clone(),
                        phenomenon_entity.clone(),
                        TraceRelation::RequirementTouchesPhenomenon,
                    );
                }
            }
        }

        for phenomenon_name in &requirement.phenomena {
            if let Some(phenomena) = name_to_phenomena.get(phenomenon_name) {
                for phenomenon_entity in phenomena {
                    graph.insert_edge(
                        requirement_entity.clone(),
                        phenomenon_entity.clone(),
                        TraceRelation::RequirementTouchesPhenomenon,
                    );
                }
            }
        }
    }

    for subproblem in &problem.subproblems {
        let subproblem_entity = TraceEntity::Subproblem(subproblem.name.clone());
        graph.insert_node(subproblem_entity.clone());

        if let Some(machine) = &subproblem.machine {
            graph.insert_edge(
                subproblem_entity.clone(),
                TraceEntity::Domain(machine.name.clone()),
                TraceRelation::SubproblemMachineDomain,
            );
        }

        for participant in &subproblem.participants {
            graph.insert_edge(
                subproblem_entity.clone(),
                TraceEntity::Domain(participant.name.clone()),
                TraceRelation::SubproblemParticipantDomain,
            );
        }

        for requirement in &subproblem.requirements {
            graph.insert_edge(
                subproblem_entity.clone(),
                TraceEntity::Requirement(requirement.name.clone()),
                TraceRelation::SubproblemIncludesRequirement,
            );
        }
    }

    graph
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span() -> Span {
        Span { start: 0, end: 0 }
    }

    fn reference(name: &str) -> Reference {
        Reference {
            name: name.to_string(),
            span: span(),
        }
    }

    fn domain(name: &str, kind: DomainKind, role: DomainRole) -> Domain {
        Domain {
            name: name.to_string(),
            kind,
            role,
            span: span(),
            source_path: None,
        }
    }

    fn phenomenon(
        name: &str,
        type_: PhenomenonType,
        from: &str,
        to: &str,
        controlled_by: &str,
    ) -> Phenomenon {
        Phenomenon {
            name: name.to_string(),
            type_,
            from: reference(from),
            to: reference(to),
            controlled_by: reference(controlled_by),
            span: span(),
        }
    }

    fn interface(name: &str, connects: &[&str], shared_phenomena: Vec<Phenomenon>) -> Interface {
        Interface {
            name: name.to_string(),
            connects: connects.iter().map(|name| reference(name)).collect(),
            shared_phenomena,
            span: span(),
            source_path: None,
        }
    }

    #[test]
    fn builds_relationship_graph_for_multi_subproblem_model() {
        let problem = Problem {
            name: "TraceabilityDogfood".to_string(),
            span: span(),
            imports: vec![],
            domains: vec![
                domain("Machine", DomainKind::Causal, DomainRole::Machine),
                domain("Operator", DomainKind::Biddable, DomainRole::Given),
                domain("Sensor", DomainKind::Causal, DomainRole::Given),
                domain("Ledger", DomainKind::Lexical, DomainRole::Given),
            ],
            interfaces: vec![
                interface(
                    "Operator-Machine",
                    &["Operator", "Machine"],
                    vec![phenomenon(
                        "IssueCommand",
                        PhenomenonType::Event,
                        "Operator",
                        "Machine",
                        "Operator",
                    )],
                ),
                interface(
                    "Sensor-Machine",
                    &["Sensor", "Machine"],
                    vec![phenomenon(
                        "PushReading",
                        PhenomenonType::Value,
                        "Sensor",
                        "Machine",
                        "Sensor",
                    )],
                ),
                interface(
                    "Machine-Ledger",
                    &["Machine", "Ledger"],
                    vec![phenomenon(
                        "PersistRecord",
                        PhenomenonType::Value,
                        "Machine",
                        "Ledger",
                        "Machine",
                    )],
                ),
            ],
            requirements: vec![
                Requirement {
                    name: "DisplayState".to_string(),
                    frame: FrameType::InformationDisplay,
                    phenomena: vec![],
                    constraint: "show data".to_string(),
                    constrains: Some(reference("Sensor")),
                    reference: Some(reference("Operator")),
                    span: span(),
                    source_path: None,
                },
                Requirement {
                    name: "StoreRecord".to_string(),
                    frame: FrameType::Transformation,
                    phenomena: vec!["PersistRecord".to_string()],
                    constraint: "store transformed record".to_string(),
                    constrains: Some(reference("Ledger")),
                    reference: None,
                    span: span(),
                    source_path: None,
                },
            ],
            subproblems: vec![
                Subproblem {
                    name: "DisplayFlow".to_string(),
                    machine: Some(reference("Machine")),
                    participants: vec![
                        reference("Machine"),
                        reference("Operator"),
                        reference("Sensor"),
                    ],
                    requirements: vec![reference("DisplayState")],
                    span: span(),
                    source_path: None,
                },
                Subproblem {
                    name: "StorageFlow".to_string(),
                    machine: Some(reference("Machine")),
                    participants: vec![reference("Machine"), reference("Ledger")],
                    requirements: vec![reference("StoreRecord")],
                    span: span(),
                    source_path: None,
                },
            ],
            assertion_sets: vec![],
            correctness_arguments: vec![],
        };

        let graph = build_traceability_graph(&problem);

        assert!(graph
            .nodes()
            .contains(&TraceEntity::Requirement("DisplayState".to_string())));
        assert!(graph
            .nodes()
            .contains(&TraceEntity::Subproblem("StorageFlow".to_string())));
        assert!(graph
            .nodes()
            .contains(&TraceEntity::Interface("Sensor-Machine".to_string())));
        assert!(graph.nodes().contains(&TraceEntity::Phenomenon {
            interface: "Machine-Ledger".to_string(),
            name: "PersistRecord".to_string(),
        }));

        assert!(graph.edges().contains(&TraceEdge {
            from: TraceEntity::Requirement("DisplayState".to_string()),
            to: TraceEntity::Domain("Sensor".to_string()),
            relation: TraceRelation::RequirementConstrainsDomain,
        }));
        assert!(graph.edges().contains(&TraceEdge {
            from: TraceEntity::Requirement("DisplayState".to_string()),
            to: TraceEntity::Interface("Operator-Machine".to_string()),
            relation: TraceRelation::RequirementTouchesInterface,
        }));
        assert!(graph.edges().contains(&TraceEdge {
            from: TraceEntity::Requirement("StoreRecord".to_string()),
            to: TraceEntity::Phenomenon {
                interface: "Machine-Ledger".to_string(),
                name: "PersistRecord".to_string(),
            },
            relation: TraceRelation::RequirementTouchesPhenomenon,
        }));
        assert!(graph.edges().contains(&TraceEdge {
            from: TraceEntity::Subproblem("StorageFlow".to_string()),
            to: TraceEntity::Requirement("StoreRecord".to_string()),
            relation: TraceRelation::SubproblemIncludesRequirement,
        }));

        let impacted_from_sensor =
            graph.impacted_requirements(&TraceEntity::Domain("Sensor".to_string()));
        assert!(impacted_from_sensor.contains("DisplayState"));
        assert!(!impacted_from_sensor.contains("StoreRecord"));
    }
}
