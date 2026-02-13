use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn make_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn run_pf_dsl(path: &PathBuf, mode: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_pf_dsl"))
        .arg(path)
        .arg(mode)
        .output()
        .expect("failed to execute pf_dsl binary")
}

#[test]
fn dogfooding_cli_generates_artifacts_for_imported_model() {
    let dir = make_temp_dir("pf-cli-dogfooding-success");
    let root_path = dir.join("root.pf");
    let import_path = dir.join("imp.pf");

    let imported = r#"
problem: Imported
domain M kind causal role machine
domain A kind causal role given
interface "M-A" connects M, A {
  shared: {
    phenomenon Observe : event [A -> M] controlledBy A
  }
}
requirement "R1" {
  frame: RequiredBehavior
  constrains: A
}
worldProperties W_base {
  assert "world stable"
}
specification S_ctrl {
  assert "controller reacts"
}
requirementAssertions R_goal {
  assert "goal reached"
}
correctnessArgument A1 {
  prove S_ctrl and W_base entail R_goal
}
"#;
    fs::write(&import_path, imported).expect("failed to write import file");
    fs::write(&root_path, "problem: Root\nimport \"imp.pf\"\n").expect("failed to write root file");

    let report = run_pf_dsl(&root_path, "--report");
    assert!(
        report.status.success(),
        "report mode should succeed: {}",
        String::from_utf8_lossy(&report.stderr)
    );
    assert!(
        String::from_utf8_lossy(&report.stdout).contains("# Problem Report: Root"),
        "report output should contain root report header"
    );

    let obligations = run_pf_dsl(&root_path, "--obligations");
    assert!(
        obligations.status.success(),
        "obligations mode should succeed: {}",
        String::from_utf8_lossy(&obligations.stderr)
    );
    let obligations_stdout = String::from_utf8_lossy(&obligations.stdout);
    assert!(
        obligations_stdout.contains("# Proof Obligations: Root"),
        "obligations output should contain root obligations header"
    );
    assert!(
        obligations_stdout.contains("obl_A1"),
        "obligations output should contain generated obligation id"
    );

    let alloy = run_pf_dsl(&root_path, "--alloy");
    assert!(
        alloy.status.success(),
        "alloy mode should succeed: {}",
        String::from_utf8_lossy(&alloy.stderr)
    );
    let alloy_stdout = String::from_utf8_lossy(&alloy.stdout);
    assert!(
        alloy_stdout.contains("module Root"),
        "alloy output should contain root module declaration"
    );
    assert!(
        alloy_stdout.contains("pred Obl_A1"),
        "alloy output should contain obligation predicate"
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn dogfooding_cli_reports_import_requirement_collision() {
    let dir = make_temp_dir("pf-cli-dogfooding-collision");
    let root_path = dir.join("root.pf");
    let a_path = dir.join("a.pf");
    let b_path = dir.join("b.pf");

    let imported = r#"
problem: Imported
domain M kind causal role machine
domain A kind causal role given
interface "M-A" connects M, A {
  shared: {
    phenomenon Observe : event [A -> M] controlledBy A
  }
}
requirement "R_shared" {
  frame: RequiredBehavior
  constrains: A
}
"#;
    fs::write(&a_path, imported).expect("failed to write first import");
    fs::write(&b_path, imported).expect("failed to write second import");
    fs::write(
        &root_path,
        "problem: Root\nimport \"a.pf\"\nimport \"b.pf\"\n",
    )
    .expect("failed to write root file");

    let output = run_pf_dsl(&root_path, "--report");
    assert!(
        !output.status.success(),
        "validation should fail when imports collide"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Duplicate requirement definition"),
        "stderr should report duplicate requirement collision"
    );

    let _ = fs::remove_dir_all(dir);
}
