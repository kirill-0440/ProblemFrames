use pf_dsl::language::{
    DOMAIN_TYPES, FRAME_TYPES, PHENOMENON_TYPES, REQUIREMENT_FIELDS, STATEMENT_KEYWORDS,
};
use pf_lsp::completion::get_completions;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[test]
fn completion_items_cover_language_spec() {
    let completions = get_completions();
    let labels: HashSet<&str> = completions
        .items
        .iter()
        .map(|item| item.label.as_str())
        .collect();

    for keyword in STATEMENT_KEYWORDS {
        assert!(
            labels.contains(keyword),
            "missing completion keyword: {keyword}"
        );
    }
    for field in REQUIREMENT_FIELDS {
        assert!(labels.contains(field), "missing completion field: {field}");
    }
    for domain in DOMAIN_TYPES {
        assert!(
            labels.contains(domain),
            "missing domain completion: {domain}"
        );
    }
    for phenomenon in PHENOMENON_TYPES {
        assert!(
            labels.contains(phenomenon),
            "missing phenomenon completion: {phenomenon}"
        );
    }
    for frame in FRAME_TYPES {
        assert!(labels.contains(frame), "missing frame completion: {frame}");
    }
}

#[test]
fn vscode_syntax_contains_language_tokens() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let syntax_path = manifest_dir.join("../../editors/code/syntaxes/pf.tmLanguage.json");
    let syntax = fs::read_to_string(&syntax_path).expect("failed to read tmLanguage");
    let json: Value = serde_json::from_str(&syntax).expect("failed to parse tmLanguage json");

    let keyword_match = json["repository"]["keywords"]["patterns"][0]["match"]
        .as_str()
        .expect("missing keyword regex");
    for keyword in STATEMENT_KEYWORDS.iter().chain(REQUIREMENT_FIELDS.iter()) {
        let token = keyword.trim_end_matches(':');
        assert!(
            keyword_match.contains(token),
            "keyword regex does not contain token: {token}"
        );
    }

    let storage_match = json["repository"]["types"]["patterns"][0]["match"]
        .as_str()
        .expect("missing storage type regex");
    for domain in DOMAIN_TYPES {
        assert!(
            storage_match.contains(domain),
            "domain type regex does not contain token: {domain}"
        );
    }

    let support_match = json["repository"]["types"]["patterns"][1]["match"]
        .as_str()
        .expect("missing support class regex");
    for frame in FRAME_TYPES {
        assert!(
            support_match.contains(frame),
            "frame regex does not contain token: {frame}"
        );
    }
    for phenomenon in PHENOMENON_TYPES {
        assert!(
            support_match.contains(phenomenon),
            "phenomenon regex does not contain token: {phenomenon}"
        );
    }
}
