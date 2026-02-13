use lsp_types::{CompletionItem, CompletionItemKind, CompletionList};
use pf_dsl::language::{
    DOMAIN_KINDS, DOMAIN_ROLES, FRAME_TYPES, PHENOMENON_TYPES, REQUIREMENT_FIELDS,
    STATEMENT_KEYWORDS,
};

pub fn get_completions() -> CompletionList {
    let mut keywords: Vec<(&str, String)> = Vec::new();

    for keyword in STATEMENT_KEYWORDS {
        keywords.push((*keyword, statement_keyword_detail(keyword).to_string()));
    }
    for field in REQUIREMENT_FIELDS {
        keywords.push((*field, requirement_field_detail(field).to_string()));
    }
    for kind in DOMAIN_KINDS {
        keywords.push((*kind, format!("Domain kind: {kind}")));
    }
    for role in DOMAIN_ROLES {
        keywords.push((*role, format!("Domain role: {role}")));
    }
    for phenomenon in PHENOMENON_TYPES {
        keywords.push((
            *phenomenon,
            format!("Phenomenon type: {}", phenomenon_name(phenomenon)),
        ));
    }
    for frame in FRAME_TYPES {
        keywords.push((*frame, format!("Frame type: {frame}")));
    }

    let items = keywords
        .into_iter()
        .map(|(label, detail)| CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(detail),
            ..Default::default()
        })
        .collect();

    CompletionList {
        is_incomplete: false,
        items,
    }
}

fn statement_keyword_detail(keyword: &str) -> &'static str {
    match keyword {
        "problem:" => "Define a new problem",
        "domain" => "Define a problem domain",
        "kind" => "Set domain kind",
        "role" => "Set domain role",
        "interface" => "Define an interface between domains",
        "connects" => "List domains connected by an interface",
        "phenomenon" => "Declare a shared phenomenon",
        "controlledBy" => "Declare the controlling domain for a phenomenon",
        "requirement" => "Define a requirement",
        "shared:" => "Shared phenomena block",
        "subproblem" => "Define a decomposition subproblem",
        "machine:" => "Set machine domain for a subproblem",
        "participants:" => "List participant domains in a subproblem",
        "requirements:" => "List referenced requirements in a subproblem",
        "worldProperties" => "Declare world-property assertions (W)",
        "specification" => "Declare specification assertions (S)",
        "requirementAssertions" => "Declare requirement assertions (R)",
        "correctnessArgument" => "Declare a proof obligation block",
        "assert" => "Declare an assertion statement",
        "prove" => "Start a correctness proof statement",
        "and" => "Combine specification and world sets",
        "entail" => "Declare the required entailment relation",
        _ => "DSL keyword",
    }
}

fn requirement_field_detail(field: &str) -> &'static str {
    match field {
        "frame:" => "Requirement frame type",
        "constraint:" => "Requirement textual constraint",
        "constrains:" => "Domain constrained by requirement",
        "reference:" => "Reference domain for frame",
        _ => "Requirement field",
    }
}

fn phenomenon_name(phenomenon: &str) -> &'static str {
    match phenomenon {
        "event" => "Event",
        "command" => "Command",
        "state" => "State",
        "value" => "Value",
        _ => "Phenomenon",
    }
}
