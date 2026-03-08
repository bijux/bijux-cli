use bijux_dag_app as _;
use clap as _;
use clap_complete as _;
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;
use tempfile as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn dag_command() -> Command {
    if let Some(path) = option_env!("CARGO_BIN_EXE_bijux") {
        if std::path::Path::new(path).exists() {
            return Command::new(path);
        }
    }
    let root = repo_root();
    let cargo_bin = option_env!("CARGO").unwrap_or("cargo");
    let mut command = Command::new(cargo_bin);
    command.env("CARGO_TARGET_DIR", root.join("artifacts/target"));
    command.args([
        "run",
        "--quiet",
        "-p",
        "bijux-dag-cli",
        "--bin",
        "bijux",
        "--",
    ]);
    command
}

fn write_temp_dag() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "bijux-dag-cli-completion-{}.json",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    std::fs::write(
        &path,
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"const1","kind":"const","inputs":[],"outputs":[{"name":"value","path":"value.txt"}],"params":{"value":"hello"}}],"edges":[]}"#,
    )
    .expect("write dag");
    path
}

#[test]
fn cli_surface_policy_docs_and_coverage_report_exist() {
    let root = repo_root();
    for rel in [
        "docs/spec/CLI_SURFACE_STABILITY_POLICY.md",
        "docs/spec/CLI_COMMAND_STABILITY_DOCUMENTATION.md",
        "docs/reports/foundation/cli_command_coverage_report.md",
    ] {
        let text = std::fs::read_to_string(root.join(rel)).expect("read required doc");
        assert!(!text.trim().is_empty(), "empty required doc/report: {rel}");
    }
}

#[test]
fn cli_validate_json_contract_and_exit_codes_are_stable() {
    let dag = write_temp_dag();
    let ok = dag_command()
        .args(["dag", "validate", dag.to_str().unwrap(), "--json"])
        .output()
        .expect("validate");
    assert!(ok.status.success());
    let payload: Value = serde_json::from_slice(&ok.stdout).expect("json");
    assert_eq!(payload["command"], "dag.validate");
    assert!(payload["data"].is_object());

    let missing = dag_command()
        .args(["dag", "validate", "/definitely/missing/file.json"])
        .output()
        .expect("missing");
    assert_eq!(missing.status.code(), Some(3));
}

#[test]
fn cli_handles_malformed_inputs_and_corrupted_run_dirs_without_panics() {
    let temp = tempfile::tempdir().expect("temp");
    let malformed = temp.path().join("broken.json");
    std::fs::write(&malformed, "{not-json").expect("write malformed");
    let malformed_out = dag_command()
        .args(["dag", "validate", malformed.to_str().unwrap()])
        .output()
        .expect("validate malformed");
    assert!(!malformed_out.status.success());

    let run = temp.path().join("run-corrupt");
    std::fs::create_dir_all(&run).expect("mkdir");
    std::fs::write(run.join("manifest.json"), "{not-json").expect("manifest");
    std::fs::write(run.join("graph.snapshot.json"), "{not-json").expect("snapshot");
    let prove_out = dag_command()
        .args(["dag", "prove", run.to_str().unwrap(), "--json"])
        .output()
        .expect("prove corrupt");
    assert!(!prove_out.status.success());
}

#[test]
fn cli_pipeline_and_deterministic_validate_output_hold() {
    let dag = write_temp_dag();
    let out_dir = tempfile::tempdir().expect("out");
    let run = dag_command()
        .args([
            "dag",
            "run",
            dag.to_str().unwrap(),
            "--out",
            out_dir.path().to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("run");
    assert!(run.status.success());

    let replay = dag_command()
        .args([
            "dag",
            "replay",
            out_dir.path().join("run").to_str().unwrap_or(""),
            "--out",
            out_dir.path().to_str().unwrap(),
        ])
        .output()
        .expect("replay invocation");
    let _ = replay.status.code();

    let first = dag_command()
        .args(["dag", "validate", dag.to_str().unwrap(), "--json"])
        .output()
        .expect("validate1");
    let second = dag_command()
        .args(["dag", "validate", dag.to_str().unwrap(), "--json"])
        .output()
        .expect("validate2");
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn cli_latency_memory_scenarios_and_command_coverage_are_declared() {
    let root = repo_root();
    for rel in [
        "evidence/perf/scenarios/cli_validate_latency.json",
        "evidence/perf/scenarios/cli_validate_memory.json",
        "docs/reports/foundation/top_10_slowest_commands.md",
        "docs/reports/foundation/cli_command_coverage_report.md",
    ] {
        assert!(root.join(rel).exists(), "missing {rel}");
    }
}

#[test]
fn cli_environment_isolation_does_not_leak_user_secret_in_validate_output() {
    let dag = write_temp_dag();
    let output = dag_command()
        .env("BIJUX_DAG_TEST_SECRET", "never-print-me")
        .args(["dag", "validate", dag.to_str().unwrap(), "--json"])
        .output()
        .expect("validate");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains("never-print-me"));
    assert!(!stderr.contains("never-print-me"));
}
