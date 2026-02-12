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
    lsp_types::Url::from_file_path(path)
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

#[test]
fn diagnostics_follow_did_change_buffer_state() {
    let dir = make_temp_dir("pf-lsp-diagnostics");
    let path = dir.join("problem.pf");
    let uri = file_uri(&path);
    let mut client = TestLspClient::spawn();

    let invalid_text = "problem: P\ninterface \"A-B\" { shared: { event e [A -> B] } }\n";
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

    let fixed_text = "problem: P\ndomain A [Machine]\ndomain B [Causal]\ninterface \"A-B\" { shared: { event e [A -> B] } }\n";
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
    let disk_text =
        "problem: P\ndomain Old [Machine]\ndomain T [Causal]\ninterface \"I\" { shared: { event e [Old -> T] } }\n";
    fs::write(&path, disk_text).expect("failed to write test file");

    let live_text =
        "problem: P\ndomain New [Machine]\ndomain T [Causal]\ninterface \"I\" { shared: { event e [New -> T] } }\n";
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
