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
use std::process::Command;
use tar as _;
use tempfile as _;
use thiserror as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run_dag(args: &[&str], cwd: &Path) -> (i32, String, String) {
    let output = Command::new("cargo")
        .current_dir(cwd)
        .env("RUSTFLAGS", "-Awarnings")
        .env("CARGO_TARGET_DIR", cwd.join("artifacts/target"))
        .args(["run", "-p", "bijux-dag-cli", "--", "dag"])
        .args(args)
        .output()
        .expect("run dag command");
    (
        output.status.code().unwrap_or(1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn run_json(args: &[&str], cwd: &Path) -> Value {
    let (code, stdout, stderr) = run_dag(args, cwd);
    assert!(code == 0, "command failed: code={code} stderr={stderr}");
    serde_json::from_str(&stdout).expect("parse json envelope")
}

fn output_path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn required_fields(schema_rel: &str) -> Vec<String> {
    let schema: Value = serde_json::from_str(
        &fs::read_to_string(repo_root().join(schema_rel)).expect("read schema"),
    )
    .expect("parse schema");
    schema["required"]
        .as_array()
        .expect("required")
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

fn run_dir_from_response(payload: &Value) -> PathBuf {
    PathBuf::from(payload["data"]["run_dir"].as_str().expect("run_dir"))
}

#[test]
fn diff_specs_and_regression_fixture_corpus_exist() {
    let root = repo_root();
    for rel in [
        "docs/spec/GRAPH_DIFF_SPEC_v0.1.md",
        "docs/spec/RUN_DIFF_SPEC_v0.1.md",
        "docs/spec/ARTIFACT_DIFF_SEMANTICS.md",
        "evidence/cache/diff/regression_corpus.json",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing diff completion artifact: {rel}"
        );
    }
}

#[test]
fn semantic_diff_classification_and_explain_surfaces_are_stable() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");

    let run_a = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "diff-a",
        ],
        &root,
    );
    let run_b = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "diff-b",
        ],
        &root,
    );
    let run_a_dir = run_dir_from_response(&run_a);
    let run_b_dir = run_dir_from_response(&run_b);
    let diff = run_json(
        &[
            "diff",
            "--json",
            &output_path_string(&run_a_dir),
            &output_path_string(&run_b_dir),
        ],
        &root,
    );
    let replay_eq = &diff["data"]["replay_equivalence"];
    assert_eq!(replay_eq["equivalent"], true);
    assert_eq!(
        replay_eq["reason_report"]["summary"],
        "runs are semantically equivalent under replay contract"
    );

    let why_rerun = run_json(
        &[
            "why-rerun",
            "--json",
            &output_path_string(&run_a_dir),
            &output_path_string(&run_b_dir),
        ],
        &root,
    );
    assert_eq!(why_rerun["data"]["equivalent"], true);
    assert!(why_rerun["data"]["reasons"].is_array());
}

#[test]
fn semantic_diff_reports_environment_resource_and_artifact_drift_cause_groups() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");

    let run_a = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "drift-a",
        ],
        &root,
    );
    let run_b = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "drift-b",
        ],
        &root,
    );
    let run_a_dir = run_dir_from_response(&run_a);
    let run_b_dir = run_dir_from_response(&run_b);

    let mut manifest: Value = serde_json::from_str(
        &fs::read_to_string(run_b_dir.join("manifest.json")).expect("manifest"),
    )
    .expect("manifest json");
    manifest["jobs"] = json!(64);
    manifest["policy"]["deny_env"] = json!(false);
    fs::write(
        run_b_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).expect("encode"),
    )
    .expect("write");

    let first_node_dir = fs::read_dir(run_b_dir.join("nodes"))
        .expect("read nodes")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.is_dir())
        .expect("node dir");
    let outputs_index = first_node_dir.join("outputs/index.json");
    if outputs_index.exists() {
        let mut index: Value =
            serde_json::from_str(&fs::read_to_string(&outputs_index).expect("index"))
                .expect("json");
        if let Some(files) = index["files"].as_array_mut() {
            if let Some(first) = files.first_mut() {
                first["sha256"] = json!("deadbeef");
            }
        }
        fs::write(
            outputs_index,
            serde_json::to_vec_pretty(&index).expect("encode"),
        )
        .expect("write");
    }

    let diff = run_json(
        &[
            "diff",
            "--json",
            &output_path_string(&run_a_dir),
            &output_path_string(&run_b_dir),
        ],
        &root,
    );
    let groups = &diff["data"]["replay_equivalence"]["cause_groups"];
    assert!(groups.get("manifest_drift").is_some());
    assert!(groups.get("artifact_payload").is_some());
}

#[test]
fn diff_schema_lockstep_human_snapshot_and_determinism_hold_under_stress() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");

    let run_a = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "det-a",
        ],
        &root,
    );
    let run_b = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "det-b",
        ],
        &root,
    );
    let run_a_dir = run_dir_from_response(&run_a);
    let run_b_dir = run_dir_from_response(&run_b);

    let diff1 = run_json(
        &[
            "diff",
            "--json",
            &output_path_string(&run_a_dir),
            &output_path_string(&run_b_dir),
        ],
        &root,
    );
    let diff2 = run_json(
        &[
            "diff",
            "--json",
            &output_path_string(&run_a_dir),
            &output_path_string(&run_b_dir),
        ],
        &root,
    );
    assert_eq!(diff1, diff2);
    for field in required_fields("configs/schema/operator/run_diff.schema.json") {
        assert!(
            diff1["data"].get(&field).is_some(),
            "run diff missing required field {field}"
        );
    }

    let (code, human, stderr) = run_dag(
        &[
            "diff",
            &output_path_string(&run_a_dir),
            &output_path_string(&run_b_dir),
            "--explain",
        ],
        &root,
    );
    assert!(code == 0, "human diff failed: {stderr}");
    assert_eq!(
        human,
        include_str!("snapshots/diff_human_output_contract.txt")
    );
}
