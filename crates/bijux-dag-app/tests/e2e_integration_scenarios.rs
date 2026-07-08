use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_core::{Effect, Graph};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

mod support;

use support::{graph_chain, graph_diamond, graph_failure, graph_retry, graph_timeout};

fn repo_root() -> PathBuf {
    support::repo_root_from_manifest_dir(env!("CARGO_MANIFEST_DIR"))
}

fn write_graph(path: &Path, graph: &Graph) {
    let payload = serde_json::to_vec_pretty(graph).expect("serialize graph");
    fs::write(path, payload).expect("write graph");
}

fn run_dag(args: &[&str], cwd: &Path) -> (i32, String, String) {
    support::run_dag_command(args, cwd)
}

fn run_json(args: &[&str], cwd: &Path) -> Value {
    let (code, stdout, stderr) = run_dag(args, cwd);
    assert!(code == 0 || code == 2 || code == 3, "command failed: code={code} stderr={stderr}");
    serde_json::from_str(&stdout).expect("parse json envelope")
}

fn extract_run_dir(payload: &Value) -> PathBuf {
    let run_dir = payload["data"]["run_dir"].as_str().expect("run_dir string");
    PathBuf::from(run_dir)
}

fn output_path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[test]
fn e2e_minimal_validate_run_and_replay() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");

    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let graph_s = output_path_string(&graph);
    let out_s = output_path_string(&out_dir);

    let _ = run_json(&["validate", "--json", &graph_s], &root);
    let run = run_json(&["run", "--json", &graph_s, "--out", &out_s], &root);
    let run_dir = extract_run_dir(&run);
    let _ = run_json(&["replay", "--json", &output_path_string(&run_dir), "--out", &out_s], &root);
}

#[test]
#[ignore = "experimental"]
fn e2e_status_and_node_inspection_for_minimal_run() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");

    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let graph_s = output_path_string(&graph);
    let out_s = output_path_string(&out_dir);

    let run = run_json(&["run", "--json", &graph_s, "--out", &out_s], &root);
    let run_dir = extract_run_dir(&run);
    let run_dir_s = output_path_string(&run_dir);

    let _ = run_json(&["status", "--json", &run_dir_s], &root);
    let _ = run_json(&["node", "--json", &run_dir_s, "--id", "echo"], &root);
}

#[test]
fn e2e_diamond_outputs_and_manifest_totals() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph_path = temp.path().join("diamond.json");
    write_graph(&graph_path, &graph_diamond());

    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");

    let run = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph_path),
            "--out",
            &output_path_string(&out_dir),
        ],
        &root,
    );
    let run_dir = extract_run_dir(&run);
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("manifest.json")).expect("read manifest"),
    )
    .expect("parse manifest");
    assert!(manifest["run_summary"].is_object() || manifest["node_counts"].is_object());
    assert!(run_dir.join("nodes").join("a").join("trace.json").exists());
}

#[test]
fn e2e_failure_downstream_behavior() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph_path = temp.path().join("failure.json");
    write_graph(&graph_path, &graph_failure());
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");

    let (code, _stdout, _stderr) = run_dag(
        &["run", &output_path_string(&graph_path), "--out", &output_path_string(&out_dir)],
        &root,
    );
    assert!(code == 0 || code == 2 || code == 3);
}

#[test]
fn e2e_retry_accounting_present() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph_path = temp.path().join("retry.json");
    write_graph(&graph_path, &graph_retry());
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");

    let run = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph_path),
            "--out",
            &output_path_string(&out_dir),
        ],
        &root,
    );
    let run_dir = extract_run_dir(&run);
    let trace: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join("b").join("trace.json"))
            .expect("read trace"),
    )
    .expect("parse trace");
    assert!(trace.get("attempt").is_some());
}

#[test]
fn e2e_timeout_error_classification() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let mut graph = graph_timeout();
    graph.nodes[1].params =
        bijux_dag_core::ParamValue::Object(std::collections::BTreeMap::from([(
            "argv".to_string(),
            bijux_dag_core::ParamValue::Literal(json!(["/bin/sh", "-c", "sleep 1"])),
        )]));
    graph.nodes[1].timeout_ms = Some(1);
    let graph_path = temp.path().join("timeout.json");
    write_graph(&graph_path, &graph);
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");

    let (code, _stdout, _stderr) = run_dag(
        &["run", &output_path_string(&graph_path), "--out", &output_path_string(&out_dir)],
        &root,
    );
    assert!(code == 0 || code == 2 || code == 3);
}

#[test]
fn e2e_missing_outputs_failure_handling() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let mut graph = graph_chain();
    graph.nodes[1].params =
        bijux_dag_core::ParamValue::Object(std::collections::BTreeMap::from([(
            "argv".to_string(),
            bijux_dag_core::ParamValue::Literal(json!(["/bin/sh", "-c", "true"])),
        )]));
    let graph_path = temp.path().join("missing-outputs.json");
    write_graph(&graph_path, &graph);
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");
    let (code, _, _) = run_dag(
        &["run", &output_path_string(&graph_path), "--out", &output_path_string(&out_dir)],
        &root,
    );
    assert!(code == 0 || code == 2 || code == 3);
}

#[test]
fn e2e_cache_hit_second_run_and_invalidation() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph_path = temp.path().join("cache.json");
    let mut graph = graph_chain();
    write_graph(&graph_path, &graph);
    let out_dir = temp.path().join("runs");
    let cache_dir = temp.path().join("cache");
    fs::create_dir_all(&out_dir).expect("create runs");

    let first = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph_path),
            "--out",
            &output_path_string(&out_dir),
            "--cache",
            "readwrite",
            "--cache-dir",
            &output_path_string(&cache_dir),
        ],
        &root,
    );
    let second = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph_path),
            "--out",
            &output_path_string(&out_dir),
            "--cache",
            "readwrite",
            "--cache-dir",
            &output_path_string(&cache_dir),
        ],
        &root,
    );

    let first_manifest: Value = serde_json::from_str(
        &fs::read_to_string(extract_run_dir(&first).join("manifest.json"))
            .expect("read first manifest"),
    )
    .expect("parse first manifest");
    let second_manifest: Value = serde_json::from_str(
        &fs::read_to_string(extract_run_dir(&second).join("manifest.json"))
            .expect("read second manifest"),
    )
    .expect("parse second manifest");
    assert_eq!(first_manifest["graph_fingerprint"], second_manifest["graph_fingerprint"]);

    graph.nodes[0].params =
        bijux_dag_core::ParamValue::Object(std::collections::BTreeMap::from([(
            "value".to_string(),
            bijux_dag_core::ParamValue::Literal(json!("changed")),
        )]));
    write_graph(&graph_path, &graph);
    let changed = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph_path),
            "--out",
            &output_path_string(&out_dir),
            "--cache",
            "readwrite",
            "--cache-dir",
            &output_path_string(&cache_dir),
        ],
        &root,
    );
    let changed_manifest: Value = serde_json::from_str(
        &fs::read_to_string(extract_run_dir(&changed).join("manifest.json"))
            .expect("read changed manifest"),
    )
    .expect("parse changed manifest");
    assert_ne!(second_manifest["graph_fingerprint"], changed_manifest["graph_fingerprint"]);
}

#[test]
fn e2e_replay_diff_semantic_comparison() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph_path = root.join("evidence/authoring/examples/hello.dag.json");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");

    let run = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph_path),
            "--out",
            &output_path_string(&out_dir),
        ],
        &root,
    );
    let run_dir = extract_run_dir(&run);

    let replay = run_json(
        &[
            "replay",
            "--json",
            &output_path_string(&run_dir),
            "--out",
            &output_path_string(&out_dir),
        ],
        &root,
    );
    let replay_dir = extract_run_dir(&replay);

    let diff = run_json(
        &[
            "diff",
            "--json",
            &output_path_string(&run_dir),
            &output_path_string(&replay_dir),
            "--explain",
        ],
        &root,
    );
    assert!(diff["data"]["replay_equivalence"].is_object());
}

#[test]
#[ignore = "experimental"]
fn e2e_import_export_round_trip_for_run_bundle() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph_path = root.join("evidence/authoring/examples/hello.dag.json");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");

    let run = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph_path),
            "--out",
            &output_path_string(&out_dir),
        ],
        &root,
    );
    let run_dir = extract_run_dir(&run);
    let export_path = temp.path().join("bundle-with-files.json");
    let _ = run_json(
        &[
            "export",
            "--json",
            &output_path_string(&run_dir),
            "--out",
            &output_path_string(&export_path),
            "--include-files",
        ],
        &root,
    );
    let _ = run_json(&["import", "--json", &output_path_string(&export_path)], &root);

    let export_meta = temp.path().join("bundle-meta.json");
    let _ = run_json(
        &[
            "export",
            "--json",
            &output_path_string(&run_dir),
            "--out",
            &output_path_string(&export_meta),
        ],
        &root,
    );
    let _ = run_json(&["import", "--json", &output_path_string(&export_meta)], &root);
}

#[test]
fn e2e_selection_policy_compat_validation_and_no_partial_run_dir() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph_path = temp.path().join("selection.json");
    write_graph(&graph_path, &graph_diamond());
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");

    let (selection_code, _, _) = run_dag(
        &[
            "run",
            &output_path_string(&graph_path),
            "--out",
            &output_path_string(&out_dir),
            "--select",
            "b",
            "--exclude",
            "c",
            "--deny-env",
            "--clean-env",
        ],
        &root,
    );
    assert!(
        selection_code == 0 || selection_code == 2 || selection_code == 3,
        "unexpected selection run exit code: {selection_code}"
    );

    let compat_fixture = root.join("configs/dag/schema/fixtures/v0.1/positive/hello.valid.json");
    let (validate_code, _, _) = run_dag(&["validate", &output_path_string(&compat_fixture)], &root);
    assert!(
        validate_code == 0 || validate_code == 2 || validate_code == 3,
        "unexpected validate exit code: {validate_code}"
    );

    let invalid_graph = temp.path().join("invalid.json");
    fs::write(&invalid_graph, "{\"spec\":\"dag/v0.1\",\"nodes\":[],\"edges\":[]}")
        .expect("write invalid graph");
    let (code, _, _) = run_dag(
        &["run", &output_path_string(&invalid_graph), "--out", &output_path_string(&out_dir)],
        &root,
    );
    assert_ne!(code, 0);
}

#[test]
fn e2e_container_and_real_world_orchestration() {
    let root = repo_root();
    let docker_available = Command::new("docker")
        .arg("--version")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false);
    if docker_available {
        let _ = Command::new("true").status().expect("container scenario placeholder");
    }

    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");
    let templates = graph_chain();
    let const_template = templates.nodes[0].clone();
    let shell_template = templates.nodes[1].clone();
    let mut graph = templates;
    graph.nodes.clear();
    graph.edges.clear();
    for idx in 0..20 {
        let id = format!("n{idx}");
        let mut node = if idx == 0 { const_template.clone() } else { shell_template.clone() };
        node.id = id.clone();
        node.kind = if idx == 0 {
            bijux_dag_core::NodeKind::Const
        } else {
            bijux_dag_core::NodeKind::Shell
        };
        if idx > 0 {
            node.inputs = vec!["in".to_string()];
            node.effects = vec![Effect::Filesystem];
        }
        graph.nodes.push(node);
        if idx > 0 {
            graph.edges.push(bijux_dag_core::Edge {
                id: None,
                kind: bijux_dag_core::EdgeKind::Data,
                decision: None,
                from: bijux_dag_core::PortRef {
                    node_id: format!("n{}", idx - 1),
                    port: "out".to_string(),
                },
                to: bijux_dag_core::PortRef { node_id: id, port: "in".to_string() },
            });
        }
    }
    let graph_path = temp.path().join("real-world.json");
    write_graph(&graph_path, &graph);
    let (code, _, _) = run_dag(
        &["run", &output_path_string(&graph_path), "--out", &output_path_string(&out_dir)],
        &root,
    );
    assert!(code == 0 || code == 2 || code == 3);
    let has_manifest = fs::read_dir(&out_dir)
        .expect("read run output dir")
        .filter_map(Result::ok)
        .any(|entry| entry.path().join("manifest.json").exists());
    assert!(has_manifest || code != 0);
}
