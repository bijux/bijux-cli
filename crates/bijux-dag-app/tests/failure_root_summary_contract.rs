use base64 as _;
use bijux_dag_app::explain_failure;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json::json;
use sha2 as _;
use std::fs;
use tar as _;
use tempfile as _;
use thiserror as _;

#[test]
fn explain_failure_separates_primary_failure_from_propagated_impact() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let run_dir = tmp.path().join("run-primary-failure");
    fs::create_dir_all(run_dir.join("nodes").join("build")).expect("build dir");
    fs::create_dir_all(run_dir.join("nodes").join("report")).expect("report dir");
    fs::create_dir_all(run_dir.join("nodes").join("publish")).expect("publish dir");
    fs::write(
        run_dir.join("graph.snapshot.json"),
        serde_json::to_vec_pretty(&json!({
            "graph": {
                "spec": "bijux-dag/v0.1",
                "nodes": [
                    {"id":"build","kind":"shell","inputs":[],"outputs":[{"name":"out","path":"out"}],"params":{}},
                    {"id":"report","kind":"shell","inputs":[],"outputs":[{"name":"out","path":"out"}],"params":{}},
                    {"id":"publish","kind":"shell","inputs":[],"outputs":[{"name":"out","path":"out"}],"params":{}}
                ],
                "edges": [
                    {"from":{"node_id":"build","port":"out"},"to":{"node_id":"report","port":"in"}},
                    {"from":{"node_id":"report","port":"out"},"to":{"node_id":"publish","port":"in"}}
                ]
            },
            "graph_fingerprint": "graph-fingerprint"
        }))
        .expect("snapshot"),
    )
    .expect("write snapshot");
    fs::write(
        run_dir.join("nodes").join("build").join("trace.json"),
        serde_json::to_vec_pretty(&json!({
            "node_id":"build",
            "status":"failed",
            "started_unix_ms":10,
            "finished_unix_ms":20,
            "attempt":1,
            "fingerprint":"fp-build",
            "adapter_id":"shell",
            "adapter_version":"1",
            "adapter_outputs_schema_version":"v1",
            "outputs":[],
            "failure":{"kind":"Execution","code":"EXEC_FAIL","message":"compiler exited with status 7"},
            "transition_cause":"ExecutionFailed",
            "lifecycle_transitions":[]
        }))
        .expect("build trace"),
    )
    .expect("write build trace");
    fs::write(
        run_dir.join("nodes").join("report").join("trace.json"),
        serde_json::to_vec_pretty(&json!({
            "node_id":"report",
            "status":"failed",
            "started_unix_ms":21,
            "finished_unix_ms":30,
            "attempt":1,
            "fingerprint":"fp-report",
            "adapter_id":"shell",
            "adapter_version":"1",
            "adapter_outputs_schema_version":"v1",
            "outputs":[],
            "failure":{"kind":"Dependency","code":"UPSTREAM_FAILED","message":"dependency trigger blocked execution for report"},
            "transition_cause":"DependencyFailed",
            "lifecycle_transitions":[]
        }))
        .expect("report trace"),
    )
    .expect("write report trace");
    fs::write(
        run_dir.join("nodes").join("publish").join("trace.json"),
        serde_json::to_vec_pretty(&json!({
            "node_id":"publish",
            "status":"skipped",
            "started_unix_ms":31,
            "finished_unix_ms":31,
            "attempt":1,
            "fingerprint":"fp-publish",
            "adapter_id":"shell",
            "adapter_version":"1",
            "adapter_outputs_schema_version":"v1",
            "outputs":[],
            "skip_reason":{"reason":"upstream_failed"},
            "transition_cause":"DependencyFailed",
            "lifecycle_transitions":[]
        }))
        .expect("publish trace"),
    )
    .expect("write publish trace");

    let report = explain_failure(&run_dir).expect("report");
    assert_eq!(report["roots"], json!(["build:execution_failed"]));
    assert_eq!(report["root_failure"], "build");
    assert_eq!(report["root_failure_class"], "execution");
    assert_eq!(report["root_failure_code"], "EXEC_FAIL");
    assert_eq!(report["root_failure_message"], "compiler exited with status 7");
    assert_eq!(report["root_failure_reason"], "execution_failed");
    assert_eq!(report["failed_nodes"], json!(["build", "report"]));
    assert_eq!(report["propagated_failures"][0]["node_id"], "report");
    assert_eq!(report["propagated_failures"][0]["blocking_nodes"], json!(["build"]));
    assert_eq!(report["propagated_skips"][0]["node_id"], "publish");
    assert_eq!(report["propagated_or_skipped_nodes"], json!(["publish", "report"]));
    assert_eq!(report["downstream_affected_nodes"], json!(["publish", "report"]));
    assert_eq!(report["downstream_affected_groups"]["failed"], json!(["report"]));
    assert_eq!(report["downstream_affected_groups"]["skipped"], json!(["publish"]));
}

#[test]
fn explain_failure_treats_isolated_branch_skips_as_propagated_impact() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let run_dir = tmp.path().join("run-isolated-branch");
    fs::create_dir_all(run_dir.join("nodes").join("fail")).expect("fail dir");
    fs::create_dir_all(run_dir.join("nodes").join("join")).expect("join dir");
    fs::create_dir_all(run_dir.join("nodes").join("publish")).expect("publish dir");
    fs::write(
        run_dir.join("graph.snapshot.json"),
        serde_json::to_vec_pretty(&json!({
            "graph": {
                "spec": "bijux-dag/v0.1",
                "nodes": [
                    {"id":"fail","kind":"shell","inputs":[],"outputs":[{"name":"out","path":"out"}],"params":{}},
                    {"id":"join","kind":"shell","inputs":[],"outputs":[{"name":"out","path":"out"}],"params":{}},
                    {"id":"publish","kind":"shell","inputs":[],"outputs":[{"name":"out","path":"out"}],"params":{}}
                ],
                "edges": [
                    {"from":{"node_id":"fail","port":"out"},"to":{"node_id":"join","port":"in"}},
                    {"from":{"node_id":"join","port":"out"},"to":{"node_id":"publish","port":"in"}}
                ]
            },
            "graph_fingerprint": "graph-fingerprint"
        }))
        .expect("snapshot"),
    )
    .expect("write snapshot");
    fs::write(
        run_dir.join("nodes").join("fail").join("trace.json"),
        serde_json::to_vec_pretty(&json!({
            "node_id":"fail",
            "status":"failed",
            "started_unix_ms":10,
            "finished_unix_ms":20,
            "attempt":1,
            "fingerprint":"fp-fail",
            "adapter_id":"shell",
            "adapter_version":"1",
            "adapter_outputs_schema_version":"v1",
            "outputs":[],
            "failure":{"kind":"Execution","code":"EXEC_FAIL","message":"failed branch exited 1"},
            "transition_cause":"ExecutionFailed",
            "lifecycle_transitions":[]
        }))
        .expect("fail trace"),
    )
    .expect("write fail trace");
    fs::write(
        run_dir.join("nodes").join("join").join("trace.json"),
        serde_json::to_vec_pretty(&json!({
            "node_id":"join",
            "status":"skipped",
            "started_unix_ms":21,
            "finished_unix_ms":21,
            "attempt":1,
            "fingerprint":"fp-join",
            "adapter_id":"shell",
            "adapter_version":"1",
            "adapter_outputs_schema_version":"v1",
            "outputs":[],
            "skip_reason":{"reason":"isolated_branch_failure"},
            "transition_cause":"DependencyFailed",
            "lifecycle_transitions":[]
        }))
        .expect("join trace"),
    )
    .expect("write join trace");
    fs::write(
        run_dir.join("nodes").join("publish").join("trace.json"),
        serde_json::to_vec_pretty(&json!({
            "node_id":"publish",
            "status":"skipped",
            "started_unix_ms":22,
            "finished_unix_ms":22,
            "attempt":1,
            "fingerprint":"fp-publish",
            "adapter_id":"shell",
            "adapter_version":"1",
            "adapter_outputs_schema_version":"v1",
            "outputs":[],
            "skip_reason":{"reason":"isolated_branch_failure"},
            "transition_cause":"DependencyFailed",
            "lifecycle_transitions":[]
        }))
        .expect("publish trace"),
    )
    .expect("write publish trace");

    let report = explain_failure(&run_dir).expect("report");
    assert_eq!(report["roots"], json!(["fail:execution_failed"]));
    assert_eq!(report["propagated_skips"][0]["node_id"], "join");
    assert_eq!(report["propagated_skips"][0]["reason"], "isolated_branch_failure");
    assert_eq!(report["propagated_skips"][0]["blocking_nodes"], json!(["fail"]));
    assert_eq!(report["propagated_skips"][1]["node_id"], "publish");
    assert_eq!(report["propagated_skips"][1]["reason"], "isolated_branch_failure");
    assert_eq!(report["propagated_skips"][1]["blocking_nodes"], json!(["join"]));
    assert_eq!(report["downstream_affected_groups"]["skipped"], json!(["join", "publish"]));
}
