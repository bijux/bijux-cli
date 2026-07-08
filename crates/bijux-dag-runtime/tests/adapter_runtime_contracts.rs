use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
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
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

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

fn python_graph(
    module: &str,
    function: &str,
    payload: Value,
    outputs: &[(&str, &str)],
    effects: &[&str],
    env_allowlist: &[&str],
    timeout_ms: Option<u64>,
) -> String {
    let effect_values: Vec<Value> =
        effects.iter().map(|effect| Value::String((*effect).to_string())).collect();
    let env_values: Vec<Value> =
        env_allowlist.iter().map(|key| Value::String((*key).to_string())).collect();
    let output_values = outputs
        .iter()
        .map(|(name, path)| serde_json::json!({"name": name, "path": path, "media_type":"application/json"}))
        .collect::<Vec<_>>();
    let mut params = serde_json::Map::new();
    params.insert("module".to_string(), Value::String(module.to_string()));
    params.insert("function".to_string(), Value::String(function.to_string()));
    if let Value::Object(entries) = payload {
        for (key, value) in entries {
            params.insert(key, value);
        }
    }
    let mut node = serde_json::json!({
        "id":"python",
        "kind":"python",
        "outputs": output_values,
        "params": Value::Object(params),
        "effects": effect_values,
        "env_allowlist": env_values,
    });
    if let Some(timeout_ms) = timeout_ms {
        node["timeout_ms"] = serde_json::json!(timeout_ms);
    }
    serde_json::json!({
        "spec":"bijux-dag/v0.1",
        "nodes":[node],
        "edges":[]
    })
    .to_string()
}

fn http_graph(
    method: &str,
    url: &str,
    headers: Option<Value>,
    body: Option<Value>,
    timeout_ms: Option<u64>,
    retry: Option<(u32, u64)>,
    nondeterminism_allowed: bool,
) -> String {
    let mut params = serde_json::Map::new();
    params.insert("method".to_string(), Value::String(method.to_string()));
    params.insert("url".to_string(), Value::String(url.to_string()));
    if let Some(headers) = headers {
        params.insert("headers".to_string(), headers);
    }
    if let Some(body) = body {
        params.insert("body".to_string(), body);
    }

    let mut node = serde_json::json!({
        "id":"http",
        "kind":"http",
        "outputs":[{"name":"response","path":"response.json","media_type":"application/json"}],
        "params": Value::Object(params),
        "effects":["filesystem", "network"],
    });
    if let Some(timeout_ms) = timeout_ms {
        node["timeout_ms"] = serde_json::json!(timeout_ms);
    }
    if let Some((max_attempts, backoff_ms)) = retry {
        node["retry"] = serde_json::json!({
            "max_attempts": max_attempts,
            "backoff_ms": backoff_ms,
        });
    }

    serde_json::json!({
        "spec":"bijux-dag/v0.1",
        "nondeterminism_allowed": nondeterminism_allowed,
        "nodes":[node],
        "edges":[]
    })
    .to_string()
}

fn file_transform_graph(
    seeds: &[(&str, &str, &str)],
    params: Value,
    outputs: &[Value],
    timeout_ms: Option<u64>,
) -> String {
    let mut nodes = seeds
        .iter()
        .map(|(node_id, output_name, contents)| {
            let command =
                format!("cat <<'BIJUX_EOF' > ../outputs/{output_name}.txt\n{contents}\nBIJUX_EOF");
            serde_json::json!({
                "id": node_id,
                "kind": "shell",
                "outputs": [{"name": output_name, "path": format!("{output_name}.txt")}],
                "params": {"argv": ["/bin/sh", "-c", command]},
                "effects": ["filesystem"],
            })
        })
        .collect::<Vec<_>>();
    let inputs = seeds
        .iter()
        .map(|(_, output_name, _)| Value::String((*output_name).to_string()))
        .collect::<Vec<_>>();
    let mut file_node = serde_json::json!({
        "id":"file_transform",
        "kind":"file_transform",
        "inputs": inputs,
        "outputs": outputs,
        "params": params,
        "effects": ["filesystem"],
    });
    if let Some(timeout_ms) = timeout_ms {
        file_node["timeout_ms"] = serde_json::json!(timeout_ms);
    }
    nodes.push(file_node);

    let edges = seeds
        .iter()
        .map(|(node_id, output_name, _)| {
            serde_json::json!({
                "from": {"node_id": node_id, "port": output_name},
                "to": {"node_id": "file_transform", "port": output_name},
            })
        })
        .collect::<Vec<_>>();

    serde_json::json!({
        "spec":"bijux-dag/v0.1",
        "nodes": nodes,
        "edges": edges,
    })
    .to_string()
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

fn read_operation_summary(run_dir: &Path, node_id: &str) -> Value {
    serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join(node_id).join("operation-summary.json"))
            .expect("operation summary"),
    )
    .expect("summary json")
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

fn write_python_module(path: &Path, contents: &str) {
    fs::write(path, contents).expect("write python module");
}

#[derive(Clone)]
struct ScriptedHttpResponse {
    status_line: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    delay_ms: u64,
}

#[derive(Debug, Clone)]
struct CapturedHttpRequest {
    method: String,
    path: String,
    headers: std::collections::BTreeMap<String, String>,
    body: Vec<u8>,
}

struct ScriptedHttpServer {
    base_url: String,
    requests: Arc<Mutex<Vec<CapturedHttpRequest>>>,
    join: Option<std::thread::JoinHandle<()>>,
}

impl ScriptedHttpServer {
    fn spawn(responses: Vec<ScriptedHttpResponse>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
        listener.set_nonblocking(false).expect("blocking listener");
        let address = listener.local_addr().expect("listener addr");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let requests_thread = Arc::clone(&requests);
        let join = thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().expect("accept");
                let request = read_http_request(&mut stream);
                requests_thread.lock().expect("requests").push(request);
                if response.delay_ms > 0 {
                    thread::sleep(Duration::from_millis(response.delay_ms));
                }
                write_http_response(&mut stream, &response);
            }
        });
        Self { base_url: format!("http://{}", address), requests, join: Some(join) }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn requests(&self) -> Vec<CapturedHttpRequest> {
        self.requests.lock().expect("requests").clone()
    }
}

impl Drop for ScriptedHttpServer {
    fn drop(&mut self) {
        if let Some(join) = self.join.take() {
            join.join().expect("server join");
        }
    }
}

fn read_http_request(stream: &mut TcpStream) -> CapturedHttpRequest {
    stream.set_read_timeout(Some(Duration::from_secs(2))).expect("read timeout");
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        let read = stream.read(&mut chunk).expect("read request");
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
        if let Some(end) = find_header_end(&buffer) {
            let content_length = parse_content_length(&buffer[..end]);
            if buffer.len() >= end + 4 + content_length {
                break;
            }
        }
    }

    let header_end = find_header_end(&buffer).expect("header end");
    let header_text = String::from_utf8(buffer[..header_end].to_vec()).expect("header text");
    let mut lines = header_text.split("\r\n");
    let request_line = lines.next().expect("request line");
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().expect("method").to_string();
    let path = request_parts.next().expect("path").to_string();
    let headers = lines
        .filter_map(|line| {
            let (name, value) = line.split_once(':')?;
            Some((name.trim().to_ascii_lowercase(), value.trim().to_string()))
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let content_length = parse_content_length(&buffer[..header_end]);
    let body = buffer[(header_end + 4)..(header_end + 4 + content_length)].to_vec();

    CapturedHttpRequest { method, path, headers, body }
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn parse_content_length(headers: &[u8]) -> usize {
    let header_text = String::from_utf8_lossy(headers);
    header_text
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0)
}

fn write_http_response(stream: &mut TcpStream, response: &ScriptedHttpResponse) {
    let mut response_bytes = format!("HTTP/1.1 {}\r\n", response.status_line).into_bytes();
    let mut has_content_length = false;
    for (name, value) in &response.headers {
        if name.eq_ignore_ascii_case("content-length") {
            has_content_length = true;
        }
        response_bytes.extend_from_slice(format!("{name}: {value}\r\n").as_bytes());
    }
    if !has_content_length {
        response_bytes
            .extend_from_slice(format!("Content-Length: {}\r\n", response.body.len()).as_bytes());
    }
    response_bytes.extend_from_slice(b"Connection: close\r\n\r\n");
    response_bytes.extend_from_slice(&response.body);
    stream.write_all(&response_bytes).expect("write response");
    stream.flush().expect("flush response");
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
            dynamic: None,
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
fn python_adapter_descriptor_exposes_timeout_cache_and_protocol_contracts() {
    let descriptors = registered_adapter_descriptors();
    let python =
        descriptors.iter().find(|descriptor| descriptor.id == "python").expect("python descriptor");
    assert_eq!(python.protocol_version, "bijux-dag-adapter/v1");
    assert!(python.supports_timeout);
    assert!(!python.supports_cancel);
    assert_eq!(
        python.cache_compatibility,
        bijux_dag_runtime::CacheCompatibilityMode::FingerprintExact
    );
}

#[test]
fn http_adapter_descriptor_exposes_timeout_cache_and_protocol_contracts() {
    let descriptors = registered_adapter_descriptors();
    let http =
        descriptors.iter().find(|descriptor| descriptor.id == "http").expect("http descriptor");
    assert_eq!(http.protocol_version, "bijux-dag-adapter/v1");
    assert!(http.supports_timeout);
    assert!(!http.supports_cancel);
    assert_eq!(
        http.cache_compatibility,
        bijux_dag_runtime::CacheCompatibilityMode::FingerprintExact
    );
    assert!(http.required_effects.filesystem);
    assert!(http.required_effects.network);
}

#[test]
fn file_transform_adapter_descriptor_exposes_timeout_cache_and_protocol_contracts() {
    let descriptors = registered_adapter_descriptors();
    let file_transform = descriptors
        .iter()
        .find(|descriptor| descriptor.id == "file_transform")
        .expect("file_transform descriptor");
    assert_eq!(file_transform.protocol_version, "bijux-dag-adapter/v1");
    assert!(file_transform.supports_timeout);
    assert!(!file_transform.supports_cancel);
    assert_eq!(
        file_transform.cache_compatibility,
        bijux_dag_runtime::CacheCompatibilityMode::FingerprintExact
    );
    assert!(file_transform.required_effects.filesystem);
    assert!(!file_transform.required_effects.network);
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
    let trace = read_trace(&run_dir);
    assert_eq!(trace["exit_code"], 0);
    assert_eq!(trace["stdout"]["path"], "nodes/shell/stdout.log");
    assert_eq!(trace["stdout"]["size_bytes"], 2);
    assert_eq!(trace["stdout"]["tail_lines"], serde_json::json!(["ok"]));
    assert_eq!(trace["stderr"]["path"], "nodes/shell/stderr.log");
    assert_eq!(trace["stderr"]["size_bytes"], 4);
    assert_eq!(trace["stderr"]["tail_lines"], serde_json::json!(["warn"]));
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
    assert_eq!(trace["exit_code"], 7);
    assert_eq!(trace["failure"]["code"], "EXEC_FAIL");
    assert_eq!(trace["failure"]["class"], "execution");
    assert_eq!(trace["failure"]["details"]["exit_code"], 7);
    assert_eq!(trace["stderr"]["tail_lines"], serde_json::json!(["boom"]));
}

#[test]
fn shell_adapter_limits_trace_tail_for_large_logs() {
    let graph = parse_graph_strict(&shell_graph(
        "printf 'ready' > ../outputs/value.txt; i=1; while [ $i -le 2500 ]; do printf 'stdout-%04d\\n' $i; printf 'stderr-%04d\\n' $i >&2; i=$((i + 1)); done",
        &["filesystem"],
    ))
    .expect("graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("tmpdir");
    let run_dir = runtime.run(&graph, temp.path(), RuntimeConfig::default()).expect("run");

    let stdout =
        fs::read_to_string(run_dir.join("nodes").join("shell").join("stdout.log")).expect("stdout");
    let stderr =
        fs::read_to_string(run_dir.join("nodes").join("shell").join("stderr.log")).expect("stderr");
    let attempt_stdout = fs::read_to_string(
        run_dir.join("nodes").join("shell").join("attempts").join("1").join("stdout.log"),
    )
    .expect("attempt stdout");
    let attempt_stderr = fs::read_to_string(
        run_dir.join("nodes").join("shell").join("attempts").join("1").join("stderr.log"),
    )
    .expect("attempt stderr");

    assert_eq!(attempt_stdout, stdout);
    assert_eq!(attempt_stderr, stderr);

    let trace = read_trace(&run_dir);
    assert_eq!(trace["stdout"]["size_bytes"].as_u64(), Some(stdout.len() as u64));
    assert_eq!(trace["stderr"]["size_bytes"].as_u64(), Some(stderr.len() as u64));
    assert_eq!(trace["stdout"]["tail_lines"][0], "stdout-2481");
    assert_eq!(trace["stdout"]["tail_lines"][19], "stdout-2500");
    assert_eq!(trace["stderr"]["tail_lines"][0], "stderr-2481");
    assert_eq!(trace["stderr"]["tail_lines"][19], "stderr-2500");
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
    assert_eq!(trace["failure"]["details"]["executable"], "definitely-missing-bijux-command");
    assert_eq!(trace["failure"]["details"]["io_error_kind"], "not_found");
}

#[test]
fn shell_adapter_rejects_non_array_argv_structurally() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"shell",
              "kind":"shell",
              "outputs":[{"name":"value","path":"value.txt"}],
              "params":{"argv":"not-an-array"},
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
    assert_eq!(trace["failure"]["code"], "EXEC_ERROR");
    assert_eq!(trace["failure"]["class"], "user");
    assert_eq!(trace["failure"]["details"]["field"], "argv");
    assert_eq!(trace["failure"]["details"]["reason"], "expected_array");
}

#[test]
fn shell_adapter_rejects_empty_argv_structurally() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"shell",
              "kind":"shell",
              "outputs":[{"name":"value","path":"value.txt"}],
              "params":{"argv":[]},
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
    assert_eq!(trace["failure"]["code"], "EXEC_ERROR");
    assert_eq!(trace["failure"]["class"], "user");
    assert_eq!(trace["failure"]["details"]["field"], "argv");
    assert_eq!(trace["failure"]["details"]["reason"], "empty");
}

#[test]
fn shell_adapter_rejects_blank_executable_structurally() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"shell",
              "kind":"shell",
              "outputs":[{"name":"value","path":"value.txt"}],
              "params":{"argv":["   ","-c","printf hi > ../outputs/value.txt"]},
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
    assert_eq!(trace["failure"]["code"], "EXEC_ERROR");
    assert_eq!(trace["failure"]["class"], "user");
    assert_eq!(trace["failure"]["details"]["field"], "argv");
    assert_eq!(trace["failure"]["details"]["reason"], "blank_executable");
    assert!(trace["failure"]["message"]
        .as_str()
        .expect("message")
        .contains("non-empty executable"));
}

#[test]
fn shell_adapter_executes_from_isolated_work_dir() {
    let graph = parse_graph_strict(&shell_graph(
        "pwd > ../outputs/value.txt; printf isolated > marker.txt",
        &["filesystem"],
    ))
    .expect("graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("tmpdir");
    let run_dir = runtime.run(&graph, temp.path(), RuntimeConfig::default()).expect("run");
    let work_dir = run_dir.join("nodes").join("shell").join("work");
    let observed_dir =
        fs::read_to_string(run_dir.join("nodes").join("shell").join("outputs").join("value.txt"))
            .expect("output");
    let observed_path = Path::new(observed_dir.trim());
    assert!(observed_path.ends_with(Path::new("nodes").join("shell").join("work")));
    assert_ne!(observed_path, std::env::current_dir().expect("current dir"));
    assert!(work_dir.join("marker.txt").exists());
    assert!(!run_dir.join("marker.txt").exists());
}

#[test]
fn shell_adapter_rejects_undeclared_outputs() {
    let graph = parse_graph_strict(&shell_graph(
        "printf ok > ../outputs/value.txt; printf extra > ../outputs/extra.txt",
        &["filesystem"],
    ))
    .expect("graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("tmpdir");
    let run_dir = runtime.run(&graph, temp.path(), RuntimeConfig::default()).expect("run");
    let trace = read_trace(&run_dir);
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["failure"]["code"], "OUTPUT_UNDECLARED");
    assert_eq!(trace["failure"]["class"], "user");
}

#[test]
fn shell_adapter_timeout_preserves_partial_stdout_and_stderr() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"shell",
              "kind":"shell",
              "outputs":[{"name":"value","path":"value.txt"}],
              "params":{"argv":["/bin/sh","-c","printf partial-out; printf partial-err >&2; sleep 1"]},
              "timeout_ms":50,
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
    assert_eq!(
        fs::read_to_string(run_dir.join("nodes").join("shell").join("stdout.log")).expect("stdout"),
        "partial-out"
    );
    let stderr =
        fs::read_to_string(run_dir.join("nodes").join("shell").join("stderr.log")).expect("stderr");
    assert!(stderr.starts_with("partial-err"));
    let trace = read_trace(&run_dir);
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["failure"]["code"], "EXEC_TIMEOUT");
    assert_eq!(trace["failure"]["class"], "timeout");
}

#[test]
fn python_adapter_writes_json_output_from_function_result() {
    let _lock = process_env_lock();
    let dir = tempfile::tempdir().expect("tmpdir");
    let module_dir = dir.path().join("python");
    fs::create_dir_all(&module_dir).expect("mkdir");
    write_python_module(
        &module_dir.join("demo_python_adapter.py"),
        "def emit(payload):\n    return {\"value\": payload[\"value\"], \"kind\": \"python\"}\n",
    );
    std::env::set_var("PYTHONPATH", &module_dir);

    let graph = parse_graph_strict(&python_graph(
        "demo_python_adapter",
        "emit",
        serde_json::json!({"value": {"left": 1, "right": 2}}),
        &[("result", "result.json")],
        &["filesystem", "env"],
        &["PYTHONPATH"],
        None,
    ))
    .expect("graph");
    let runtime = Runtime::new();
    let run_dir = runtime.run(&graph, dir.path(), RuntimeConfig::default()).expect("run");
    let payload: Value = serde_json::from_str(
        &fs::read_to_string(
            run_dir.join("nodes").join("python").join("outputs").join("result.json"),
        )
        .expect("output"),
    )
    .expect("json output");
    assert_eq!(payload["kind"], "python");
    assert_eq!(payload["value"]["left"], 1);
    let trace = read_node_trace(&run_dir, "python");
    assert_eq!(trace["status"], "success");
    std::env::remove_var("PYTHONPATH");
}

#[test]
fn python_adapter_maps_named_result_object_to_multiple_outputs() {
    let _lock = process_env_lock();
    let dir = tempfile::tempdir().expect("tmpdir");
    let module_dir = dir.path().join("python");
    fs::create_dir_all(&module_dir).expect("mkdir");
    write_python_module(
        &module_dir.join("split_python_adapter.py"),
        "def split(payload):\n    return {\"left\": payload[\"left\"], \"right\": payload[\"right\"]}\n",
    );
    std::env::set_var("PYTHONPATH", &module_dir);

    let graph = parse_graph_strict(&python_graph(
        "split_python_adapter",
        "split",
        serde_json::json!({"left": 1, "right": 2}),
        &[("left", "left.json"), ("right", "right.json")],
        &["filesystem", "env"],
        &["PYTHONPATH"],
        None,
    ))
    .expect("graph");
    let runtime = Runtime::new();
    let run_dir = runtime.run(&graph, dir.path(), RuntimeConfig::default()).expect("run");
    let left: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join("python").join("outputs").join("left.json"))
            .expect("left output"),
    )
    .expect("left json");
    let right: Value = serde_json::from_str(
        &fs::read_to_string(
            run_dir.join("nodes").join("python").join("outputs").join("right.json"),
        )
        .expect("right output"),
    )
    .expect("right json");
    assert_eq!(left, serde_json::json!(1));
    assert_eq!(right, serde_json::json!(2));
    std::env::remove_var("PYTHONPATH");
}

#[test]
fn python_adapter_structures_function_exceptions() {
    let _lock = process_env_lock();
    let dir = tempfile::tempdir().expect("tmpdir");
    let module_dir = dir.path().join("python");
    fs::create_dir_all(&module_dir).expect("mkdir");
    write_python_module(
        &module_dir.join("failing_python_adapter.py"),
        "def explode(payload):\n    raise ValueError(f\"bad {payload['value']}\")\n",
    );
    std::env::set_var("PYTHONPATH", &module_dir);

    let graph = parse_graph_strict(&python_graph(
        "failing_python_adapter",
        "explode",
        serde_json::json!({"value": "payload"}),
        &[("result", "result.json")],
        &["filesystem", "env"],
        &["PYTHONPATH"],
        None,
    ))
    .expect("graph");
    let runtime = Runtime::new();
    let run_dir = runtime.run(&graph, dir.path(), RuntimeConfig::default()).expect("run");
    let trace = read_node_trace(&run_dir, "python");
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["failure"]["code"], "PYTHON_EXCEPTION");
    assert_eq!(trace["failure"]["class"], "execution");
    assert_eq!(trace["failure"]["details"]["phase"], "call_function");
    assert_eq!(trace["failure"]["details"]["exception_type"], "ValueError");
    std::env::remove_var("PYTHONPATH");
}

#[test]
fn python_adapter_timeout_is_reported_structurally() {
    let _lock = process_env_lock();
    let dir = tempfile::tempdir().expect("tmpdir");
    let module_dir = dir.path().join("python");
    fs::create_dir_all(&module_dir).expect("mkdir");
    write_python_module(
        &module_dir.join("slow_python_adapter.py"),
        "import time\n\ndef sleep_then_emit(payload):\n    time.sleep(1)\n    return payload\n",
    );
    std::env::set_var("PYTHONPATH", &module_dir);

    let graph = parse_graph_strict(&python_graph(
        "slow_python_adapter",
        "sleep_then_emit",
        serde_json::json!({"value": "payload"}),
        &[("result", "result.json")],
        &["filesystem", "env"],
        &["PYTHONPATH"],
        Some(50),
    ))
    .expect("graph");
    let runtime = Runtime::new();
    let run_dir = runtime.run(&graph, dir.path(), RuntimeConfig::default()).expect("run");
    let trace = read_node_trace(&run_dir, "python");
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["failure"]["code"], "EXEC_TIMEOUT");
    assert_eq!(trace["failure"]["class"], "timeout");
    std::env::remove_var("PYTHONPATH");
}

#[test]
fn http_adapter_captures_response_status_headers_and_body() {
    let server = ScriptedHttpServer::spawn(vec![ScriptedHttpResponse {
        status_line: "200 OK".to_string(),
        headers: vec![("content-type".to_string(), "application/json".to_string())],
        body: br#"{"ok":true,"source":"adapter"}"#.to_vec(),
        delay_ms: 0,
    }]);
    let graph = parse_graph_strict(&http_graph(
        "POST",
        &server.url("/v1/run"),
        Some(serde_json::json!({"x-token":"secret"})),
        Some(serde_json::json!({"value": 7})),
        None,
        None,
        false,
    ))
    .expect("graph");

    let runtime = Runtime::new();
    let dir = tempfile::tempdir().expect("tmpdir");
    let run_dir = runtime.run(&graph, dir.path(), RuntimeConfig::default()).expect("run");
    let response: Value = serde_json::from_str(
        &fs::read_to_string(
            run_dir.join("nodes").join("http").join("outputs").join("response.json"),
        )
        .expect("response output"),
    )
    .expect("response json");
    assert_eq!(response["request"]["method"], "POST");
    assert_eq!(response["response"]["status"], 200);
    assert_eq!(response["response"]["body"]["json"]["ok"], true);
    assert_eq!(response["response"]["body"]["text"], "{\"ok\":true,\"source\":\"adapter\"}");

    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, "POST");
    assert_eq!(requests[0].path, "/v1/run");
    assert_eq!(requests[0].headers.get("x-token").map(String::as_str), Some("secret"));
    let request_body: Value = serde_json::from_slice(&requests[0].body).expect("request body");
    assert_eq!(request_body["value"], 7);
}

#[test]
fn http_adapter_structures_http_status_failures() {
    let server = ScriptedHttpServer::spawn(vec![ScriptedHttpResponse {
        status_line: "503 Service Unavailable".to_string(),
        headers: vec![("content-type".to_string(), "text/plain".to_string())],
        body: b"service down".to_vec(),
        delay_ms: 0,
    }]);
    let graph = parse_graph_strict(&http_graph(
        "GET",
        &server.url("/health"),
        None,
        None,
        None,
        None,
        false,
    ))
    .expect("graph");

    let runtime = Runtime::new();
    let dir = tempfile::tempdir().expect("tmpdir");
    let run_dir = runtime.run(&graph, dir.path(), RuntimeConfig::default()).expect("run");
    let trace = read_node_trace(&run_dir, "http");
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["failure"]["code"], "HTTP_STATUS_ERROR");
    assert_eq!(trace["failure"]["class"], "execution");
    assert_eq!(trace["failure"]["details"]["status"], 503);
    assert_eq!(trace["failure"]["details"]["response_body"]["text"], "service down");

    let response: Value = serde_json::from_str(
        &fs::read_to_string(
            run_dir.join("nodes").join("http").join("outputs").join("response.json"),
        )
        .expect("response output"),
    )
    .expect("response json");
    assert_eq!(response["response"]["status"], 503);
}

#[test]
fn http_adapter_timeout_is_reported_structurally() {
    let server = ScriptedHttpServer::spawn(vec![ScriptedHttpResponse {
        status_line: "200 OK".to_string(),
        headers: vec![("content-type".to_string(), "application/json".to_string())],
        body: br#"{"ok":true}"#.to_vec(),
        delay_ms: 200,
    }]);
    let graph = parse_graph_strict(&http_graph(
        "GET",
        &server.url("/slow"),
        None,
        None,
        Some(50),
        None,
        false,
    ))
    .expect("graph");

    let runtime = Runtime::new();
    let dir = tempfile::tempdir().expect("tmpdir");
    let run_dir = runtime.run(&graph, dir.path(), RuntimeConfig::default()).expect("run");
    let trace = read_node_trace(&run_dir, "http");
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["failure"]["code"], "EXEC_TIMEOUT");
    assert_eq!(trace["failure"]["class"], "timeout");
}

#[test]
fn http_adapter_retry_succeeds_on_second_attempt() {
    let server = ScriptedHttpServer::spawn(vec![
        ScriptedHttpResponse {
            status_line: "503 Service Unavailable".to_string(),
            headers: vec![("content-type".to_string(), "text/plain".to_string())],
            body: b"retry later".to_vec(),
            delay_ms: 0,
        },
        ScriptedHttpResponse {
            status_line: "200 OK".to_string(),
            headers: vec![("content-type".to_string(), "application/json".to_string())],
            body: br#"{"attempt":"second"}"#.to_vec(),
            delay_ms: 0,
        },
    ]);
    let graph = parse_graph_strict(&http_graph(
        "GET",
        &server.url("/flaky"),
        None,
        None,
        None,
        Some((1, 10)),
        true,
    ))
    .expect("graph");

    let runtime = Runtime::new();
    let dir = tempfile::tempdir().expect("tmpdir");
    let run_dir = runtime.run(&graph, dir.path(), RuntimeConfig::default()).expect("run");
    let trace = read_node_trace(&run_dir, "http");
    assert_eq!(trace["status"], "success");
    assert_eq!(trace["attempt"], 2);

    let response: Value = serde_json::from_str(
        &fs::read_to_string(
            run_dir.join("nodes").join("http").join("outputs").join("response.json"),
        )
        .expect("response output"),
    )
    .expect("response json");
    assert_eq!(response["response"]["body"]["json"]["attempt"], "second");

    let attempts: Vec<Value> = serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join("http").join("attempts.json"))
            .expect("attempts"),
    )
    .expect("attempts json");
    assert_eq!(attempts.len(), 2);
}

#[test]
fn http_adapter_network_policy_denial_is_structured() {
    let graph = parse_graph_strict(&http_graph(
        "GET",
        "http://127.0.0.1:1/policy",
        None,
        None,
        None,
        None,
        false,
    ))
    .expect("graph");

    let runtime = Runtime::new();
    let dir = tempfile::tempdir().expect("tmpdir");
    let run_dir = runtime
        .run(
            &graph,
            dir.path(),
            RuntimeConfig {
                policy: bijux_dag_runtime::PolicyConfig {
                    deny_network: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .expect("run");
    let trace = read_node_trace(&run_dir, "http");
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["failure"]["code"], "POLICY_DENIED");
    assert_eq!(trace["failure"]["details"]["effect"], "network");
}

#[test]
fn file_transform_copy_writes_output_and_structured_summary() {
    let graph = parse_graph_strict(&file_transform_graph(
        &[("seed", "source", "copy payload")],
        serde_json::json!({
            "operation": "copy",
            "source": "seed/source",
        }),
        &[serde_json::json!({"name":"copied","path":"copied.txt"})],
        None,
    ))
    .expect("graph");

    let runtime = Runtime::new();
    let dir = tempfile::tempdir().expect("tmpdir");
    let run_dir = runtime.run(&graph, dir.path(), RuntimeConfig::default()).expect("run");
    let copied = fs::read_to_string(
        run_dir.join("nodes").join("file_transform").join("outputs").join("copied.txt"),
    )
    .expect("copied output");
    assert_eq!(copied, "copy payload\n");

    let summary = read_operation_summary(&run_dir, "file_transform");
    assert_eq!(summary["operation"], "copy");
    assert_eq!(summary["sources"], serde_json::json!(["seed/source"]));
    assert_eq!(summary["outputs"][0]["path"], "copied.txt");
    assert!(summary["outputs"][0]["sha256"].as_str().is_some());
}

#[test]
fn file_transform_concatenate_preserves_source_order() {
    let graph = parse_graph_strict(&file_transform_graph(
        &[("left_seed", "left", "left"), ("right_seed", "right", "right")],
        serde_json::json!({
            "operation": "concatenate",
            "sources": ["left_seed/left", "right_seed/right"],
        }),
        &[serde_json::json!({"name":"joined","path":"joined.txt"})],
        None,
    ))
    .expect("graph");

    let runtime = Runtime::new();
    let dir = tempfile::tempdir().expect("tmpdir");
    let run_dir = runtime.run(&graph, dir.path(), RuntimeConfig::default()).expect("run");
    let joined = fs::read_to_string(
        run_dir.join("nodes").join("file_transform").join("outputs").join("joined.txt"),
    )
    .expect("joined output");
    assert_eq!(joined, "left\nright\n");
}

#[test]
fn file_transform_split_materializes_chunks_in_declared_order() {
    let graph = parse_graph_strict(&file_transform_graph(
        &[("seed", "source", "abcdefghi")],
        serde_json::json!({
            "operation": "split",
            "source": "seed/source",
            "chunk_bytes": 4,
        }),
        &[
            serde_json::json!({"name":"chunk_a","path":"chunk-a.txt"}),
            serde_json::json!({"name":"chunk_b","path":"chunk-b.txt"}),
            serde_json::json!({"name":"chunk_c","path":"chunk-c.txt"}),
        ],
        None,
    ))
    .expect("graph");

    let runtime = Runtime::new();
    let dir = tempfile::tempdir().expect("tmpdir");
    let run_dir = runtime.run(&graph, dir.path(), RuntimeConfig::default()).expect("run");
    let outputs_dir = run_dir.join("nodes").join("file_transform").join("outputs");
    assert_eq!(fs::read_to_string(outputs_dir.join("chunk-a.txt")).expect("chunk a"), "abcd");
    assert_eq!(fs::read_to_string(outputs_dir.join("chunk-b.txt")).expect("chunk b"), "efgh");
    assert_eq!(fs::read_to_string(outputs_dir.join("chunk-c.txt")).expect("chunk c"), "i\n");

    let summary = read_operation_summary(&run_dir, "file_transform");
    assert_eq!(summary["chunk_bytes"], 4);
    assert_eq!(summary["outputs"][0]["source_offset"], 0);
    assert_eq!(summary["outputs"][1]["source_offset"], 4);
    assert_eq!(summary["outputs"][2]["source_offset"], 8);
}

#[test]
fn file_transform_gzip_roundtrip_preserves_source_bytes() {
    let graph = parse_graph_strict(
        &serde_json::json!({
            "spec":"bijux-dag/v0.1",
            "nodes":[
                {
                    "id":"seed",
                    "kind":"shell",
                    "outputs":[{"name":"source","path":"source.txt"}],
                    "params":{"argv":["/bin/sh","-c","cat <<'BIJUX_EOF' > ../outputs/source.txt\ncompressed payload\nBIJUX_EOF"]},
                    "effects":["filesystem"]
                },
                {
                    "id":"compress",
                    "kind":"file_transform",
                    "inputs":["source"],
                    "outputs":[{"name":"archive","path":"archive.txt.gz"}],
                    "params":{"operation":"gzip_compress","source":"seed/source"},
                    "effects":["filesystem"]
                },
                {
                    "id":"decompress",
                    "kind":"file_transform",
                    "inputs":["archive"],
                    "outputs":[{"name":"plain","path":"plain.txt"}],
                    "params":{"operation":"gzip_decompress","source":"compress/archive"},
                    "effects":["filesystem"]
                }
            ],
            "edges":[
                {"from":{"node_id":"seed","port":"source"},"to":{"node_id":"compress","port":"source"}},
                {"from":{"node_id":"compress","port":"archive"},"to":{"node_id":"decompress","port":"archive"}}
            ]
        })
        .to_string(),
    )
    .expect("graph");

    let runtime = Runtime::new();
    let dir = tempfile::tempdir().expect("tmpdir");
    let run_dir = runtime.run(&graph, dir.path(), RuntimeConfig::default()).expect("run");
    let plain = fs::read_to_string(
        run_dir.join("nodes").join("decompress").join("outputs").join("plain.txt"),
    )
    .expect("plain output");
    assert_eq!(plain, "compressed payload\n");
}

#[test]
fn file_transform_checksum_emits_structured_json_artifact() {
    let graph = parse_graph_strict(&file_transform_graph(
        &[("seed", "source", "checksum payload")],
        serde_json::json!({
            "operation": "checksum",
            "source": "seed/source",
            "checksum_algorithm": "sha256",
        }),
        &[serde_json::json!({
            "name":"digest",
            "path":"digest.json",
            "media_type":"application/json"
        })],
        None,
    ))
    .expect("graph");

    let runtime = Runtime::new();
    let dir = tempfile::tempdir().expect("tmpdir");
    let run_dir = runtime.run(&graph, dir.path(), RuntimeConfig::default()).expect("run");
    let digest: Value = serde_json::from_str(
        &fs::read_to_string(
            run_dir.join("nodes").join("file_transform").join("outputs").join("digest.json"),
        )
        .expect("digest output"),
    )
    .expect("digest json");
    let expected_sha256 = {
        use sha2::Digest as _;
        hex::encode(sha2::Sha256::digest(b"checksum payload\n"))
    };
    assert_eq!(digest["operation"], "checksum");
    assert_eq!(digest["algorithm"], "sha256");
    assert_eq!(digest["sha256"], expected_sha256);
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
    assert_eq!(trace["exit_code"], 143);
    assert_eq!(trace["failure"]["code"], "EXEC_TIMEOUT");
    assert_eq!(trace["failure"]["class"], "timeout");
    assert_eq!(trace["stdout"]["tail_lines"], serde_json::json!(["partial-stdout"]));
    assert_eq!(trace["stderr"]["path"], "nodes/container/stderr.log");
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
fn external_adapter_failure_envelope_is_mapped_structurally() {
    let _lock = process_env_lock();
    let dir = tempfile::tempdir().expect("tmpdir");
    let adapter_dir = dir.path().join("adapters");
    fs::create_dir_all(&adapter_dir).expect("mkdir");
    let adapter_path = adapter_dir.join("failing-adapter");
    fs::write(
        &adapter_path,
        r#"#!/bin/sh
if [ "$1" = "info" ]; then
  echo '{"protocol_version":"bijux-dag-adapter/v1","adapter_id":"failing","adapter_version":"0.1","required_effects":{"filesystem":true,"env":false,"network":false,"clock":false},"supported_kinds":["fake"],"output_schema":"v0.1"}'
  exit 0
fi
if [ "$1" = "execute" ]; then
  failure_path=""
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --failure-path) failure_path="$2"; shift 2;;
      --outdir|--workdir|--node-spec) shift 2;;
      *) shift;;
    esac
  done
  cat > "$failure_path" <<'JSON'
{"class":"execution","kind":"Execution","code":"REMOTE_DATA_SOURCE_REJECTED","message":"upstream rejected the request","details":{"reason":"quota_exhausted","retryable":false}}
JSON
  printf 'adapter stderr\n' >&2
  exit 9
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
    let graph = external_graph("fake", None);
    let runtime = Runtime::new();
    let run_dir = runtime.run(&graph, dir.path(), RuntimeConfig::default()).expect("run");
    std::env::remove_var("BIJUX_DAG_ADAPTERS_DIR");

    let trace: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join("n1").join("trace.json")).expect("trace"),
    )
    .expect("trace json");
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["exit_code"], 9);
    assert_eq!(trace["failure"]["code"], "REMOTE_DATA_SOURCE_REJECTED");
    assert_eq!(trace["failure"]["class"], "execution");
    assert_eq!(trace["failure"]["details"]["reason"], "quota_exhausted");
    assert_eq!(trace["failure"]["details"]["retryable"], false);
    assert_eq!(trace["failure"]["details"]["exit_code"], 9);
    assert_eq!(trace["stderr"]["tail_lines"], serde_json::json!(["adapter stderr"]));
}

#[test]
fn external_adapter_binary_change_forces_cache_miss_and_rerun() {
    let _lock = process_env_lock();
    let dir = tempfile::tempdir().expect("tmpdir");
    let cache_dir = dir.path().join("cache");
    let adapter_dir = dir.path().join("adapters");
    fs::create_dir_all(&adapter_dir).expect("mkdir");
    let adapter_path = adapter_dir.join("fake-adapter");
    let runtime_config = RuntimeConfig {
        cache_mode: bijux_dag_runtime::CacheMode::ReadWrite,
        cache_dir: Some(cache_dir.clone()),
        ..RuntimeConfig::default()
    };

    let write_adapter = |payload: &str| {
        fs::write(
            &adapter_path,
            format!(
                r#"#!/bin/sh
if [ "$1" = "info" ]; then
  echo '{{"protocol_version":"bijux-dag-adapter/v1","adapter_id":"fake","adapter_version":"0.1","required_effects":{{"filesystem":true,"env":false,"network":false,"clock":false}},"supported_kinds":["fake"],"output_schema":"v0.1"}}'
  exit 0
fi
if [ "$1" = "execute" ]; then
  outdir=""
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --outdir) outdir="$2"; shift 2;;
      --workdir|--node-spec|--failure-path) shift 2;;
      *) shift;;
    esac
  done
  mkdir -p "$outdir"
  printf '%s' '{payload}' > "$outdir/out"
  exit 0
fi
exit 1
"#
            ),
        )
        .expect("write adapter");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&adapter_path).expect("meta").permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&adapter_path, perms).expect("chmod");
        }
    };

    std::env::set_var("BIJUX_DAG_ADAPTERS_DIR", &adapter_dir);
    write_adapter("first");
    let graph = external_graph("fake", None);
    let first_run = Runtime::new()
        .run(&graph, dir.path(), runtime_config.clone())
        .expect("first run");
    assert_eq!(
        fs::read_to_string(first_run.join("nodes").join("n1").join("outputs").join("out"))
            .expect("first output"),
        "first"
    );

    write_adapter("second");
    let second_run = Runtime::new()
        .run(&graph, dir.path(), runtime_config)
        .expect("second run");
    std::env::remove_var("BIJUX_DAG_ADAPTERS_DIR");

    assert_eq!(
        fs::read_to_string(second_run.join("nodes").join("n1").join("outputs").join("out"))
            .expect("second output"),
        "second"
    );
    let cache_entries = fs::read_dir(&cache_dir)
        .expect("cache entries")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            entry.path().join("meta.json").exists().then_some(entry.path())
        })
        .count();
    assert_eq!(cache_entries, 2);
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
fn adapter_conformance_suite_covers_shell_hardening_and_output_contract_scenarios() {
    let suites = adapter_conformance_suite().expect("suite");
    let file_transform =
        suites.iter().find(|suite| suite.adapter_id == "file_transform").expect("file_transform");
    let http = suites.iter().find(|suite| suite.adapter_id == "http").expect("http");
    let shell = suites.iter().find(|suite| suite.adapter_id == "shell").expect("shell");
    let python = suites.iter().find(|suite| suite.adapter_id == "python").expect("python");
    let file_transform_scenarios = &file_transform.scenarios;
    let http_scenarios = &http.scenarios;
    let python_scenarios = &python.scenarios;
    assert!(file_transform_scenarios.iter().any(|scenario| {
        scenario.scenario == "success"
            && scenario.status == bijux_dag_runtime::AdapterScenarioStatus::Pass
    }));
    assert!(file_transform_scenarios.iter().any(|scenario| {
        scenario.scenario == "failure"
            && scenario.status == bijux_dag_runtime::AdapterScenarioStatus::Pass
    }));
    assert!(file_transform_scenarios.iter().any(|scenario| {
        scenario.scenario == "cache_output"
            && scenario.status == bijux_dag_runtime::AdapterScenarioStatus::Pass
    }));
    assert!(file_transform_scenarios.iter().any(|scenario| {
        scenario.scenario == "missing_executable"
            && scenario.status == bijux_dag_runtime::AdapterScenarioStatus::Skip
    }));
    assert!(shell.scenarios.iter().any(|scenario| scenario.scenario == "argv_contract"));
    let argv_contract = shell
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario == "argv_contract")
        .expect("shell argv contract");
    assert!(argv_contract.reason.contains("non-blank executable"));
    assert!(shell.scenarios.iter().any(|scenario| scenario.scenario == "timeout"));
    assert!(shell.scenarios.iter().any(|scenario| scenario.scenario == "undeclared_output"));
    assert!(shell.scenarios.iter().any(|scenario| scenario.scenario == "workdir_isolation"));
    assert!(shell.scenarios.iter().any(|scenario| scenario.scenario == "missing_executable"));
    assert!(shell.scenarios.iter().any(|scenario| scenario.scenario == "cache_output"));
    assert!(shell.scenarios.iter().any(|scenario| scenario.scenario == "non_utf8_output"));
    assert!(python_scenarios.iter().any(|scenario| scenario.scenario == "timeout"));
    assert!(python_scenarios.iter().any(|scenario| {
        scenario.scenario == "failure"
            && scenario.status == bijux_dag_runtime::AdapterScenarioStatus::Pass
    }));
    assert!(python_scenarios.iter().any(|scenario| {
        scenario.scenario == "env_policy"
            && scenario.status == bijux_dag_runtime::AdapterScenarioStatus::Pass
    }));
    assert!(python_scenarios.iter().any(|scenario| {
        scenario.scenario == "workdir_isolation"
            && scenario.status == bijux_dag_runtime::AdapterScenarioStatus::Pass
    }));
    assert!(python_scenarios.iter().any(|scenario| {
        scenario.scenario == "missing_executable"
            && scenario.status == bijux_dag_runtime::AdapterScenarioStatus::Pass
    }));
    assert!(http_scenarios.iter().any(|scenario| {
        scenario.scenario == "failure"
            && scenario.status == bijux_dag_runtime::AdapterScenarioStatus::Pass
    }));
    assert!(http_scenarios.iter().any(|scenario| {
        scenario.scenario == "timeout"
            && scenario.status == bijux_dag_runtime::AdapterScenarioStatus::Pass
    }));
    assert!(http_scenarios.iter().any(|scenario| {
        scenario.scenario == "missing_executable"
            && scenario.status == bijux_dag_runtime::AdapterScenarioStatus::Skip
    }));
}
