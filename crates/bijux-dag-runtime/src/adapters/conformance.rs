//! Adapter conformance checks.

use crate::adapter::{AdapterDescriptor, AdapterOrigin, CacheCompatibilityMode};
use crate::backend::fake::FakeBatchExecutorContract;
use crate::backend_cluster::{KubernetesAdapterContractReport, SlurmAdapterDesignContractReport};
use crate::{NodeTrace, OutputsIndex, PolicyConfig, Runtime, RuntimeConfig};
use bijux_dag_core::parse_graph_strict;
use bijux_dag_core::Severity;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::ffi::OsString;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterConformanceReport {
    pub adapter_id: String,
    pub passed: bool,
    pub violations: Vec<String>,
}

pub fn validate_descriptor(descriptor: &AdapterDescriptor) -> AdapterConformanceReport {
    let mut violations = Vec::new();
    if descriptor.id.trim().is_empty() {
        violations.push("missing adapter id".to_string());
    }
    if descriptor.version.trim().is_empty() {
        violations.push("missing adapter version".to_string());
    }
    if descriptor.supported_kinds.is_empty() {
        violations.push("missing supported kinds".to_string());
    }
    if descriptor.produces_outputs_schema_version.trim().is_empty() {
        violations.push("missing outputs schema version".to_string());
    }
    if descriptor.protocol_version.trim().is_empty() {
        violations.push("missing adapter protocol version".to_string());
    }
    if matches!(descriptor.origin, AdapterOrigin::External)
        && !descriptor.required_effects.filesystem
        && !descriptor.required_effects.env
        && !descriptor.required_effects.network
        && !descriptor.required_effects.clock
    {
        violations.push("external adapter declares no required effects".to_string());
    }
    if matches!(descriptor.origin, AdapterOrigin::External) && descriptor.binary_hash.is_none() {
        violations.push("external adapter missing binary hash".to_string());
    }

    AdapterConformanceReport {
        adapter_id: descriptor.id.clone(),
        passed: violations.is_empty(),
        violations,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdapterScenarioStatus {
    Pass,
    Fail,
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdapterScenarioResult {
    pub scenario: String,
    pub status: AdapterScenarioStatus,
    pub enforced_by_runtime: bool,
    pub advisory_only: bool,
    pub checked_by_execution: bool,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observation: Option<AdapterScenarioObservation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdapterConformanceSuiteReport {
    pub adapter_id: String,
    pub adapter_version: String,
    pub origin: AdapterOrigin,
    pub scenarios: Vec<AdapterScenarioResult>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdapterScenarioObservation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_class: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter_outputs_schema_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter_binary_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdapterOutputSchemaCompatibilityReport {
    pub compatible: bool,
    pub compatibility_mode: CacheCompatibilityMode,
    pub produced_schema_version: String,
    pub expected_schema_version: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdapterReferenceDocument {
    pub descriptors: Vec<AdapterDescriptor>,
    pub conformance: Vec<AdapterConformanceSuiteReport>,
    pub slurm: SlurmAdapterDesignContractReport,
    pub kubernetes: KubernetesAdapterContractReport,
    pub fake_batch: FakeBatchExecutorContract,
}

pub fn validate_output_schema_compatibility(
    mode: CacheCompatibilityMode,
    produced_schema_version: &str,
    expected_schema_version: &str,
) -> AdapterOutputSchemaCompatibilityReport {
    let compatible = match mode {
        CacheCompatibilityMode::FingerprintExact => {
            produced_schema_version == expected_schema_version
        }
    };
    let reason = if compatible {
        "produced output schema matches the expected adapter schema".to_string()
    } else {
        match mode {
            CacheCompatibilityMode::FingerprintExact => format!(
                "cache entry schema '{}' is incompatible with expected schema '{}' under fingerprint-exact compatibility",
                produced_schema_version, expected_schema_version
            ),
        }
    };
    AdapterOutputSchemaCompatibilityReport {
        compatible,
        compatibility_mode: mode,
        produced_schema_version: produced_schema_version.to_string(),
        expected_schema_version: expected_schema_version.to_string(),
        reason,
    }
}

fn scenario(
    name: &str,
    status: AdapterScenarioStatus,
    enforced_by_runtime: bool,
    advisory_only: bool,
    checked_by_execution: bool,
    reason: &str,
    observation: Option<AdapterScenarioObservation>,
) -> AdapterScenarioResult {
    AdapterScenarioResult {
        scenario: name.to_string(),
        status,
        enforced_by_runtime,
        advisory_only,
        checked_by_execution,
        reason: reason.to_string(),
        observation,
    }
}

pub fn build_adapter_conformance_suite(
    descriptor: &AdapterDescriptor,
) -> AdapterConformanceSuiteReport {
    let scenarios = match descriptor.id.as_str() {
        "const" => const_adapter_scenarios(descriptor),
        "shell" => shell_adapter_scenarios(descriptor),
        "python" => python_adapter_scenarios(descriptor),
        "http" => http_adapter_scenarios(descriptor),
        "file_transform" => file_transform_adapter_scenarios(descriptor),
        "container" => container_adapter_scenarios(descriptor),
        _ if matches!(descriptor.origin, AdapterOrigin::External) => {
            external_adapter_scenarios(descriptor)
        }
        _ => unsupported_adapter_scenarios(descriptor),
    };
    AdapterConformanceSuiteReport {
        adapter_id: descriptor.id.clone(),
        adapter_version: descriptor.version.clone(),
        origin: descriptor.origin,
        scenarios,
    }
}

pub fn generate_adapter_reference_markdown(document: &AdapterReferenceDocument) -> String {
    let mut lines = Vec::new();
    lines.push("# Adapter Contract".to_string());
    lines.push(String::new());
    lines.push("This document is generated from runtime adapter descriptors and backend contract references.".to_string());
    lines.push(String::new());
    lines.push("## Registered adapters".to_string());
    for descriptor in &document.descriptors {
        lines.push(format!(
            "- `{}` `{}`: kinds={:?}, origin={:?}, schema={}, timeout={}, cancel={}, cache={:?}",
            descriptor.id,
            descriptor.version,
            descriptor.supported_kinds,
            descriptor.origin,
            descriptor.produces_outputs_schema_version,
            descriptor.supports_timeout,
            descriptor.supports_cancel,
            descriptor.cache_compatibility
        ));
    }
    lines.push(String::new());
    lines.push("## Conformance scenarios".to_string());
    for report in &document.conformance {
        lines.push(format!("### {} {}", report.adapter_id, report.adapter_version));
        for scenario in &report.scenarios {
            lines.push(format!(
                "- `{}`: {:?} (enforced_by_runtime={}, advisory_only={}, checked_by_execution={}) - {}",
                scenario.scenario,
                scenario.status,
                scenario.enforced_by_runtime,
                scenario.advisory_only,
                scenario.checked_by_execution,
                scenario.reason
            ));
            if let Some(observation) = &scenario.observation {
                let node_status = observation.node_status.as_deref().unwrap_or("none");
                let failure_code = observation.failure_code.as_deref().unwrap_or("none");
                let adapter_id = observation.adapter_id.as_deref().unwrap_or("unknown");
                let adapter_version = observation.adapter_version.as_deref().unwrap_or("unknown");
                let schema =
                    observation.adapter_outputs_schema_version.as_deref().unwrap_or("unknown");
                let output_files = if observation.output_files.is_empty() {
                    "none".to_string()
                } else {
                    observation.output_files.join(", ")
                };
                lines.push(format!(
                    "  observed status={}, failure_code={}, adapter={}@{}, schema={}, outputs={}",
                    node_status, failure_code, adapter_id, adapter_version, schema, output_files,
                ));
            }
        }
        lines.push(String::new());
    }
    lines.push("## External adapter protocol boundary".to_string());
    lines.push("- `info --json` must emit machine JSON on stdout only.".to_string());
    lines.push("- non-empty stderr during the info handshake is rejected.".to_string());
    lines.push(
        "- `execute` receives `--node-spec`, `--workdir`, `--outdir`, and `--failure-path`."
            .to_string(),
    );
    lines.push(
        "- nonzero adapter exits should write a `FailureInfo` JSON envelope to `--failure-path` for precise runtime failure mapping.".to_string(),
    );
    lines.push(
        "- external adapter binaries are fingerprinted into node trace evidence and cache identity."
            .to_string(),
    );
    lines.push(String::new());
    lines.push("## Slurm contract".to_string());
    lines.push(format!(
        "- submit=`{}`, poll=`{}`, cancel=`{}`",
        document.slurm.contract.submit_command,
        document.slurm.contract.poll_command,
        document.slurm.contract.cancel_command
    ));
    lines.push(format!("- logs: {}", document.slurm.log_collection_mode));
    lines.push(format!("- artifacts: {}", document.slurm.artifact_collection_mode));
    lines.push(String::new());
    lines.push("## Kubernetes contract".to_string());
    lines.push(format!("- namespace: `{}`", document.kubernetes.contract.namespace));
    lines.push(format!("- job spec mapping: {}", document.kubernetes.job_spec_mapping));
    lines.push(format!("- pod status mapping: {}", document.kubernetes.pod_status_mapping));
    lines.push(format!("- logs: {}", document.kubernetes.log_collection_mode));
    lines.push(format!("- artifacts: {}", document.kubernetes.artifact_collection_mode));
    lines.push(format!(
        "- unsupported fields rejected: {}",
        document.kubernetes.unsupported_field_rejection.join(", ")
    ));
    lines.push(String::new());
    lines.push("## Fake batch executor".to_string());
    lines.push(format!(
        "- submit=`{}`, poll=`{}`, cancel=`{}`",
        document.fake_batch.submit_api,
        document.fake_batch.poll_api,
        document.fake_batch.cancel_api
    ));
    lines.push(format!("- states: {}", document.fake_batch.supported_states.join(", ")));
    lines.join("\n")
}

#[derive(Debug)]
struct ConformanceRunRecord {
    trace: NodeTrace,
    outputs_index: Option<OutputsIndex>,
}

impl ConformanceRunRecord {
    fn observation(&self) -> AdapterScenarioObservation {
        AdapterScenarioObservation {
            node_status: Some(self.trace.status.clone()),
            failure_code: self.trace.failure.as_ref().map(|failure| failure.code.clone()),
            failure_class: self
                .trace
                .failure
                .as_ref()
                .and_then(|failure| failure.class.map(|class| class.as_str().to_string())),
            output_files: self
                .outputs_index
                .as_ref()
                .map(|index| index.files.iter().map(|file| file.name.clone()).collect())
                .unwrap_or_default(),
            adapter_id: Some(self.trace.adapter_id.clone()),
            adapter_version: Some(self.trace.adapter_version.clone()),
            adapter_outputs_schema_version: Some(self.trace.adapter_outputs_schema_version.clone()),
            adapter_binary_sha256: self.trace.adapter_binary_sha256.clone(),
        }
    }
}

#[derive(Debug)]
struct ConformanceWorkspace {
    path: PathBuf,
}

impl ConformanceWorkspace {
    fn new(adapter_id: &str, scenario: &str) -> Result<Self, String> {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos();
        let unique = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "bijux-adapter-conformance-{adapter_id}-{scenario}-{}-{nanos}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&path).map_err(|error| error.to_string())?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ConformanceWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[derive(Debug, Clone)]
struct ScriptedHttpResponse {
    status_line: &'static str,
    body: &'static [u8],
    content_type: &'static str,
    delay: Duration,
}

struct ScriptedHttpServer {
    base_url: String,
    join: Option<thread::JoinHandle<()>>,
}

impl ScriptedHttpServer {
    fn spawn(response: ScriptedHttpResponse) -> Result<Self, String> {
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|error| error.to_string())?;
        let address = listener.local_addr().map_err(|error| error.to_string())?;
        let join = thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let _ = stream.set_read_timeout(Some(Duration::from_millis(200)));
                let mut request = [0_u8; 4096];
                let _ = stream.read(&mut request);
                if !response.delay.is_zero() {
                    thread::sleep(response.delay);
                }
                let headers = format!(
                    "HTTP/1.1 {}\r\ncontent-type: {}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                    response.status_line,
                    response.content_type,
                    response.body.len()
                );
                let _ = stream.write_all(headers.as_bytes());
                let _ = stream.write_all(response.body);
                let _ = stream.flush();
            }
        });
        Ok(Self { base_url: format!("http://{address}"), join: Some(join) })
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

impl Drop for ScriptedHttpServer {
    fn drop(&mut self) {
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

struct ScopedEnvVar {
    key: &'static str,
    previous: Option<OsString>,
}

impl ScopedEnvVar {
    fn set(key: &'static str, value: &Path) -> Self {
        let previous = std::env::var_os(key);
        std::env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for ScopedEnvVar {
    fn drop(&mut self) {
        if let Some(previous) = &self.previous {
            std::env::set_var(self.key, previous);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

fn python_env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|error| error.into_inner())
}

fn pass_scenario(
    name: &str,
    enforced_by_runtime: bool,
    advisory_only: bool,
    reason: impl Into<String>,
    observation: Option<AdapterScenarioObservation>,
) -> AdapterScenarioResult {
    scenario(
        name,
        AdapterScenarioStatus::Pass,
        enforced_by_runtime,
        advisory_only,
        true,
        &reason.into(),
        observation,
    )
}

fn fail_scenario(
    name: &str,
    enforced_by_runtime: bool,
    advisory_only: bool,
    reason: impl Into<String>,
    observation: Option<AdapterScenarioObservation>,
) -> AdapterScenarioResult {
    scenario(
        name,
        AdapterScenarioStatus::Fail,
        enforced_by_runtime,
        advisory_only,
        true,
        &reason.into(),
        observation,
    )
}

fn skip_scenario(
    name: &str,
    enforced_by_runtime: bool,
    advisory_only: bool,
    reason: impl Into<String>,
) -> AdapterScenarioResult {
    scenario(
        name,
        AdapterScenarioStatus::Skip,
        enforced_by_runtime,
        advisory_only,
        false,
        &reason.into(),
        None,
    )
}

fn run_scenario_check(
    name: &str,
    record: &ConformanceRunRecord,
    enforced_by_runtime: bool,
    check: impl FnOnce(&ConformanceRunRecord) -> Result<String, String>,
) -> AdapterScenarioResult {
    let observation = Some(record.observation());
    match check(record) {
        Ok(reason) => pass_scenario(name, enforced_by_runtime, false, reason, observation),
        Err(reason) => fail_scenario(name, enforced_by_runtime, false, reason, observation),
    }
}

fn execute_graph_record(
    workspace: &ConformanceWorkspace,
    node_id: &str,
    graph_json: &str,
    config: RuntimeConfig,
) -> Result<ConformanceRunRecord, String> {
    let graph = parse_graph_strict(graph_json)
        .map_err(|error| format!("parse error: {error:?}: {error}"))?;
    let validation_errors = graph
        .validate_with_warnings()
        .into_iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .map(|diagnostic| diagnostic.message)
        .collect::<Vec<_>>();
    if !validation_errors.is_empty() {
        return Err(format!("graph validation failed: {}", validation_errors.join("; ")));
    }
    let run_dir = Runtime::new()
        .run(&graph, workspace.path(), config)
        .map_err(|error| format!("run error: {error:?}: {error}"))?;
    let trace_path = run_dir.join("nodes").join(node_id).join("trace.json");
    let trace: NodeTrace =
        serde_json::from_str(&fs::read_to_string(&trace_path).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    let outputs_index_path = run_dir.join("nodes").join(node_id).join("outputs").join("index.json");
    let outputs_index = if outputs_index_path.exists() {
        Some(
            serde_json::from_str(
                &fs::read_to_string(outputs_index_path).map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?,
        )
    } else {
        None
    };
    Ok(ConformanceRunRecord { trace, outputs_index })
}

fn expect_success(record: &ConformanceRunRecord, adapter_id: &str) -> Result<String, String> {
    if record.trace.status == "success" {
        Ok(format!("runtime completed {adapter_id} execution successfully"))
    } else {
        Err(format!(
            "expected successful execution for {adapter_id}, observed status {}",
            record.trace.status
        ))
    }
}

fn expect_failure(
    record: &ConformanceRunRecord,
    adapter_id: &str,
    failure_code: &str,
    failure_class: &str,
) -> Result<String, String> {
    let Some(failure) = &record.trace.failure else {
        return Err(format!(
            "expected structured failure for {adapter_id}, but trace contains no failure payload"
        ));
    };
    let observed_class = failure.class.map(|class| class.as_str().to_string()).unwrap_or_default();
    if record.trace.status != "failed" {
        return Err(format!(
            "expected failed status for {adapter_id}, observed {}",
            record.trace.status
        ));
    }
    if failure.code != failure_code {
        return Err(format!(
            "expected failure code {failure_code} for {adapter_id}, observed {}",
            failure.code
        ));
    }
    if observed_class != failure_class {
        return Err(format!(
            "expected failure class {failure_class} for {adapter_id}, observed {observed_class}"
        ));
    }
    Ok(format!(
        "runtime recorded structured failure {failure_code} ({failure_class}) for {adapter_id}"
    ))
}

fn expect_output_manifest(
    record: &ConformanceRunRecord,
    adapter_id: &str,
    expected_files: &[&str],
) -> Result<String, String> {
    let Some(index) = &record.outputs_index else {
        return Err(format!("expected outputs manifest for {adapter_id}, but none was written"));
    };
    let files = index.files.iter().map(|file| file.name.as_str()).collect::<Vec<_>>();
    if expected_files.iter().all(|expected| files.iter().any(|file| file == expected)) {
        Ok(format!(
            "runtime wrote outputs manifest for {adapter_id} with files {}",
            files.join(", ")
        ))
    } else {
        Err(format!(
            "expected outputs manifest for {adapter_id} to contain {:?}, observed {:?}",
            expected_files, files
        ))
    }
}

fn expect_identity_schema(
    record: &ConformanceRunRecord,
    descriptor: &AdapterDescriptor,
) -> Result<String, String> {
    if record.trace.adapter_id != descriptor.id {
        return Err(format!(
            "expected adapter id {}, observed {}",
            descriptor.id, record.trace.adapter_id
        ));
    }
    if record.trace.adapter_version != descriptor.version {
        return Err(format!(
            "expected adapter version {}, observed {}",
            descriptor.version, record.trace.adapter_version
        ));
    }
    if record.trace.adapter_outputs_schema_version != descriptor.produces_outputs_schema_version {
        return Err(format!(
            "expected adapter outputs schema {}, observed {}",
            descriptor.produces_outputs_schema_version, record.trace.adapter_outputs_schema_version
        ));
    }
    Ok(format!(
        "trace recorded adapter identity {}@{} with schema {}",
        descriptor.id, descriptor.version, descriptor.produces_outputs_schema_version
    ))
}

fn execution_error(name: &str, reason: impl Into<String>) -> AdapterScenarioResult {
    scenario(name, AdapterScenarioStatus::Fail, true, false, false, &reason.into(), None)
}

fn runtime_config_with_env_policy() -> RuntimeConfig {
    RuntimeConfig {
        policy: PolicyConfig { clean_env: false, ..PolicyConfig::default() },
        ..RuntimeConfig::default()
    }
}

fn const_graph() -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [{
            "id": "const",
            "kind": "const",
            "outputs": [{"name": "value", "path": "value.json", "media_type": "application/json"}],
            "params": {"value": {"message": "hello"}}
        }],
        "edges": []
    })
    .to_string()
}

fn shell_graph(command: &str, timeout_ms: Option<u64>) -> String {
    let mut node = json!({
        "id": "shell",
        "kind": "shell",
        "outputs": [{"name": "value", "path": "value.txt"}],
        "params": {"argv": ["/bin/sh", "-c", command]},
        "effects": ["filesystem"],
    });
    if let Some(timeout_ms) = timeout_ms {
        node["timeout_ms"] = json!(timeout_ms);
    }
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [node],
        "edges": []
    })
    .to_string()
}

fn python_graph(module: &str, function: &str, timeout_ms: Option<u64>) -> String {
    let mut node = json!({
        "id": "python",
        "kind": "python",
        "outputs": [{"name": "result", "path": "result.json", "media_type": "application/json"}],
        "params": {
            "module": module,
            "function": function,
            "value": "payload"
        },
        "effects": ["filesystem", "env"],
        "env_allowlist": ["PYTHONPATH"]
    });
    if let Some(timeout_ms) = timeout_ms {
        node["timeout_ms"] = json!(timeout_ms);
    }
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [node],
        "edges": []
    })
    .to_string()
}

fn http_graph(url: &str, timeout_ms: Option<u64>) -> String {
    let mut node = json!({
        "id": "http",
        "kind": "http",
        "outputs": [{"name": "response", "path": "response.json", "media_type": "application/json"}],
        "params": {
            "method": "GET",
            "url": url
        },
        "effects": ["filesystem", "network"]
    });
    if let Some(timeout_ms) = timeout_ms {
        node["timeout_ms"] = json!(timeout_ms);
    }
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [node],
        "edges": []
    })
    .to_string()
}

fn file_transform_graph(params: Value, outputs: Vec<Value>, timeout_ms: Option<u64>) -> String {
    let seed_command = "printf 'alpha\\nbeta\\n' > ../outputs/source.txt";
    let mut file_node = json!({
        "id": "file_transform",
        "kind": "file_transform",
        "inputs": ["source"],
        "outputs": outputs,
        "params": params,
        "effects": ["filesystem"]
    });
    if let Some(timeout_ms) = timeout_ms {
        file_node["timeout_ms"] = json!(timeout_ms);
    }
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "seed",
                "kind": "shell",
                "outputs": [{"name": "source", "path": "source.txt"}],
                "params": {"argv": ["/bin/sh", "-c", seed_command]},
                "effects": ["filesystem"]
            },
            file_node
        ],
        "edges": [{
            "from": {"node_id": "seed", "port": "source"},
            "to": {"node_id": "file_transform", "port": "source"}
        }]
    })
    .to_string()
}

fn python_runtime_available() -> bool {
    ["python3", "python"].iter().any(|candidate| {
        std::process::Command::new(candidate)
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    })
}

fn write_python_fixture(
    module_dir: &Path,
    module_name: &str,
    contents: &str,
) -> Result<(), String> {
    fs::create_dir_all(module_dir).map_err(|error| error.to_string())?;
    fs::write(module_dir.join(format!("{module_name}.py")), contents)
        .map_err(|error| error.to_string())
}

fn const_adapter_scenarios(descriptor: &AdapterDescriptor) -> Vec<AdapterScenarioResult> {
    let workspace = match ConformanceWorkspace::new("const", "success") {
        Ok(workspace) => workspace,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let success =
        match execute_graph_record(&workspace, "const", &const_graph(), RuntimeConfig::default()) {
            Ok(record) => record,
            Err(error) => return canonical_execution_failure_suite(descriptor, error),
        };
    vec![
        run_scenario_check("success", &success, true, |record| expect_success(record, "const")),
        skip_scenario(
            "failure",
            false,
            true,
            "const adapter has no runtime failure path for valid node definitions",
        ),
        skip_scenario(
            "missing_output",
            false,
            true,
            "const adapter always materializes its declared value output",
        ),
        skip_scenario("timeout", false, true, "const adapter does not expose timeout-sensitive work"),
        run_scenario_check("output_manifest", &success, true, |record| {
            expect_output_manifest(record, "const", &["value"])
        }),
        skip_scenario(
            "failure_schema",
            false,
            true,
            "const adapter does not emit structured failure payloads for successful value materialization",
        ),
        run_scenario_check("adapter_identity_schema", &success, true, |record| {
            expect_identity_schema(record, descriptor)
        }),
    ]
}

fn shell_adapter_scenarios(descriptor: &AdapterDescriptor) -> Vec<AdapterScenarioResult> {
    let success_workspace = match ConformanceWorkspace::new("shell", "success") {
        Ok(workspace) => workspace,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let success = match execute_graph_record(
        &success_workspace,
        "shell",
        &shell_graph("printf 'hello' > ../outputs/value.txt", None),
        RuntimeConfig::default(),
    ) {
        Ok(record) => record,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let failure_workspace = match ConformanceWorkspace::new("shell", "failure") {
        Ok(workspace) => workspace,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let failure = match execute_graph_record(
        &failure_workspace,
        "shell",
        &shell_graph("printf 'partial' > ../outputs/value.txt; printf 'boom' >&2; exit 7", None),
        RuntimeConfig::default(),
    ) {
        Ok(record) => record,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let missing_output_workspace = match ConformanceWorkspace::new("shell", "missing-output") {
        Ok(workspace) => workspace,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let missing_output = match execute_graph_record(
        &missing_output_workspace,
        "shell",
        &shell_graph("printf 'no-output'", None),
        RuntimeConfig::default(),
    ) {
        Ok(record) => record,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let timeout_workspace = match ConformanceWorkspace::new("shell", "timeout") {
        Ok(workspace) => workspace,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let timeout = match execute_graph_record(
        &timeout_workspace,
        "shell",
        &shell_graph("sleep 1", Some(50)),
        RuntimeConfig::default(),
    ) {
        Ok(record) => record,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    vec![
        run_scenario_check("success", &success, true, |record| expect_success(record, "shell")),
        run_scenario_check("failure", &failure, true, |record| {
            expect_failure(record, "shell", "EXEC_FAIL", "execution")
        }),
        run_scenario_check("missing_output", &missing_output, true, |record| {
            expect_failure(record, "shell", "OUTPUT_MISSING", "user")
        }),
        run_scenario_check("timeout", &timeout, true, |record| {
            expect_failure(record, "shell", "EXEC_TIMEOUT", "timeout")
        }),
        run_scenario_check("output_manifest", &success, true, |record| {
            expect_output_manifest(record, "shell", &["value"])
        }),
        run_scenario_check("failure_schema", &failure, true, |record| {
            expect_failure(record, "shell", "EXEC_FAIL", "execution")
        }),
        run_scenario_check("adapter_identity_schema", &success, true, |record| {
            expect_identity_schema(record, descriptor)
        }),
    ]
}

fn python_adapter_scenarios(descriptor: &AdapterDescriptor) -> Vec<AdapterScenarioResult> {
    if !python_runtime_available() {
        return canonical_skip_suite(
            descriptor,
            "python interpreter is unavailable, so runtime-backed python conformance could not execute",
        );
    }

    let _env_lock = python_env_lock();
    let workspace = match ConformanceWorkspace::new("python", "fixtures") {
        Ok(workspace) => workspace,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let module_dir = workspace.path().join("python");
    if let Err(error) = write_python_fixture(
        &module_dir,
        "conformance_python_adapter",
        "import time\n\ndef emit(payload):\n    return payload\n\ndef explode(payload):\n    raise ValueError('boom')\n\ndef stall(payload):\n    time.sleep(1)\n    return payload\n",
    ) {
        return canonical_execution_failure_suite(descriptor, error);
    }
    let _pythonpath = ScopedEnvVar::set("PYTHONPATH", &module_dir);

    let success_workspace = match ConformanceWorkspace::new("python", "success") {
        Ok(workspace) => workspace,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let success = match execute_graph_record(
        &success_workspace,
        "python",
        &python_graph("conformance_python_adapter", "emit", None),
        runtime_config_with_env_policy(),
    ) {
        Ok(record) => record,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let failure_workspace = match ConformanceWorkspace::new("python", "failure") {
        Ok(workspace) => workspace,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let failure = match execute_graph_record(
        &failure_workspace,
        "python",
        &python_graph("conformance_python_adapter", "explode", None),
        runtime_config_with_env_policy(),
    ) {
        Ok(record) => record,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let timeout_workspace = match ConformanceWorkspace::new("python", "timeout") {
        Ok(workspace) => workspace,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let timeout = match execute_graph_record(
        &timeout_workspace,
        "python",
        &python_graph("conformance_python_adapter", "stall", Some(50)),
        runtime_config_with_env_policy(),
    ) {
        Ok(record) => record,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    vec![
        run_scenario_check("success", &success, true, |record| expect_success(record, "python")),
        run_scenario_check("failure", &failure, true, |record| {
            expect_failure(record, "python", "PYTHON_EXCEPTION", "execution")
        }),
        skip_scenario(
            "missing_output",
            false,
            true,
            "python adapter failures are reported as structured execution exceptions before runtime output inspection",
        ),
        run_scenario_check("timeout", &timeout, true, |record| {
            expect_failure(record, "python", "EXEC_TIMEOUT", "timeout")
        }),
        run_scenario_check("output_manifest", &success, true, |record| {
            expect_output_manifest(record, "python", &["result"])
        }),
        run_scenario_check("failure_schema", &failure, true, |record| {
            expect_failure(record, "python", "PYTHON_EXCEPTION", "execution")
        }),
        run_scenario_check("adapter_identity_schema", &success, true, |record| {
            expect_identity_schema(record, descriptor)
        }),
    ]
}

fn http_adapter_scenarios(descriptor: &AdapterDescriptor) -> Vec<AdapterScenarioResult> {
    let success_server = match ScriptedHttpServer::spawn(ScriptedHttpResponse {
        status_line: "200 OK",
        body: br#"{"ok":true}"#,
        content_type: "application/json",
        delay: Duration::ZERO,
    }) {
        Ok(server) => server,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let success_workspace = match ConformanceWorkspace::new("http", "success") {
        Ok(workspace) => workspace,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let success = match execute_graph_record(
        &success_workspace,
        "http",
        &http_graph(&success_server.url("/ok"), None),
        RuntimeConfig::default(),
    ) {
        Ok(record) => record,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };

    let failure_server = match ScriptedHttpServer::spawn(ScriptedHttpResponse {
        status_line: "503 Service Unavailable",
        body: b"service down",
        content_type: "text/plain",
        delay: Duration::ZERO,
    }) {
        Ok(server) => server,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let failure_workspace = match ConformanceWorkspace::new("http", "failure") {
        Ok(workspace) => workspace,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let failure = match execute_graph_record(
        &failure_workspace,
        "http",
        &http_graph(&failure_server.url("/fail"), None),
        RuntimeConfig::default(),
    ) {
        Ok(record) => record,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };

    let timeout_server = match ScriptedHttpServer::spawn(ScriptedHttpResponse {
        status_line: "200 OK",
        body: br#"{"ok":true}"#,
        content_type: "application/json",
        delay: Duration::from_millis(200),
    }) {
        Ok(server) => server,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let timeout_workspace = match ConformanceWorkspace::new("http", "timeout") {
        Ok(workspace) => workspace,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let timeout = match execute_graph_record(
        &timeout_workspace,
        "http",
        &http_graph(&timeout_server.url("/slow"), Some(50)),
        RuntimeConfig::default(),
    ) {
        Ok(record) => record,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };

    vec![
        run_scenario_check("success", &success, true, |record| expect_success(record, "http")),
        run_scenario_check("failure", &failure, true, |record| {
            expect_failure(record, "http", "HTTP_STATUS_ERROR", "execution")
        }),
        skip_scenario(
            "missing_output",
            false,
            true,
            "http adapter always materializes the response artifact before runtime output inspection",
        ),
        run_scenario_check("timeout", &timeout, true, |record| {
            expect_failure(record, "http", "EXEC_TIMEOUT", "timeout")
        }),
        run_scenario_check("output_manifest", &success, true, |record| {
            expect_output_manifest(record, "http", &["response"])
        }),
        run_scenario_check("failure_schema", &failure, true, |record| {
            expect_failure(record, "http", "HTTP_STATUS_ERROR", "execution")
        }),
        run_scenario_check("adapter_identity_schema", &success, true, |record| {
            expect_identity_schema(record, descriptor)
        }),
    ]
}

fn file_transform_adapter_scenarios(descriptor: &AdapterDescriptor) -> Vec<AdapterScenarioResult> {
    let success_workspace = match ConformanceWorkspace::new("file-transform", "success") {
        Ok(workspace) => workspace,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let success = match execute_graph_record(
        &success_workspace,
        "file_transform",
        &file_transform_graph(
            json!({
                "operation": "copy",
                "source": "seed/source",
            }),
            vec![json!({"name": "artifact", "path": "artifact.txt"})],
            None,
        ),
        RuntimeConfig::default(),
    ) {
        Ok(record) => record,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let failure_workspace = match ConformanceWorkspace::new("file-transform", "failure") {
        Ok(workspace) => workspace,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    let failure = match execute_graph_record(
        &failure_workspace,
        "file_transform",
        &file_transform_graph(
            json!({
                "operation": "copy",
                "source": "seed/missing",
            }),
            vec![json!({"name": "artifact", "path": "artifact.txt"})],
            None,
        ),
        RuntimeConfig::default(),
    ) {
        Ok(record) => record,
        Err(error) => return canonical_execution_failure_suite(descriptor, error),
    };
    vec![
        run_scenario_check("success", &success, true, |record| {
            expect_success(record, "file_transform")
        }),
        run_scenario_check("failure", &failure, true, |record| {
            expect_failure(record, "file_transform", "EXEC_ERROR", "user")
        }),
        skip_scenario(
            "missing_output",
            false,
            true,
            "file_transform validates operation-specific output cardinality before generic runtime missing-output inspection",
        ),
        skip_scenario(
            "timeout",
            descriptor.supports_timeout,
            true,
            "file_transform timeout coverage remains adapter-specific and is not emitted by the generic conformance harness",
        ),
        run_scenario_check("output_manifest", &success, true, |record| {
            expect_output_manifest(record, "file_transform", &["artifact"])
        }),
        run_scenario_check("failure_schema", &failure, true, |record| {
            expect_failure(record, "file_transform", "EXEC_ERROR", "user")
        }),
        run_scenario_check("adapter_identity_schema", &success, true, |record| {
            expect_identity_schema(record, descriptor)
        }),
    ]
}

fn container_adapter_scenarios(descriptor: &AdapterDescriptor) -> Vec<AdapterScenarioResult> {
    canonical_skip_suite(
        descriptor,
        "container adapter conformance requires a repository-owned image fixture and remains intentionally skipped until that fixture is defined",
    )
}

fn external_adapter_scenarios(descriptor: &AdapterDescriptor) -> Vec<AdapterScenarioResult> {
    canonical_skip_suite(
        descriptor,
        "external adapters require adapter-specific fixtures before runtime-backed conformance can execute safely",
    )
}

fn unsupported_adapter_scenarios(descriptor: &AdapterDescriptor) -> Vec<AdapterScenarioResult> {
    canonical_skip_suite(
        descriptor,
        "no runtime-backed conformance fixture is registered for this adapter",
    )
}

fn canonical_skip_suite(
    descriptor: &AdapterDescriptor,
    reason: &str,
) -> Vec<AdapterScenarioResult> {
    vec![
        skip_scenario("success", true, false, reason),
        skip_scenario("failure", true, false, reason),
        skip_scenario("missing_output", true, false, reason),
        skip_scenario("timeout", descriptor.supports_timeout, !descriptor.supports_timeout, reason),
        skip_scenario("output_manifest", true, false, reason),
        skip_scenario("failure_schema", true, false, reason),
        skip_scenario("adapter_identity_schema", true, false, reason),
    ]
}

fn canonical_execution_failure_suite(
    descriptor: &AdapterDescriptor,
    error: String,
) -> Vec<AdapterScenarioResult> {
    vec![
        execution_error("success", error.clone()),
        execution_error("failure", error.clone()),
        execution_error("missing_output", error.clone()),
        execution_error("timeout", error.clone()),
        execution_error("output_manifest", error.clone()),
        execution_error("failure_schema", error.clone()),
        execution_error("adapter_identity_schema", error),
    ]
    .into_iter()
    .map(|mut scenario| {
        scenario.enforced_by_runtime =
            descriptor.supports_timeout || scenario.scenario != "timeout";
        scenario
    })
    .collect()
}
