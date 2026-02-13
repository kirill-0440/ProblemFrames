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
        "interface" => "Define an interface between domains",
        "requirement" => "Define a requirement",
        "shared:" => "Shared phenomena block",
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
