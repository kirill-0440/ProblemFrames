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
subproblem S1 {
  machine: M
  participants: M, A
  requirements: "R1"
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

    let decomposition = run_pf_dsl(&root_path, "--decomposition-closure");
    assert!(
        decomposition.status.success(),
        "decomposition mode should succeed: {}",
        String::from_utf8_lossy(&decomposition.stderr)
    );
    let decomposition_stdout = String::from_utf8_lossy(&decomposition.stdout);
    assert!(
        decomposition_stdout.contains("Closure status: PASS"),
        "decomposition closure should pass for covered model"
    );

    let concern_coverage = run_pf_dsl(&root_path, "--concern-coverage");
    assert!(
        concern_coverage.status.success(),
        "concern coverage mode should succeed: {}",
        String::from_utf8_lossy(&concern_coverage.stderr)
    );
    let concern_stdout = String::from_utf8_lossy(&concern_coverage.stdout);
    assert!(concern_stdout.contains("# Concern Coverage Report: Root"));
    assert!(concern_stdout.contains("- Concern coverage status: PASS"));

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

    let wrspm = run_pf_dsl(&root_path, "--wrspm-report");
    assert!(
        wrspm.status.success(),
        "wrspm report mode should succeed: {}",
        String::from_utf8_lossy(&wrspm.stderr)
    );
    let wrspm_stdout = String::from_utf8_lossy(&wrspm.stdout);
    assert!(
        wrspm_stdout.contains("# WRSPM Report: Root"),
        "wrspm output should contain report header"
    );
    assert!(
        wrspm_stdout.contains("Artifact Projection"),
        "wrspm output should contain artifact section"
    );

    let wrspm_json = run_pf_dsl(&root_path, "--wrspm-json");
    assert!(
        wrspm_json.status.success(),
        "wrspm json mode should succeed: {}",
        String::from_utf8_lossy(&wrspm_json.stderr)
    );
    let wrspm_json_stdout = String::from_utf8_lossy(&wrspm_json.stdout);
    assert!(
        wrspm_json_stdout.contains("\"problem\": \"Root\""),
        "wrspm json output should contain problem field"
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
domain Sensor kind causal role given marks: {
  @ddd.bounded_context("Telemetry")
  @ddd.aggregate_root
}
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
  marks: {
    @sysml.requirement
    @ddd.application_service("DisplaySensor")
  }
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
    assert!(markdown_stdout.contains("`domain:Sensor` -> requirements: R_display"));
    assert!(markdown_stdout.contains("`domain:Sensor` -> generated targets:"));
    assert!(
        markdown_stdout.contains("ddd.application_service:ddd.application_service.displaysensor")
    );

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
    assert!(csv_stdout.contains("impact,,,,,,requirement,R_display,R_display,,2"));
    assert!(csv_stdout.contains(
        "impact_target,,,,,,requirement,R_display,,ddd.application_service:ddd.application_service.displaysensor,2"
    ));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn dogfooding_fixture_cross_model_traceability_includes_generated_targets() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("dogfooding/traceability_cross_model/main.pf");

    let markdown = run_pf_dsl_with_args(
        &fixture,
        "--traceability-md",
        &["--impact=domain:Sensor", "--impact-hops=2"],
    );
    assert!(
        markdown.status.success(),
        "traceability markdown mode should succeed for cross-model fixture: {}",
        String::from_utf8_lossy(&markdown.stderr)
    );

    let stdout = String::from_utf8_lossy(&markdown.stdout);
    assert!(stdout.contains("# Traceability Report: TraceabilityCrossModel"));
    assert!(stdout.contains("`domain:Sensor` -> requirements: R_display_sensor"));
    assert!(stdout.contains("ddd.application_service:ddd.application_service.displaysensor"));
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
fn dogfooding_cli_generates_pim_and_trace_map_outputs() {
    let dir = make_temp_dir("pf-cli-pim");
    let path = dir.join("pim.pf");
    fs::write(
        &path,
        r#"
problem: PimCli
domain Tool kind causal role machine
domain Payments kind causal role given marks: {
  @ddd.bounded_context("Payments")
  @ddd.aggregate_root
  @sysml.block
}
domain User kind biddable role given
interface "User-Tool" connects User, Tool {
  shared: {
    phenomenon Command : command [User -> Tool] controlledBy User
  }
}
interface "Tool-Payments" connects Tool, Payments {
  shared: {
    phenomenon Execute : event [Tool -> Payments] controlledBy Tool
    phenomenon Updated : event [Payments -> Tool] controlledBy Payments
  }
}
requirement "R1" {
  frame: CommandedBehavior
  reference: User
  constrains: Payments
  marks: {
    @sysml.requirement
    @ddd.application_service("ExecutePayment")
  }
}
"#,
    )
    .expect("failed to write model");

    let ddd = run_pf_dsl(&path, "--ddd-pim");
    assert!(
        ddd.status.success(),
        "{}",
        String::from_utf8_lossy(&ddd.stderr)
    );
    let ddd_stdout = String::from_utf8_lossy(&ddd.stdout);
    assert!(ddd_stdout.contains("# DDD PIM Report: PimCli"));
    assert!(ddd_stdout.contains("ExecutePayment (R1)"));

    let sysml_text = run_pf_dsl(&path, "--sysml2-text");
    assert!(
        sysml_text.status.success(),
        "{}",
        String::from_utf8_lossy(&sysml_text.stderr)
    );
    let sysml_text_stdout = String::from_utf8_lossy(&sysml_text.stdout);
    assert!(sysml_text_stdout.contains("package PimCli"));
    assert!(sysml_text_stdout.contains("requirement R1"));

    let sysml_json = run_pf_dsl(&path, "--sysml2-json");
    assert!(
        sysml_json.status.success(),
        "{}",
        String::from_utf8_lossy(&sysml_json.stderr)
    );
    let sysml_json_stdout = String::from_utf8_lossy(&sysml_json.stdout);
    assert!(sysml_json_stdout.contains("\"target\": \"sysml-v2-json\""));

    let trace_map = run_pf_dsl(&path, "--trace-map-json");
    assert!(
        trace_map.status.success(),
        "{}",
        String::from_utf8_lossy(&trace_map.stderr)
    );
    let trace_map_stdout = String::from_utf8_lossy(&trace_map.stdout);
    assert!(trace_map_stdout.contains("\"status\": \"PASS\""));
    assert!(trace_map_stdout.contains("ddd.application_service.executepayment"));

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

    let concern_output = run_pf_dsl(&path, "--concern-coverage");
    assert!(
        concern_output.status.success(),
        "concern coverage mode should succeed: {}",
        String::from_utf8_lossy(&concern_output.stderr)
    );
    let concern_stdout = String::from_utf8_lossy(&concern_output.stdout);
    assert!(concern_stdout.contains("# Concern Coverage Report: Closure"));
    assert!(concern_stdout.contains("- Concern coverage status: FAIL"));
    assert!(concern_stdout.contains("| R_uncovered | - | - |"));
    assert!(concern_stdout.contains("## Explicit Uncovered Entries"));
    assert!(concern_stdout.contains("- R_uncovered: requirement is not mapped to any subproblem"));

    let _ = fs::remove_dir_all(dir);
}
