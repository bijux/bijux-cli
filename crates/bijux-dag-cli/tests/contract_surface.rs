use bijux_dag_app as _;
use clap as _;
use clap_complete as _;
use serde_json as _;
use tempfile as _;

use std::process::Command;
use tempfile::{tempdir, NamedTempFile};

fn dag_command() -> Command {
    let path = env!("CARGO_BIN_EXE_bijux-dag");
    assert!(
        std::path::Path::new(path).exists(),
        "resolved bijux test binary path does not exist: {path}"
    );
    Command::new(path)
}

fn write_temp_dag() -> String {
    let path = std::env::temp_dir().join(format!(
        "bijux-dag-cli-contract-{}.json",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    let content = r#"{
  "spec": "bijux-dag/v0.1",
  "nodes": [
    {
      "id": "const1",
      "kind": "const",
      "inputs": [],
      "outputs": [
        {
          "name": "value",
          "path": "value.txt"
        }
      ],
      "params": {
        "value": "hello"
      }
    }
  ],
  "edges": []
}
"#;
    std::fs::write(&path, content).expect("write dag");
    path.to_string_lossy().into_owned()
}

fn run_simple_dag_json() -> (tempfile::TempDir, String, serde_json::Value) {
    let dag = write_temp_dag();
    let out_dir = tempfile::tempdir().expect("run out");
    let output = dag_command()
        .args(["dag", "--json", "run", &dag, "--out", out_dir.path().to_str().unwrap()])
        .output()
        .expect("run json");
    assert!(output.status.success(), "run stderr: {}", String::from_utf8_lossy(&output.stderr));
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("run payload");
    (out_dir, dag, payload)
}

#[test]
fn dag_validate_help_is_stable_enough() {
    let output = dag_command().args(["dag", "validate", "--help"]).output().expect("validate help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("Usage:"));
    assert!(text.contains("dag validate [OPTIONS] <DAG>"));
}

#[test]
fn dag_unknown_subcommand_fails_with_code() {
    let output = dag_command().args(["foo"]).output().expect("unknown subcommand");

    assert!(!output.status.success());
}

#[test]
fn dag_validate_json_schema_contract() {
    let dag = write_temp_dag();
    let output =
        dag_command().args(["dag", "validate", &dag, "--json"]).output().expect("json validate");

    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("validate json");
    assert_eq!(payload["command"], "dag.validate");
    assert!(payload["status"].as_str().is_some());
    assert!(payload["data"].is_object());
}

#[test]
fn dag_root_help_lists_umbrella_commands() {
    let output = dag_command().arg("--help").output().expect("global help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("dag"));
    assert!(text.contains("completions"));
    assert!(text.contains("Git for computation graphs"));
}

#[test]
fn dag_command_help_surface_contract() {
    let output = dag_command().args(["dag"]).output().expect("dag help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    for token in [
        "validate",
        "run",
        "replay",
        "diff",
        "explain",
        "status",
        "cache",
        "adapters",
        "commands",
        "doctor",
        "trace-node",
        "run-bundle",
        "lab",
    ] {
        assert!(text.contains(token));
    }
}

#[test]
fn dag_run_help_surface_contract() {
    let output = dag_command().args(["dag", "run", "--help"]).output().expect("run help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    for token in [
        "--out",
        "--hermetic",
        "--deny-network",
        "--clean-env",
        "--preflight-only",
        "--explain-scheduling",
        "run",
    ] {
        assert!(text.contains(token));
    }
}

#[test]
fn dag_commands_groups_surface_is_stable_enough() {
    let output = dag_command().args(["dag", "commands", "--groups"]).output().expect("commands");
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    for token in ["core", "runtime", "evidence", "cache", "diagnostics", "lab"] {
        assert!(text.contains(token), "missing group token: {token}");
    }
}

#[test]
fn dag_commands_json_exposes_group_and_maturity_metadata() {
    let output =
        dag_command().args(["dag", "--json", "commands"]).output().expect("commands json");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("commands json");
    let commands = payload["data"]["commands"].as_array().expect("commands array");
    assert!(commands.iter().any(|entry| entry["path"] == "doctor"));
    assert!(commands.iter().any(|entry| entry["path"] == "lab federation schedule"));
    assert!(commands.iter().all(|entry| entry.get("maturity").is_some()));
    assert!(commands.iter().all(|entry| entry.get("group").is_some()));
}

#[test]
fn dag_doctor_json_includes_schema_and_runtime_config_status() {
    let output = dag_command().args(["dag", "--json", "doctor"]).output().expect("doctor json");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("doctor payload");
    assert!(payload["data"]["schema_files"]["count"].as_u64().is_some());
    assert!(payload["data"]["runtime_config"]["defaults_fingerprint"].as_str().is_some());
}

#[test]
fn dag_explain_plan_alias_and_legacy_alias_both_work() {
    let dag = write_temp_dag();
    for args in [
        vec!["dag", "--json", "explain-plan", &dag],
        vec!["dag", "--json", "show-effective-plan", &dag],
    ] {
        let output = dag_command().args(args).output().expect("explain plan");
        assert!(output.status.success());
        let payload: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("explain plan payload");
        assert!(payload["data"]["planner_contract_version"].as_str().is_some());
        assert!(payload["data"]["planned_nodes"].is_array());
    }
}

#[test]
fn dag_lab_namespace_help_exposes_simulation_families() {
    let output = dag_command().args(["dag", "lab", "--help"]).output().expect("lab help");
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    for token in ["federation", "incident", "enterprise", "release", "security"] {
        assert!(text.contains(token));
    }
}

#[test]
fn dag_run_preflight_and_scheduling_surfaces_work_end_to_end() {
    let dag = write_temp_dag();
    let out_dir = tempfile::tempdir().expect("out");

    let preflight = dag_command()
        .args([
            "dag",
            "--json",
            "run",
            &dag,
            "--out",
            out_dir.path().to_str().unwrap(),
            "--preflight-only",
        ])
        .output()
        .expect("preflight");
    assert!(preflight.status.success());
    let preflight_payload: serde_json::Value =
        serde_json::from_slice(&preflight.stdout).expect("preflight payload");
    assert!(preflight_payload["data"]["scheduling"]["planned_nodes"].is_array());

    let run = dag_command()
        .args([
            "dag",
            "--json",
            "run",
            &dag,
            "--out",
            out_dir.path().to_str().unwrap(),
            "--explain-scheduling",
        ])
        .output()
        .expect("run explain scheduling");
    assert!(run.status.success(), "run stderr: {}", String::from_utf8_lossy(&run.stderr));
    let run_payload: serde_json::Value = serde_json::from_slice(&run.stdout).expect("run payload");
    assert!(run_payload["data"]["run_dir"].as_str().is_some());
    assert!(run_payload["data"]["scheduling"]["planned_nodes"].is_array());
}

#[test]
fn dag_trace_node_artifact_fetch_and_bundle_surfaces_work_end_to_end() {
    let (_out_dir, _dag, run_payload) = run_simple_dag_json();
    let run_dir = run_payload["data"]["run_dir"].as_str().expect("run dir");

    let trace = dag_command()
        .args(["dag", "--json", "trace-node", run_dir, "--id", "const1"])
        .output()
        .expect("trace node");
    assert!(trace.status.success(), "trace stderr: {}", String::from_utf8_lossy(&trace.stderr));
    let trace_payload: serde_json::Value = serde_json::from_slice(&trace.stdout).expect("trace json");
    assert_eq!(trace_payload["data"]["node_id"], "const1");

    let copied = tempdir().expect("copy out");
    let copied_path = copied.path().join("value.txt");
    let fetch = dag_command()
        .args([
            "dag",
            "--json",
            "artifact",
            "fetch",
            run_dir,
            "const1:value.txt",
            "--out",
            copied_path.to_str().unwrap(),
        ])
        .output()
        .expect("artifact fetch");
    assert!(fetch.status.success(), "fetch stderr: {}", String::from_utf8_lossy(&fetch.stderr));
    assert_eq!(std::fs::read_to_string(&copied_path).expect("copied artifact"), "\"hello\"");

    let bundle = tempdir().expect("bundle out");
    let bundle_path = bundle.path().join("bundle.json");
    let bundle_output = dag_command()
        .args([
            "dag",
            "--json",
            "run-bundle",
            run_dir,
            "--out",
            bundle_path.to_str().unwrap(),
        ])
        .output()
        .expect("run bundle");
    assert!(bundle_output.status.success());
    let bundle_payload: serde_json::Value =
        serde_json::from_slice(&bundle_output.stdout).expect("bundle payload");
    assert_eq!(bundle_payload["data"]["bundle"], bundle_path.to_string_lossy().to_string());
    let bundle_json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&bundle_path).expect("bundle file")).expect("bundle json");
    assert_eq!(bundle_json["bundle_version"], "export-bundle/v0.1");
    assert!(bundle_json["files"].is_object());
}

#[test]
fn dag_replay_help_surface_contract() {
    let output = dag_command().args(["dag", "replay", "--help"]).output().expect("replay help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    for token in ["--out", "--run-id", "--reuse-cache", "replay"] {
        assert!(text.contains(token));
    }
}

#[test]
fn dag_diff_help_surface_contract() {
    let output = dag_command().args(["dag", "diff", "--help"]).output().expect("diff help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("dag diff"));
    assert!(text.contains("--json"));
}

#[test]
fn dag_explain_help_surface_contract() {
    let output = dag_command().args(["dag", "explain", "--help"]).output().expect("explain help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("dag explain"));
    assert!(text.contains("--node"));
}

#[test]
fn dag_cache_help_surface_contract() {
    let output = dag_command().args(["dag", "cache", "--help"]).output().expect("cache help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    for token in ["cache", "verify", "pack", "explain"] {
        assert!(text.contains(token));
    }
}

#[test]
fn dag_adapters_help_surface_contract() {
    let output = dag_command().args(["dag", "adapters", "--help"]).output().expect("adapters help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("adapters"));
    assert!(text.contains("ls"));
    assert!(text.contains("doctor"));
}

#[test]
fn dag_validate_text_output_contract() {
    let dag = write_temp_dag();
    let output = dag_command().args(["dag", "validate", &dag]).output().expect("validate text");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("status:"));
}

#[test]
fn dag_validate_invalid_argument_fails() {
    let output = dag_command()
        .args(["dag", "validate", "non-existent-dag.json"])
        .output()
        .expect("invalid validate arg");

    assert!(!output.status.success());
}

#[test]
fn dag_validate_rejects_invalid_spec_with_validation_exit_code() {
    let invalid = NamedTempFile::new().expect("temp invalid");
    let invalid_path = invalid.path().to_path_buf();
    std::fs::write(&invalid_path, r#"{"spec":"bijux-dag/v9.9","nodes":[],"edges":[]}"#)
        .expect("write invalid spec");

    let output = dag_command()
        .args(["dag", "validate", invalid_path.to_str().unwrap()])
        .output()
        .expect("invalid validate");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn dag_run_exit_code_success() {
    let dag = write_temp_dag();
    let out_dir = tempfile::tempdir().expect("run out");

    let output = dag_command()
        .args(["dag", "run", &dag, "--out", out_dir.path().to_str().unwrap()])
        .output()
        .expect("run success");

    assert!(output.status.success());
}

#[test]
fn dag_run_runtime_failure_returns_nonzero_exit() {
    let dag = {
        let path = std::env::temp_dir().join(format!(
            "bijux-dag-cli-contract-failing-{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        let content = r#"{
          "spec": "bijux-dag/v0.1",
          "nodes": [{
            "id": "fail",
            "kind": "shell",
            "inputs": [],
            "outputs": [{ "name": "value", "path": "value.txt" }],
            "params": {
              "argv": ["/bin/sh","-c","exit 7"]
            }
          }],
          "edges": []
        }"#;
        std::fs::write(&path, content).expect("write dag");
        path.to_string_lossy().into_owned()
    };
    let out_dir = tempfile::tempdir().expect("run out");

    let output = dag_command()
        .args(["dag", "run", &dag, "--out", out_dir.path().to_str().unwrap()])
        .output()
        .expect("run fail");

    assert!(!output.status.success());
    assert!(output.status.code().is_some_and(|code| code != 0));
}

#[test]
fn completions_generation_supports_all_supported_shells() {
    for shell in ["bash", "zsh", "fish", "elvish", "powershell"] {
        let output = dag_command()
            .args(["completions", "--shell", shell])
            .output()
            .expect("completion command");
        assert!(output.status.success(), "shell {shell} failed");
        assert!(!output.stdout.is_empty(), "shell {shell} emitted empty completion");
    }
}

#[test]
fn fsck_alias_surface_runs_on_valid_run_dir() {
    let dag = write_temp_dag();
    let out_dir = tempdir().expect("run out");
    let run_output = dag_command()
        .args(["dag", "run", &dag, "--out", out_dir.path().to_str().expect("run out path")])
        .output()
        .expect("run");
    assert!(run_output.status.success(), "run must succeed for fsck setup");

    let mut entries: Vec<_> =
        std::fs::read_dir(out_dir.path()).expect("read out dir").filter_map(Result::ok).collect();
    entries.sort_by_key(|entry| entry.file_name());
    let run_dir =
        entries.last().expect("expected run directory").path().to_string_lossy().into_owned();

    let fsck_output =
        dag_command().args(["dag", "fsck", &run_dir, "--strict", "--json"]).output().expect("fsck");
    assert!(fsck_output.status.success(), "fsck on valid run directory should succeed");

    let payload: serde_json::Value =
        serde_json::from_slice(&fsck_output.stdout).expect("fsck json payload");
    assert_eq!(payload["command"], "dag.fsck");
    assert_eq!(payload["ok"], true);
}

#[test]
fn fsck_alias_supports_bundle_verification_mode() {
    let tmp = tempdir().expect("tempdir");
    let bundle_path = tmp.path().join("bundle.json");
    std::fs::write(
        &bundle_path,
        r#"{
  "bundle_version":"export-bundle/v0.1",
  "export_mode":"manifest-only",
  "manifest":{"manifest_version":"run-manifest/v0.1"},
  "graph_snapshot":{"spec":"bijux-dag/v0.1","nodes":[],"edges":[]},
  "node_traces":{},
  "outputs":{}
}"#,
    )
    .expect("write bundle");

    let output = dag_command()
        .args(["dag", "fsck", bundle_path.to_str().expect("bundle path"), "--json"])
        .output()
        .expect("bundle fsck");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("bundle fsck json");
    assert_eq!(payload["command"], "dag.fsck");
    assert_eq!(payload["data"]["kind"], "bundle");
    assert_eq!(payload["ok"], true);
}

#[test]
fn capabilities_backend_query_supports_kubernetes() {
    let output = dag_command()
        .args(["dag", "capabilities", "--backend", "kubernetes", "--json"])
        .output()
        .expect("capabilities backend");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("capabilities backend json");
    assert_eq!(payload["command"], "dag.capabilities");
    assert_eq!(payload["data"]["backend"], "kubernetes");
    assert_eq!(payload["data"]["status"], "simulated");
}

#[test]
fn capabilities_backend_query_supports_hpc() {
    let output = dag_command()
        .args(["dag", "capabilities", "--backend", "hpc", "--json"])
        .output()
        .expect("capabilities hpc backend");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("capabilities hpc json");
    assert_eq!(payload["command"], "dag.capabilities");
    assert_eq!(payload["data"]["backend"], "hpc");
    assert_eq!(payload["data"]["status"], "simulated");
}

#[test]
#[ignore = "flaky in mixed backend simulation environments"]
fn capabilities_backend_query_supports_remote() {
    let output = dag_command()
        .args(["dag", "capabilities", "--backend", "remote", "--json"])
        .output()
        .expect("capabilities remote backend");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("capabilities remote json");
    assert_eq!(payload["command"], "dag.capabilities");
    assert_eq!(payload["data"]["backend"], "remote");
    assert_eq!(payload["data"]["status"], "simulated");
    assert_eq!(payload["data"]["capabilities"]["worker_pool_capability_negotiation"], true);
}

#[test]
#[ignore = "slow"]
fn semantic_portability_backend_query_surface_is_available() {
    let output = dag_command()
        .args(["dag", "semantic-portability", "--backend", "kubernetes", "--json"])
        .output()
        .expect("semantic portability");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("semantic portability json");
    assert_eq!(payload["command"], "dag.semantic-portability");
    assert_eq!(payload["data"]["backend"], "kubernetes");
}

#[test]
fn equivalence_proof_surface_reports_for_two_runs() {
    let dag = write_temp_dag();
    let run_a = tempfile::tempdir().expect("run a dir");
    let run_b = tempfile::tempdir().expect("run b dir");
    let run_a_out = dag_command()
        .args(["dag", "run", &dag, "--out", run_a.path().to_str().expect("run_a path"), "--json"])
        .output()
        .expect("run a");
    assert!(run_a_out.status.success());
    let run_b_out = dag_command()
        .args(["dag", "run", &dag, "--out", run_b.path().to_str().expect("run_b path"), "--json"])
        .output()
        .expect("run b");
    assert!(run_b_out.status.success());

    let run_a_payload: serde_json::Value = serde_json::from_slice(&run_a_out.stdout).expect("a");
    let run_b_payload: serde_json::Value = serde_json::from_slice(&run_b_out.stdout).expect("b");
    let run_a_dir = run_a_payload["data"]["run_dir"].as_str().expect("run a dir");
    let run_b_dir = run_b_payload["data"]["run_dir"].as_str().expect("run b dir");

    let output = dag_command()
        .args([
            "dag",
            "equivalence-proof",
            run_a_dir,
            run_b_dir,
            "--backend-a",
            "kubernetes",
            "--backend-b",
            "hpc",
            "--json",
        ])
        .output()
        .expect("equivalence proof");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("equivalence proof payload");
    assert_eq!(payload["command"], "dag.equivalence-proof");
    assert!(payload["data"]["status"].as_str().is_some());
}

#[test]
fn export_import_help_includes_bundle_control_flags() {
    let export_help =
        dag_command().args(["dag", "export", "--help"]).output().expect("export help");
    assert!(export_help.status.success());
    let export_text = String::from_utf8_lossy(&export_help.stdout);
    assert!(export_text.contains("--from-run"));
    assert!(export_text.contains("--without-artifacts"));
    assert!(export_text.contains("--provenance-only"));
    assert!(export_text.contains("--redact"));

    let import_help =
        dag_command().args(["dag", "import", "--help"]).output().expect("import help");
    assert!(import_help.status.success());
    let import_text = String::from_utf8_lossy(&import_help.stdout);
    assert!(import_text.contains("--verify-only"));
}

#[test]
fn prove_help_and_json_surface_are_available() {
    let help = dag_command().args(["dag", "prove", "--help"]).output().expect("prove help");
    assert!(help.status.success());
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(text.contains("dag prove"));
}

#[test]
fn proof_summary_help_surface_is_available() {
    let help = dag_command()
        .args(["dag", "proof-summary", "--help"])
        .output()
        .expect("proof-summary help");
    assert!(help.status.success());
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(text.contains("dag proof-summary"));
}

#[test]
fn migrate_help_includes_dry_run_preview_flag() {
    let help =
        dag_command().args(["dag", "migrate", "dag", "--help"]).output().expect("migrate help");
    assert!(help.status.success());
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(text.contains("--dry-run"));
}

#[test]
fn dag_status_json_schema_contract() {
    let dag = write_temp_dag();
    let run_dir = tempfile::tempdir().expect("run out");
    let run = dag_command()
        .args(["dag", "run", "--json", &dag, "--out", run_dir.path().to_str().unwrap()])
        .output()
        .expect("run json");
    let run_payload: serde_json::Value =
        serde_json::from_slice(&run.stdout).expect("parse run payload");
    let run_path = run_payload["data"]["run_dir"].as_str().unwrap();

    let output =
        dag_command().args(["dag", "status", "--json", run_path]).output().expect("status json");

    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse status payload");
    assert_eq!(payload["command"], "dag.status");
    assert_eq!(payload["ok"], true);
    assert!(payload["data"]["manifest"].is_object());
    assert!(payload["data"]["traces"].is_array());
}

#[test]
fn dag_diff_json_schema_contract() {
    let dag = write_temp_dag();
    let first_run_dir = tempfile::tempdir().expect("first run out");
    let second_run_dir = tempfile::tempdir().expect("second run out");

    let run_a = dag_command()
        .args(["dag", "run", "--json", &dag, "--out", first_run_dir.path().to_str().unwrap()])
        .output()
        .expect("run a");
    let run_b = dag_command()
        .args(["dag", "run", "--json", &dag, "--out", second_run_dir.path().to_str().unwrap()])
        .output()
        .expect("run b");
    assert!(run_a.status.success(), "run a failed: {}", String::from_utf8_lossy(&run_a.stderr));
    assert!(run_b.status.success(), "run b failed: {}", String::from_utf8_lossy(&run_b.stderr));

    let payload_a: serde_json::Value =
        serde_json::from_slice(&run_a.stdout).expect("parse run a payload");
    let payload_b: serde_json::Value =
        serde_json::from_slice(&run_b.stdout).expect("parse run b payload");
    let run_a_path = payload_a["data"]["run_dir"].as_str().unwrap();
    let run_b_path = payload_b["data"]["run_dir"].as_str().unwrap();

    let output = dag_command()
        .args(["dag", "diff", "--json", run_a_path, run_b_path])
        .output()
        .expect("diff json");

    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse diff payload");
    assert_eq!(payload["command"], "dag.diff");
    assert!(payload["data"]["manifest"].is_object());
    assert!(payload["data"]["nodes"].is_object());
    assert!(payload["data"]["outputs"].is_object());
}

#[test]
fn dag_validate_json_exists_with_human_and_machine_contracts() {
    let dag = write_temp_dag();
    let output = dag_command().args(["dag", "validate", &dag]).output().expect("validate text");

    assert!(output.status.success());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("{\"ok\""));

    let output_json =
        dag_command().args(["dag", "validate", "--json", &dag]).output().expect("validate json");

    assert!(output_json.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output_json.stdout).expect("validate json parse");
    assert_eq!(payload["command"], "dag.validate");
    assert_eq!(payload["ok"], true);
    assert!(payload["data"].is_object());
}

#[test]
fn dag_run_json_output_contract_and_exit_code() {
    let dag = write_temp_dag();
    let out_dir = tempdir().expect("temp out");

    let output = dag_command()
        .args(["dag", "run", "--json", &dag, "--out", out_dir.path().to_str().unwrap()])
        .output()
        .expect("run with json");

    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("run json parse");
    assert_eq!(payload["command"], "dag.run");
    assert_eq!(payload["status"], "ok");
    assert!(payload["data"].get("run_dir").and_then(|v| v.as_str()).is_some());
}
