use base64 as _;
use bijux_dag_app::{dag_command, dag_run};
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
fn why_rerun_and_trace_artifact_commands_are_json_capable() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    let run_a = root.join("run-a");
    let run_b = root.join("run-b");
    for run in [&run_a, &run_b] {
        fs::create_dir_all(run.join("nodes/extract/outputs")).expect("mkdir");
        fs::create_dir_all(run.join("outputs")).expect("mkdir");
        fs::write(run.join("nodes/extract/outputs/data.txt"), b"x").expect("payload");
        fs::write(
            run.join("manifest.json"),
            serde_json::to_vec_pretty(&json!({
                "manifest_version":"run-manifest/v0.1",
                "run_id": run.file_name().unwrap().to_string_lossy(),
                "created_unix_ms":1,"started_unix_ms":1,"finished_unix_ms":2,
                "graph_snapshot":"graph.snapshot.json","status":"success","spec":"bijux-dag/v0.1",
                "graph_fingerprint":"g1","tool_version":"0.1.0","jobs":1,
                "adapters":[],"outputs":[],"node_counts":{"success":1,"failed":0,"skipped":0,"cached":0},
                "policy":{"deny_network":true,"deny_env":true,"deny_clock":true,"clean_env":true}
            }))
            .expect("manifest"),
        )
        .expect("write manifest");
        fs::write(
            run.join("graph.snapshot.json"),
            serde_json::to_vec_pretty(&json!({"graph_fingerprint":"g1"})).expect("snap"),
        )
        .expect("write snap");
        fs::write(
            run.join("outputs/index.json"),
            serde_json::to_vec_pretty(&json!({"files":[{"node_id":"extract","node_fingerprint":"fp1","name":"out","kind":"file","media_type":"text/plain","size_bytes":5,"sha256":"2d711642b726b04401627ca9fbac32f5c8530fb1903cc4db02258717921a4881","path":"nodes/extract/outputs/data.txt"}]}))
                .expect("index"),
        )
        .expect("write index");
    }

    let why = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "why-rerun",
            run_a.to_string_lossy().as_ref(),
            run_b.to_string_lossy().as_ref(),
        ])
        .expect("parse why-rerun");
    assert_eq!(dag_run(&why).expect("why-rerun"), std::process::ExitCode::SUCCESS);

    let trace = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "trace-artifact",
            run_a.to_string_lossy().as_ref(),
            "extract:data.txt",
        ])
        .expect("parse trace-artifact");
    assert_eq!(dag_run(&trace).expect("trace-artifact"), std::process::ExitCode::SUCCESS);
}

#[test]
fn why_cache_missed_command_is_json_capable() {
    let tmp = tempfile::tempdir().expect("tmp");
    let cache_dir = tmp.path().join("cache");
    fs::create_dir_all(&cache_dir).expect("mkdir cache");
    let cmd = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "why-cache-missed",
            "missing-key",
            "--expected-adapter-id",
            "shell",
            "--expected-adapter-version",
            "1.0.0",
            "--cache-dir",
            cache_dir.to_string_lossy().as_ref(),
        ])
        .expect("parse why-cache-missed");
    assert_eq!(dag_run(&cmd).expect("why-cache-missed"), std::process::ExitCode::SUCCESS);
}

#[test]
fn why_cache_missed_run_node_mode_is_json_capable() {
    let tmp = tempfile::tempdir().expect("tmp");
    let cache_dir = tmp.path().join("cache");
    let run_dir = tmp.path().join("run");
    fs::create_dir_all(run_dir.join("nodes/node")).expect("mkdir node");
    fs::write(
        run_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "manifest_version":"run-manifest/v0.1",
            "run_id":"run-1",
            "created_unix_ms":1,"started_unix_ms":1,"finished_unix_ms":2,
            "graph_snapshot":"graph.snapshot.json","status":"success","spec":"bijux-dag/v0.1",
            "graph_fingerprint":"g1","tool_version":"0.1.0","jobs":1,
            "adapters":[],"outputs":[],"node_counts":{"success":1,"failed":0,"skipped":0,"cached":0},
            "policy":{"deny_network":true,"deny_env":true,"deny_clock":true,"clean_env":true},
            "cache_mode":"Off",
            "cache_dir": cache_dir.display().to_string()
        }))
        .expect("manifest"),
    )
    .expect("write manifest");
    fs::write(
        run_dir.join("graph.snapshot.json"),
        serde_json::to_vec_pretty(&json!({
            "graph":{
                "spec":"bijux-dag/v0.1",
                "meta":{"name":"x","owners":[],"tags":[]},
                "inputs":{},
                "nodes":[{
                    "id":"node",
                    "kind":"shell",
                    "inputs":[],
                    "outputs":[{"name":"value","path":"value.txt"}],
                    "params":{"argv":["/bin/sh","-c","printf '%s' ok > ../outputs/value.txt"]},
                    "cache":{"enabled":true},
                    "effects":["filesystem"]
                }],
                "edges":[]
            },
            "graph_fingerprint":"g1"
        }))
        .expect("snapshot"),
    )
    .expect("write snapshot");
    fs::write(
        run_dir.join("nodes/node/trace.json"),
        serde_json::to_vec_pretty(&json!({
            "node_id":"node",
            "status":"success",
            "started_unix_ms":1,
            "finished_unix_ms":2,
            "attempt":1,
            "fingerprint":"exec-current",
            "adapter_id":"shell",
            "adapter_version":"1.0.0",
            "adapter_outputs_schema_version":"schema/v1",
            "cache_identity":{
                "cache_key":"cache-key",
                "node_definition_fingerprint":"node-current",
                "declared_environment_fingerprint":"env-current",
                "input_lineage_fingerprint":"inputs-current",
                "params_fingerprint":"params-current",
                "command_fingerprint":"command-current",
                "policy_fingerprint":"policy-current",
                "execution_contract_fingerprint":"exec-contract-current",
                "backend_class":"local"
            }
        }))
        .expect("trace"),
    )
    .expect("write trace");

    let cmd = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "why-cache-missed",
            "--run-dir",
            run_dir.to_string_lossy().as_ref(),
            "--node",
            "node",
        ])
        .expect("parse why-cache-missed run node mode");
    assert_eq!(dag_run(&cmd).expect("why-cache-missed"), std::process::ExitCode::SUCCESS);
}

#[test]
fn why_rerun_reports_equivalence_for_identical_runs() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    let run_a = root.join("run-a");
    let run_b = root.join("run-b");
    for run in [&run_a, &run_b] {
        fs::create_dir_all(run.join("outputs")).expect("mkdir");
        fs::write(
            run.join("manifest.json"),
            serde_json::to_vec_pretty(&json!({
                "status":"success","spec":"bijux-dag/v0.1","graph_fingerprint":"g"
            }))
            .expect("manifest"),
        )
        .expect("write");
        fs::write(
            run.join("graph.snapshot.json"),
            serde_json::to_vec_pretty(&json!({"graph_fingerprint":"g"})).expect("snap"),
        )
        .expect("write");
    }
    let cmd = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "diff",
            "--mode",
            "summary",
            "--node",
            "extract",
            run_a.to_string_lossy().as_ref(),
            run_b.to_string_lossy().as_ref(),
        ])
        .expect("parse");
    assert_eq!(dag_run(&cmd).expect("run"), std::process::ExitCode::SUCCESS);

    let why = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "why-rerun",
            "--node",
            "extract",
            run_a.to_string_lossy().as_ref(),
            run_b.to_string_lossy().as_ref(),
        ])
        .expect("parse");
    assert_eq!(dag_run(&why).expect("run"), std::process::ExitCode::SUCCESS);
}
