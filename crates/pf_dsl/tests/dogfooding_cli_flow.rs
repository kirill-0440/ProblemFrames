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

fn run_pf_dsl_with_args(path: &PathBuf, mode: &str, extra_args: &[&str]) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_pf_dsl"));
    cmd.arg(path).arg(mode);
    for arg in extra_args {
        cmd.arg(arg);
    }
    cmd.output().expect("failed to execute pf_dsl binary")
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
    assert!(
        String::from_utf8_lossy(&report.stdout).contains("## 5. Decomposition Closure"),
        "report output should include decomposition closure section"
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

#[test]
fn dogfooding_cli_generates_traceability_exports_with_impact() {
    let dir = make_temp_dir("pf-cli-traceability-success");
    let root_path = dir.join("root.pf");
    let import_path = dir.join("imp.pf");

    let imported = r#"
problem: Imported
domain M kind causal role machine
domain Operator kind biddable role given
domain Sensor kind causal role given
interface "Operator-M" connects Operator, M {
  shared: {
    phenomenon Command : event [Operator -> M] controlledBy Operator
  }
}
interface "Sensor-M" connects Sensor, M {
  shared: {
    phenomenon Reading : value [Sensor -> M] controlledBy Sensor
  }
}
requirement "R_display" {
  frame: InformationDisplay
  constrains: Sensor
  reference: Operator
}
subproblem Display {
  machine: M
  participants: M, Operator, Sensor
  requirements: "R_display"
}
"#;
    fs::write(&import_path, imported).expect("failed to write import file");
    fs::write(&root_path, "problem: Root\nimport \"imp.pf\"\n").expect("failed to write root file");

    let markdown =
        run_pf_dsl_with_args(&root_path, "--traceability-md", &["--impact=domain:Sensor"]);
    assert!(
        markdown.status.success(),
        "traceability markdown mode should succeed: {}",
        String::from_utf8_lossy(&markdown.stderr)
    );
    let markdown_stdout = String::from_utf8_lossy(&markdown.stdout);
    assert!(markdown_stdout.contains("# Traceability Report: Root"));
    assert!(markdown_stdout.contains("`domain:Sensor` -> R_display"));

    let csv = run_pf_dsl_with_args(
        &root_path,
        "--traceability-csv",
        &["--impact=requirement:R_display", "--impact-hops=2"],
    );
    assert!(
        csv.status.success(),
        "traceability csv mode should succeed: {}",
        String::from_utf8_lossy(&csv.stderr)
    );
    let csv_stdout = String::from_utf8_lossy(&csv.stdout);
    assert!(csv_stdout.starts_with("record_type,from_kind,from_id,relation,to_kind,to_id"));
    assert!(csv_stdout.contains("impact,,,,,,requirement,R_display,R_display,2"));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn dogfooding_cli_rejects_unknown_traceability_impact_seed() {
    let dir = make_temp_dir("pf-cli-traceability-error");
    let path = dir.join("simple.pf");
    fs::write(
        &path,
        r#"
problem: P
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
"#,
    )
    .expect("failed to write model");

    let output = run_pf_dsl_with_args(
        &path,
        "--traceability-md",
        &["--impact=requirement:MissingRequirement"],
    );
    assert!(
        !output.status.success(),
        "traceability should fail on unknown impact seed"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("unknown requirement impact seed 'MissingRequirement'"),
        "stderr should explain the invalid impact seed"
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn dogfooding_cli_generates_view_specific_dot_exports() {
    let dir = make_temp_dir("pf-cli-dot-views");
    let path = dir.join("views.pf");
    fs::write(
        &path,
        r#"
problem: Views
domain M kind causal role machine
domain User kind biddable role given
domain Ledger kind lexical role given
interface "User-M" connects User, M {
  shared: {
    phenomenon Command : event [User -> M] controlledBy User
  }
}
interface "M-Ledger" connects M, Ledger {
  shared: {
    phenomenon Persist : value [M -> Ledger] controlledBy M
  }
}
requirement "R1" {
  frame: SimpleWorkpieces
  constrains: Ledger
  reference: User
}
subproblem Core {
  machine: M
  participants: M, User, Ledger
  requirements: "R1"
}
"#,
    )
    .expect("failed to write model");

    let context = run_pf_dsl(&path, "--dot-context");
    assert!(
        context.status.success(),
        "context dot export should succeed"
    );
    let context_stdout = String::from_utf8_lossy(&context.stdout);
    assert!(context_stdout.contains("Persist [V]"));
    assert!(!context_stdout.contains("R1\\n[SimpleWorkpieces]"));

    let problem = run_pf_dsl(&path, "--dot-problem");
    assert!(
        problem.status.success(),
        "problem dot export should succeed"
    );
    let problem_stdout = String::from_utf8_lossy(&problem.stdout);
    assert!(problem_stdout.contains("\"R1\" [shape=note"));
    assert!(problem_stdout.contains("label=\"constrains\""));

    let decomposition = run_pf_dsl(&path, "--dot-decomposition");
    assert!(
        decomposition.status.success(),
        "decomposition dot export should succeed"
    );
    let decomposition_stdout = String::from_utf8_lossy(&decomposition.stdout);
    assert!(decomposition_stdout.contains("\"subproblem:Core\""));
    assert!(decomposition_stdout.contains("label=\"includes\""));
    assert!(!decomposition_stdout.contains("[dir=both"));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn dogfooding_cli_generates_decomposition_closure_report() {
    let dir = make_temp_dir("pf-cli-decomposition-closure");
    let path = dir.join("closure.pf");
    fs::write(
        &path,
        r#"
problem: Closure
domain M kind causal role machine
domain Operator kind biddable role given
domain Device kind causal role given
interface "Operator-M" connects Operator, M {
  shared: {
    phenomenon Command : command [Operator -> M] controlledBy Operator
  }
}
interface "M-Device" connects M, Device {
  shared: {
    phenomenon Actuate : event [M -> Device] controlledBy M
  }
}
requirement "R_covered" {
  frame: CommandedBehavior
  constrains: Device
  reference: Operator
}
requirement "R_uncovered" {
  frame: RequiredBehavior
  constrains: Device
}
subproblem Execution {
  machine: M
  participants: M, Operator, Device
  requirements: "R_covered"
}
"#,
    )
    .expect("failed to write model");

    let output = run_pf_dsl(&path, "--decomposition-closure");
    assert!(
        output.status.success(),
        "decomposition closure mode should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("# Decomposition Closure Report: Closure"));
    assert!(stdout.contains("| R_uncovered | - | uncovered |"));
    assert!(stdout.contains("### Orphan Subproblems"));
    assert!(stdout.contains("- None."));

    let _ = fs::remove_dir_all(dir);
}
