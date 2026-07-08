use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json::Value;
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

fn run_json(args: &[&str], cwd: &Path) -> Value {
    let (code, stdout, stderr) = support::run_dag_command(args, cwd);
    assert_eq!(code, 0, "command failed: stderr={stderr}");
    serde_json::from_str(&stdout).expect("parse json envelope")
}

fn run_json_owned(args: Vec<String>, cwd: &Path) -> Value {
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    run_json(&refs, cwd)
}

fn run_dir_from_response(payload: &Value) -> PathBuf {
    PathBuf::from(payload["data"]["run_dir"].as_str().expect("run dir"))
}

fn read_manifest(run_dir: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(run_dir.join("manifest.json")).expect("manifest"))
        .expect("manifest json")
}

fn read_trace(run_dir: &Path, node_id: &str) -> Value {
    serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join(node_id).join("trace.json")).expect("trace"),
    )
    .expect("trace json")
}

fn workflow_graph(root: &Path) -> PathBuf {
    root.join("evidence/dag/authoring/examples/audience-branch-bulletin.dag.json")
}

fn copy_source_note(root: &Path, destination: &Path, fixture_name: &str) -> PathBuf {
    let source =
        root.join("evidence/dag/authoring/examples/audience-branch-source").join(fixture_name);
    fs::create_dir_all(destination).expect("inputs dir");
    let note = destination.join(fixture_name);
    fs::copy(source, &note).expect("copy note");
    note
}

fn bulletin_path(run_dir: &Path) -> PathBuf {
    run_dir
        .join("nodes")
        .join("publish_bulletin")
        .join("outputs")
        .join("publish")
        .join("bulletin.md")
}

fn artifact_id_for_bulletin(registry_payload: &Value) -> String {
    registry_payload["data"]["artifacts"]
        .as_array()
        .expect("artifacts array")
        .iter()
        .find(|artifact| {
            artifact["path"].as_str().is_some_and(|path| {
                path.ends_with("nodes/publish_bulletin/outputs/publish/bulletin.md")
            })
        })
        .and_then(|artifact| artifact["artifact_id"].as_str())
        .expect("bulletin artifact id")
        .to_string()
}

fn json_array_strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .expect("json array")
        .iter()
        .map(|item| item.as_str().expect("string item").to_string())
        .collect()
}

#[test]
fn branch_bulletin_workflow_supports_complete_operator_story() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let original_note = copy_source_note(&root, &temp.path().join("inputs"), "team-update.md");
    let revised_note =
        copy_source_note(&root, &temp.path().join("inputs"), "team-update-revised.md");
    let runs_dir = temp.path().join("runs");
    let cache_dir = temp.path().join("cache");
    let deliverables_dir = temp.path().join("deliverables");
    fs::create_dir_all(&runs_dir).expect("runs dir");
    fs::create_dir_all(&cache_dir).expect("cache dir");
    fs::create_dir_all(&deliverables_dir).expect("deliverables dir");

    let graph = workflow_graph(&root);
    let cold = run_json_owned(
        vec![
            "run".to_string(),
            "--json".to_string(),
            output_path_string(&graph),
            "--out".to_string(),
            output_path_string(&runs_dir),
            "--run-id".to_string(),
            "branch-bulletin-cold".to_string(),
            "--cache".to_string(),
            "readwrite".to_string(),
            "--cache-dir".to_string(),
            output_path_string(&cache_dir),
            "--input".to_string(),
            format!("source_note={}", output_path_string(&original_note)),
            "--input".to_string(),
            "audience_mode=technical".to_string(),
        ],
        &root,
    );

    let cold_run = run_dir_from_response(&cold);
    let cold_manifest = read_manifest(&cold_run);
    assert_eq!(cold_manifest["status"], "success");
    assert_eq!(
        cold_manifest["run_metadata"]["graph_inputs"]["source_note"],
        output_path_string(&original_note)
    );
    assert_eq!(cold_manifest["run_metadata"]["graph_inputs"]["audience_mode"], "technical");
    assert_eq!(cold_manifest["node_counts"]["cached"], 0);

    let cold_bulletin = fs::read_to_string(bulletin_path(&cold_run)).expect("cold bulletin");
    assert!(cold_bulletin.starts_with("# Technical Bulletin\n"));
    assert!(cold_bulletin.contains("Audience lane: technical"));

    let warm = run_json_owned(
        vec![
            "run".to_string(),
            "--json".to_string(),
            output_path_string(&graph),
            "--out".to_string(),
            output_path_string(&runs_dir),
            "--run-id".to_string(),
            "branch-bulletin-warm".to_string(),
            "--cache".to_string(),
            "readwrite".to_string(),
            "--cache-dir".to_string(),
            output_path_string(&cache_dir),
            "--input".to_string(),
            format!("source_note={}", output_path_string(&original_note)),
            "--input".to_string(),
            "audience_mode=technical".to_string(),
        ],
        &root,
    );

    let warm_run = run_dir_from_response(&warm);
    let warm_manifest = read_manifest(&warm_run);
    assert!(
        warm_manifest["node_counts"]["cached"].as_u64().unwrap_or(0) >= 3,
        "expected warm cache reuse for prepare, branch, and selected render nodes"
    );
    assert_eq!(read_trace(&warm_run, "prepare_note")["status"], "cached");
    assert_eq!(read_trace(&warm_run, "choose_audience_lane")["status"], "cached");
    assert_eq!(read_trace(&warm_run, "render_technical_bulletin")["status"], "cached");
    assert_eq!(read_trace(&warm_run, "render_executive_bulletin")["status"], "skipped");
    assert_eq!(read_trace(&warm_run, "publish_bulletin")["status"], "success");

    let registry = run_json_owned(
        vec![
            "artifact".to_string(),
            "registry".to_string(),
            output_path_string(&cold_run),
            "--json".to_string(),
        ],
        &root,
    );
    let artifact_id = artifact_id_for_bulletin(&registry);

    let artifact_inspect = run_json_owned(
        vec![
            "artifact-inspect".to_string(),
            output_path_string(&cold_run),
            artifact_id.clone(),
            "--json".to_string(),
        ],
        &root,
    );
    let artifact_sha =
        artifact_inspect["data"]["artifact_sha256"].as_str().expect("artifact sha").to_string();
    assert_eq!(artifact_inspect["data"]["promotable"], true);
    assert_eq!(artifact_sha.len(), 64);
    assert!(
        artifact_inspect["data"]["lineage"]["upstream_artifact_ids"]
            .as_array()
            .is_some_and(|items| !items.is_empty()),
        "expected final bulletin to expose upstream lineage"
    );

    let artifact_hash = run_json_owned(
        vec![
            "hash".to_string(),
            "artifact".to_string(),
            "--json".to_string(),
            output_path_string(&bulletin_path(&cold_run)),
        ],
        &root,
    );
    assert_eq!(artifact_hash["data"]["artifact_sha256"], artifact_sha);

    let lineage = run_json_owned(
        vec![
            "artifact".to_string(),
            "lineage".to_string(),
            output_path_string(&cold_run),
            "--json".to_string(),
        ],
        &root,
    );
    assert!(
        lineage["data"]["edge_count"].as_u64().unwrap_or(0) >= 3,
        "expected branch bulletin lineage snapshot to record the publishing chain"
    );

    let changed = run_json_owned(
        vec![
            "run".to_string(),
            "--json".to_string(),
            output_path_string(&graph),
            "--out".to_string(),
            output_path_string(&runs_dir),
            "--run-id".to_string(),
            "branch-bulletin-updated".to_string(),
            "--cache".to_string(),
            "readwrite".to_string(),
            "--cache-dir".to_string(),
            output_path_string(&cache_dir),
            "--input".to_string(),
            format!("source_note={}", output_path_string(&revised_note)),
            "--input".to_string(),
            "audience_mode=executive".to_string(),
        ],
        &root,
    );

    let changed_run = run_dir_from_response(&changed);
    let changed_manifest = read_manifest(&changed_run);
    assert_eq!(changed_manifest["status"], "success");
    assert_eq!(read_trace(&changed_run, "choose_audience_lane")["branch_decision"], "executive");
    assert_eq!(read_trace(&changed_run, "render_executive_bulletin")["status"], "success");
    assert_eq!(
        read_trace(&changed_run, "render_technical_bulletin")["skip_reason"]["reason"],
        "branch_decision_not_selected"
    );

    let compare = run_json_owned(
        vec![
            "runs".to_string(),
            "compare".to_string(),
            "branch-bulletin-warm".to_string(),
            "branch-bulletin-updated".to_string(),
            "--root".to_string(),
            output_path_string(&runs_dir),
            "--json".to_string(),
        ],
        &root,
    );

    let changed_inputs = json_array_strings(&compare["data"]["input_values"]["changed_inputs"]);
    assert!(changed_inputs.iter().any(|item| item == "source_note"));
    assert!(changed_inputs.iter().any(|item| item == "audience_mode"));

    let changed_nodes = json_array_strings(&compare["data"]["node_statuses"]["changed_nodes"]);
    for node_id in ["render_executive_bulletin", "render_technical_bulletin"] {
        assert!(
            changed_nodes.iter().any(|item| item == node_id),
            "expected runs compare to surface {node_id} as changed"
        );
    }

    let changed_outputs = json_array_strings(&compare["data"]["output_hashes"]["changed_outputs"]);
    assert!(
        changed_outputs.iter().any(|item| item == "publish_bulletin:publish/bulletin.md"),
        "expected final bulletin artifact to be attributed as changed"
    );
    assert!(
        changed_outputs.iter().any(|item| item == "publish_bulletin:publish/selection.json"),
        "expected selected-lane evidence to be attributed as changed"
    );

    let proof_source = run_json_owned(
        vec![
            "run".to_string(),
            "--json".to_string(),
            output_path_string(&graph),
            "--out".to_string(),
            output_path_string(&runs_dir),
            "--run-id".to_string(),
            "branch-bulletin-proof-source".to_string(),
            "--input".to_string(),
            format!("source_note={}", output_path_string(&original_note)),
            "--input".to_string(),
            "audience_mode=executive".to_string(),
        ],
        &root,
    );
    let proof_source_run = run_dir_from_response(&proof_source);
    let proof_source_manifest = read_manifest(&proof_source_run);
    assert_eq!(proof_source_manifest["status"], "success");

    let replay = run_json_owned(
        vec![
            "replay".to_string(),
            "--json".to_string(),
            "--source-run-id".to_string(),
            "branch-bulletin-proof-source".to_string(),
            "--source-run-root".to_string(),
            output_path_string(&runs_dir),
            "--out".to_string(),
            output_path_string(&runs_dir),
            "--run-id".to_string(),
            "branch-bulletin-replay".to_string(),
            "--select".to_string(),
            "id:publish_bulletin".to_string(),
            "--dependency-closure".to_string(),
            "--prove".to_string(),
        ],
        &root,
    );

    let replay_run = run_dir_from_response(&replay);
    let replay_manifest = read_manifest(&replay_run);
    assert_eq!(replay_manifest["status"], "success");
    assert_eq!(replay_manifest["run_metadata"]["parent_run_id"], "branch-bulletin-proof-source");
    assert_eq!(replay_manifest["run_metadata"]["source_run_id"], "branch-bulletin-proof-source");
    assert_eq!(replay["data"]["replay_proof"]["equivalent"], true);
    assert_eq!(
        replay["data"]["replay_proof"]["branch_decision_drift_nodes"]
            .as_array()
            .expect("branch drift nodes")
            .len(),
        0
    );

    let verify = run_json_owned(
        vec![
            "verify".to_string(),
            "--json".to_string(),
            output_path_string(&replay_run),
            "--strict".to_string(),
        ],
        &root,
    );
    assert_eq!(verify["ok"], true);

    let promote = run_json_owned(
        vec![
            "artifact".to_string(),
            "promote".to_string(),
            output_path_string(&changed_run),
            "publish_bulletin:bulletin.md".to_string(),
            "--deliverables-root".to_string(),
            output_path_string(&deliverables_dir),
            "--to".to_string(),
            "release".to_string(),
            "--json".to_string(),
        ],
        &root,
    );

    let destination = PathBuf::from(promote["data"]["destination"].as_str().expect("destination"));
    assert!(destination.join("payload").join("bulletin.md").exists());
    assert!(destination.join("promotion.json").exists());
}
