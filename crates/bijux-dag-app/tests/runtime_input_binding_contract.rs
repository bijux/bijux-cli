use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json::{json, Value};
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use std::fs;
use std::path::{Path, PathBuf};

mod support;

fn repo_root() -> PathBuf {
    support::repo_root_from_manifest_dir(env!("CARGO_MANIFEST_DIR"))
}

fn output_path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn write_runtime_input_graph(path: &Path) {
    fs::write(
        path,
        serde_json::to_vec_pretty(&json!({
            "spec":"bijux-dag/v0.1",
            "meta":{"name":"runtime-inputs","owners":[],"tags":[]},
            "inputs":{
                "region":{"type":"string","default":"default-region"},
                "payload":{
                    "type":"object",
                    "properties":{
                        "tenant":{"type":"string","required":true}
                    },
                    "required":true
                },
                "api_token":{"type":"string","required":true}
            },
            "nodes":[
                {
                    "id":"emit_region",
                    "kind":"const",
                    "inputs":[],
                    "outputs":[{"name":"value","path":"emit/region.json"}],
                    "params":{"value":{"graph_input":"region"}}
                },
                {
                    "id":"emit_payload",
                    "kind":"const",
                    "inputs":[],
                    "outputs":[{"name":"value","path":"emit/payload.json"}],
                    "params":{"value":{"graph_input":"payload"}}
                },
                {
                    "id":"emit_secret",
                    "kind":"const",
                    "inputs":[],
                    "outputs":[{"name":"value","path":"emit/secret.json"}],
                    "params":{"value":{"graph_input":"api_token"}}
                }
            ],
            "edges":[]
        }))
        .expect("graph json"),
    )
    .expect("write graph");
}

fn run_json(args: &[&str], cwd: &Path) -> Value {
    let (code, stdout, stderr) = support::run_dag_command(args, cwd);
    assert_eq!(code, 0, "command failed: stderr={stderr}");
    serde_json::from_str(&stdout).expect("parse json envelope")
}

fn extract_run_dir(payload: &Value) -> PathBuf {
    PathBuf::from(payload["data"]["run_dir"].as_str().expect("run dir"))
}

#[test]
fn cli_inputs_override_defaults_and_land_in_manifest() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph = temp.path().join("graph.json");
    write_runtime_input_graph(&graph);
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("runs dir");

    let payload = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--input",
            "region=us-east-1",
            "--input",
            "payload={\"tenant\":\"atlas\"}",
            "--input",
            "api_token=secret-123",
        ],
        &root,
    );
    let run_dir = extract_run_dir(&payload);
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("manifest.json")).expect("manifest"))
            .expect("manifest json");
    assert_eq!(manifest["run_metadata"]["graph_inputs"]["region"], "us-east-1");
    assert_eq!(manifest["run_metadata"]["graph_inputs"]["payload"]["tenant"], "atlas");
    assert_eq!(manifest["run_metadata"]["graph_inputs"]["api_token"], "secret-123");

    let emitted_region: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes/emit_region/outputs/emit/region.json"))
            .expect("region output"),
    )
    .expect("region json");
    assert_eq!(emitted_region, "us-east-1");
}

#[test]
fn inputs_file_supports_json_and_cli_overrides_file_values() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph = temp.path().join("graph.json");
    write_runtime_input_graph(&graph);
    let out_dir = temp.path().join("runs");
    let inputs_file = temp.path().join("inputs.json");
    fs::create_dir_all(&out_dir).expect("runs dir");
    fs::write(
        &inputs_file,
        serde_json::to_vec_pretty(&json!({
            "region":"file-region",
            "payload":{"tenant":"from-file"},
            "api_token":"from-file-secret"
        }))
        .expect("inputs json"),
    )
    .expect("write inputs file");

    let payload = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--inputs-file",
            &output_path_string(&inputs_file),
            "--input",
            "region=cli-region",
        ],
        &root,
    );
    let run_dir = extract_run_dir(&payload);
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("manifest.json")).expect("manifest"))
            .expect("manifest json");
    assert_eq!(manifest["run_metadata"]["graph_inputs"]["region"], "cli-region");
    assert_eq!(manifest["run_metadata"]["graph_inputs"]["payload"]["tenant"], "from-file");
}

#[test]
fn run_rejects_missing_required_runtime_input_before_execution() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph = temp.path().join("graph.json");
    write_runtime_input_graph(&graph);
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("runs dir");

    let (code, stdout, stderr) = support::run_dag_command(
        &[
            "run",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--input",
            "region=only-region",
        ],
        &root,
    );
    assert_eq!(code, 2);
    assert!(stdout.is_empty());
    assert!(stderr.contains("missing required runtime inputs: api_token, payload"));
    assert!(
        fs::read_dir(&out_dir).expect("read out dir").next().is_none(),
        "run directory should not be created when required inputs are missing"
    );
}

#[test]
fn run_json_error_includes_user_error_class_for_missing_required_inputs() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph = temp.path().join("graph.json");
    write_runtime_input_graph(&graph);
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("runs dir");

    let (code, stdout, stderr) = support::run_dag_command(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--input",
            "region=only-region",
        ],
        &root,
    );
    assert_eq!(code, 2);
    assert!(stderr.is_empty(), "json errors should not write human stderr: {stderr}");
    let payload: Value = serde_json::from_str(&stdout).expect("parse json error");
    assert_eq!(payload["status"], "invalid");
    assert_eq!(payload["data"]["error_class"], "user");
    assert_eq!(payload["data"]["missing_inputs"], json!(["api_token", "payload"]));
}

#[test]
fn run_human_output_redacts_secret_like_input_values() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph = temp.path().join("graph.json");
    write_runtime_input_graph(&graph);
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("runs dir");

    let (code, stdout, stderr) = support::run_dag_command(
        &[
            "run",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--input",
            "region=human-region",
            "--input",
            "payload={\"tenant\":\"atlas\"}",
            "--input",
            "api_token=top-secret-token",
        ],
        &root,
    );
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(stdout.contains("\"region\":\"human-region\""));
    assert!(stdout.contains("\"api_token\":\"[REDACTED]\""));
    assert!(stdout.contains("redacted_inputs: [\"api_token\"]"));
    assert!(!stdout.contains("top-secret-token"));
}

#[test]
fn run_reports_exact_path_for_invalid_typed_runtime_input() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph = temp.path().join("graph.json");
    write_runtime_input_graph(&graph);
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("runs dir");

    let (code, stdout, stderr) = support::run_dag_command(
        &[
            "run",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--input",
            "region=human-region",
            "--input",
            "payload={\"tenant\":7}",
            "--input",
            "api_token=top-secret-token",
        ],
        &root,
    );
    assert_eq!(code, 2);
    assert!(stdout.is_empty());
    assert!(stderr.contains("/inputs/payload/tenant"));
    assert!(stderr.contains("expected string"));
}
