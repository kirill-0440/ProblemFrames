use lsp_types::{CompletionItem, CompletionItemKind, CompletionList};

pub fn get_completions() -> CompletionList {
    let keywords = vec![
        ("problem:", "Define a new problem"),
        ("domain", "Define a problem domain"),
        ("interface", "Define an interface between domains"),
        ("requirement", "Define a requirement"),
        ("Machine", "Domain type: Machine"),
        ("Causal", "Domain type: Causal"),
        ("Biddable", "Domain type: Biddable"),
        ("Lexical", "Domain type: Lexical"),
        ("event", "Phenomenon type: Event"),
        ("state", "Phenomenon type: State"),
        ("value", "Phenomenon type: Value"),
        ("shared:", "Shared phenomena block"),
    ];

    let items = keywords
        .into_iter()
        .map(|(label, detail)| CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(detail.to_string()),
            ..Default::default()
        })
        .collect();

    CompletionList {
        is_incomplete: false,
        items,
    }
}
