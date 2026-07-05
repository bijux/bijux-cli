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
    container_network_policy_args, container_volume_contract, probe_external_adapters,
    registered_adapter_descriptors, validate_container_mount_contract,
    validate_output_schema_compatibility, Runtime, RuntimeConfig,
};
use serde_json::Value;
use std::fs;
use std::sync::{Mutex, OnceLock};

fn process_env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|error| error.into_inner())
}

fn shell_graph(command: &str, effects: &[&str]) -> String {
    let effects =
        effects.iter().map(|effect| format!("\"{effect}\"")).collect::<Vec<_>>().join(",");
    let env_allowlist = if effects.contains(&"env") {
        ",\n              \"env_allowlist\":[\"PATH\"]"
    } else {
        ""
    };
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
