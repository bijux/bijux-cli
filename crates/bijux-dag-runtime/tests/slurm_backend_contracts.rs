use bijux_dag_artifacts as _;
use bijux_dag_core::{parse_graph_strict, Graph};
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json::json;
use sha2::{Digest, Sha256};
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::{
    build_slurm_execution_request, AbsolutePathPolicy, MockSlurmBackend, NodeStatus, PolicyConfig,
    RemoteExecutionFingerprintSet, RemoteExecutionIdentity, RemoteExecutionWorkspace,
    RemoteInputArtifact, RemoteNodeExecutionPayload, SlurmBackendExecutor, SlurmJobStatus,
    SystemSlurmBackend, SystemSlurmBackendConfig,
};
use std::fs;
use std::path::Path;

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn shell_payload(
    backend_id: &str,
    out_base: &Path,
    run_id: &str,
    script: &str,
    timeout_ms: Option<u64>,
) -> RemoteNodeExecutionPayload {
    let graph: Graph = parse_graph_strict(&format!(
        r#"{{
          "spec": "bijux-dag/v0.1",
          "nodes": [
            {{
              "id": "shell-node",
              "kind": "shell",
              "outputs": [{{"name": "result", "path": "result.txt"}}],
              "params": {{"argv": ["/bin/sh", "-c", {script:?}]}},
              "resources": {{"cpu": 4, "mem_mb": 8192}},
              "tags": ["slurm.partition:gpu", "slurm.queue:priority", "slurm.account:research"]
            }}
          ],
          "edges": []
        }}"#
    ))
    .expect("parse graph");
    let mut node = graph.nodes[0].clone();
    node.timeout_ms = timeout_ms;
    RemoteNodeExecutionPayload {
        identity: RemoteExecutionIdentity {
            run_id: run_id.to_string(),
            node_id: node.id.clone(),
            attempt_id: "1".to_string(),
            backend_id: backend_id.to_string(),
        },
        graph,
        node,
        params: json!({"argv": ["/bin/sh", "-c", script]}),
        input_artifacts: vec![RemoteInputArtifact {
            relative_path: "seed/in.txt".to_string(),
            sha256: sha256_hex(b"slurm-input"),
            bytes: b"slurm-input".to_vec(),
        }],
        workspace: RemoteExecutionWorkspace {
            out_base: out_base.display().to_string(),
            cache_dir: None,
        },
        policy: PolicyConfig::default(),
        absolute_path_policy: AbsolutePathPolicy::AllowLiteral,
        planner_contract_version: "bijux-dag-planner/v1".to_string(),
        fingerprints: RemoteExecutionFingerprintSet {
            node_fingerprint: "node".to_string(),
            node_definition_fingerprint: "node-def".to_string(),
            declared_environment_fingerprint: "env".to_string(),
            params_fingerprint: "params".to_string(),
            command_fingerprint: Some("command".to_string()),
            execution_fingerprint: "execution".to_string(),
            evidence_fingerprint: "evidence".to_string(),
            execution_contract_fingerprint: "contract".to_string(),
        },
    }
}

#[test]
fn mock_slurm_backend_executes_shell_payload_and_captures_logs() {
    let temp = tempfile::tempdir().expect("temp dir");
    let payload = shell_payload(
        "slurm",
        temp.path(),
        "slurm-shell-success",
        "cat ../inputs/seed/in.txt > ../outputs/result.txt && echo slurm-stdout && echo slurm-stderr 1>&2",
        Some(120_000),
    );
    let request = build_slurm_execution_request(payload, "general", "cpu");
    let backend = MockSlurmBackend::default();

    let result = backend.execute_job(request).expect("slurm execute");

    assert_eq!(result.scheduler_status, SlurmJobStatus::Completed);
    assert_eq!(result.node_status, NodeStatus::Success);
    assert_eq!(result.job.lifecycle.len(), 3);
    assert_eq!(result.job.lifecycle[0].status, SlurmJobStatus::Submitted);
    assert_eq!(result.job.lifecycle[1].status, SlurmJobStatus::Running);
    assert_eq!(result.job.lifecycle[2].status, SlurmJobStatus::Completed);
    assert!(result.job.job_id.starts_with("slurm-"));
    assert!(result.job.metadata.resource_request.contains("queue=priority"));
    assert!(result.job.metadata.resource_request.contains("partition=gpu"));
    assert!(result.job.metadata.resource_request.contains("account=research"));
    assert!(result.logs.stdout.contains("slurm-stdout"));
    assert!(result.logs.stderr.contains("slurm-stderr"));

    let rendered =
        fs::read_to_string(Path::new(&result.node_result.outputs_dir).join("result.txt"))
            .expect("rendered output");
    assert_eq!(rendered, "slurm-input");

    let persisted = backend.job_record(&result.job.job_id).expect("job record");
    assert_eq!(persisted.terminal_status, SlurmJobStatus::Completed);
    assert_eq!(backend.submitted_requests().len(), 1);
}

#[test]
fn mock_slurm_backend_marks_failed_shell_payload_as_failed_job() {
    let temp = tempfile::tempdir().expect("temp dir");
    let payload = shell_payload(
        "hpc",
        temp.path(),
        "slurm-shell-failure",
        "echo broken 1>&2; exit 17",
        Some(120_000),
    );
    let request = build_slurm_execution_request(payload, "general", "cpu");
    let backend = MockSlurmBackend::default();

    let result = backend.execute_job(request).expect("slurm execute");

    assert_eq!(result.scheduler_status, SlurmJobStatus::Failed);
    assert_eq!(result.node_status, NodeStatus::Failed);
    assert_eq!(result.node_result.status, NodeStatus::Failed);
    assert!(result.logs.stderr.contains("broken"));
    assert_eq!(result.job.terminal_status, SlurmJobStatus::Failed);
}

fn write_executable(path: &Path, script: &str) {
    fs::write(path, script).expect("write script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path).expect("stat script").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("chmod script");
    }
}

#[test]
fn system_slurm_backend_submits_polls_and_collects_worker_result() {
    let temp = tempfile::tempdir().expect("temp dir");
    let tool_dir = temp.path().join("bin");
    let state_dir = temp.path().join("state");
    let log_dir = temp.path().join("logs");
    fs::create_dir_all(&tool_dir).expect("tool dir");
    fs::create_dir_all(&state_dir).expect("state dir");
    fs::create_dir_all(&log_dir).expect("log dir");

    let stdout_log = log_dir.join("node.stdout.log");
    let stderr_log = log_dir.join("node.stderr.log");
    let outputs_dir = temp.path().join("outputs");
    fs::create_dir_all(&outputs_dir).expect("outputs dir");

    let worker = tool_dir.join("worker");
    write_executable(
        &worker,
        &format!(
            "#!/bin/sh\nset -eu\nPAYLOAD=\"$1\"\nshift\nRESULT=\"\"\nwhile [ $# -gt 0 ]; do\n  case \"$1\" in\n    --result) RESULT=\"$2\"; shift 2 ;;\n    --in-place) shift ;;\n    *) shift ;;\n  esac\ndone\nmkdir -p \"$(dirname \"$RESULT\")\"\nprintf 'worker-stdout\\n' > {stdout:?}\nprintf 'worker-stderr\\n' > {stderr:?}\nprintf 'done\\n' > {outputs:?}/result.txt\ncat > \"$RESULT\" <<'JSON'\n{{\n  \"identity\": {{\"run_id\":\"run-system\",\"node_id\":\"shell-node\",\"attempt_id\":\"1\",\"backend_id\":\"slurm\"}},\n  \"node_result\": {{\n    \"status\":\"Success\",\n    \"stdout_path\": {stdout_json:?},\n    \"stderr_path\": {stderr_json:?},\n    \"outputs_dir\": {outputs_json:?},\n    \"output_evidence\": [],\n    \"failure\": null,\n    \"attempts\": 1,\n    \"attempt_events\": [],\n    \"container_meta\": null,\n    \"adapter_binary_sha256\": null\n  }},\n  \"started_unix_ms\": 10,\n  \"finished_unix_ms\": 20\n}}\nJSON\n",
            stdout = stdout_log.display().to_string(),
            stderr = stderr_log.display().to_string(),
            outputs = outputs_dir.display().to_string(),
            stdout_json = stdout_log.display().to_string(),
            stderr_json = stderr_log.display().to_string(),
            outputs_json = outputs_dir.display().to_string(),
        ),
    );

    let sbatch = tool_dir.join("sbatch");
    write_executable(
        &sbatch,
        &format!(
            "#!/bin/sh\nset -eu\nSTATE_DIR={state:?}\nOUT=\"\"\nERR=\"\"\nSCRIPT=\"\"\nwhile [ $# -gt 0 ]; do\n  case \"$1\" in\n    --parsable) shift ;;\n    --cpus-per-task|--mem|--time|--partition|--qos|--account|--output|--error)\n      if [ \"$1\" = \"--output\" ]; then OUT=\"$2\"; fi\n      if [ \"$1\" = \"--error\" ]; then ERR=\"$2\"; fi\n      shift 2 ;;\n    *) SCRIPT=\"$1\"; shift ;;\n  esac\ndone\nmkdir -p \"$STATE_DIR\" \"$(dirname \"$OUT\")\" \"$(dirname \"$ERR\")\"\nif sh \"$SCRIPT\" > \"$OUT\" 2> \"$ERR\"; then\n  printf 'COMPLETED' > \"$STATE_DIR/job-1.state\"\nelse\n  printf 'FAILED' > \"$STATE_DIR/job-1.state\"\nfi\nprintf 'job-1\\n'\n",
            state = state_dir.display().to_string(),
        ),
    );

    let sacct = tool_dir.join("sacct");
    write_executable(
        &sacct,
        &format!(
            "#!/bin/sh\nset -eu\nSTATE_DIR={state:?}\nSTATE=$(cat \"$STATE_DIR/job-1.state\")\nprintf '%s|0:0\\n' \"$STATE\"\n",
            state = state_dir.display().to_string(),
        ),
    );

    let payload = shell_payload("slurm", temp.path(), "run-system", "printf ignored", Some(30_000));
    let request = build_slurm_execution_request(payload, "general", "cpu");
    let backend = SystemSlurmBackend::new(SystemSlurmBackendConfig {
        sbatch_command: sbatch.display().to_string(),
        sacct_command: sacct.display().to_string(),
        poll_interval_ms: 50,
        worker_command: vec![worker.display().to_string()],
    })
    .expect("system backend");

    let result = backend.execute_job(request).expect("system slurm execute");

    assert_eq!(result.scheduler_status, SlurmJobStatus::Completed);
    assert_eq!(result.node_status, NodeStatus::Success);
    assert_eq!(result.job.job_id, "job-1");
    assert!(result.logs.stdout.contains("worker-stdout"));
    assert!(result.logs.stderr.contains("worker-stderr"));
    assert_eq!(
        fs::read_to_string(outputs_dir.join("result.txt")).expect("result output"),
        "done\n"
    );
}
