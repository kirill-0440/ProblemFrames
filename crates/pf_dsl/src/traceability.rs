use crate::ast::*;
use crate::trace_map::build_trace_map;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TraceEntity {
    Requirement(String),
    Domain(String),
    Interface(String),
    Phenomenon { interface: String, name: String },
    Subproblem(String),
}

impl TraceEntity {
    pub fn kind(&self) -> &'static str {
        match self {
            TraceEntity::Requirement(_) => "requirement",
            TraceEntity::Domain(_) => "domain",
            TraceEntity::Interface(_) => "interface",
            TraceEntity::Phenomenon { .. } => "phenomenon",
            TraceEntity::Subproblem(_) => "subproblem",
        }
    }

    pub fn id(&self) -> String {
        match self {
            TraceEntity::Requirement(name)
            | TraceEntity::Domain(name)
            | TraceEntity::Interface(name)
            | TraceEntity::Subproblem(name) => name.clone(),
            TraceEntity::Phenomenon { interface, name } => format!("{interface}::{name}"),
        }
    }
}

impl fmt::Display for TraceEntity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.kind(), self.id())
    }
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

impl TraceRelation {
    pub fn as_str(&self) -> &'static str {
        match self {
            TraceRelation::RequirementConstrainsDomain => "requirement_constrains_domain",
            TraceRelation::RequirementReferencesDomain => "requirement_references_domain",
            TraceRelation::RequirementTouchesInterface => "requirement_touches_interface",
            TraceRelation::RequirementTouchesPhenomenon => "requirement_touches_phenomenon",
            TraceRelation::InterfaceConnectsDomain => "interface_connects_domain",
            TraceRelation::InterfaceHasPhenomenon => "interface_has_phenomenon",
            TraceRelation::PhenomenonFromDomain => "phenomenon_from_domain",
            TraceRelation::PhenomenonToDomain => "phenomenon_to_domain",
            TraceRelation::PhenomenonControlledByDomain => "phenomenon_controlled_by_domain",
            TraceRelation::SubproblemMachineDomain => "subproblem_machine_domain",
            TraceRelation::SubproblemParticipantDomain => "subproblem_participant_domain",
            TraceRelation::SubproblemIncludesRequirement => "subproblem_includes_requirement",
        }
    }
}

impl fmt::Display for TraceRelation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
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
        self.reachable_within_hops(seed, max_hops)
            .into_iter()
            .filter_map(|entity| match entity {
                TraceEntity::Requirement(name) => Some(name),
                _ => None,
            })
            .collect()
    }

    pub fn reachable_within_hops(
        &self,
        seed: &TraceEntity,
        max_hops: usize,
    ) -> BTreeSet<TraceEntity> {
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

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn sorted_join(values: BTreeSet<String>) -> String {
    if values.is_empty() {
        "-".to_string()
    } else {
        values.into_iter().collect::<Vec<_>>().join(", ")
    }
}

fn trace_source_key(entity: &TraceEntity) -> Option<(String, String)> {
    match entity {
        TraceEntity::Requirement(name) => Some(("requirement".to_string(), name.clone())),
        TraceEntity::Domain(name) => Some(("domain".to_string(), name.clone())),
        TraceEntity::Interface(name) => Some(("interface".to_string(), name.clone())),
        TraceEntity::Phenomenon { interface, name } => {
            Some(("phenomenon".to_string(), format!("{interface}.{name}")))
        }
        TraceEntity::Subproblem(_) => None,
    }
}

fn build_trace_target_index(problem: &Problem) -> BTreeMap<(String, String), BTreeSet<String>> {
    let trace_map = build_trace_map(problem);
    let mut index: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
    for link in trace_map.links {
        index
            .entry((link.source_kind, link.source_id))
            .or_default()
            .insert(format!("{}:{}", link.target_kind, link.target_id));
    }
    index
}

fn collect_generated_targets_for_reachable(
    reachable: &BTreeSet<TraceEntity>,
    trace_target_index: &BTreeMap<(String, String), BTreeSet<String>>,
) -> BTreeSet<String> {
    let mut targets = BTreeSet::new();
    for entity in reachable {
        if let Some(source_key) = trace_source_key(entity) {
            if let Some(mapped_targets) = trace_target_index.get(&source_key) {
                targets.extend(mapped_targets.iter().cloned());
            }
        }
    }
    targets
}

pub fn generate_traceability_markdown(
    problem: &Problem,
    impact_seeds: &[TraceEntity],
    max_hops: usize,
) -> String {
    let graph = build_traceability_graph(problem);
    let trace_target_index = build_trace_target_index(problem);
    let mut output = String::new();

    output.push_str(&format!("# Traceability Report: {}\n\n", problem.name));
    output.push_str("## Relationship Summary\n");
    output.push_str(&format!("- Nodes: {}\n", graph.nodes().len()));
    output.push_str(&format!("- Edges: {}\n\n", graph.edges().len()));

    output.push_str("## Requirement Relationship Matrix\n");
    output.push_str(
        "| Requirement | Constrains | References | Interfaces | Phenomena | Subproblems |\n",
    );
    output.push_str("| --- | --- | --- | --- | --- | --- |\n");

    for requirement in &problem.requirements {
        let requirement_entity = TraceEntity::Requirement(requirement.name.clone());

        let mut constrains = BTreeSet::new();
        let mut references = BTreeSet::new();
        let mut interfaces = BTreeSet::new();
        let mut phenomena = BTreeSet::new();
        let mut subproblems = BTreeSet::new();

        for edge in graph.edges() {
            match edge.relation {
                TraceRelation::RequirementConstrainsDomain if edge.from == requirement_entity => {
                    constrains.insert(edge.to.id());
                }
                TraceRelation::RequirementReferencesDomain if edge.from == requirement_entity => {
                    references.insert(edge.to.id());
                }
                TraceRelation::RequirementTouchesInterface if edge.from == requirement_entity => {
                    interfaces.insert(edge.to.id());
                }
                TraceRelation::RequirementTouchesPhenomenon if edge.from == requirement_entity => {
                    phenomena.insert(edge.to.id());
                }
                TraceRelation::SubproblemIncludesRequirement
                    if edge.to == requirement_entity
                        && matches!(edge.from, TraceEntity::Subproblem(_)) =>
                {
                    subproblems.insert(edge.from.id());
                }
                _ => {}
            }
        }

        output.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            requirement.name,
            sorted_join(constrains),
            sorted_join(references),
            sorted_join(interfaces),
            sorted_join(phenomena),
            sorted_join(subproblems)
        ));
    }
    output.push('\n');

    output.push_str("## Impact Analysis\n");
    output.push_str(&format!("- Max hops: {}\n", max_hops));
    if impact_seeds.is_empty() {
        output.push_str(
            "- No impact seeds provided. Use `--impact=requirement:<name>,domain:<name>`.\n",
        );
    } else {
        for seed in impact_seeds {
            let impacted = graph.impacted_requirements_within_hops(seed, max_hops);
            let reachable = graph.reachable_within_hops(seed, max_hops);
            let generated_targets =
                collect_generated_targets_for_reachable(&reachable, &trace_target_index);
            if impacted.is_empty() {
                output.push_str(&format!("- `{}` -> requirements: (none)\n", seed));
            } else {
                output.push_str(&format!(
                    "- `{}` -> requirements: {}\n",
                    seed,
                    impacted.into_iter().collect::<Vec<_>>().join(", ")
                ));
            }
            if generated_targets.is_empty() {
                output.push_str(&format!("- `{}` -> generated targets: (none)\n", seed));
            } else {
                output.push_str(&format!(
                    "- `{}` -> generated targets: {}\n",
                    seed,
                    generated_targets.into_iter().collect::<Vec<_>>().join(", ")
                ));
            }
        }
    }

    output
}

pub fn generate_traceability_csv(
    problem: &Problem,
    impact_seeds: &[TraceEntity],
    max_hops: usize,
) -> String {
    let graph = build_traceability_graph(problem);
    let trace_target_index = build_trace_target_index(problem);
    let mut output = String::new();

    output.push_str("record_type,from_kind,from_id,relation,to_kind,to_id,seed_kind,seed_id,impacted_requirement,impacted_target,max_hops\n");
    for edge in graph.edges() {
        output.push_str(&format!(
            "edge,{},{},{},{},{},,,,,\n",
            csv_escape(edge.from.kind()),
            csv_escape(&edge.from.id()),
            csv_escape(edge.relation.as_str()),
            csv_escape(edge.to.kind()),
            csv_escape(&edge.to.id())
        ));
    }

    for seed in impact_seeds {
        let impacted = graph.impacted_requirements_within_hops(seed, max_hops);
        let reachable = graph.reachable_within_hops(seed, max_hops);
        let generated_targets =
            collect_generated_targets_for_reachable(&reachable, &trace_target_index);
        if impacted.is_empty() {
            output.push_str(&format!(
                "impact,,,,,,{},{},,,{}\n",
                csv_escape(seed.kind()),
                csv_escape(&seed.id()),
                max_hops
            ));
        } else {
            for requirement_name in impacted {
                output.push_str(&format!(
                    "impact,,,,,,{},{},{},,{}\n",
                    csv_escape(seed.kind()),
                    csv_escape(&seed.id()),
                    csv_escape(&requirement_name),
                    max_hops
                ));
            }
        }

        for target in generated_targets {
            output.push_str(&format!(
                "impact_target,,,,,,{},{},,{},{}\n",
                csv_escape(seed.kind()),
                csv_escape(&seed.id()),
                csv_escape(&target),
                max_hops
            ));
        }
    }

    output
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
            marks: vec![],
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

    fn sample_problem() -> Problem {
        Problem {
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
                    marks: vec![],
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
                    marks: vec![],
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
        }
    }

    #[test]
    fn builds_relationship_graph_for_multi_subproblem_model() {
        let problem = sample_problem();

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

    #[test]
    fn renders_traceability_markdown_with_impact_section() {
        let problem = sample_problem();
        let markdown = generate_traceability_markdown(
            &problem,
            &[TraceEntity::Domain("Sensor".to_string())],
            2,
        );

        assert!(markdown.contains("# Traceability Report: TraceabilityDogfood"));
        assert!(markdown.contains("## Requirement Relationship Matrix"));
        assert!(markdown.contains("| DisplayState | Sensor | Operator |"));
        assert!(markdown.contains("## Impact Analysis"));
        assert!(markdown.contains("`domain:Sensor` -> requirements: DisplayState"));
        assert!(markdown.contains("`domain:Sensor` -> generated targets:"));
        assert!(markdown.contains("sysml.block:sysml.block.sensor"));
    }

    #[test]
    fn renders_traceability_csv_with_edges_and_impact_rows() {
        let problem = sample_problem();
        let csv = generate_traceability_csv(
            &problem,
            &[TraceEntity::Requirement("StoreRecord".into())],
            2,
        );

        assert!(csv.starts_with(
            "record_type,from_kind,from_id,relation,to_kind,to_id,seed_kind,seed_id,impacted_requirement,impacted_target,max_hops"
        ));
        assert!(csv.contains(
            "edge,requirement,StoreRecord,requirement_constrains_domain,domain,Ledger,,,,,"
        ));
        assert!(csv.contains("impact,,,,,,requirement,StoreRecord,StoreRecord,,2"));
        assert!(csv.contains("impact_target,,,,,,requirement,StoreRecord,,sysml.requirement:sysml.requirement.storerecord,2"));
    }
}
