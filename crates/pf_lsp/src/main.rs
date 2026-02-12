use anyhow::Result;
use lsp_server::{Connection, Message, Notification};
use lsp_types::{
    Diagnostic, DiagnosticSeverity, InitializeParams, Position, PublishDiagnosticsParams, Range,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};
use pf_dsl::parser::parse;
use pf_dsl::validator::validate;
use serde_json::Value;

fn main() -> Result<()> {
    eprintln!("Starting PF LSP Server...");

    // Create the transport
    let (connection, io_threads) = Connection::stdio();

    // Initialize
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
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

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                // Handle requests (none for now)
            }
            Message::Response(_resp) => {
                // handle responses
            }
            Message::Notification(not) => {
                // handle notifications
                match not.method.as_str() {
                    "textDocument/didOpen" => {
                        let params: lsp_types::DidOpenTextDocumentParams =
                            serde_json::from_value(not.params)?;
                        validate_document(
                            &connection,
                            params.text_document.uri,
                            &params.text_document.text,
                        )?;
                    }
                    "textDocument/didChange" => {
                        let params: lsp_types::DidChangeTextDocumentParams =
                            serde_json::from_value(not.params)?;
                        // FULL sync, so content_changes[0].text is the whole file
                        if let Some(change) = params.content_changes.first() {
                            validate_document(&connection, params.text_document.uri, &change.text)?;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
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
    let mut line = 0;
    let mut character = 0;
    let mut current_byte = 0;

    for (i, c) in text.char_indices() {
        if i >= offset {
            break;
        }
        if c == '\n' {
            line += 1;
            character = 0;
        } else {
            character += 1; // logical character count (LSP uses utf-16 usually, but let's assume simple for now)
        }
        current_byte = i + c.len_utf8();
    }

    // If offset is passed the last char
    if offset > current_byte && offset <= text.len() {
        // This logic is a bit linear and slow for large files but fine for now.
        // We might be slightly off if we broke early.
        // A better approach is `line_index` crate.
    }

    Position { line, character }
}

fn validate_document(connection: &Connection, uri: Url, text: &str) -> Result<()> {
    let mut diagnostics = Vec::new();

    // 1. Parse
    match parse(text) {
        Ok(problem) => {
            // 2. Semantic Validate
            match validate(&problem) {
                Ok(_) => {}
                Err(errors) => {
                    for err in errors {
                        // Extract Span from ValidationError
                        let span = match &err {
                            pf_dsl::validator::ValidationError::UndefinedDomainInInterface(
                                _,
                                _,
                                s,
                            ) => *s,
                            pf_dsl::validator::ValidationError::UndefinedDomainInRequirement(
                                _,
                                _,
                                s,
                            ) => *s,
                            pf_dsl::validator::ValidationError::InvalidFrameDomain(_, _, _, s) => {
                                *s
                            }
                        };

                        let diagnostic = Diagnostic {
                            range: span_to_range(text, span),
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: None,
                            code_description: None,
                            source: Some("pf-lsp".to_string()),
                            message: err.to_string(),
                            related_information: None,
                            tags: None,
                            data: None,
                        };
                        diagnostics.push(diagnostic);
                    }
                }
            }
        }
        Err(e) => {
            // Parser error
            // We can parse generic pest error to get location if we want
            // For now, default to top of file or try to extract location
            let range = Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 1,
                },
            };

            let diagnostic = Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                code: None,
                code_description: None,
                source: Some("pf-lsp".to_string()),
                message: e.to_string(),
                related_information: None,
                tags: None,
                data: None,
            };
            diagnostics.push(diagnostic);
        }
    }

    let params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version: None,
    };

    let not = Notification::new("textDocument/publishDiagnostics".to_string(), params);
    connection.sender.send(Message::Notification(not))?;

    Ok(())
}
