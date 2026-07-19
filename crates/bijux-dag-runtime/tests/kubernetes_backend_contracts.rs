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
    build_kubernetes_execution_request, AbsolutePathPolicy, KubernetesBackendExecutor,
    KubernetesPodPhase, MockKubernetesBackend, NodeStatus, PolicyConfig,
    RemoteExecutionFingerprintSet, RemoteExecutionIdentity, RemoteExecutionWorkspace,
    RemoteInputArtifact, RemoteNodeExecutionPayload,
};
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn process_env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|error| error.into_inner())
}

fn write_executable(path: &Path, contents: &str) {
    fs::write(path, contents).expect("write executable");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path).expect("meta").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).expect("chmod");
    }
}

struct PathGuard(Option<OsString>);

impl Drop for PathGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.0.take() {
            std::env::set_var("PATH", previous);
        } else {
            std::env::remove_var("PATH");
        }
    }
}

fn prepend_path(dir: &Path) -> PathGuard {
    let previous = std::env::var_os("PATH");
    let mut entries = vec![dir.display().to_string()];
    if let Some(value) = &previous {
        entries.push(value.to_string_lossy().to_string());
    }
    std::env::set_var("PATH", entries.join(":"));
    PathGuard(previous)
}

fn shell_payload(
    backend_id: &str,
    out_base: &Path,
    run_id: &str,
    script: &str,
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
              "resources": {{"cpu": 2, "mem_mb": 1024}}
            }}
          ],
          "edges": []
        }}"#
    ))
    .expect("parse graph");
    let node = graph.nodes[0].clone();
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
            sha256: sha256_hex(b"k8s-input"),
            bytes: b"k8s-input".to_vec(),
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

fn container_payload(
    backend_id: &str,
    out_base: &Path,
    run_id: &str,
) -> RemoteNodeExecutionPayload {
    let graph: Graph = parse_graph_strict(
        r#"{
          "spec": "bijux-dag/v0.1",
          "nodes": [
            {
              "id": "container-node",
              "kind": "container",
              "inputs": ["seed"],
              "outputs": [
                {"name": "result", "path": "result.txt"},
                {"name": "network", "path": "network.txt"},
                {"name": "workdir", "path": "workdir.txt"}
              ],
              "effects": ["filesystem"],
              "resources": {"cpu": 2, "mem_mb": 1024},
              "container": {
                "image": "example.local/runner@sha256:feedface",
                "argv": ["/bin/sh", "-c", "cat /bijux/node/inputs/seed/in.txt > /bijux/node/outputs/result.txt"],
                "workdir": "{work_dir}/scratch",
                "engine": "docker"
              },
              "params": {}
            }
          ],
          "edges": []
        }"#,
    )
    .expect("parse graph");
    let node = graph.nodes[0].clone();
    RemoteNodeExecutionPayload {
        identity: RemoteExecutionIdentity {
            run_id: run_id.to_string(),
            node_id: node.id.clone(),
            attempt_id: "1".to_string(),
            backend_id: backend_id.to_string(),
        },
        graph,
        node,
        params: json!({}),
        input_artifacts: vec![RemoteInputArtifact {
            relative_path: "seed/in.txt".to_string(),
            sha256: sha256_hex(b"k8s-container-input"),
            bytes: b"k8s-container-input".to_vec(),
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
fn mock_kubernetes_backend_executes_container_payload_and_records_workspace_contract() {
    let _lock = process_env_lock();
    let temp = tempfile::tempdir().expect("temp dir");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&bin_dir).expect("bin dir");
    let docker = bin_dir.join("docker");
    write_executable(
        &docker,
        r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "docker fake 1.0"
  exit 0
fi
if [ "$1" = "image" ] && [ "$2" = "inspect" ]; then
  echo "sha256:feedface"
  exit 0
fi
if [ "$1" = "run" ]; then
  shift
  inputs_dir=""
  outputs_dir=""
  workdir=""
  network_mode="default"
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --network)
        network_mode="$2"
        shift 2
        ;;
      --workdir)
        workdir="$2"
        shift 2
        ;;
      -v)
        mount="$2"
        host_path=$(printf '%s' "$mount" | cut -d: -f1)
        container_path=$(printf '%s' "$mount" | cut -d: -f2)
        if [ "$container_path" = "/bijux/node/inputs" ]; then
          inputs_dir="$host_path"
        elif [ "$container_path" = "/bijux/node/outputs" ]; then
          outputs_dir="$host_path"
        fi
        shift 2
        ;;
      -e)
        shift 2
        ;;
      --rm)
        shift
        ;;
      -*)
        shift
        ;;
      *)
        break
        ;;
    esac
  done
  cat "$inputs_dir/seed/in.txt" > "$outputs_dir/result.txt"
  printf '%s' "$network_mode" > "$outputs_dir/network.txt"
  printf '%s' "$workdir" > "$outputs_dir/workdir.txt"
  printf 'k8s-container-stdout'
  printf 'k8s-container-stderr' >&2
  exit 0
fi
exit 1
"#,
    );
    let _path_guard = prepend_path(&bin_dir);

    let payload = container_payload("kubernetes", temp.path(), "k8s-container-success");
    let request = build_kubernetes_execution_request(payload, "bijux-jobs");
    let backend = MockKubernetesBackend::default();

    let result = backend.execute_job(request).expect("kubernetes execute");

    assert_eq!(result.pod_status.phase, KubernetesPodPhase::Succeeded);
    assert_eq!(result.node_status, NodeStatus::Success);
    assert_eq!(result.job.lifecycle.len(), 3);
    assert_eq!(result.job.lifecycle[0].status.phase, KubernetesPodPhase::Pending);
    assert_eq!(result.job.lifecycle[1].status.phase, KubernetesPodPhase::Running);
    assert_eq!(result.job.lifecycle[2].status.phase, KubernetesPodPhase::Succeeded);
    assert!(result.job.job_id.starts_with("k8s-"));
    assert_eq!(
        fs::read_to_string(Path::new(&result.node_result.outputs_dir).join("result.txt"))
            .expect("rendered output"),
        "k8s-container-input"
    );
    assert_eq!(
        fs::read_to_string(Path::new(&result.node_result.outputs_dir).join("network.txt"))
            .expect("network mode"),
        "none"
    );
    assert_eq!(
        fs::read_to_string(Path::new(&result.node_result.outputs_dir).join("workdir.txt"))
            .expect("workdir"),
        "/bijux/node/work/scratch"
    );
    assert!(result.logs.stdout.contains("k8s-container-stdout"));
    assert!(result.logs.stderr.contains("k8s-container-stderr"));
    assert_eq!(result.job.workspace.input_artifact_count, 1);
    assert_eq!(result.job.workspace.declared_output_count, 3);
    assert!(result.job.metadata.resource_request.contains("transfer_mode=mounted_workdir"));
    assert_eq!(backend.submitted_requests().len(), 1);
    assert_eq!(
        backend.job_record(&result.job.job_id).expect("job record").terminal_status.phase,
        KubernetesPodPhase::Succeeded
    );
}

#[test]
fn mock_kubernetes_backend_maps_failed_shell_payload_into_failed_pod_status() {
    let temp = tempfile::tempdir().expect("temp dir");
    let payload =
        shell_payload("k8s", temp.path(), "k8s-shell-failure", "echo broken 1>&2; exit 17");
    let request = build_kubernetes_execution_request(payload, "bijux-jobs");
    let backend = MockKubernetesBackend::default();

    let result = backend.execute_job(request).expect("kubernetes execute");

    assert_eq!(result.pod_status.phase, KubernetesPodPhase::Failed);
    assert_eq!(result.node_status, NodeStatus::Failed);
    assert_eq!(result.node_result.status, NodeStatus::Failed);
    assert_eq!(result.pod_status.reason.as_deref(), Some("Error"));
    assert!(result.logs.stderr.contains("broken"));
    assert_eq!(result.job.terminal_status.phase, KubernetesPodPhase::Failed);
    assert!(result.job.metadata.resource_request.contains("transfer_mode=staged_artifacts"));
}
