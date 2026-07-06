use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_core::{
    parse_graph_strict, Effect, FileOutput, Graph, Node, NodeKind, ParamValue, RetryPolicy,
    SemanticNodeKind, TriggerRule,
};
use bijux_dag_runtime::{
    adapter_admission_matrix, adapter_conformance_suite, container_engine_discovery,
    container_gpu_runtime_args, container_network_policy_args, container_volume_contract,
    probe_external_adapters, registered_adapter_descriptors, validate_container_mount_contract,
    validate_output_schema_compatibility, Runtime, RuntimeConfig,
};
use serde_json::Value;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

fn process_env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|error| error.into_inner())
}

fn shell_graph(command: &str, effects: &[&str]) -> String {
    let effects =
        effects.iter().map(|effect| format!("\"{effect}\"")).collect::<Vec<_>>().join(",");
    let env_allowlist =
        if effects.contains("env") { ",\n              \"env_allowlist\":[\"PATH\"]" } else { "" };
    format!(
        r#"{{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {{
              "id":"shell",
              "kind":"shell",
              "outputs":[{{"name":"value","path":"value.txt"}}],
              "params":{{"argv":["/bin/sh","-c","{command}"]}},
              "effects":[{effects}]{env_allowlist}
            }}
          ],
          "edges":[]
        }}"#
    )
}

fn read_trace(run_dir: &std::path::Path) -> Value {
    serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join("shell").join("trace.json")).expect("trace"),
    )
    .expect("trace json")
}

fn read_node_trace(run_dir: &Path, node_id: &str) -> Value {
    serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join(node_id).join("trace.json")).expect("trace"),
    )
    .expect("trace json")
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

fn container_graph(
    effects: &[&str],
    timeout_ms: Option<u64>,
    image: &str,
    container_command: &str,
) -> Graph {
    let effects =
        effects.iter().map(|effect| format!("\"{effect}\"")).collect::<Vec<_>>().join(",");
    let timeout = timeout_ms.map(|value| format!(",\"timeout_ms\":{value}")).unwrap_or_default();
    parse_graph_strict(&format!(
        r#"{{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {{
              "id":"upstream",
              "kind":"shell",
              "outputs":[{{"name":"seed","path":"seed.txt"}}],
              "params":{{"argv":["/bin/sh","-c","printf 'seed-data' > ../outputs/seed.txt"]}},
              "effects":["filesystem"]
            }},
            {{
              "id":"container",
              "kind":"container",
              "inputs":["seed"],
              "outputs":[
                {{"name":"result","path":"result.txt"}},
                {{"name":"network","path":"network.txt"}},
                {{"name":"workdir","path":"workdir.txt"}}
              ],
              "effects":[{effects}],
              "container":{{
                "image":"{image}",
                "argv":["/bin/sh","-c","{container_command}"],
                "workdir":"{{work_dir}}/scratch",
                "engine":"docker"
              }}{timeout}
            }}
          ],
          "edges":[
            {{"from":{{"node_id":"upstream","port":"seed"}},"to":{{"node_id":"container","port":"seed"}}}}
          ]
        }}"#
    ))
    .expect("graph")
}

fn external_graph(kind: &str, timeout_ms: Option<u64>) -> Graph {
    Graph {
        spec: "bijux-dag/v0.1".to_string(),
        meta: None,
        inputs: std::collections::BTreeMap::new(),
        nondeterminism_allowed: false,
        subgraphs: std::collections::BTreeMap::new(),
        subgraph_instances: Vec::new(),
        nodes: vec![Node {
            id: "n1".to_string(),
            kind: NodeKind::External(kind.to_string()),
            semantic_kind: SemanticNodeKind::Task,
            inputs: vec![],
            outputs: vec![FileOutput::new("out".to_string(), "out".to_string())],
            params: ParamValue::default(),
            container: None,
            timeout_ms,
            resources: None,
            tags: vec![],
            retry: RetryPolicy::default(),
            cache: Default::default(),
            effects: vec![Effect::Filesystem],
            env_allowlist: vec![],
            group: None,
            trigger_rule: TriggerRule::AllSuccess,
            branch: None,
        }],
        edges: vec![],
    }
}

#[test]
fn adapter_descriptors_expose_timeout_cache_and_protocol_contracts() {
    let descriptors = registered_adapter_descriptors();
    assert!(!descriptors.is_empty());
    let shell =
        descriptors.iter().find(|descriptor| descriptor.id == "shell").expect("shell descriptor");
    assert_eq!(shell.protocol_version, "bijux-dag-adapter/v1");
    assert!(shell.supports_timeout);
    assert!(!shell.supports_cancel);
    assert_eq!(
        shell.cache_compatibility,
        bijux_dag_runtime::CacheCompatibilityMode::FingerprintExact
    );
}

#[test]
fn adapter_admission_matrix_reports_unsupported_node_kinds() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[{"id":"x","kind":"missing.kind","outputs":[{"name":"out","path":"out"}],"params":{}}],
          "edges":[]
        }"#,
    )
    .expect("graph");
    let report = adapter_admission_matrix(&graph);
    assert!(!report.supported);
    assert_eq!(report.entries.len(), 1);
    assert!(report.entries[0].reasons[0].contains("no registered adapter"));
}

#[test]
fn shell_adapter_writes_declared_output_and_captures_streams() {
    let graph = parse_graph_strict(&shell_graph(
        "printf 'hello' > ../outputs/value.txt; printf 'ok' >&1; printf 'warn' >&2",
        &["filesystem"],
    ))
    .expect("graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("tmpdir");
    let run_dir = runtime.run(&graph, temp.path(), RuntimeConfig::default()).expect("run");
    assert_eq!(
        fs::read_to_string(run_dir.join("nodes").join("shell").join("outputs").join("value.txt"))
            .expect("output"),
        "hello"
    );
    assert_eq!(
        fs::read_to_string(run_dir.join("nodes").join("shell").join("stdout.log")).expect("stdout"),
        "ok"
    );
    assert_eq!(
        fs::read_to_string(run_dir.join("nodes").join("shell").join("stderr.log")).expect("stderr"),
        "warn"
    );
}

#[test]
fn shell_adapter_missing_declared_output_fails_contract() {
    let graph =
        parse_graph_strict(&shell_graph("printf 'no-output'", &["filesystem"])).expect("graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("tmpdir");
    let run_dir = runtime.run(&graph, temp.path(), RuntimeConfig::default()).expect("run");
    let trace = read_trace(&run_dir);
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["failure"]["code"], "OUTPUT_MISSING");
    assert_eq!(trace["failure"]["class"], "user");
}

#[test]
fn shell_adapter_failure_records_exit_code() {
    let graph = parse_graph_strict(&shell_graph(
        "printf 'partial' > ../outputs/value.txt; printf 'boom' >&2; exit 7",
        &["filesystem"],
    ))
    .expect("graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("tmpdir");
    let run_dir = runtime.run(&graph, temp.path(), RuntimeConfig::default()).expect("run");
    let trace = read_trace(&run_dir);
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["failure"]["code"], "EXEC_FAIL");
    assert_eq!(trace["failure"]["class"], "execution");
    assert_eq!(trace["failure"]["details"]["exit_code"], 7);
}

#[test]
fn shell_adapter_env_policy_denial_is_structured() {
    let graph = parse_graph_strict(&shell_graph("env", &["filesystem", "env"])).expect("graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("tmpdir");
    let run_dir = runtime
        .run(
            &graph,
            temp.path(),
            RuntimeConfig {
                policy: bijux_dag_runtime::PolicyConfig {
                    deny_env: true,
                    ..bijux_dag_runtime::PolicyConfig::default()
                },
                ..RuntimeConfig::default()
            },
        )
        .expect("run");
    let trace = read_trace(&run_dir);
    assert_eq!(trace["failure"]["kind"], "Policy");
    assert_eq!(trace["failure"]["code"], "POLICY_DENIED");
    assert_eq!(trace["failure"]["class"], "policy");
}

#[test]
fn shell_adapter_missing_executable_is_infrastructure_error() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"shell",
              "kind":"shell",
              "outputs":[{"name":"value","path":"value.txt"}],
              "params":{"argv":["definitely-missing-bijux-command"]},
              "effects":["filesystem"]
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("tmpdir");
    let run_dir = runtime.run(&graph, temp.path(), RuntimeConfig::default()).expect("run");
    let trace = read_trace(&run_dir);
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["failure"]["code"], "MISSING_EXECUTABLE");
    assert_eq!(trace["failure"]["class"], "infrastructure");
}

#[test]
fn container_volume_contract_is_isolated_and_validated() {
    let dir = tempfile::tempdir().expect("tmpdir");
    let mounts = container_volume_contract(dir.path());
    assert_eq!(mounts.len(), 3);
    assert!(mounts
        .iter()
        .any(|mount| mount.container_path == "/bijux/node/inputs" && mount.readonly));
    assert!(mounts
        .iter()
        .any(|mount| mount.container_path == "/bijux/node/outputs" && !mount.readonly));
    assert!(mounts
        .iter()
        .any(|mount| mount.container_path == "/bijux/node/work" && !mount.readonly));
    validate_container_mount_contract(&mounts, dir.path()).expect("valid mount contract");
}

#[test]
fn container_network_policy_rejects_unknown_engine_when_isolation_is_required() {
    let error = container_network_policy_args("custom-engine", true).expect_err("must reject");
    assert!(error.contains("cannot enforce deny_network"));
}

#[test]
fn container_engine_discovery_reports_unavailable_engine_structurally() {
    let error = container_engine_discovery("definitely-missing-engine").expect_err("missing");
    assert!(error.contains("container engine unavailable"));
}

#[test]
fn container_gpu_runtime_args_reject_unknown_engine() {
    let error = container_gpu_runtime_args("custom-engine", 1).expect_err("must reject");
    assert!(error.contains("cannot request gpu devices"));
}

#[test]
fn container_adapter_passes_gpu_flags_to_supported_engine() {
    let _lock = process_env_lock();
    let dir = tempfile::tempdir().expect("tmpdir");
    let bin_dir = dir.path().join("bin");
    fs::create_dir_all(&bin_dir).expect("bin dir");
    let docker = bin_dir.join("docker");
    fs::write(
        &docker,
        r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "docker fake"
  exit 0
fi
if [ "$1" = "run" ]; then
  outputs_dir=""
  gpu_arg=""
  while [ $# -gt 0 ]; do
    case "$1" in
      --gpus=*)
        gpu_arg="$1"
        shift
        ;;
      -v)
        mount="$2"
        host_path=$(printf '%s' "$mount" | cut -d: -f1)
        container_path=$(printf '%s' "$mount" | cut -d: -f2)
        if [ "$container_path" = "/bijux/node/outputs" ]; then
          outputs_dir="$host_path"
        fi
        shift 2
        ;;
      *)
        shift
        ;;
    esac
  done
  printf '%s' "$gpu_arg" > "$outputs_dir/gpu-args.txt"
  printf 'ok' > "$outputs_dir/result.txt"
  exit 0
fi
exit 1
"#,
    )
    .expect("docker shim");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&docker).expect("docker meta").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&docker, perms).expect("docker chmod");
    }

    let path_backup = std::env::var_os("PATH");
    let mut path_entries = vec![bin_dir.display().to_string()];
    if let Some(path_backup) = &path_backup {
        path_entries.push(path_backup.to_string_lossy().to_string());
    }
    std::env::set_var("PATH", path_entries.join(":"));

    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"container",
              "kind":"container",
              "outputs":[
                {"name":"result","path":"result.txt"},
                {"name":"gpu_args","path":"gpu-args.txt"}
              ],
              "resources":{"cpu":1,"mem_mb":64,"gpu_devices":1},
              "effects":["filesystem"],
              "container":{
                "image":"example.local/runner@sha256:feedface",
                "argv":["/bin/true"],
                "engine":"docker"
              },
              "params":{}
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("graph");

    let runtime = Runtime::new();
    let run_dir = runtime
        .run(
            &graph,
            dir.path(),
            RuntimeConfig { gpu_device_budget: Some(1), ..RuntimeConfig::default() },
        )
        .expect("container run");

    if let Some(path_backup) = path_backup {
        std::env::set_var("PATH", path_backup);
    } else {
        std::env::remove_var("PATH");
    }

    let gpu_args = fs::read_to_string(
        run_dir.join("nodes").join("container").join("outputs").join("gpu-args.txt"),
    )
    .expect("gpu args");
    assert_eq!(gpu_args, "--gpus=1");
}

#[test]
fn container_adapter_materializes_inputs_collects_outputs_and_records_identity() {
    let _lock = process_env_lock();
    let dir = tempfile::tempdir().expect("tmpdir");
    let bin_dir = dir.path().join("bin");
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
  cat "$inputs_dir/upstream/seed" > "$outputs_dir/result.txt"
  printf '%s' "$network_mode" > "$outputs_dir/network.txt"
  printf '%s' "$workdir" > "$outputs_dir/workdir.txt"
  printf 'container-stdout'
  printf 'container-stderr' >&2
  exit 0
fi
exit 1
"#,
    );
    let _path_guard = prepend_path(&bin_dir);

    let graph = container_graph(
        &["filesystem"],
        None,
        "example.local/runner@sha256:feedface",
        "cat /bijux/node/inputs/upstream/seed > /bijux/node/outputs/result.txt",
    );
    let runtime = Runtime::new();
    let run_dir = runtime.run(&graph, dir.path(), RuntimeConfig::default()).expect("run");

    assert_eq!(
        fs::read_to_string(
            run_dir.join("nodes").join("container").join("outputs").join("result.txt")
        )
        .expect("result"),
        "seed-data"
    );
    assert_eq!(
        fs::read_to_string(
            run_dir.join("nodes").join("container").join("outputs").join("network.txt")
        )
        .expect("network mode"),
        "none"
    );
    assert_eq!(
        fs::read_to_string(
            run_dir.join("nodes").join("container").join("outputs").join("workdir.txt")
        )
        .expect("workdir"),
        "/bijux/node/work/scratch"
    );
    assert_eq!(
        fs::read_to_string(run_dir.join("nodes").join("container").join("stdout.log"))
            .expect("stdout"),
        "container-stdout"
    );
    assert_eq!(
        fs::read_to_string(run_dir.join("nodes").join("container").join("stderr.log"))
            .expect("stderr"),
        "container-stderr"
    );
    let trace = read_node_trace(&run_dir, "container");
    assert_eq!(trace["status"], "success");
    assert_eq!(trace["container"]["image"], "example.local/runner@sha256:feedface");
    assert_eq!(trace["container"]["image_digest"], "sha256:feedface");
    assert_eq!(trace["container"]["engine"], "docker");
    assert_eq!(trace["container"]["engine_version"], "docker fake 1.0");
}

#[test]
fn container_adapter_rejects_unpinned_image_reference_by_default() {
    let dir = tempfile::tempdir().expect("tmpdir");
    let graph = container_graph(&["filesystem"], None, "example.local/runner:latest", "true");
    let runtime = Runtime::new();
    let run_dir = runtime.run(&graph, dir.path(), RuntimeConfig::default()).expect("run");

    let trace = read_node_trace(&run_dir, "container");
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["failure"]["class"], "policy");
    assert_eq!(trace["failure"]["code"], "POLICY_CONTAINER_IMAGE_REFERENCE_DENIED");
    assert_eq!(trace["failure"]["details"]["image"], "example.local/runner:latest");
    assert_eq!(trace["failure"]["details"]["container_image_reference_policy"], "require_digest");
}

#[test]
fn container_adapter_allows_unpinned_image_reference_with_explicit_override() {
    let _lock = process_env_lock();
    let dir = tempfile::tempdir().expect("tmpdir");
    let bin_dir = dir.path().join("bin");
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
  outputs_dir=""
  network_mode="default"
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --network)
        network_mode="$2"
        shift 2
        ;;
      -v)
        mount="$2"
        host_path=$(printf '%s' "$mount" | cut -d: -f1)
        container_path=$(printf '%s' "$mount" | cut -d: -f2)
        if [ "$container_path" = "/bijux/node/outputs" ]; then
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
      --workdir)
        shift 2
        ;;
      -*)
        shift
        ;;
      *)
        break
        ;;
    esac
  done
  printf 'ok' > "$outputs_dir/result.txt"
  printf '%s' "$network_mode" > "$outputs_dir/network.txt"
  printf '/bijux/node/work/scratch' > "$outputs_dir/workdir.txt"
  exit 0
fi
exit 1
"#,
    );
    let _path_guard = prepend_path(&bin_dir);

    let graph = container_graph(&["filesystem"], None, "example.local/runner:latest", "true");
    let runtime = Runtime::new();
    let run_dir = runtime
        .run(
            &graph,
            dir.path(),
            RuntimeConfig {
                policy: bijux_dag_runtime::PolicyConfig {
                    container_image_reference_policy:
                        bijux_dag_runtime::ContainerImageReferencePolicy::AllowUnpinned,
                    ..bijux_dag_runtime::PolicyConfig::default()
                },
                ..RuntimeConfig::default()
            },
        )
        .expect("run");

    let trace = read_node_trace(&run_dir, "container");
    assert_eq!(trace["status"], "success");
    assert_eq!(trace["container"]["image"], "example.local/runner:latest");
    assert_eq!(trace["container"]["image_digest"], "sha256:feedface");
}

#[test]
fn container_adapter_omits_no_network_flag_when_network_effect_is_declared() {
    let _lock = process_env_lock();
    let dir = tempfile::tempdir().expect("tmpdir");
    let bin_dir = dir.path().join("bin");
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
  outputs_dir=""
  network_mode="default"
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --network)
        network_mode="$2"
        shift 2
        ;;
      -v)
        mount="$2"
        host_path=$(printf '%s' "$mount" | cut -d: -f1)
        container_path=$(printf '%s' "$mount" | cut -d: -f2)
        if [ "$container_path" = "/bijux/node/outputs" ]; then
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
      --workdir)
        shift 2
        ;;
      -*)
        shift
        ;;
      *)
        break
        ;;
    esac
  done
  printf '%s' "$network_mode" > "$outputs_dir/network.txt"
  printf 'ok' > "$outputs_dir/result.txt"
  printf '/bijux/node/work/scratch' > "$outputs_dir/workdir.txt"
  exit 0
fi
exit 1
"#,
    );
    let _path_guard = prepend_path(&bin_dir);

    let graph = container_graph(
        &["filesystem", "network"],
        None,
        "example.local/runner@sha256:feedface",
        "true",
    );
    let runtime = Runtime::new();
    let run_dir = runtime.run(&graph, dir.path(), RuntimeConfig::default()).expect("run");

    assert_eq!(
        fs::read_to_string(
            run_dir.join("nodes").join("container").join("outputs").join("network.txt")
        )
        .expect("network mode"),
        "default"
    );
}

#[test]
fn container_adapter_preserves_streams_and_identity_on_timeout() {
    let _lock = process_env_lock();
    let dir = tempfile::tempdir().expect("tmpdir");
    let bin_dir = dir.path().join("bin");
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
  trap 'exit 143' TERM
  printf 'partial-stdout'
  printf 'partial-stderr' >&2
  sleep 1
  exit 0
fi
exit 1
"#,
    );
    let _path_guard = prepend_path(&bin_dir);

    let graph = container_graph(
        &["filesystem"],
        Some(50),
        "example.local/runner@sha256:feedface",
        "sleep 1",
    );
    let runtime = Runtime::new();
    let run_dir = runtime.run(&graph, dir.path(), RuntimeConfig::default()).expect("run");

    assert_eq!(
        fs::read_to_string(run_dir.join("nodes").join("container").join("stdout.log"))
            .expect("stdout"),
        "partial-stdout"
    );
    let stderr = fs::read_to_string(run_dir.join("nodes").join("container").join("stderr.log"))
        .expect("stderr");
    assert!(stderr.starts_with("partial-stderr"));
    let trace = read_node_trace(&run_dir, "container");
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["failure"]["code"], "EXEC_TIMEOUT");
    assert_eq!(trace["failure"]["class"], "timeout");
    assert_eq!(trace["container"]["image"], "example.local/runner@sha256:feedface");
    assert_eq!(trace["container"]["image_digest"], "sha256:feedface");
    assert_eq!(trace["container"]["engine_version"], "docker fake 1.0");
}

#[test]
fn external_adapter_probe_reports_exact_manifest_failure_reason() {
    let _lock = process_env_lock();
    let dir = tempfile::tempdir().expect("tmpdir");
    let adapters = dir.path().join("adapters");
    fs::create_dir_all(&adapters).expect("mkdir");
    let script = adapters.join("bad-adapter");
    fs::write(
        &script,
        "#!/bin/sh\nif [ \"$1\" = \"info\" ]; then echo '{\"adapter_id\":\"bad\"}'; exit 0; fi\nexit 1\n",
    )
    .expect("write");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script).expect("meta").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script, perms).expect("chmod");
    }
    std::env::set_var("BIJUX_DAG_ADAPTERS_DIR", &adapters);
    let reports = probe_external_adapters().expect("probe");
    std::env::remove_var("BIJUX_DAG_ADAPTERS_DIR");
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].status, bijux_dag_runtime::ExternalAdapterHandshakeStatus::Rejected);
    assert!(reports[0].reason.as_deref().unwrap_or_default().contains("invalid adapter manifest"));
}

#[test]
fn external_adapter_info_stderr_boundary_is_rejected() {
    let _lock = process_env_lock();
    let dir = tempfile::tempdir().expect("tmpdir");
    let adapters = dir.path().join("adapters");
    fs::create_dir_all(&adapters).expect("mkdir");
    let script = adapters.join("stderr-adapter");
    fs::write(
        &script,
        "#!/bin/sh\nif [ \"$1\" = \"info\" ]; then echo '{\"protocol_version\":\"bijux-dag-adapter/v1\",\"adapter_id\":\"noisy\",\"adapter_version\":\"0.1\",\"required_effects\":{\"filesystem\":true,\"env\":false,\"network\":false,\"clock\":false},\"supported_kinds\":[\"fake\"],\"output_schema\":\"v0.1\"}'; echo 'log-noise' >&2; exit 0; fi\nexit 1\n",
    )
    .expect("write");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script).expect("meta").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script, perms).expect("chmod");
    }
    std::env::set_var("BIJUX_DAG_ADAPTERS_DIR", &adapters);
    let reports = probe_external_adapters().expect("probe");
    std::env::remove_var("BIJUX_DAG_ADAPTERS_DIR");
    assert_eq!(reports[0].status, bijux_dag_runtime::ExternalAdapterHandshakeStatus::Rejected);
    assert!(reports[0].reason.as_deref().unwrap_or_default().contains("stdout only"));
}

#[test]
fn external_adapter_timeout_quarantines_partial_outputs_and_preserves_binary_hash() {
    let _lock = process_env_lock();
    let dir = tempfile::tempdir().expect("tmpdir");
    let adapter_dir = dir.path().join("adapters");
    fs::create_dir_all(&adapter_dir).expect("mkdir");
    let adapter_path = adapter_dir.join("slow-adapter");
    fs::write(
        &adapter_path,
        r#"#!/bin/sh
if [ "$1" = "info" ]; then
  echo '{"protocol_version":"bijux-dag-adapter/v1","adapter_id":"slow","adapter_version":"0.1","required_effects":{"filesystem":true,"env":false,"network":false,"clock":false},"supported_kinds":["fake"],"output_schema":"v0.1"}'
  exit 0
fi
if [ "$1" = "execute" ]; then
  outdir=""
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --outdir) outdir="$2"; shift 2;;
      --workdir|--node-spec) shift 2;;
      *) shift;;
    esac
  done
  mkdir -p "$outdir"
  echo "partial" > "$outdir/out"
  sleep 1
  exit 0
fi
exit 1
"#,
    )
    .expect("write");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&adapter_path).expect("meta").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&adapter_path, perms).expect("chmod");
    }
    std::env::set_var("BIJUX_DAG_ADAPTERS_DIR", &adapter_dir);
    let graph = external_graph("fake", Some(50));
    let runtime = Runtime::new();
    let run_dir = runtime.run(&graph, dir.path(), RuntimeConfig::default()).expect("run");
    std::env::remove_var("BIJUX_DAG_ADAPTERS_DIR");
    let trace: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join("n1").join("trace.json")).expect("trace"),
    )
    .expect("trace json");
    assert_eq!(trace["failure"]["code"], "EXEC_TIMEOUT");
    assert_eq!(trace["failure"]["class"], "timeout");
    assert_eq!(trace["failure"]["details"]["timeout_class"], "external_adapter_process");
    assert!(trace["adapter_binary_sha256"].as_str().is_some());
    let quarantine_dir =
        trace["failure"]["details"]["quarantined_outputs_dir"].as_str().expect("quarantine dir");
    let quarantine_path = {
        let raw = std::path::Path::new(quarantine_dir);
        if raw.is_absolute() {
            raw.to_path_buf()
        } else {
            run_dir.join("nodes").join("n1").join(raw)
        }
    };
    assert!(quarantine_path.exists());
    assert!(quarantine_path.join("out").exists());
}

#[test]
fn external_adapter_trace_records_binary_hash_on_success() {
    let _lock = process_env_lock();
    let dir = tempfile::tempdir().expect("tmpdir");
    let adapter_dir = dir.path().join("adapters");
    fs::create_dir_all(&adapter_dir).expect("mkdir");
    let adapter_path = adapter_dir.join("fake-adapter");
    fs::write(&adapter_path, include_str!("bin/fake_adapter.sh")).expect("write");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&adapter_path).expect("meta").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&adapter_path, perms).expect("chmod");
    }
    std::env::set_var("BIJUX_DAG_ADAPTERS_DIR", &adapter_dir);
    let graph = external_graph("fake", None);
    let runtime = Runtime::new();
    let run_dir = runtime.run(&graph, dir.path(), RuntimeConfig::default()).expect("run");
    std::env::remove_var("BIJUX_DAG_ADAPTERS_DIR");
    let trace: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join("n1").join("trace.json")).expect("trace"),
    )
    .expect("trace json");
    assert!(trace["adapter_binary_sha256"].as_str().is_some());
}

#[test]
fn adapter_output_schema_compatibility_reports_exact_mismatch() {
    let report = validate_output_schema_compatibility(
        bijux_dag_runtime::CacheCompatibilityMode::FingerprintExact,
        "schema/v1",
        "schema/v2",
    );
    assert!(!report.compatible);
    assert!(report.reason.contains("fingerprint-exact"));
}

#[test]
fn adapter_conformance_suite_covers_timeout_cache_and_non_utf8_scenarios() {
    let suites = adapter_conformance_suite().expect("suite");
    let shell = suites.iter().find(|suite| suite.adapter_id == "shell").expect("shell");
    assert!(shell.scenarios.iter().any(|scenario| scenario.scenario == "timeout"));
    assert!(shell.scenarios.iter().any(|scenario| scenario.scenario == "cache_output"));
    assert!(shell.scenarios.iter().any(|scenario| scenario.scenario == "non_utf8_output"));
}
