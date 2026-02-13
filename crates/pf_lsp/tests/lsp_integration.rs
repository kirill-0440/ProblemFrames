use serde_json::{json, Value};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const TIMEOUT: Duration = Duration::from_secs(5);

struct TestLspClient {
    child: Child,
    stdin: ChildStdin,
    rx: Receiver<Value>,
}

impl TestLspClient {
    fn spawn() -> Self {
        let exe = env!("CARGO_BIN_EXE_pf_lsp");
        let mut child = Command::new(exe)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to start pf_lsp");

        let stdin = child.stdin.take().expect("failed to open child stdin");
        let stdout = child.stdout.take().expect("failed to open child stdout");
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            while let Some(msg) = read_lsp_message(&mut reader) {
                if tx.send(msg).is_err() {
                    break;
                }
            }
        });

        let mut client = Self { child, stdin, rx };
        client.initialize();
        client
    }

    fn initialize(&mut self) {
        self.send(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "capabilities": {}
            }
        }));
        self.wait_for(|msg| msg.get("id") == Some(&json!(1)))
            .expect("did not receive initialize response");

        self.send(json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        }));
    }

    fn send(&mut self, body: Value) {
        let encoded = serde_json::to_vec(&body).expect("failed to serialize message");
        write!(self.stdin, "Content-Length: {}\r\n\r\n", encoded.len())
            .expect("failed to write lsp header");
        self.stdin
            .write_all(&encoded)
            .expect("failed to write lsp body");
        self.stdin.flush().expect("failed to flush lsp input");
    }

    fn wait_for<F>(&self, predicate: F) -> Option<Value>
    where
        F: Fn(&Value) -> bool,
    {
        let deadline = Instant::now() + TIMEOUT;
        loop {
            let remaining = deadline.checked_duration_since(Instant::now())?;
            let msg = self.rx.recv_timeout(remaining).ok()?;
            if predicate(&msg) {
                return Some(msg);
            }
        }
    }
}

impl Drop for TestLspClient {
    fn drop(&mut self) {
        self.send(json!({
            "jsonrpc": "2.0",
            "id": 999,
            "method": "shutdown",
            "params": null
        }));
        let _ = self.wait_for(|msg| msg.get("id") == Some(&json!(999)));
        self.send(json!({
            "jsonrpc": "2.0",
            "method": "exit",
            "params": null
        }));
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn read_lsp_message<R: BufRead + Read>(reader: &mut R) -> Option<Value> {
    let mut content_length: Option<usize> = None;

    loop {
        let mut header = String::new();
        let bytes = reader.read_line(&mut header).ok()?;
        if bytes == 0 {
            return None;
        }
        if header == "\r\n" {
            break;
        }
        let lower = header.to_ascii_lowercase();
        if let Some(length) = lower.strip_prefix("content-length:") {
            content_length = length.trim().parse::<usize>().ok();
        }
    }

    let len = content_length?;
    let mut payload = vec![0_u8; len];
    reader.read_exact(&mut payload).ok()?;
    serde_json::from_slice(&payload).ok()
}

fn make_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn file_uri(path: &Path) -> String {
    url::Url::from_file_path(path)
        .expect("failed to build file uri")
        .to_string()
}

fn position_of(text: &str, needle: &str, nth: usize) -> lsp_types::Position {
    let mut start = 0;
    let mut offset = None;
    for _ in 0..=nth {
        let idx = text[start..]
            .find(needle)
            .expect("needle not found for position");
        let absolute = start + idx;
        offset = Some(absolute);
        start = absolute + needle.len();
    }
    let target = offset.expect("missing offset");

    let mut line = 0_u32;
    let mut character = 0_u32;
    for (byte_idx, ch) in text.char_indices() {
        if byte_idx >= target {
            break;
        }
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16() as u32;
        }
    }

    lsp_types::Position { line, character }
}

fn position_for_offset(text: &str, target: usize) -> lsp_types::Position {
    let mut line = 0_u32;
    let mut character = 0_u32;
    for (byte_idx, ch) in text.char_indices() {
        if byte_idx >= target {
            break;
        }
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16() as u32;
        }
    }
    lsp_types::Position { line, character }
}

#[test]
fn diagnostics_follow_did_change_buffer_state() {
    let dir = make_temp_dir("pf-lsp-diagnostics");
    let path = dir.join("problem.pf");
    let uri = file_uri(&path);
    let mut client = TestLspClient::spawn();

    let invalid_text =
        "problem: P\ninterface \"A-B\" connects A, B { shared: { phenomenon e : event [A -> B] controlledBy A } }\n";
    client.send(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": uri,
                "languageId": "pf",
                "version": 1,
                "text": invalid_text
            }
        }
    }));

    let open_diag = client
        .wait_for(|msg| msg.get("method") == Some(&json!("textDocument/publishDiagnostics")))
        .expect("did not receive diagnostics after didOpen");
    let diagnostics = open_diag["params"]["diagnostics"]
        .as_array()
        .expect("diagnostics must be array");
    assert!(
        !diagnostics.is_empty(),
        "expected non-empty diagnostics for invalid buffer"
    );

    let fixed_text = "problem: P\ndomain A kind causal role machine\ndomain B kind causal role given\ninterface \"A-B\" connects A, B { shared: { phenomenon e : event [A -> B] controlledBy A } }\n";
    client.send(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didChange",
        "params": {
            "textDocument": {
                "uri": uri,
                "version": 2
            },
            "contentChanges": [
                { "text": fixed_text }
            ]
        }
    }));

    let change_diag = client
        .wait_for(|msg| msg.get("method") == Some(&json!("textDocument/publishDiagnostics")))
        .expect("did not receive diagnostics after didChange");
    let diagnostics = change_diag["params"]["diagnostics"]
        .as_array()
        .expect("diagnostics must be array");
    assert!(
        diagnostics.is_empty(),
        "expected empty diagnostics after fix"
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn definition_uses_unsaved_buffer_content() {
    let dir = make_temp_dir("pf-lsp-definition");
    let path = dir.join("problem.pf");
    let uri = file_uri(&path);

    // On-disk content intentionally differs from open buffer content.
    let disk_text = "problem: P\ndomain Old kind causal role machine\ndomain T kind causal role given\ninterface \"I\" connects Old, T { shared: { phenomenon e : event [Old -> T] controlledBy Old } }\n";
    fs::write(&path, disk_text).expect("failed to write test file");

    let live_text = "problem: P\ndomain New kind causal role machine\ndomain T kind causal role given\ninterface \"I\" connects New, T { shared: { phenomenon e : event [New -> T] controlledBy New } }\n";
    let position = position_of(live_text, "New", 1);

    let mut client = TestLspClient::spawn();
    client.send(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": uri,
                "languageId": "pf",
                "version": 1,
                "text": live_text
            }
        }
    }));

    let _ =
        client.wait_for(|msg| msg.get("method") == Some(&json!("textDocument/publishDiagnostics")));

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "textDocument/definition",
        "params": {
            "textDocument": { "uri": uri },
            "position": {
                "line": position.line,
                "character": position.character
            }
        }
    }));

    let response = client
        .wait_for(|msg| msg.get("id") == Some(&json!(2)))
        .expect("did not receive definition response");

    assert_ne!(
        response.get("result"),
        Some(&Value::Null),
        "definition should resolve from unsaved buffer state"
    );
    assert_eq!(
        response["result"]["uri"].as_str(),
        Some(uri.as_str()),
        "definition should stay in current file"
    );
    assert_eq!(
        response["result"]["range"]["start"]["line"].as_u64(),
        Some(1),
        "definition should point to 'domain New'"
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn definition_resolves_subproblem_and_requirement_references() {
    let dir = make_temp_dir("pf-lsp-subproblem-definition");
    let path = dir.join("problem.pf");
    let uri = file_uri(&path);

    let text = "problem: P\ndomain M kind causal role machine\ndomain Sensor kind causal role given\nrequirement \"R_sub\" {\n  frame: RequiredBehavior\n  constrains: Sensor\n}\nsubproblem Core {\n  machine: M\n  participants: M, Sensor\n  requirements: \"R_sub\"\n}\n";
    let participant_position = position_of(text, "Sensor", 2);
    let requirement_position = position_of(text, "\"R_sub\"", 1);

    let mut client = TestLspClient::spawn();
    client.send(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": uri,
                "languageId": "pf",
                "version": 1,
                "text": text
            }
        }
    }));

    let _ =
        client.wait_for(|msg| msg.get("method") == Some(&json!("textDocument/publishDiagnostics")));

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 21,
        "method": "textDocument/definition",
        "params": {
            "textDocument": { "uri": uri },
            "position": {
                "line": participant_position.line,
                "character": participant_position.character
            }
        }
    }));

    let participant_response = client
        .wait_for(|msg| msg.get("id") == Some(&json!(21)))
        .expect("did not receive subproblem participant definition response");
    assert_eq!(
        participant_response["result"]["uri"].as_str(),
        Some(uri.as_str()),
        "participant definition should stay in current file"
    );
    assert_eq!(
        participant_response["result"]["range"]["start"]["line"].as_u64(),
        Some(2),
        "participant reference should resolve to domain Sensor declaration"
    );

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 22,
        "method": "textDocument/definition",
        "params": {
            "textDocument": { "uri": uri },
            "position": {
                "line": requirement_position.line,
                "character": requirement_position.character
            }
        }
    }));

    let requirement_response = client
        .wait_for(|msg| msg.get("id") == Some(&json!(22)))
        .expect("did not receive subproblem requirement definition response");
    assert_eq!(
        requirement_response["result"]["uri"].as_str(),
        Some(uri.as_str()),
        "requirement definition should stay in current file"
    );
    assert_eq!(
        requirement_response["result"]["range"]["start"]["line"].as_u64(),
        Some(3),
        "requirement reference should resolve to requirement declaration"
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn definition_resolves_correctness_argument_assertion_sets() {
    let dir = make_temp_dir("pf-lsp-correctness-definition");
    let path = dir.join("problem.pf");
    let uri = file_uri(&path);

    let text = "problem: P\ndomain M kind causal role machine\nworldProperties W_base {\n  assert \"world\"\n}\nspecification S_control {\n  assert \"spec\"\n}\nrequirementAssertions R_goal {\n  assert \"goal\"\n}\ncorrectnessArgument A1 {\n  prove S_control and W_base entail R_goal\n}\n";
    let spec_position = position_of(text, "S_control", 1);
    let world_position = position_of(text, "W_base", 1);
    let req_position = position_of(text, "R_goal", 1);

    let mut client = TestLspClient::spawn();
    client.send(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": uri,
                "languageId": "pf",
                "version": 1,
                "text": text
            }
        }
    }));

    let _ =
        client.wait_for(|msg| msg.get("method") == Some(&json!("textDocument/publishDiagnostics")));

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 23,
        "method": "textDocument/definition",
        "params": {
            "textDocument": { "uri": uri },
            "position": {
                "line": spec_position.line,
                "character": spec_position.character
            }
        }
    }));
    let spec_response = client
        .wait_for(|msg| msg.get("id") == Some(&json!(23)))
        .expect("did not receive correctness specification definition response");
    assert_eq!(
        spec_response["result"]["range"]["start"]["line"].as_u64(),
        Some(5),
        "S reference should resolve to specification declaration"
    );

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 24,
        "method": "textDocument/definition",
        "params": {
            "textDocument": { "uri": uri },
            "position": {
                "line": world_position.line,
                "character": world_position.character
            }
        }
    }));
    let world_response = client
        .wait_for(|msg| msg.get("id") == Some(&json!(24)))
        .expect("did not receive correctness world definition response");
    assert_eq!(
        world_response["result"]["range"]["start"]["line"].as_u64(),
        Some(2),
        "W reference should resolve to worldProperties declaration"
    );

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 25,
        "method": "textDocument/definition",
        "params": {
            "textDocument": { "uri": uri },
            "position": {
                "line": req_position.line,
                "character": req_position.character
            }
        }
    }));
    let requirement_response = client
        .wait_for(|msg| msg.get("id") == Some(&json!(25)))
        .expect("did not receive correctness requirement definition response");
    assert_eq!(
        requirement_response["result"]["range"]["start"]["line"].as_u64(),
        Some(8),
        "R reference should resolve to requirementAssertions declaration"
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn parse_errors_report_precise_range() {
    let dir = make_temp_dir("pf-lsp-parse-range");
    let path = dir.join("problem.pf");
    let uri = file_uri(&path);
    let mut client = TestLspClient::spawn();

    let invalid_text = "problem: P\ndomain A kind causal role machine\ndomain B kind causal role given\ninterface \"A-B\" connects A, B {\n  shared: {\n    phenomenon e : event [A -> B] controlledBy\n  }\n}\n";

    client.send(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": uri,
                "languageId": "pf",
                "version": 1,
                "text": invalid_text
            }
        }
    }));

    let open_diag = client
        .wait_for(|msg| msg.get("method") == Some(&json!("textDocument/publishDiagnostics")))
        .expect("did not receive diagnostics after didOpen");
    let diagnostics = open_diag["params"]["diagnostics"]
        .as_array()
        .expect("diagnostics must be array");
    assert!(
        !diagnostics.is_empty(),
        "expected non-empty diagnostics for parse error"
    );

    let start_line = diagnostics[0]["range"]["start"]["line"]
        .as_u64()
        .expect("missing range.start.line");
    let start_character = diagnostics[0]["range"]["start"]["character"]
        .as_u64()
        .expect("missing range.start.character");
    assert!(
        start_line > 0 || start_character > 0,
        "parse diagnostic should not default to 0:0"
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn definition_ignores_imported_span_collisions() {
    let dir = make_temp_dir("pf-lsp-definition-collision");
    let root_path = dir.join("root.pf");
    let import_path = dir.join("imp.pf");
    let root_uri = file_uri(&root_path);

    let imported_text = "problem: Imported\ndomain A kind causal role machine\ndomain B kind causal role given\ninterface \"I\" connects A, B { shared: { phenomenon ev : event [A -> B] controlledBy A } }\n";
    fs::write(&import_path, imported_text).expect("failed to write imported file");

    let mut root_text = String::from("problem: Root\nimport \"imp.pf\"\n");
    for idx in 0..40 {
        let role = if idx == 0 { "machine" } else { "given" };
        root_text.push_str(&format!("domain D{idx} kind causal role {role}\n"));
    }

    let probe_offset = 60_usize.min(root_text.len().saturating_sub(1));
    let probe_position = position_for_offset(&root_text, probe_offset);

    let mut client = TestLspClient::spawn();
    client.send(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": root_uri,
                "languageId": "pf",
                "version": 1,
                "text": root_text
            }
        }
    }));

    let _ =
        client.wait_for(|msg| msg.get("method") == Some(&json!("textDocument/publishDiagnostics")));

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "textDocument/definition",
        "params": {
            "textDocument": { "uri": root_uri },
            "position": {
                "line": probe_position.line,
                "character": probe_position.character
            }
        }
    }));

    let response = client
        .wait_for(|msg| msg.get("id") == Some(&json!(3)))
        .expect("did not receive definition response");

    assert_eq!(
        response.get("result"),
        Some(&Value::Null),
        "definition on non-reference offset should not jump to imported file"
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn diagnostics_are_attributed_to_imported_file_uri() {
    let dir = make_temp_dir("pf-lsp-import-diagnostics");
    let root_path = dir.join("root.pf");
    let import_path = dir.join("imp.pf");
    let root_uri = file_uri(&root_path);
    let import_uri = file_uri(&import_path);

    let imported_text = "problem: Imported\ndomain M kind causal role machine\nrequirement \"Broken\" {\n  frame: RequiredBehavior\n  constrains: Missing\n}\n";
    fs::write(&import_path, imported_text).expect("failed to write imported file");

    let root_text =
        "problem: Root\nimport \"imp.pf\"\ndomain RootMachine kind causal role machine\n";

    let mut client = TestLspClient::spawn();
    client.send(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": root_uri,
                "languageId": "pf",
                "version": 1,
                "text": root_text
            }
        }
    }));

    let imported_diag = client
        .wait_for(|msg| {
            msg.get("method") == Some(&json!("textDocument/publishDiagnostics"))
                && msg["params"]["uri"] == json!(import_uri)
        })
        .expect("did not receive diagnostics for imported file");
    let diagnostics = imported_diag["params"]["diagnostics"]
        .as_array()
        .expect("diagnostics must be array");
    assert!(
        !diagnostics.is_empty(),
        "expected imported file diagnostics to be non-empty"
    );
    assert!(
        diagnostics[0]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("Missing"),
        "expected message to mention missing domain"
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn imported_diagnostics_are_cleared_after_import_removed() {
    let dir = make_temp_dir("pf-lsp-import-clear");
    let import_path = dir.join("imp.pf");
    let root_path = dir.join("root.pf");
    let import_uri = file_uri(&import_path);
    let root_uri = file_uri(&root_path);

    let imported_text = "problem: Imported\ndomain M kind causal role machine\nrequirement \"Broken\" {\n  frame: RequiredBehavior\n  constrains: Missing\n}\n";
    fs::write(&import_path, imported_text).expect("failed to write imported file");

    let root_text =
        "problem: Root\nimport \"imp.pf\"\ndomain RootMachine kind causal role machine\n";
    let root_without_import = "problem: Root\ndomain RootMachine kind causal role machine\n";

    let mut client = TestLspClient::spawn();
    client.send(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": root_uri,
                "languageId": "pf",
                "version": 1,
                "text": root_text
            }
        }
    }));

    let imported_diag = client
        .wait_for(|msg| {
            msg.get("method") == Some(&json!("textDocument/publishDiagnostics"))
                && msg["params"]["uri"] == json!(import_uri)
        })
        .expect("did not receive diagnostics for imported file");
    let diagnostics = imported_diag["params"]["diagnostics"]
        .as_array()
        .expect("diagnostics must be array");
    assert!(
        !diagnostics.is_empty(),
        "expected imported diagnostics before removing import"
    );

    client.send(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didChange",
        "params": {
            "textDocument": {
                "uri": root_uri,
                "version": 2
            },
            "contentChanges": [
                { "text": root_without_import }
            ]
        }
    }));

    let cleared_diag = client
        .wait_for(|msg| {
            msg.get("method") == Some(&json!("textDocument/publishDiagnostics"))
                && msg["params"]["uri"] == json!(import_uri)
                && msg["params"]["diagnostics"]
                    .as_array()
                    .map(|arr| arr.is_empty())
                    .unwrap_or(false)
        })
        .expect("did not receive clearing diagnostics for imported file");

    let diagnostics = cleared_diag["params"]["diagnostics"]
        .as_array()
        .expect("diagnostics must be array");
    assert!(
        diagnostics.is_empty(),
        "expected imported diagnostics to be cleared after removing import"
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn dogfooding_flow_handles_import_collision_and_definition_rebind() {
    let dir = make_temp_dir("pf-lsp-dogfooding-flow");
    let root_path = dir.join("root.pf");
    let import_path = dir.join("imp.pf");
    let root_uri = file_uri(&root_path);
    let import_uri = file_uri(&import_path);

    let imported_text = "problem: Imported\ndomain M kind causal role machine\ndomain A kind causal role given\ninterface \"M-A\" connects M, A {\n  shared: {\n    phenomenon Observe : event [A -> M] controlledBy A\n  }\n}\nrequirement \"R_shared\" {\n  frame: RequiredBehavior\n  constrains: A\n}\n";
    fs::write(&import_path, imported_text).expect("failed to write imported model");

    let root_v1 = "problem: Root\nimport \"imp.pf\"\nsubproblem Core {\n  machine: M\n  participants: M, A\n  requirements: \"R_shared\"\n}\n";
    let root_v2 = "problem: Root\nimport \"imp.pf\"\nrequirement \"R_shared\" {\n  frame: RequiredBehavior\n  constrains: A\n}\nsubproblem Core {\n  machine: M\n  participants: M, A\n  requirements: \"R_shared\"\n}\n";
    let reference_pos_v1 = position_of(root_v1, "\"R_shared\"", 0);
    let reference_pos_v2 = position_of(root_v2, "\"R_shared\"", 1);

    let mut client = TestLspClient::spawn();
    client.send(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": root_uri,
                "languageId": "pf",
                "version": 1,
                "text": root_v1
            }
        }
    }));

    let root_diag_v1 = client
        .wait_for(|msg| {
            msg.get("method") == Some(&json!("textDocument/publishDiagnostics"))
                && msg["params"]["uri"] == json!(root_uri)
        })
        .expect("did not receive diagnostics for root on didOpen");
    let root_v1_diagnostics = root_diag_v1["params"]["diagnostics"]
        .as_array()
        .expect("diagnostics must be array");
    assert!(
        root_v1_diagnostics.is_empty(),
        "initial dogfooding model should be valid"
    );

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 31,
        "method": "textDocument/definition",
        "params": {
            "textDocument": { "uri": root_uri },
            "position": {
                "line": reference_pos_v1.line,
                "character": reference_pos_v1.character
            }
        }
    }));

    let def_v1 = client
        .wait_for(|msg| msg.get("id") == Some(&json!(31)))
        .expect("did not receive initial definition response");
    assert_eq!(
        def_v1["result"]["uri"].as_str(),
        Some(import_uri.as_str()),
        "initial definition should resolve into imported model"
    );

    client.send(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didChange",
        "params": {
            "textDocument": {
                "uri": root_uri,
                "version": 2
            },
            "contentChanges": [
                { "text": root_v2 }
            ]
        }
    }));

    let imported_duplicate_diag = client
        .wait_for(|msg| {
            msg.get("method") == Some(&json!("textDocument/publishDiagnostics"))
                && msg["params"]["uri"] == json!(import_uri)
                && msg["params"]["diagnostics"]
                    .as_array()
                    .map(|diagnostics| {
                        diagnostics.iter().any(|diagnostic| {
                            diagnostic["message"]
                                .as_str()
                                .unwrap_or_default()
                                .contains("Duplicate requirement definition")
                        })
                    })
                    .unwrap_or(false)
        })
        .expect("did not receive duplicate requirement diagnostics for import");
    let imported_v2_diagnostics = imported_duplicate_diag["params"]["diagnostics"]
        .as_array()
        .expect("diagnostics must be array");
    assert!(
        !imported_v2_diagnostics.is_empty(),
        "expected diagnostics after introducing import collision"
    );

    client.send(json!({
        "jsonrpc": "2.0",
        "id": 32,
        "method": "textDocument/definition",
        "params": {
            "textDocument": { "uri": root_uri },
            "position": {
                "line": reference_pos_v2.line,
                "character": reference_pos_v2.character
            }
        }
    }));

    let def_v2 = client
        .wait_for(|msg| msg.get("id") == Some(&json!(32)))
        .expect("did not receive updated definition response");
    assert_eq!(
        def_v2["result"]["uri"].as_str(),
        Some(root_uri.as_str()),
        "after adding local symbol, definition should resolve in root file"
    );

    let _ = fs::remove_dir_all(dir);
}
