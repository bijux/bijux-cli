use bijux_dag_app::{
    doctor_run, explain_failure, format_inspect_human, inspect_summary, list_runs, run_timeline, run_tree,
};
use serde_json::json;
use std::fs;

fn write_run_fixture(base: &std::path::Path, run_id: &str) -> std::path::PathBuf {
    let run = base.join(run_id);
    fs::create_dir_all(run.join("nodes").join("a")).expect("mkdir node a");
    fs::create_dir_all(run.join("nodes").join("b")).expect("mkdir node b");
    fs::write(
        run.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "run_id": run_id,
            "status": "failed",
            "graph_fingerprint": "g1",
            "started_unix_ms": 1000u64,
            "finished_unix_ms": 1100u64,
            "node_counts": {"success": 1, "failed": 1}
        }))
        .expect("manifest json"),
    )
    .expect("write manifest");
    fs::write(
        run.join("snapshot.json"),
        serde_json::to_vec_pretty(&json!({
            "graph": {
                "nodes": [{"id":"a"},{"id":"b"}],
                "edges": [{"from":{"node_id":"a"}, "to":{"node_id":"b"}}]
            }
        }))
        .expect("snapshot json"),
    )
    .expect("write snapshot");
    fs::write(
        run.join("outputs.index.json"),
        serde_json::to_vec_pretty(&json!({"files":[{"path":"x"}]})).expect("outputs index"),
    )
    .expect("write outputs index");
    fs::write(
        run.join("nodes").join("a").join("trace.json"),
        serde_json::to_vec_pretty(&json!({"status":"success","started_unix_ms":1001u64,"finished_unix_ms":1050u64,"attempt":1})).expect("trace a"),
    )
    .expect("write trace a");
    fs::write(
        run.join("nodes").join("b").join("trace.json"),
        serde_json::to_vec_pretty(&json!({"status":"failed","started_unix_ms":1055u64,"finished_unix_ms":1099u64,"attempt":2})).expect("trace b"),
    )
    .expect("write trace b");
    run
}

#[test]
fn operator_summary_and_human_output_are_stable() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let run = write_run_fixture(tmp.path(), "run-1");
    let summary = inspect_summary(&run).expect("summary");
    assert_eq!(summary["retry_count"], 1);
    assert_eq!(summary["artifact_count"], 1);
    let text = format_inspect_human(&summary);
    assert!(text.contains("run_id: \"run-1\""));
    assert!(text.contains("status: \"failed\""));
}

#[test]
fn operator_tree_timeline_and_failure_explain_work_from_explicit_run_dir() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let run = write_run_fixture(tmp.path(), "run-2");
    let tree = run_tree(&run).expect("tree");
    assert_eq!(tree["nodes"].as_array().expect("nodes").len(), 2);
    let timeline = run_timeline(&run).expect("timeline");
    assert_eq!(timeline["events"].as_array().expect("events").len(), 2);
    let explain = explain_failure(&run).expect("explain");
    assert_eq!(explain["root_failure"], "b");
}

#[test]
fn operator_commands_tolerate_partial_corruption() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let run = tmp.path().join("run-bad");
    fs::create_dir_all(&run).expect("mkdir");
    fs::write(run.join("manifest.json"), "{bad-json").expect("write bad manifest");
    let summary = inspect_summary(&run).expect("summary from bad manifest");
    assert_eq!(summary["artifact_count"], 0);
    let doctor = doctor_run(&run);
    assert_eq!(doctor["status"], "corrupt");
}

#[test]
fn run_list_reads_only_explicit_root() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let _ = write_run_fixture(tmp.path(), "run-a");
    let _ = write_run_fixture(tmp.path(), "run-b");
    let listed = list_runs(tmp.path()).expect("list runs");
    assert_eq!(listed, vec!["run-a".to_string(), "run-b".to_string()]);
}
