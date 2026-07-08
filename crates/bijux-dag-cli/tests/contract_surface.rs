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

fn write_temp_owned_dag() -> String {
    let path = std::env::temp_dir().join(format!(
        "bijux-dag-cli-governance-{}.json",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    let content = r#"{
  "spec": "bijux-dag/v0.1",
  "meta": {
    "name": "owned-workflow",
    "owners": ["ops@example.com"],
    "tags": ["release"]
  },
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
    std::fs::write(&path, content).expect("write owned dag");
    path.to_string_lossy().into_owned()
}

fn write_temp_downstream_dag() -> String {
    let path = std::env::temp_dir().join(format!(
        "bijux-dag-cli-downstream-{}.json",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    let content = r#"{
  "spec": "bijux-dag/v0.1",
  "meta": {"name":"downstream-cli","owners":[],"tags":[]},
  "nodes": [
    {
      "id": "source",
      "kind": "const",
      "outputs": [{"name": "out", "path": "source/out.json"}],
      "params": {"value": "seed"}
    },
    {
      "id": "branch",
      "kind": "const",
      "inputs": ["in"],
      "outputs": [{"name": "out", "path": "branch/out.json"}],
      "params": {"value": "branch"}
    },
    {
      "id": "sink",
      "kind": "const",
      "inputs": ["in"],
      "outputs": [{"name": "out", "path": "sink/out.json"}],
      "params": {"value": "sink"}
    },
    {
      "id": "sidecar",
      "kind": "const",
      "outputs": [{"name": "out", "path": "sidecar/out.json"}],
      "params": {"value": "sidecar"}
    }
  ],
  "edges": [
    {"from": {"node_id": "source", "port": "out"}, "to": {"node_id": "branch", "port": "in"}},
    {"from": {"node_id": "branch", "port": "out"}, "to": {"node_id": "sink", "port": "in"}}
  ]
}
"#;
    std::fs::write(&path, content).expect("write downstream dag");
    path.to_string_lossy().into_owned()
}

fn write_temp_named_resource_dag() -> String {
    let path = std::env::temp_dir().join(format!(
        "bijux-dag-cli-named-resource-{}.json",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    let content = r#"{
  "spec": "bijux-dag/v0.1",
  "meta": {"name":"named-resource-preview","owners":[],"tags":[]},
  "nodes": [
    {
      "id": "root",
      "kind": "const",
      "outputs": [{"name": "out", "path": "root/out.json"}],
      "params": {"value": 1}
    },
    {
      "id": "left",
      "kind": "shell",
      "inputs": ["in"],
      "outputs": [{"name": "out", "path": "left/out.json"}],
      "params": {"argv": ["echo", "left"], "estimated_duration_ms": 10000},
      "resources": {"cpu": 1, "mem_mb": 64, "named_resources": {"database_slot": 1}}
    },
    {
      "id": "right",
      "kind": "shell",
      "inputs": ["in"],
      "outputs": [{"name": "out", "path": "right/out.json"}],
      "params": {"argv": ["echo", "right"], "estimated_duration_ms": 10000},
      "resources": {"cpu": 1, "mem_mb": 64, "named_resources": {"database_slot": 1}}
    },
    {
      "id": "join",
      "kind": "shell",
      "inputs": ["left", "right"],
      "outputs": [{"name": "out", "path": "join/out.json"}],
      "params": {"argv": ["echo", "join"]}
    }
  ],
  "edges": [
    {"from": {"node_id": "root", "port": "out"}, "to": {"node_id": "left", "port": "in"}},
    {"from": {"node_id": "root", "port": "out"}, "to": {"node_id": "right", "port": "in"}},
    {"from": {"node_id": "left", "port": "out"}, "to": {"node_id": "join", "port": "left"}},
    {"from": {"node_id": "right", "port": "out"}, "to": {"node_id": "join", "port": "right"}}
  ]
}
"#;
    std::fs::write(&path, content).expect("write named resource dag");
    path.to_string_lossy().into_owned()
}

fn write_temp_dag_fragments() -> (tempfile::TempDir, String, String) {
    let dir = tempfile::tempdir().expect("tmp");
    let foundation = dir.path().join("foundation.json");
    let publication = dir.path().join("publication.json");
    std::fs::write(
        &foundation,
        r#"{
  "spec": "bijux-dag/v0.1",
  "meta": {"name":"foundation","owners":[],"tags":[]},
  "nodes": [
    {
      "id": "extract",
      "kind": "const",
      "outputs": [
        {
          "name": "report",
          "path": "extract/report.json"
        }
      ],
      "params": {
        "value": "hello"
      }
    }
  ],
  "edges": []
}
"#,
    )
    .expect("write foundation");
    std::fs::write(
        &publication,
        r#"{
  "spec": "bijux-dag/v0.1",
  "meta": {"name":"publication","owners":[],"tags":[]},
  "nodes": [
    {
      "id": "publish",
      "kind": "const",
      "inputs": ["report"],
      "outputs": [
        {
          "name": "value",
          "path": "publish/value.txt"
        }
      ],
      "params": {
        "seed": {"node_output": {"node_id": "extract", "output_name": "report"}}
      }
    }
  ],
  "edges": [
    {
      "from": {"node_id": "extract", "port": "report"},
      "to": {"node_id": "publish", "port": "report"}
    }
  ]
}
"#,
    )
    .expect("write publication");
    (dir, foundation.to_string_lossy().into_owned(), publication.to_string_lossy().into_owned())
}

fn write_invalid_validation_dags() -> (tempfile::TempDir, String, String) {
    let dir = tempfile::tempdir().expect("tmp");
    let missing_input = dir.path().join("missing-input.json");
    let invalid_workdir = dir.path().join("invalid-workdir.json");
    std::fs::write(
        &missing_input,
        r#"{
  "spec": "bijux-dag/v0.1",
  "meta": {"name":"missing-input-binding","owners":[],"tags":[]},
  "nodes": [
    {
      "id": "emit",
      "kind": "const",
      "outputs": [
        {
          "name": "value",
          "path": "emit/value.json"
        }
      ],
      "params": {
        "value": "seed"
      }
    },
    {
      "id": "consume",
      "kind": "const",
      "inputs": ["payload"],
      "outputs": [
        {
          "name": "result",
          "path": "consume/result.json"
        }
      ],
      "params": {
        "value": 1
      }
    }
  ],
  "edges": []
}
"#,
    )
    .expect("write missing input");
    std::fs::write(
        &invalid_workdir,
        r#"{
  "spec": "bijux-dag/v0.1",
  "meta": {"name":"invalid-container-workdir","owners":[],"tags":[]},
  "nodes": [
    {
      "id": "publish",
      "kind": "container",
      "outputs": [
        {
          "name": "result",
          "path": "publish/result.txt"
        }
      ],
      "container": {
        "image": "alpine:3.20",
        "argv": ["sh", "-c", "echo ok > result.txt"],
        "engine": "docker",
        "workdir": "{work_dir}/../escape"
      },
      "effects": ["filesystem"]
    }
  ],
  "edges": []
}
"#,
    )
    .expect("write invalid workdir");
    (
        dir,
        missing_input.to_string_lossy().into_owned(),
        invalid_workdir.to_string_lossy().into_owned(),
    )
}

fn run_simple_dag_json() -> (tempfile::TempDir, String, serde_json::Value) {
    let dag = write_temp_dag();
    let out_dir = tempfile::tempdir().expect("run out");
    let output = dag_command()
        .args(["--json", "run", &dag, "--out", out_dir.path().to_str().unwrap()])
        .output()
        .expect("run json");
    assert!(output.status.success(), "run stderr: {}", String::from_utf8_lossy(&output.stderr));
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("run payload");
    (out_dir, dag, payload)
}

#[test]
fn dag_validate_help_is_stable_enough() {
    let output = dag_command().args(["validate", "--help"]).output().expect("validate help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("Usage:"));
    assert!(text.contains("bijux-dag validate [OPTIONS] <DAGS>..."));
}

#[test]
fn dag_unknown_subcommand_fails_with_code() {
    let output = dag_command().args(["foo"]).output().expect("unknown subcommand");

    assert!(!output.status.success());
}

#[test]
fn dag_validate_json_schema_contract() {
    let dag = write_temp_dag();
    let output = dag_command().args(["validate", &dag, "--json"]).output().expect("json validate");

    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("validate json");
    assert_eq!(payload["command"], "dag.validate");
    assert!(payload["status"].as_str().is_some());
    assert!(payload["data"].is_object());
}

#[test]
fn dag_validate_accepts_composed_graph_fragments() {
    let (_dir, foundation, publication) = write_temp_dag_fragments();
    let output = dag_command()
        .args(["validate", &foundation, &publication, "--json"])
        .output()
        .expect("composed validate");

    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("validate json");
    assert_eq!(payload["command"], "dag.validate");
    assert!(payload["status"].as_str().is_some());
}

#[test]
fn dag_validate_json_reports_missing_required_input_binding() {
    let (_dir, missing_input, _invalid_workdir) = write_invalid_validation_dags();
    let output = dag_command()
        .args(["validate", &missing_input, "--json"])
        .output()
        .expect("validate missing input");

    assert!(!output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("validate json");
    assert_eq!(payload["command"], "dag.validate");
    assert_eq!(payload["ok"], false);
    assert!(payload["diagnostics"].as_array().unwrap().iter().any(|diagnostic| {
        diagnostic["code"] == "E1005"
            && diagnostic["message"] == "missing required input binding: consume.payload"
    }));
}

#[test]
fn dag_validate_json_reports_invalid_container_workdir_path() {
    let (_dir, _missing_input, invalid_workdir) = write_invalid_validation_dags();
    let output = dag_command()
        .args(["validate", &invalid_workdir, "--json"])
        .output()
        .expect("validate invalid workdir");

    assert!(!output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("validate json");
    assert_eq!(payload["command"], "dag.validate");
    assert_eq!(payload["ok"], false);
    assert!(payload["diagnostics"].as_array().unwrap().iter().any(|diagnostic| {
        diagnostic["code"] == "E1025"
            && diagnostic["path"] == "/nodes/publish/container/workdir"
            && diagnostic["message"] == "invalid path variable suffix: ../escape"
    }));
}

#[test]
fn dag_root_help_lists_top_level_commands() {
    let output = dag_command().arg("--help").output().expect("global help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("validate"));
    assert!(text.contains("run"));
    assert!(text.contains("completions"));
    assert!(text
        .contains("Validate, run, replay, explain, and compare reproducible computation graphs"));
    assert!(text.contains("v0.4.0 surface truth table:"));
    assert!(text.contains("commands --lane experimental"));
    assert!(text.contains("commands --lane simulated"));
    assert!(text.contains("commands --lane internal"));
    assert!(text.contains("BIJUX_DAG_ENABLE_SIMULATED=1"));
    assert!(text.contains("BIJUX_DAG_ENABLE_INTERNAL=1"));
    assert!(text.contains("Use `bijux-dag commands` for the stable operator surface"));
    assert!(!text.contains("enterprise"));
    assert!(!text.contains("governance"));
}

#[test]
fn dag_root_help_surface_contract() {
    let output = dag_command().arg("--help").output().expect("dag root help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    for token in [
        "validate",
        "artifact-inspect",
        "artifact",
        "commands",
        "plan",
        "run",
        "replay",
        "runs",
        "diff",
        "explain",
        "verify",
        "doctor",
        "cache",
        "version",
    ] {
        assert!(text.contains(token));
    }
    for hidden in [
        "adapters",
        "canonicalize",
        "export",
        "fsck",
        "hash",
        "import",
        "init",
        "lint",
        "policy",
        "prove",
        "status",
        "trace-artifact",
        "why-rerun",
        "control-plane",
        "state-store",
        "enterprise",
        "fleet",
        "governance",
        "incident",
        "lab",
        "federation",
        "security",
        "durability",
        "performance",
        "release",
        "runtime",
        "schedule",
        "run-bundle",
        "trace-node",
        "version-inspect",
        "capabilities",
        "semantic-portability",
        "equivalence-proof",
    ] {
        assert!(
            !text.contains(&format!("\n  {hidden}")),
            "hidden namespace leaked into root help: {hidden}"
        );
    }
    assert!(!text.contains("\n  graph"), "hidden namespace leaked into root help: graph");
}

#[test]
fn dag_root_rejects_redundant_dag_subcommand() {
    let output = dag_command().args(["dag", "--help"]).output().expect("nested dag help");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unrecognized subcommand 'dag'"));
    assert!(stderr.contains("Usage: bijux-dag [OPTIONS] [COMMAND]"));
}

#[test]
fn dag_run_help_surface_contract() {
    let output = dag_command().args(["run", "--help"]).output().expect("run help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    for token in [
        "--out",
        "--hermetic",
        "--deny-network",
        "--clean-env",
        "--resource-capacity",
        "--preflight-only",
        "--explain-scheduling",
        "run",
    ] {
        assert!(text.contains(token));
    }
    for detail in [
        "deny declared network effects",
        "does not firewall sockets",
        "deny declared clock effects",
        "does not virtualize wall clock access",
        "declare a named runtime capacity as <name=count>",
        "curated bijux environment",
        "best-effort local policy profile",
        "does not claim syscall sandboxing or host filesystem isolation",
    ] {
        assert!(text.contains(detail), "missing run help detail: {detail}");
    }
}

#[test]
fn dag_commands_groups_surface_is_stable_enough() {
    let output = dag_command().args(["commands", "--groups"]).output().expect("commands");
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    for token in [
        "artifact", "cache", "config", "doctor", "graph", "inspect", "plan", "prove", "replay",
        "run",
    ] {
        assert!(text.contains(token), "missing group token: {token}");
    }
}

#[test]
fn dag_commands_json_exposes_group_and_maturity_metadata() {
    let output = dag_command().args(["--json", "commands"]).output().expect("commands json");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("commands json");
    let commands = payload["data"]["commands"].as_array().expect("commands array");
    assert!(commands.iter().any(|entry| entry["path"] == "doctor"));
    assert!(commands.iter().all(|entry| entry["lane"] == "stable"));
    assert!(commands.iter().all(|entry| entry["availability"] == "default"));
    assert!(!commands.iter().any(|entry| entry["path"] == "artifact fetch"));
    assert!(!commands.iter().any(|entry| entry["path"] == "status"));
    assert!(!commands.iter().any(|entry| entry["path"] == "init"));
    assert!(!commands.iter().any(|entry| entry["path"] == "lab federation schedule"));
    assert!(commands.iter().all(|entry| entry.get("lane").is_some()));
    assert!(commands.iter().all(|entry| entry.get("availability").is_some()));
    assert!(commands.iter().all(|entry| entry.get("group").is_some()));
}

#[test]
fn dag_commands_json_can_target_experimental_lane_inventory() {
    let output = dag_command()
        .args(["--json", "commands", "--lane", "experimental"])
        .output()
        .expect("commands json experimental");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("commands json experimental");
    let commands = payload["data"]["commands"].as_array().expect("commands array");
    assert!(commands.iter().any(|entry| entry["path"] == "artifact fetch"
        && entry["lane"] == "experimental"
        && entry["availability"] == "explicit-path"));
    assert!(commands.iter().any(|entry| entry["path"] == "trace-node"));
    assert!(!commands.iter().any(|entry| entry["path"] == "lab federation schedule"));
    assert!(!commands.iter().any(|entry| entry["path"] == "doctor"));
}

#[test]
fn dag_commands_json_can_target_simulated_lane_inventory() {
    let output = dag_command()
        .args(["--json", "commands", "--lane", "simulated"])
        .output()
        .expect("commands json simulated");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("commands json simulated");
    let commands = payload["data"]["commands"].as_array().expect("commands array");
    assert!(commands.iter().any(|entry| entry["path"] == "lab federation schedule"
        && entry["lane"] == "simulation"
        && entry["availability"] == "opt-in"
        && entry["opt_in_env"] == "BIJUX_DAG_ENABLE_SIMULATED"));
    assert!(!commands.iter().any(|entry| entry["path"] == "trace-node"));
    assert!(!commands.iter().any(|entry| entry["path"] == "doctor"));
}

#[test]
fn dag_commands_human_output_marks_non_stable_routes_as_opt_in() {
    let default_output = dag_command().args(["commands"]).output().expect("commands");
    assert!(default_output.status.success());
    let default_text = String::from_utf8_lossy(&default_output.stdout);
    assert!(default_text.contains("validate"));
    assert!(default_text.contains("commands"));
    assert!(!default_text.contains("capabilities"));
    assert!(!default_text.contains("enterprise"));
    assert!(!default_text.contains("lab federation schedule"));

    let experimental_output = dag_command()
        .args(["commands", "--lane", "experimental"])
        .output()
        .expect("commands experimental");
    assert!(experimental_output.status.success());
    let experimental_text = String::from_utf8_lossy(&experimental_output.stdout);
    assert!(experimental_text.contains("trace-node [inspect | experimental | explicit-path]"));
    assert!(!experimental_text.contains("enterprise"));
    assert!(!experimental_text.contains("capabilities"));

    let simulated_output = dag_command()
        .args(["commands", "--lane", "simulated"])
        .output()
        .expect("commands simulated");
    assert!(simulated_output.status.success());
    let simulated_text = String::from_utf8_lossy(&simulated_output.stdout);
    assert!(simulated_text
        .contains("enterprise [config | simulated | opt-in via BIJUX_DAG_ENABLE_SIMULATED]"));
    assert!(!simulated_text.contains("trace-node"));

    let internal_output =
        dag_command().args(["commands", "--lane", "internal"]).output().expect("commands internal");
    assert!(internal_output.status.success());
    let internal_text = String::from_utf8_lossy(&internal_output.stdout);
    assert!(internal_text
        .contains("capabilities [config | internal | opt-in via BIJUX_DAG_ENABLE_INTERNAL]"));
    assert!(!internal_text.contains("enterprise"));
}

#[test]
fn dag_artifact_help_hides_experimental_fetch_route() {
    let output = dag_command().args(["artifact", "--help"]).output().expect("artifact help");
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("registry"));
    assert!(text.contains("lineage"));
    assert!(text.contains("retention"));
    assert!(!text.contains("fetch"));
}

#[test]
fn dag_doctor_json_includes_schema_and_runtime_config_status() {
    let output = dag_command().args(["--json", "doctor"]).output().expect("doctor json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("doctor payload");
    assert!(payload["data"]["schema_files"]["count"].as_u64().is_some());
    assert!(payload["data"]["runtime_config"]["defaults_fingerprint"].as_str().is_some());
}

#[test]
fn dag_explain_plan_alias_and_legacy_alias_both_work() {
    let dag = write_temp_dag();
    for args in [vec!["--json", "explain-plan", &dag], vec!["--json", "show-effective-plan", &dag]]
    {
        let output = dag_command().args(args).output().expect("explain plan");
        assert!(output.status.success());
        let payload: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("explain plan payload");
        assert!(payload["data"]["planner_contract_version"].as_str().is_some());
        assert!(payload["data"]["planned_nodes"].is_array());
    }
}

#[test]
fn dag_explain_plan_aliases_accept_composed_graph_fragments() {
    let (_dir, foundation, publication) = write_temp_dag_fragments();
    for args in [
        vec!["--json", "explain-plan", &foundation, &publication],
        vec!["--json", "show-effective-plan", &foundation, &publication],
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
fn dag_explain_plan_alias_surfaces_downstream_selection() {
    let dag = write_temp_downstream_dag();
    let output = dag_command()
        .args(["--json", "explain-plan", "--from-node", "branch", &dag])
        .output()
        .expect("explain plan downstream selection");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("explain plan payload");
    assert_eq!(payload["data"]["selection"]["downstream_roots"], serde_json::json!(["branch"]));
    assert_eq!(
        payload["data"]["selection"]["selected_nodes"],
        serde_json::json!(["branch", "sink"])
    );
    assert_eq!(
        payload["data"]["selection"]["omitted_nodes"],
        serde_json::json!([
            {"node_id":"sidecar","reason":"not_selected_by_from_node"},
            {"node_id":"source","reason":"not_selected_by_from_node"}
        ])
    );
}

#[test]
fn dag_explain_plan_alias_surfaces_resource_aware_preview_budgets() {
    let dag = write_temp_named_resource_dag();
    let output = dag_command()
        .args([
            "--json",
            "explain-plan",
            "--jobs",
            "2",
            "--resource-capacity",
            "database_slot=1",
            &dag,
        ])
        .output()
        .expect("explain plan resource preview");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("explain plan payload");
    assert_eq!(
        payload["data"]["execution_cost_estimate"]["scheduling_simulation"]["run_bound"],
        "resource_bound"
    );
    assert_eq!(
        payload["data"]["execution_cost_estimate"]["scheduling_simulation"]["bottlenecks"][0]
            ["resource"],
        "named_resource:database_slot"
    );
}

#[test]
fn dag_hidden_simulation_help_remains_discoverable_by_explicit_path() {
    let output = dag_command().args(["lab", "--help"]).output().expect("lab help");
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(!text.contains("federation"));

    let nested = dag_command()
        .args(["lab", "federation", "schedule", "--help"])
        .output()
        .expect("nested lab help");
    assert!(nested.status.success());
    let nested_text = String::from_utf8_lossy(&nested.stdout);
    assert!(nested_text.contains("bijux-dag lab federation schedule"));
}

#[test]
fn dag_simulated_routes_require_opt_in_before_execution() {
    let dag = write_temp_owned_dag();

    let denied = dag_command()
        .args(["--json", "governance", "ownership", &dag])
        .output()
        .expect("governance ownership denied");
    assert!(!denied.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&denied.stdout).expect("governance denial payload");
    assert_eq!(payload["command"], "dag.governance");
    assert_eq!(payload["data"]["command_family"], "governance");
    assert_eq!(payload["data"]["lane"], "simulation");
    assert_eq!(payload["data"]["access"], "opt-in");
    assert_eq!(payload["data"]["opt_in_env"], "BIJUX_DAG_ENABLE_SIMULATED");
    assert!(payload["diagnostics"].as_array().unwrap().iter().any(|diagnostic| {
        diagnostic["code"] == "release-boundary-opt-in"
            && diagnostic["message"]
                .as_str()
                .unwrap()
                .contains("`governance` belongs to the simulated command lane")
    }));

    let allowed = dag_command()
        .env("BIJUX_DAG_ENABLE_SIMULATED", "1")
        .args(["--json", "governance", "ownership", &dag])
        .output()
        .expect("governance ownership allowed");
    assert!(
        allowed.status.success(),
        "allowed stderr: {}",
        String::from_utf8_lossy(&allowed.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&allowed.stdout).expect("governance payload");
    assert_eq!(payload["command"], "dag.governance.ownership");
    assert_eq!(payload["ok"], true);
}

#[test]
fn dag_internal_routes_require_opt_in_before_execution() {
    let dag = write_temp_dag();

    let denied = dag_command()
        .args(["--json", "version-inspect", "--dag", &dag])
        .output()
        .expect("version inspect denied");
    assert!(!denied.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&denied.stdout).expect("version inspect denial payload");
    assert_eq!(payload["command"], "dag.version-inspect");
    assert_eq!(payload["data"]["command_family"], "version-inspect");
    assert_eq!(payload["data"]["lane"], "internal");
    assert_eq!(payload["data"]["access"], "opt-in");
    assert_eq!(payload["data"]["opt_in_env"], "BIJUX_DAG_ENABLE_INTERNAL");
    assert!(payload["diagnostics"].as_array().unwrap().iter().any(|diagnostic| {
        diagnostic["code"] == "release-boundary-opt-in"
            && diagnostic["message"]
                .as_str()
                .unwrap()
                .contains("`version-inspect` belongs to the internal command lane")
    }));

    let allowed = dag_command()
        .env("BIJUX_DAG_ENABLE_INTERNAL", "1")
        .args(["--json", "version-inspect", "--dag", &dag])
        .output()
        .expect("version inspect allowed");
    assert!(
        allowed.status.success(),
        "allowed stderr: {}",
        String::from_utf8_lossy(&allowed.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&allowed.stdout).expect("version inspect payload");
    assert_eq!(payload["command"], "dag.version-inspect");
    assert_eq!(payload["ok"], true);
}

#[test]
fn dag_run_preflight_and_scheduling_surfaces_work_end_to_end() {
    let dag = write_temp_dag();
    let out_dir = tempfile::tempdir().expect("out");

    let preflight = dag_command()
        .args([
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
fn dag_run_preflight_accepts_composed_graph_fragments() {
    let (_dir, foundation, publication) = write_temp_dag_fragments();
    let out_dir = tempfile::tempdir().expect("out");

    let preflight = dag_command()
        .args([
            "--json",
            "run",
            &foundation,
            &publication,
            "--out",
            out_dir.path().to_str().unwrap(),
            "--preflight-only",
        ])
        .output()
        .expect("preflight");
    assert!(preflight.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&preflight.stdout).expect("payload");
    assert!(payload["data"]["scheduling"]["planned_nodes"].is_array());
}

#[test]
fn dag_trace_node_artifact_fetch_and_bundle_surfaces_work_end_to_end() {
    let (_out_dir, _dag, run_payload) = run_simple_dag_json();
    let run_dir = run_payload["data"]["run_dir"].as_str().expect("run dir");

    let trace = dag_command()
        .args(["--json", "trace-node", run_dir, "--id", "const1"])
        .output()
        .expect("trace node");
    assert!(trace.status.success(), "trace stderr: {}", String::from_utf8_lossy(&trace.stderr));
    let trace_payload: serde_json::Value =
        serde_json::from_slice(&trace.stdout).expect("trace json");
    assert_eq!(trace_payload["data"]["node_id"], "const1");

    let copied = tempdir().expect("copy out");
    let copied_path = copied.path().join("value.txt");
    let fetch = dag_command()
        .args([
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
        .args(["--json", "run-bundle", run_dir, "--out", bundle_path.to_str().unwrap()])
        .output()
        .expect("run bundle");
    assert!(bundle_output.status.success());
    let bundle_payload: serde_json::Value =
        serde_json::from_slice(&bundle_output.stdout).expect("bundle payload");
    assert_eq!(bundle_payload["data"]["bundle"], bundle_path.to_string_lossy().to_string());
    let bundle_json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&bundle_path).expect("bundle file"))
            .expect("bundle json");
    assert_eq!(bundle_json["bundle_version"], "export-bundle/v0.1");
    assert!(bundle_json["files"].is_object());
}

#[test]
fn dag_replay_help_surface_contract() {
    let output = dag_command().args(["replay", "--help"]).output().expect("replay help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    for token in [
        "--out",
        "--run-id",
        "--reuse-cache",
        "--sandbox",
        "--hermetic",
        "--source-run-id",
        "--source-run-root",
        "--resource-capacity",
        "--from-node",
        "replay",
    ] {
        assert!(text.contains(token));
    }
    for detail in [
        "write-boundary check, not a process sandbox",
        "deny declared network effects",
        "deny declared clock effects",
        "declare a named runtime capacity as <name=count>",
        "best-effort local policy profile",
    ] {
        assert!(text.contains(detail), "missing replay help detail: {detail}");
    }
}

#[test]
fn dag_plan_explain_help_mentions_downstream_selector() {
    let output =
        dag_command().args(["plan", "explain", "--help"]).output().expect("plan explain help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("bijux-dag plan explain"));
    assert!(text.contains("--from-node"));
    assert!(text.contains("--jobs"));
    assert!(text.contains("--cpu-budget"));
    assert!(text.contains("--resource-capacity"));
    assert!(text.contains("declare a named runtime capacity as <name=count>"));
}

#[test]
fn dag_diff_help_surface_contract() {
    let output = dag_command().args(["diff", "--help"]).output().expect("diff help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("bijux-dag diff"));
    assert!(text.contains("--json"));
}

#[test]
fn dag_explain_help_surface_contract() {
    let output = dag_command().args(["explain", "--help"]).output().expect("explain help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("bijux-dag explain"));
    assert!(text.contains("--node"));
}

#[test]
fn dag_cache_help_surface_contract() {
    let output = dag_command().args(["cache", "--help"]).output().expect("cache help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    for token in ["cache", "verify", "pack", "explain"] {
        assert!(text.contains(token));
    }
}

#[test]
fn dag_adapters_help_surface_contract() {
    let output = dag_command().args(["adapters", "--help"]).output().expect("adapters help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("adapters"));
    assert!(!text.contains("ls"));
    assert!(!text.contains("doctor"));

    let nested =
        dag_command().args(["adapters", "ls", "--help"]).output().expect("adapters ls help");
    assert!(nested.status.success());
    let nested_text = String::from_utf8_lossy(&nested.stdout);
    assert!(nested_text.contains("bijux-dag adapters ls"));
}

#[test]
fn dag_validate_text_output_contract() {
    let dag = write_temp_dag();
    let output = dag_command().args(["validate", &dag]).output().expect("validate text");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("status:"));
}

#[test]
fn dag_validate_invalid_argument_fails() {
    let output = dag_command()
        .args(["validate", "non-existent-dag.json"])
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
        .args(["validate", invalid_path.to_str().unwrap()])
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
        .args(["run", &dag, "--out", out_dir.path().to_str().unwrap()])
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
        .args(["run", &dag, "--out", out_dir.path().to_str().unwrap()])
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
        .args(["run", &dag, "--out", out_dir.path().to_str().expect("run out path")])
        .output()
        .expect("run");
    assert!(run_output.status.success(), "run must succeed for fsck setup");

    let mut entries: Vec<_> =
        std::fs::read_dir(out_dir.path()).expect("read out dir").filter_map(Result::ok).collect();
    entries.sort_by_key(|entry| entry.file_name());
    let run_dir =
        entries.last().expect("expected run directory").path().to_string_lossy().into_owned();

    let fsck_output =
        dag_command().args(["fsck", &run_dir, "--strict", "--json"]).output().expect("fsck");
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
        .args(["fsck", bundle_path.to_str().expect("bundle path"), "--json"])
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
        .env("BIJUX_DAG_ENABLE_INTERNAL", "1")
        .args(["capabilities", "--backend", "kubernetes", "--json"])
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
fn capabilities_json_reports_container_execution_as_implemented() {
    let output = dag_command()
        .env("BIJUX_DAG_ENABLE_INTERNAL", "1")
        .args(["capabilities", "--json"])
        .output()
        .expect("capabilities json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("capabilities json");
    assert_eq!(payload["command"], "dag.capabilities");
    assert_eq!(payload["data"]["execution_modes"]["container"], "implemented");
    assert_eq!(payload["data"]["execution_lanes"]["container"], "ENFORCED");
}

#[test]
fn capabilities_backend_query_supports_hpc() {
    let output = dag_command()
        .env("BIJUX_DAG_ENABLE_INTERNAL", "1")
        .args(["capabilities", "--backend", "hpc", "--json"])
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
fn capabilities_backend_query_supports_slurm() {
    let output = dag_command()
        .env("BIJUX_DAG_ENABLE_INTERNAL", "1")
        .args(["capabilities", "--backend", "slurm", "--json"])
        .output()
        .expect("capabilities slurm backend");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("capabilities slurm json");
    assert_eq!(payload["command"], "dag.capabilities");
    assert_eq!(payload["data"]["backend"], "slurm");
    assert_eq!(payload["data"]["status"], "implemented");
    assert_eq!(payload["data"]["execution_lane"], "ENFORCED");
    assert_eq!(payload["data"]["production_ready"], false);
}

#[test]
fn capabilities_backend_query_supports_remote() {
    let output = dag_command()
        .env("BIJUX_DAG_ENABLE_INTERNAL", "1")
        .args(["capabilities", "--backend", "remote", "--json"])
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
fn semantic_portability_backend_query_surface_is_available() {
    let output = dag_command()
        .env("BIJUX_DAG_ENABLE_INTERNAL", "1")
        .args(["semantic-portability", "--backend", "kubernetes", "--json"])
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
        .args(["run", &dag, "--out", run_a.path().to_str().expect("run_a path"), "--json"])
        .output()
        .expect("run a");
    assert!(run_a_out.status.success());
    let run_b_out = dag_command()
        .args(["run", &dag, "--out", run_b.path().to_str().expect("run_b path"), "--json"])
        .output()
        .expect("run b");
    assert!(run_b_out.status.success());

    let run_a_payload: serde_json::Value = serde_json::from_slice(&run_a_out.stdout).expect("a");
    let run_b_payload: serde_json::Value = serde_json::from_slice(&run_b_out.stdout).expect("b");
    let run_a_dir = run_a_payload["data"]["run_dir"].as_str().expect("run a dir");
    let run_b_dir = run_b_payload["data"]["run_dir"].as_str().expect("run b dir");

    let output = dag_command()
        .env("BIJUX_DAG_ENABLE_INTERNAL", "1")
        .args([
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
    let export_help = dag_command().args(["export", "--help"]).output().expect("export help");
    assert!(export_help.status.success());
    let export_text = String::from_utf8_lossy(&export_help.stdout);
    assert!(export_text.contains("--from-run"));
    assert!(export_text.contains("--without-artifacts"));
    assert!(export_text.contains("--provenance-only"));
    assert!(export_text.contains("--redact"));

    let import_help = dag_command().args(["import", "--help"]).output().expect("import help");
    assert!(import_help.status.success());
    let import_text = String::from_utf8_lossy(&import_help.stdout);
    assert!(import_text.contains("--verify-only"));
}

#[test]
fn prove_help_and_json_surface_are_available() {
    let help = dag_command().args(["prove", "--help"]).output().expect("prove help");
    assert!(help.status.success());
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(text.contains("bijux-dag prove"));
}

#[test]
fn proof_summary_help_surface_is_available() {
    let help =
        dag_command().args(["proof-summary", "--help"]).output().expect("proof-summary help");
    assert!(help.status.success());
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(text.contains("bijux-dag proof-summary"));
}

#[test]
fn migrate_help_includes_dry_run_preview_flag() {
    let help = dag_command().args(["migrate", "dag", "--help"]).output().expect("migrate help");
    assert!(help.status.success());
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(text.contains("--dry-run"));
}

#[test]
fn dag_status_json_schema_contract() {
    let dag = write_temp_dag();
    let run_dir = tempfile::tempdir().expect("run out");
    let run = dag_command()
        .args(["run", "--json", &dag, "--out", run_dir.path().to_str().unwrap()])
        .output()
        .expect("run json");
    let run_payload: serde_json::Value =
        serde_json::from_slice(&run.stdout).expect("parse run payload");
    let run_path = run_payload["data"]["run_dir"].as_str().unwrap();

    let output = dag_command().args(["status", "--json", run_path]).output().expect("status json");

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
        .args(["run", "--json", &dag, "--out", first_run_dir.path().to_str().unwrap()])
        .output()
        .expect("run a");
    let run_b = dag_command()
        .args(["run", "--json", &dag, "--out", second_run_dir.path().to_str().unwrap()])
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

    let output =
        dag_command().args(["diff", "--json", run_a_path, run_b_path]).output().expect("diff json");

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
    let output = dag_command().args(["validate", &dag]).output().expect("validate text");

    assert!(output.status.success());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("{\"ok\""));

    let output_json =
        dag_command().args(["validate", "--json", &dag]).output().expect("validate json");

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
        .args(["run", "--json", &dag, "--out", out_dir.path().to_str().unwrap()])
        .output()
        .expect("run with json");

    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("run json parse");
    assert_eq!(payload["command"], "dag.run");
    assert_eq!(payload["status"], "ok");
    assert!(payload["data"].get("run_dir").and_then(|v| v.as_str()).is_some());
}
