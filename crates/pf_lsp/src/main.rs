use anyhow::Result;
use lsp_server::{Connection, Message, Notification};
use lsp_types::{
    Diagnostic, DiagnosticSeverity, InitializeParams, Position, PublishDiagnosticsParams, Range,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, Uri,
};
use pf_dsl::resolver::resolve;
use pf_dsl::validator::{validate_with_sources, validation_error_span};
use pf_lsp::completion::get_completions;
use serde_json::Value;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

const JSONRPC_METHOD_NOT_FOUND: i32 = -32601;
const JSONRPC_INVALID_PARAMS: i32 = -32602;

#[derive(Default)]
struct ServerState {
    documents: HashMap<Uri, String>,
    diagnostic_targets_by_owner: HashMap<String, HashSet<String>>,
}

impl ServerState {
    fn upsert_document(&mut self, uri: Uri, text: String) {
        self.documents.insert(uri, text);
    }

    fn remove_document(&mut self, uri: &Uri) {
        self.documents.remove(uri);
    }

    fn document_text(&self, uri: &Uri) -> Option<&str> {
        self.documents.get(uri).map(String::as_str)
    }

    fn update_diagnostic_targets(
        &mut self,
        owner_uri: &Uri,
        current_targets: HashSet<String>,
    ) -> Vec<String> {
        let owner_key = owner_uri.as_str().to_string();
        let previous_targets = self
            .diagnostic_targets_by_owner
            .insert(owner_key, current_targets.clone())
            .unwrap_or_default();

        previous_targets
            .into_iter()
            .filter(|uri| !current_targets.contains(uri))
            .collect()
    }

    fn clear_diagnostic_targets(&mut self, owner_uri: &Uri) -> Vec<String> {
        self.diagnostic_targets_by_owner
            .remove(owner_uri.as_str())
            .unwrap_or_default()
            .into_iter()
            .collect()
    }
}

fn main() -> Result<()> {
    eprintln!("Starting PF LSP Server...");

    // Create the transport
    let (connection, io_threads) = Connection::stdio();

    // Initialize
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        completion_provider: Some(lsp_types::CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: Some(vec![".".to_string(), " ".to_string()]),
            work_done_progress_options: Default::default(),
            all_commit_characters: None,
            completion_item: None,
        }),
        definition_provider: Some(lsp_types::OneOf::Left(true)),
        ..Default::default()
    })?;

    let initialization_params = connection.initialize(server_capabilities)?;
    main_loop(connection, initialization_params)?;
    io_threads.join()?;

    eprintln!("Shutting down PF LSP Server...");
    Ok(())
}

fn main_loop(connection: Connection, params: Value) -> Result<()> {
    let _params: InitializeParams = serde_json::from_value(params)?;
    eprintln!("Initialized.");
    let mut state = ServerState::default();

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                match req.method.as_str() {
                    "textDocument/completion" => {
                        let _params: lsp_types::CompletionParams =
                            match serde_json::from_value(req.params) {
                                Ok(params) => params,
                                Err(err) => {
                                    send_response_error(
                                        &connection,
                                        req.id,
                                        JSONRPC_INVALID_PARAMS,
                                        format!("Invalid completion params: {err}"),
                                    )?;
                                    continue;
                                }
                            };
                        let completion_list = get_completions();
                        let resp = lsp_server::Response::new_ok(req.id, completion_list);
                        connection.sender.send(Message::Response(resp))?;
                    }
                    "textDocument/definition" => {
                        let params: lsp_types::GotoDefinitionParams =
                            match serde_json::from_value(req.params) {
                                Ok(params) => params,
                                Err(err) => {
                                    send_response_error(
                                        &connection,
                                        req.id,
                                        JSONRPC_INVALID_PARAMS,
                                        format!("Invalid definition params: {err}"),
                                    )?;
                                    continue;
                                }
                            };

                        let response_payload = match resolve_definition(&state, params) {
                            Some(location) => serde_json::to_value(location)?,
                            None => Value::Null,
                        };

                        let resp = lsp_server::Response::new_ok(req.id, response_payload);
                        connection.sender.send(Message::Response(resp))?;
                    }
                    _ => {
                        send_response_error(
                            &connection,
                            req.id,
                            JSONRPC_METHOD_NOT_FOUND,
                            format!("Unsupported request method: {}", req.method),
                        )?;
                    }
                }
            }
            Message::Response(_resp) => {
                // handle responses
            }
            Message::Notification(not) => {
                // handle notifications
                match not.method.as_str() {
                    "textDocument/didOpen" => {
                        let params: lsp_types::DidOpenTextDocumentParams =
                            match serde_json::from_value(not.params) {
                                Ok(params) => params,
                                Err(err) => {
                                    eprintln!("Invalid didOpen params: {err}");
                                    continue;
                                }
                            };
                        state.upsert_document(
                            params.text_document.uri.clone(),
                            params.text_document.text.clone(),
                        );
                        if let Err(err) = validate_document(
                            &connection,
                            &mut state,
                            params.text_document.uri,
                            &params.text_document.text,
                        ) {
                            eprintln!("Failed to validate opened document: {err}");
                        }
                    }
                    "textDocument/didChange" => {
                        let params: lsp_types::DidChangeTextDocumentParams =
                            match serde_json::from_value(not.params) {
                                Ok(params) => params,
                                Err(err) => {
                                    eprintln!("Invalid didChange params: {err}");
                                    continue;
                                }
                            };
                        // FULL sync, so content_changes[0].text is the whole file
                        if let Some(change) = params.content_changes.first() {
                            state.upsert_document(
                                params.text_document.uri.clone(),
                                change.text.clone(),
                            );
                            if let Err(err) = validate_document(
                                &connection,
                                &mut state,
                                params.text_document.uri,
                                &change.text,
                            ) {
                                eprintln!("Failed to validate changed document: {err}");
                            }
                        }
                    }
                    "textDocument/didClose" => {
                        let params: lsp_types::DidCloseTextDocumentParams =
                            match serde_json::from_value(not.params) {
                                Ok(params) => params,
                                Err(err) => {
                                    eprintln!("Invalid didClose params: {err}");
                                    continue;
                                }
                            };
                        state.remove_document(&params.text_document.uri);
                        let mut targets = state.clear_diagnostic_targets(&params.text_document.uri);
                        let owner_uri = params.text_document.uri.as_str().to_string();
                        if !targets.contains(&owner_uri) {
                            targets.push(owner_uri);
                        }
                        for target_uri in targets
                            .into_iter()
                            .filter_map(|target| target.parse::<Uri>().ok())
                        {
                            if let Err(err) =
                                publish_diagnostics(&connection, target_uri, Vec::new())
                            {
                                eprintln!("Failed to clear diagnostics on close: {err}");
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn resolve_definition(
    state: &ServerState,
    params: lsp_types::GotoDefinitionParams,
) -> Option<lsp_types::Location> {
    let uri = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    let path = uri_to_path(&uri)?;
    let text = if let Some(buffer_text) = state.document_text(&uri) {
        Cow::Borrowed(buffer_text)
    } else {
        Cow::Owned(std::fs::read_to_string(&path).ok()?)
    };

    let offset = offset_at_position(text.as_ref(), position);
    let problem = resolve(&path, Some(text.as_ref())).ok()?;
    let (source_path_opt, span) = pf_dsl::resolver::find_definition(&problem, &path, offset)?;

    let target_path = source_path_opt.unwrap_or_else(|| path.clone());
    let target_uri = path_to_uri(&target_path).unwrap_or_else(|| uri.clone());
    let target_text = if target_path == path {
        Cow::Borrowed(text.as_ref())
    } else if let Some(buffer_text) = state.document_text(&target_uri) {
        Cow::Borrowed(buffer_text)
    } else {
        Cow::Owned(std::fs::read_to_string(&target_path).ok()?)
    };

    let range = span_to_range(target_text.as_ref(), span);

    Some(lsp_types::Location {
        uri: target_uri,
        range,
    })
}

fn send_response_error(
    connection: &Connection,
    id: lsp_server::RequestId,
    code: i32,
    message: String,
) -> Result<()> {
    let response = lsp_server::Response::new_err(id, code, message);
    connection.sender.send(Message::Response(response))?;
    Ok(())
}

fn span_to_range(text: &str, span: pf_dsl::ast::Span) -> Range {
    // This is a naive implementation.
    // Ideally we should use a line index to be faster.
    // Be careful with byte indices vs char indices if utf8.
    // pest Span is byte offsets.

    let start_byte = span.start;
    let end_byte = span.end;

    let start = position_at_byte(text, start_byte);
    let end = position_at_byte(text, end_byte);

    Range { start, end }
}

fn position_at_byte(text: &str, offset: usize) -> Position {
    let mut line = 0_u32;
    let mut character = 0_u32;

    for (i, c) in text.char_indices() {
        if i >= offset {
            break;
        }
        if c == '\n' {
            line += 1;
            character = 0;
        } else {
            // LSP character offsets are UTF-16 code units.
            character += c.len_utf16() as u32;
        }
    }

    Position { line, character }
}

fn offset_at_position(text: &str, position: Position) -> usize {
    let mut line = 0_u32;
    let mut character = 0_u32;
    for (i, c) in text.char_indices() {
        if line == position.line && character == position.character {
            return i;
        }

        if c == '\n' {
            line += 1;
            character = 0;
        } else {
            let next_character = character + c.len_utf16() as u32;
            if line == position.line && next_character > position.character {
                // Clamp to the start of a code point when client points into a surrogate pair.
                return i;
            }
            character = next_character;
        }
    }

    // Fallback if at end of file
    if line == position.line && character == position.character {
        return text.len();
    }

    text.len() // Invalid position fallback
}

fn parse_error_range(text: &str) -> Option<(Range, String)> {
    let (span, message) = pf_dsl::parser::parse_error_diagnostic(text)?;
    Some((span_to_range(text, span), message))
}

fn text_for_path<'a>(
    state: &'a ServerState,
    current_uri: &Uri,
    current_path: &Path,
    current_text: &'a str,
    target_path: &Path,
) -> Option<(Uri, Cow<'a, str>)> {
    if target_path == current_path {
        return Some((current_uri.clone(), Cow::Borrowed(current_text)));
    }
    let target_uri = path_to_uri(target_path)?;
    if let Some(buffer_text) = state.document_text(&target_uri) {
        return Some((target_uri, Cow::Borrowed(buffer_text)));
    }
    Some((
        target_uri,
        Cow::Owned(std::fs::read_to_string(target_path).ok()?),
    ))
}

fn push_diagnostic_for_uri(
    diagnostics_by_uri: &mut Vec<(Uri, Vec<Diagnostic>)>,
    target_uri: Uri,
    diagnostic: Diagnostic,
) {
    if let Some((_, diagnostics)) = diagnostics_by_uri
        .iter_mut()
        .find(|(uri, _)| uri == &target_uri)
    {
        diagnostics.push(diagnostic);
        return;
    }
    diagnostics_by_uri.push((target_uri, vec![diagnostic]));
}

fn validate_document(
    connection: &Connection,
    state: &mut ServerState,
    uri: Uri,
    text: &str,
) -> Result<()> {
    let mut diagnostics_by_uri: Vec<(Uri, Vec<Diagnostic>)> = vec![(uri.clone(), Vec::new())];

    // 1. Resolve (Parse + Imports)
    // We need to convert URI to Path
    let path = uri_to_path(&uri).ok_or_else(|| anyhow::anyhow!("Invalid URI scheme"))?;

    match resolve(&path, Some(text)) {
        Ok(problem) => {
            // 2. Semantic Validate
            match validate_with_sources(&problem) {
                Ok(_) => {}
                Err(issues) => {
                    for issue in issues {
                        let span = validation_error_span(&issue.error);
                        let target_path = issue.source_path.as_deref().unwrap_or(path.as_path());
                        let Some((target_uri, target_text)) =
                            text_for_path(state, &uri, path.as_path(), text, target_path)
                        else {
                            continue;
                        };

                        let diagnostic = Diagnostic {
                            range: span_to_range(target_text.as_ref(), span),
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: None,
                            code_description: None,
                            source: Some("pf-lsp".to_string()),
                            message: issue.error.to_string(),
                            related_information: None,
                            tags: None,
                            data: None,
                        };
                        push_diagnostic_for_uri(&mut diagnostics_by_uri, target_uri, diagnostic);
                    }
                }
            }
        }
        Err(e) => {
            let (range, message) = parse_error_range(text).unwrap_or_else(|| {
                (
                    Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 1,
                        },
                    },
                    e.to_string(),
                )
            });

            let diagnostic = Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                code: None,
                code_description: None,
                source: Some("pf-lsp".to_string()),
                message,
                related_information: None,
                tags: None,
                data: None,
            };
            push_diagnostic_for_uri(&mut diagnostics_by_uri, uri.clone(), diagnostic);
        }
    }

    let current_targets: HashSet<String> = diagnostics_by_uri
        .iter()
        .map(|(target_uri, _)| target_uri.as_str().to_string())
        .collect();

    for (target_uri, diagnostics) in diagnostics_by_uri {
        publish_diagnostics(connection, target_uri, diagnostics)?;
    }

    for stale_uri in state
        .update_diagnostic_targets(&uri, current_targets)
        .into_iter()
        .filter_map(|target| target.parse::<Uri>().ok())
    {
        publish_diagnostics(connection, stale_uri, Vec::new())?;
    }

    Ok(())
}

fn publish_diagnostics(
    connection: &Connection,
    uri: Uri,
    diagnostics: Vec<Diagnostic>,
) -> Result<()> {
    let params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version: None,
    };
    let not = Notification::new("textDocument/publishDiagnostics".to_string(), params);
    connection.sender.send(Message::Notification(not))?;
    Ok(())
}

fn uri_to_path(uri: &Uri) -> Option<PathBuf> {
    url::Url::parse(uri.as_str()).ok()?.to_file_path().ok()
}

fn path_to_uri(path: &Path) -> Option<Uri> {
    let file_url = url::Url::from_file_path(path).ok()?;
    file_url.as_str().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::{offset_at_position, position_at_byte};
    use lsp_types::Position;

    #[test]
    fn utf16_offset_mapping_handles_surrogate_pairs() {
        let text = "a\n😀b\n";

        assert_eq!(
            offset_at_position(
                text,
                Position {
                    line: 1,
                    character: 0
                }
            ),
            2
        );
        assert_eq!(
            offset_at_position(
                text,
                Position {
                    line: 1,
                    character: 2
                }
            ),
            6
        );
        assert_eq!(
            offset_at_position(
                text,
                Position {
                    line: 1,
                    character: 3
                }
            ),
            7
        );
    }

    #[test]
    fn byte_to_position_uses_utf16_columns() {
        let text = "a\n😀b\n";

        assert_eq!(
            position_at_byte(text, 2),
            Position {
                line: 1,
                character: 0
            }
        );
        assert_eq!(
            position_at_byte(text, 6),
            Position {
                line: 1,
                character: 2
            }
        );
        assert_eq!(
            position_at_byte(text, 7),
            Position {
                line: 1,
                character: 3
            }
        );
    }
}
