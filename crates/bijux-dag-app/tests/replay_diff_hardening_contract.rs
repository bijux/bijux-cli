use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json::{json, Value};
use sha2 as _;
use std::fs;
use std::path::{Path, PathBuf};
use tar as _;
use tempfile as _;
use thiserror as _;

mod support;

fn repo_root() -> PathBuf {
    support::repo_root_from_manifest_dir(env!("CARGO_MANIFEST_DIR"))
}

fn run_dag(args: &[&str], cwd: &Path) -> (i32, String, String) {
    support::run_dag_command(args, cwd)
}

fn run_json(args: &[&str], cwd: &Path) -> Value {
    let (code, stdout, stderr) = run_dag(args, cwd);
    assert!(code == 0, "command failed: code={code} stderr={stderr}");
    serde_json::from_str(&stdout).expect("parse json envelope")
}

fn out(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

fn required_fields(schema_rel: &str) -> Vec<String> {
    let root = repo_root();
    let schema: Value =
        serde_json::from_str(&fs::read_to_string(root.join(schema_rel)).expect("read schema"))
            .expect("parse schema");
    schema["required"]
        .as_array()
        .expect("required")
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

fn write_const_graph(path: &Path, shape: &str) {
    let payload = match shape {
        "minimal" => json!({
            "spec":"bijux-dag/v0.1",
            "meta":{"name":"minimal","owners":[],"tags":[]},
            "nodes":[{"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out.txt"}],"params":{"value":"1"}}],
            "edges":[]
        }),
        "diamond" => json!({
            "spec":"bijux-dag/v0.1",
            "meta":{"name":"diamond","owners":[],"tags":[]},
            "nodes":[
              {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":"1"}},
              {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":"2"}},
              {"id":"c","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"c/out"}],"params":{"value":"3"}},
              {"id":"d","kind":"const","inputs":["l","r"],"outputs":[{"name":"out","path":"d/out"}],"params":{"value":"4"}}
            ],
            "edges":[
              {"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}},
              {"from":{"node_id":"a","port":"out"},"to":{"node_id":"c","port":"in"}},
              {"from":{"node_id":"b","port":"out"},"to":{"node_id":"d","port":"l"}},
              {"from":{"node_id":"c","port":"out"},"to":{"node_id":"d","port":"r"}}
            ]
        }),
        _ => json!({
            "spec":"bijux-dag/v0.1",
            "meta":{"name":"cache-heavy","owners":[],"tags":[]},
            "nodes":[
              {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":"1"}},
              {"id":"b","kind":"const","inputs":[],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":"2"}},
              {"id":"c","kind":"const","inputs":[],"outputs":[{"name":"out","path":"c/out"}],"params":{"value":"3"}},
              {"id":"join","kind":"const","inputs":["x","y","z"],"outputs":[{"name":"out","path":"join/out"}],"params":{"value":"ok"}}
            ],
            "edges":[
              {"from":{"node_id":"a","port":"out"},"to":{"node_id":"join","port":"x"}},
              {"from":{"node_id":"b","port":"out"},"to":{"node_id":"join","port":"y"}},
              {"from":{"node_id":"c","port":"out"},"to":{"node_id":"join","port":"z"}}
            ]
        }),
    };
    fs::write(path, serde_json::to_vec_pretty(&payload).expect("encode")).expect("write graph");
}

fn run_and_replay_with_prove(
    root: &Path,
    graph: &Path,
    out_dir: &Path,
    source: &str,
    replay: &str,
) -> Value {
    let _ =
        run_json(&["run", "--json", &out(graph), "--out", &out(out_dir), "--run-id", source], root);
    let source_run = out_dir.join(format!("run-{source}"));
    run_json(
        &[
            "replay",
            "--json",
            &out(&source_run),
            "--out",
            &out(out_dir),
            "--run-id",
            replay,
            "--prove",
        ],
        root,
    )
}

#[test]
fn replay_proof_schema_lockstep_and_mismatch_grouping() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let graph = tmp.path().join("minimal.json");
    write_const_graph(&graph, "minimal");

    let _ = run_json(
        &["run", "--json", &out(&graph), "--out", &out(&out_dir), "--run-id", "mismatch-source"],
        &root,
    );
    let source_run = out_dir.join("run-mismatch-source");
    let mut manifest: Value = serde_json::from_str(
        &fs::read_to_string(source_run.join("manifest.json")).expect("manifest"),
    )
    .expect("json");
    manifest["graph_fingerprint"] = json!("tampered");
    fs::write(
        source_run.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).expect("encode"),
    )
    .expect("write");

    let proved = run_json(
        &[
            "replay",
            "--json",
            &out(&source_run),
            "--out",
            &out(&out_dir),
            "--run-id",
            "mismatch-replay",
            "--prove",
        ],
        &root,
    );

    let proof = &proved["data"]["replay_proof"];
    for field in required_fields("configs/dag/schema/operator/replay_proof.schema.json") {
        assert!(proof.get(&field).is_some(), "replay proof missing required field `{field}`");
    }
    assert_eq!(proof["fidelity_level"], "diverged");
    assert!(proof["cause_groups"].is_object());
}

#[test]
fn replay_exactness_covers_minimal_diamond_and_cache_oriented_graphs() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");

    for (shape, source, replay) in [
        ("minimal", "min-src", "min-replay"),
        ("diamond", "dia-src", "dia-replay"),
        ("cache", "cache-src", "cache-replay"),
    ] {
        let graph = tmp.path().join(format!("{shape}.json"));
        write_const_graph(&graph, shape);
        let proved = run_and_replay_with_prove(&root, &graph, &out_dir, source, replay);
        assert!(proved["data"]["replay_proof"]["fidelity_level"].is_string());
        if shape != "cache" {
            assert_eq!(proved["data"]["replay_proof"]["fidelity_level"], "strict_equivalent");
        }
    }
}

#[test]
fn replay_from_selected_node_dry_run_is_supported() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let graph = tmp.path().join("diamond.json");
    write_const_graph(&graph, "diamond");

    let _ = run_json(
        &["run", "--json", &out(&graph), "--out", &out(&out_dir), "--run-id", "import-src"],
        &root,
    );
    let source_run = out_dir.join("run-import-src");

    let dry = run_json(
        &[
            "replay",
            "--json",
            &out(&source_run),
            "--out",
            &out(&out_dir),
            "--dry-run",
            "--select",
            "id:d",
        ],
        &root,
    );
    assert!(dry["data"]["dry_run_plan"].is_object());
}

#[test]
#[ignore = "experimental"]
fn replay_imported_bundle_round_trip_is_supported() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let graph = tmp.path().join("diamond.json");
    write_const_graph(&graph, "diamond");

    let _ = run_json(
        &["run", "--json", &out(&graph), "--out", &out(&out_dir), "--run-id", "import-src"],
        &root,
    );
    let source_run = out_dir.join("run-import-src");
    let bundle = tmp.path().join("bundle.json");
    let _ = run_json(
        &["export", "--json", &out(&source_run), "--out", &out(&bundle), "--manifest-only"],
        &root,
    );
    let imported = run_json(&["import", "--json", &out(&bundle)], &root);
    assert!(imported["ok"].as_bool().unwrap_or(false));
}

#[test]
fn replay_missing_artifacts_and_environment_mismatch_downgrade_fidelity() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let graph = tmp.path().join("minimal.json");
    write_const_graph(&graph, "minimal");

    let _ = run_json(
        &["run", "--json", &out(&graph), "--out", &out(&out_dir), "--run-id", "drift-src"],
        &root,
    );
    let source_run = out_dir.join("run-drift-src");

    let outputs_index = source_run.join("outputs/index.json");
    let mut index: Value =
        serde_json::from_str(&fs::read_to_string(&outputs_index).expect("index")).expect("json");
    if let Some(files) = index.get_mut("files").and_then(Value::as_array_mut) {
        if let Some(first) = files.first_mut() {
            first["sha256"] = json!("deadbeef");
        }
    }
    fs::write(&outputs_index, serde_json::to_vec_pretty(&index).expect("encode")).expect("write");

    let first_node_dir = fs::read_dir(source_run.join("nodes"))
        .expect("read nodes")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|p| p.is_dir())
        .expect("node dir");
    let trace_path = first_node_dir.join("trace.json");
    let mut trace: Value =
        serde_json::from_str(&fs::read_to_string(&trace_path).expect("trace")).expect("json");
    trace["status"] = json!("failed");
    fs::write(&trace_path, serde_json::to_vec_pretty(&trace).expect("encode")).expect("write");

    let mut manifest: Value = serde_json::from_str(
        &fs::read_to_string(source_run.join("manifest.json")).expect("manifest"),
    )
    .expect("json");
    manifest["policy"]["deny_env"] = json!(false);
    fs::write(
        source_run.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).expect("encode"),
    )
    .expect("write");

    let proved = run_json(
        &[
            "replay",
            "--json",
            &out(&source_run),
            "--out",
            &out(&out_dir),
            "--run-id",
            "drift-replay",
            "--prove",
        ],
        &root,
    );
    assert_eq!(proved["data"]["replay_proof"]["fidelity_level"], "diverged");
    assert!(proved["data"]["replay_proof"]["reasons"].is_array());
}

#[test]
fn replay_diff_schema_is_lockstep_and_semantic() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let graph = tmp.path().join("diamond.json");
    write_const_graph(&graph, "diamond");

    let _ = run_json(
        &["run", "--json", &out(&graph), "--out", &out(&out_dir), "--run-id", "diff-a"],
        &root,
    );
    let _ = run_json(
        &["run", "--json", &out(&graph), "--out", &out(&out_dir), "--run-id", "diff-b"],
        &root,
    );
    let run_a = out_dir.join("run-diff-a");
    let run_b = out_dir.join("run-diff-b");

    let run_diff = run_json(&["diff", "--json", &out(&run_a), &out(&run_b)], &root);
    for field in required_fields("configs/dag/schema/operator/run_diff.schema.json") {
        assert!(run_diff["data"].get(&field).is_some(), "run diff missing field `{field}`");
    }
}

#[test]
#[ignore = "experimental"]
fn canonical_diff_and_artifact_trace_schemas_are_lockstep() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let graph = tmp.path().join("diamond.json");
    write_const_graph(&graph, "diamond");

    let _ = run_json(
        &["run", "--json", &out(&graph), "--out", &out(&out_dir), "--run-id", "diff-a"],
        &root,
    );
    let run_a = out_dir.join("run-diff-a");
    let canonical_diff = run_json(&["canonical-diff", "--json", &out(&graph)], &root);
    for field in required_fields("configs/dag/schema/operator/graph_diff.schema.json") {
        assert!(canonical_diff["data"].get(&field).is_some(), "graph diff missing field `{field}`");
    }

    let trace = run_json(&["trace-artifact", "--json", &out(&run_a), "a:out"], &root);
    for field in required_fields("configs/dag/schema/operator/artifact_trace.schema.json") {
        assert!(trace["data"].get(&field).is_some(), "artifact trace missing field `{field}`");
    }
}

#[test]
fn explain_failure_schema_lockstep_and_human_readable_snapshots_are_stable() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let graph = tmp.path().join("minimal.json");
    write_const_graph(&graph, "minimal");

    let _ = run_json(
        &["run", "--json", &out(&graph), "--out", &out(&out_dir), "--run-id", "fail-source"],
        &root,
    );
    let run_dir = out_dir.join("run-fail-source");

    let node_dir = fs::read_dir(run_dir.join("nodes"))
        .expect("nodes")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| p.is_dir())
        .expect("node dir");
    let trace_path = node_dir.join("trace.json");
    let mut trace: Value =
        serde_json::from_str(&fs::read_to_string(&trace_path).expect("trace")).expect("json");
    trace["status"] = json!("failed");
    fs::write(&trace_path, serde_json::to_vec_pretty(&trace).expect("encode")).expect("write");

    let explain = run_json(
        &["--json", "runs", "explain-failure", "fail-source", "--root", &out(&out_dir)],
        &root,
    );
    for field in required_fields("configs/dag/schema/operator/run_explain_failure.schema.json") {
        assert!(explain["data"].get(&field).is_some(), "explain-failure missing field `{field}`");
    }

    let (_code_diff, stdout_diff, stderr_diff) =
        run_dag(&["diff", &out(&run_dir), &out(&run_dir), "--explain"], &root);
    let _ = stderr_diff;
    let expected_diff = include_str!("snapshots/replay_diff_human_output.txt");
    assert_eq!(stdout_diff, expected_diff);

    let (_code_replay, stdout_replay, stderr_replay) = run_dag(
        &["replay", &out(&run_dir), "--out", &out(&out_dir), "--run-id", "human-proof", "--prove"],
        &root,
    );
    let _ = stderr_replay;
    let normalized = stdout_replay
        .lines()
        .map(|line| {
            if line.starts_with("run dir: ") {
                "run dir: <normalized>".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let expected_replay = include_str!("snapshots/replay_proof_human_output.txt");
    assert_eq!(normalized, expected_replay);
}
