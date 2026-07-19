use crate::commands::DagCli;
use crate::node_execution_explanation::{
    explain_node_execution, format_node_execution_explanation_human, NodeExecutionExplanation,
};
use crate::routes::path_resolution::{
    manifest_path, node_attempts_path, node_inputs_index_path, node_outputs_index_path,
    node_resolved_params_path, node_stderr_path, node_stdout_path, node_trace_path,
};
use crate::routes::preconditions::require_run_directory;
use crate::routes::run_lookup::read_manifest_json;
use crate::run_data::{load_snapshot, read_node_traces};
use crate::{emit_json, read_file, ExitCode};
use bijux_dag_artifacts::{InputFile, InputsIndex, NodeTrace, OutputFile, OutputsIndex};
use bijux_dag_core::{node_io_contract, CacheBehavior, Graph, Node, NodeIoContract};
use bijux_dag_runtime::AttemptEvent;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const NODE_LOG_TAIL_LINE_LIMIT: usize = 20;
const NODE_LOG_TAIL_READ_BYTES: u64 = 16 * 1024;

#[derive(Debug, Serialize)]
struct NodeLogInspection {
    path: String,
    size_bytes: u64,
    tail: Vec<String>,
}

#[derive(Debug, Serialize)]
struct NodeAttemptInspection {
    attempt: u32,
    started_unix_ms: u128,
    finished_unix_ms: u128,
    status: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    stdout: Option<NodeLogInspection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stderr: Option<NodeLogInspection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scheduled_backoff_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
struct NodeLogsInspection {
    #[serde(skip_serializing_if = "Option::is_none")]
    stdout: Option<NodeLogInspection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stderr: Option<NodeLogInspection>,
}

#[derive(Debug, Serialize)]
struct NodeCacheInspection {
    configured: CacheBehavior,
    observed_result: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    identity: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    proof: Option<Value>,
}

#[derive(Debug, Serialize)]
struct NodeFailureInspection {
    #[serde(skip_serializing_if = "Option::is_none")]
    failure: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    skip_reason: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transition_cause: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lifecycle_state: Option<String>,
}

#[derive(Debug, Serialize)]
struct NodeInspectionPayload {
    run_dir: String,
    node_id: String,
    status: String,
    planned: Node,
    dependencies: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    io_contract: Option<NodeIoContract>,
    resolved_params: Value,
    input_artifacts: Vec<InputFile>,
    output_artifacts: Vec<OutputFile>,
    terminal_attempt: u32,
    attempts: Vec<NodeAttemptInspection>,
    logs: NodeLogsInspection,
    cache: NodeCacheInspection,
    failure: NodeFailureInspection,
    execution_explanation: NodeExecutionExplanation,
    fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    evidence_gaps: Vec<String>,
}

fn concise_explain_human(
    status: &Value,
    graph_fp: &Value,
    counts: &Value,
    failed: &[String],
) -> String {
    format!(
        "status: {status}\ngraph_fingerprint: {graph_fp}\nnode_counts: {counts}\nfailed_nodes: {failed:?}"
    )
}

fn parse_optional_json(content: &str) -> Option<Value> {
    serde_json::from_str::<Value>(content).ok()
}

fn render_cache_policy(cache: &CacheBehavior) -> String {
    if cache.enabled {
        "enabled".to_string()
    } else {
        format!("disabled (reason: {})", cache.reason.as_deref().unwrap_or("unspecified"))
    }
}

fn read_required_json_file<T: DeserializeOwned>(path: &Path) -> Result<T, ExitCode> {
    let raw = fs::read_to_string(path).map_err(|_| ExitCode::from(3))?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

fn relative_run_path(run_dir: &Path, path: &Path) -> String {
    path.strip_prefix(run_dir).unwrap_or(path).display().to_string()
}

fn read_optional_json_file<T: DeserializeOwned>(
    run_dir: &Path,
    path: &Path,
    label: &str,
    evidence_gaps: &mut Vec<String>,
) -> Option<T> {
    if !path.exists() {
        evidence_gaps.push(format!("missing {label}: {}", relative_run_path(run_dir, path)));
        return None;
    }
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(_) => {
            evidence_gaps.push(format!("unreadable {label}: {}", relative_run_path(run_dir, path)));
            return None;
        }
    };
    match serde_json::from_str(&raw) {
        Ok(value) => Some(value),
        Err(_) => {
            evidence_gaps.push(format!("invalid {label}: {}", relative_run_path(run_dir, path)));
            None
        }
    }
}

fn read_optional_log_inspection(
    run_dir: &Path,
    path: &Path,
    label: &str,
    evidence_gaps: &mut Vec<String>,
) -> Option<NodeLogInspection> {
    if !path.exists() {
        evidence_gaps.push(format!("missing {label}: {}", relative_run_path(run_dir, path)));
        return None;
    }
    let size_bytes = match fs::metadata(path) {
        Ok(metadata) => metadata.len(),
        Err(_) => {
            evidence_gaps.push(format!("unreadable {label}: {}", relative_run_path(run_dir, path)));
            return None;
        }
    };
    let tail = match read_log_tail_lines(path, NODE_LOG_TAIL_LINE_LIMIT, NODE_LOG_TAIL_READ_BYTES) {
        Ok(tail) => tail,
        Err(_) => {
            evidence_gaps.push(format!("unreadable {label}: {}", relative_run_path(run_dir, path)));
            return None;
        }
    };
    Some(NodeLogInspection { path: relative_run_path(run_dir, path), size_bytes, tail })
}

fn read_log_tail_lines(
    path: &Path,
    max_lines: usize,
    max_bytes: u64,
) -> std::io::Result<Vec<String>> {
    let mut file = fs::File::open(path)?;
    let file_len = file.metadata()?.len();
    let start = file_len.saturating_sub(max_bytes);
    file.seek(SeekFrom::Start(start))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let content = String::from_utf8_lossy(&buffer);
    let mut tail = content.lines().map(ToString::to_string).collect::<Vec<_>>();
    if start > 0 && !content.starts_with('\n') && !tail.is_empty() {
        tail.remove(0);
    }
    if tail.len() > max_lines {
        tail = tail.split_off(tail.len() - max_lines);
    }
    Ok(tail)
}

fn log_inspection_from_trace(evidence: &bijux_dag_artifacts::NodeLogEvidence) -> NodeLogInspection {
    NodeLogInspection {
        path: evidence.path.clone(),
        size_bytes: evidence.size_bytes,
        tail: evidence.tail_lines.clone(),
    }
}

fn serialize_optional<T: Serialize>(value: Option<T>) -> Option<Value> {
    value.and_then(|value| serde_json::to_value(value).ok())
}

fn attempt_status_value(attempt: &AttemptEvent) -> Value {
    serde_json::to_value(&attempt.status).unwrap_or(Value::Null)
}

fn node_cache_result(node: &Node, trace: &NodeTrace) -> String {
    if !node.cache.enabled {
        return "disabled".to_string();
    }
    if trace.status.eq_ignore_ascii_case("cached") {
        return "hit".to_string();
    }
    if trace.cache_identity.is_some() || trace.cache_proof.is_some() {
        return "evaluated_without_reuse".to_string();
    }
    "not_reused".to_string()
}

fn node_inspection_payload(
    run_dir: &Path,
    node_id: &str,
) -> Result<NodeInspectionPayload, ExitCode> {
    require_run_directory(run_dir)?;
    let snapshot = load_snapshot(run_dir)?;
    let trace: NodeTrace = read_required_json_file(&node_trace_path(run_dir, node_id))?;
    let trace_json = serde_json::to_value(&trace).map_err(|_| ExitCode::from(3))?;
    let planned = snapshot
        .graph
        .nodes
        .iter()
        .find(|node| node.id == node_id)
        .cloned()
        .ok_or(ExitCode::from(3))?;
    let dependencies = snapshot
        .graph
        .edges
        .iter()
        .filter(|edge| edge.to.node_id == node_id)
        .map(|edge| edge.from.node_id.clone())
        .collect::<Vec<_>>();
    let io_contract = node_io_contract(&snapshot.graph, node_id);
    let mut evidence_gaps = Vec::new();
    let resolved_params = read_optional_json_file::<Value>(
        run_dir,
        &node_resolved_params_path(run_dir, node_id),
        "resolved params",
        &mut evidence_gaps,
    )
    .unwrap_or(Value::Null);
    let input_artifacts = read_optional_json_file::<InputsIndex>(
        run_dir,
        &node_inputs_index_path(run_dir, node_id),
        "input artifact index",
        &mut evidence_gaps,
    )
    .map(|index| index.files)
    .unwrap_or_default();
    let output_artifacts = read_optional_json_file::<OutputsIndex>(
        run_dir,
        &node_outputs_index_path(run_dir, node_id),
        "output artifact index",
        &mut evidence_gaps,
    )
    .map(|index| index.files)
    .unwrap_or_default();
    let attempts = read_optional_json_file::<Vec<AttemptEvent>>(
        run_dir,
        &node_attempts_path(run_dir, node_id),
        "attempt history",
        &mut evidence_gaps,
    )
    .unwrap_or_default()
    .into_iter()
    .map(|attempt| NodeAttemptInspection {
        attempt: attempt.attempt,
        started_unix_ms: attempt.started_unix_ms,
        finished_unix_ms: attempt.finished_unix_ms,
        status: attempt_status_value(&attempt),
        stdout: attempt
            .stdout_path
            .as_ref()
            .map(|path| run_dir.join("nodes").join(node_id).join(path))
            .and_then(|path| {
                read_optional_log_inspection(
                    run_dir,
                    &path,
                    "attempt stdout log",
                    &mut evidence_gaps,
                )
            }),
        stderr: attempt
            .stderr_path
            .as_ref()
            .map(|path| run_dir.join("nodes").join(node_id).join(path))
            .and_then(|path| {
                read_optional_log_inspection(
                    run_dir,
                    &path,
                    "attempt stderr log",
                    &mut evidence_gaps,
                )
            }),
        failure: serialize_optional(attempt.failure),
        scheduled_backoff_ms: attempt.scheduled_backoff_ms,
    })
    .collect::<Vec<_>>();
    let logs = NodeLogsInspection {
        stdout: trace.stdout.as_ref().map(log_inspection_from_trace).or_else(|| {
            read_optional_log_inspection(
                run_dir,
                &node_stdout_path(run_dir, node_id),
                "stdout log",
                &mut evidence_gaps,
            )
        }),
        stderr: trace.stderr.as_ref().map(log_inspection_from_trace).or_else(|| {
            read_optional_log_inspection(
                run_dir,
                &node_stderr_path(run_dir, node_id),
                "stderr log",
                &mut evidence_gaps,
            )
        }),
    };
    let cache = NodeCacheInspection {
        configured: planned.cache.clone(),
        observed_result: node_cache_result(&planned, &trace),
        identity: serialize_optional(trace.cache_identity.clone()),
        proof: serialize_optional(trace.cache_proof.clone()),
    };
    let failure = NodeFailureInspection {
        failure: serialize_optional(trace.failure.clone()),
        exit_code: trace.exit_code,
        skip_reason: serialize_optional(trace.skip_reason.clone()),
        transition_cause: trace.transition_cause.clone(),
        lifecycle_state: trace.lifecycle_state.clone(),
    };
    let execution_explanation =
        explain_node_execution(run_dir, &snapshot.graph, node_id, Some(&trace_json));

    Ok(NodeInspectionPayload {
        run_dir: run_dir.display().to_string(),
        node_id: node_id.to_string(),
        status: trace.status.clone(),
        planned: planned.clone(),
        dependencies,
        io_contract,
        resolved_params,
        input_artifacts,
        output_artifacts,
        terminal_attempt: trace.attempt,
        attempts,
        logs,
        cache,
        failure,
        execution_explanation,
        fingerprint: snapshot.graph.node_fingerprint(&planned).ok(),
        evidence_gaps,
    })
}

fn format_node_inspection_human(payload: &NodeInspectionPayload) -> String {
    let planned_inputs =
        serde_json::to_string(&payload.planned.inputs).unwrap_or_else(|_| "[]".to_string());
    let planned_outputs =
        payload.planned.outputs.iter().map(|output| output.name.clone()).collect::<Vec<_>>();
    let attempt_summary = if payload.attempts.is_empty() {
        "[]".to_string()
    } else {
        payload
            .attempts
            .iter()
            .map(|attempt| {
                let status = attempt
                    .status
                    .as_str()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| attempt.status.to_string());
                format!(
                    "attempt={} status={} backoff_ms={}",
                    attempt.attempt,
                    status,
                    attempt
                        .scheduled_backoff_ms
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "-".to_string())
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let failure_summary = payload
        .failure
        .failure
        .as_ref()
        .map(|failure| serde_json::to_string(failure).unwrap_or_else(|_| "null".to_string()))
        .unwrap_or_else(|| "null".to_string());
    let exit_code =
        payload.failure.exit_code.map(|value| value.to_string()).unwrap_or_else(|| "-".to_string());
    let stdout_tail =
        payload.logs.stdout.as_ref().map(|log| log.tail.join("\n")).unwrap_or_default();
    let stderr_tail =
        payload.logs.stderr.as_ref().map(|log| log.tail.join("\n")).unwrap_or_default();
    let stdout_size_bytes = payload
        .logs
        .stdout
        .as_ref()
        .map(|log| log.size_bytes.to_string())
        .unwrap_or_else(|| "-".to_string());
    let stderr_size_bytes = payload
        .logs
        .stderr
        .as_ref()
        .map(|log| log.size_bytes.to_string())
        .unwrap_or_else(|| "-".to_string());
    let evidence_gaps = if payload.evidence_gaps.is_empty() {
        "[]".to_string()
    } else {
        payload.evidence_gaps.join("; ")
    };
    format!(
        "node: {}\nstatus: {}\nplanned_kind: {}\nplanned_inputs: {}\nplanned_outputs: {:?}\nresolved_params: {}\ninput_artifact_count: {}\noutput_artifact_count: {}\nterminal_attempt: {}\nattempts:\n{}\ncache_status: configured={} observed={}\nfailure_info: {}\nexit_code: {}\nexecution_explanation: {}\nstdout_path: {}\nstdout_size_bytes: {}\nstderr_path: {}\nstderr_size_bytes: {}\nstdout_tail:\n{}\nstderr_tail:\n{}\nevidence_gaps: {}",
        payload.node_id,
        payload.status,
        payload.planned.kind.as_str(),
        planned_inputs,
        planned_outputs,
        payload.resolved_params,
        payload.input_artifacts.len(),
        payload.output_artifacts.len(),
        payload.terminal_attempt,
        attempt_summary,
        render_cache_policy(&payload.cache.configured),
        payload.cache.observed_result,
        failure_summary,
        exit_code,
        format_node_execution_explanation_human(&payload.execution_explanation),
        payload.logs.stdout.as_ref().map(|log| log.path.as_str()).unwrap_or("-"),
        stdout_size_bytes,
        payload.logs.stderr.as_ref().map(|log| log.path.as_str()).unwrap_or("-"),
        stderr_size_bytes,
        stdout_tail,
        stderr_tail,
        evidence_gaps,
    )
}

fn explain_node_payload(
    manifest: &str,
    graph: &Graph,
    node: &Node,
    node_id: &str,
    deps: Vec<String>,
    trace: Option<&str>,
    execution_explanation: &NodeExecutionExplanation,
    outputs_index: Option<&str>,
    resolved_params: Option<&str>,
) -> Value {
    let io_contract = node_io_contract(graph, node_id);
    let effective_inputs = graph.effective_inputs().unwrap_or_default();
    json!({
        "manifest": parse_optional_json(manifest),
        "node": node_id,
        "deps": deps,
        "graph_inputs": effective_inputs,
        "graph_input_schema": graph.input_schema(),
        "inputs": node.inputs.clone(),
        "input_bindings": io_contract.as_ref().map(|contract| contract.inputs.clone()),
        "outputs": node.outputs.clone(),
        "output_contracts": io_contract.as_ref().map(|contract| contract.outputs.clone()),
        "param_bindings": io_contract.as_ref().map(|contract| contract.param_bindings.clone()),
        "effects": node.effects.clone(),
        "cache": node.cache.clone(),
        "env_allowlist": node.env_allowlist.clone(),
        "outputs_index": outputs_index.and_then(parse_optional_json),
        "resolved_params": resolved_params.and_then(parse_optional_json),
        "execution_explanation": execution_explanation,
        "trace": trace.and_then(parse_optional_json),
        "fingerprint": graph.node_fingerprint(node).ok(),
    })
}

pub(crate) fn handle_explain_command(
    cli: &DagCli,
    run_dir: &Path,
    node: &Option<String>,
) -> Result<ExitCode, ExitCode> {
    require_run_directory(run_dir)?;
    let manifest = read_file(&manifest_path(run_dir))?;
    if let Some(node_id) = node.as_ref() {
        let snapshot = load_snapshot(run_dir)?;
        let trace = read_file(&node_trace_path(run_dir, node_id)).ok();
        let trace_json = trace
            .as_deref()
            .map(|raw| serde_json::from_str::<Value>(raw).map_err(|_| ExitCode::from(3)))
            .transpose()?;
        let node_info =
            snapshot.graph.nodes.iter().find(|n| n.id == *node_id).ok_or(ExitCode::from(3))?;
        let deps = snapshot
            .graph
            .edges
            .iter()
            .filter(|e| e.to.node_id == *node_id)
            .map(|e| e.from.node_id.clone())
            .collect::<Vec<_>>();
        let outputs_index = read_file(&node_outputs_index_path(run_dir, node_id)).ok();
        let resolved_params = read_file(&node_resolved_params_path(run_dir, node_id)).ok();
        let execution_explanation =
            explain_node_execution(run_dir, &snapshot.graph, node_id, trace_json.as_ref());
        if cli.json {
            let data = explain_node_payload(
                &manifest,
                &snapshot.graph,
                node_info,
                node_id,
                deps,
                trace.as_deref(),
                &execution_explanation,
                outputs_index.as_deref(),
                resolved_params.as_deref(),
            );
            return emit_json(cli, "dag.explain", true, data, Vec::new(), ExitCode::SUCCESS);
        } else {
            println!("node: {}", node_id);
            println!("deps: {:?}", deps);
            println!("graph_inputs: {:?}", snapshot.graph.effective_inputs().unwrap_or_default());
            println!("graph_input_schema: {:?}", snapshot.graph.input_schema());
            println!("inputs: {:?}", node_info.inputs);
            if let Some(io_contract) = node_io_contract(&snapshot.graph, node_id) {
                println!("input_bindings: {:?}", io_contract.inputs);
                println!("param_bindings: {:?}", io_contract.param_bindings);
                println!("output_contracts: {:?}", io_contract.outputs);
            }
            println!("outputs: {:?}", node_info.outputs);
            println!("effects: {:?}", node_info.effects);
            println!("cache: {}", render_cache_policy(&node_info.cache));
            println!("env_allowlist: {:?}", node_info.env_allowlist);
            println!(
                "execution_explanation: {}",
                format_node_execution_explanation_human(&execution_explanation)
            );
            if let Some(r) = resolved_params {
                println!("resolved_params:\n{}", r);
            }
            if let Some(o) = outputs_index {
                println!("outputs_index:\n{}", o);
            }
            println!("fingerprint: {:?}", snapshot.graph.node_fingerprint(node_info).ok());
            if let Some(trace) = trace {
                println!("trace:\n{}", trace);
            } else {
                println!("trace: <missing>");
            }
        }
    } else if cli.json {
        let m: serde_json::Value = read_manifest_json(run_dir).unwrap_or_default();
        let status = m.get("status").cloned().unwrap_or_default();
        let graph_fp = m.get("graph_fingerprint").cloned().unwrap_or_default();
        let counts = m.get("node_counts").cloned().unwrap_or_default();
        let nodes = read_node_traces(run_dir).unwrap_or_default();
        let failed: Vec<String> = nodes
            .iter()
            .filter_map(|(id, v)| {
                if v.get("status") == Some(&serde_json::Value::String("failed".to_string())) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();
        let data = json!({
            "status": status,
            "graph_fingerprint": graph_fp,
            "node_counts": counts,
            "failed_nodes": failed,
        });
        return emit_json(cli, "dag.explain", true, data, Vec::new(), ExitCode::SUCCESS);
    } else {
        let m: serde_json::Value = read_manifest_json(run_dir).unwrap_or_default();
        let status = m.get("status").cloned().unwrap_or_default();
        let graph_fp = m.get("graph_fingerprint").cloned().unwrap_or_default();
        let counts = m.get("node_counts").cloned().unwrap_or_default();
        let nodes = read_node_traces(run_dir).unwrap_or_default();
        let failed: Vec<String> = nodes
            .iter()
            .filter_map(|(id, v)| {
                if v.get("status") == Some(&serde_json::Value::String("failed".to_string())) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();
        println!("{}", concise_explain_human(&status, &graph_fp, &counts, &failed));
    }
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn handle_node_command(
    cli: &DagCli,
    run_dir: &Path,
    node: &str,
) -> Result<ExitCode, ExitCode> {
    let payload = node_inspection_payload(run_dir, node)?;
    if cli.json {
        return emit_json(
            cli,
            "dag.node",
            true,
            serde_json::to_value(&payload).map_err(|_| ExitCode::from(3))?,
            Vec::new(),
            ExitCode::SUCCESS,
        );
    }
    println!("{}", format_node_inspection_human(&payload));
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn handle_status_command(cli: &DagCli, run_dir: &Path) -> Result<ExitCode, ExitCode> {
    require_run_directory(run_dir)?;
    let manifest = read_file(&manifest_path(run_dir))?;
    let nodes_dir = run_dir.join("nodes");
    let manifest_json = serde_json::from_str::<Value>(&manifest).unwrap_or(Value::Null);
    let mut statuses = Vec::new();
    if nodes_dir.exists() {
        for entry in fs::read_dir(nodes_dir).map_err(|_| ExitCode::from(3))? {
            let entry = entry.map_err(|_| ExitCode::from(3))?;
            let trace_path = entry.path().join("trace.json");
            if trace_path.exists() {
                let t = read_file(&trace_path)?;
                let mut trace = serde_json::from_str::<Value>(&t).unwrap_or(Value::Null);
                if let Some(object) = trace.as_object_mut() {
                    object.insert(
                        "node_id".to_string(),
                        Value::String(entry.file_name().to_string_lossy().to_string()),
                    );
                }
                statuses.push(trace);
            }
        }
    }
    let summary = operator_status_summary(run_dir, &manifest_json, &statuses);
    if cli.json {
        return emit_json(
            cli,
            "dag.status",
            true,
            json!({
                "current_state": summary["current_state"],
                "next_action": summary["next_action"],
                "critical_failure": summary["critical_failure"],
                "evidence_path": summary["evidence_path"],
                "verification_result": summary["verification_result"],
                "manifest": manifest_json,
                "traces": statuses
            }),
            Vec::new(),
            ExitCode::SUCCESS,
        );
    }
    println!("{}", operator_status_human(&summary));
    Ok(ExitCode::SUCCESS)
}

fn operator_status_summary(run_dir: &Path, manifest: &Value, traces: &[Value]) -> Value {
    let current_state =
        manifest.get("status").and_then(Value::as_str).unwrap_or("unknown").to_string();
    let first_failed = traces.iter().find(|trace| {
        trace.get("status").and_then(Value::as_str) == Some("failed")
            || trace.get("state").and_then(Value::as_str) == Some("failed")
    });
    let critical_failure = first_failed.map_or(Value::Null, |trace| {
        json!({
            "node_id": trace.get("node_id").cloned().unwrap_or(Value::Null),
            "status": trace.get("status").cloned().unwrap_or(Value::Null),
            "failure": trace.get("failure").cloned().unwrap_or(Value::Null)
        })
    });
    let verification_result = status_verification_result(run_dir, manifest, traces);
    let next_action =
        status_next_action(&current_state, verification_result.as_str(), &critical_failure);
    json!({
        "current_state": current_state,
        "next_action": next_action,
        "critical_failure": critical_failure,
        "evidence_path": run_dir.display().to_string(),
        "verification_result": verification_result
    })
}

fn status_verification_result(run_dir: &Path, manifest: &Value, traces: &[Value]) -> String {
    if manifest.is_null() {
        return "manifest-invalid".to_string();
    }
    if !run_dir.join("manifest.json").is_file() {
        return "manifest-missing".to_string();
    }
    if !run_dir.join("nodes").exists() {
        return "traces-missing".to_string();
    }
    if traces.is_empty() {
        return "traces-empty".to_string();
    }
    if run_dir.join("outputs.index.json").exists()
        || run_dir.join("outputs").join("index.json").exists()
    {
        return "evidence-present".to_string();
    }
    "artifact-index-missing".to_string()
}

fn status_next_action(
    current_state: &str,
    verification_result: &str,
    critical_failure: &Value,
) -> String {
    if verification_result != "evidence-present" {
        return "run `dag verify <run_dir> --strict` and repair missing evidence files".to_string();
    }
    if !critical_failure.is_null() {
        return "run `dag runs explain-failure <run-id> --root <runs-root>` for root-cause details"
            .to_string();
    }
    match current_state {
        "failed" => "run `dag runs explain-failure <run-id> --root <runs-root>`".to_string(),
        "cancelled" => "resume with replay or inspect scheduler cancellation reason".to_string(),
        "running" => "inspect timeline and node traces for current bottlenecks".to_string(),
        "success" | "completed" => "inspect artifacts and export evidence bundle".to_string(),
        _ => "inspect manifest and traces to classify run state".to_string(),
    }
}

fn operator_status_human(summary: &Value) -> String {
    format!(
        "current_state: {}\nnext_action: {}\ncritical_failure: {}\nevidence_path: {}\nverification_result: {}",
        summary.get("current_state").unwrap_or(&Value::Null),
        summary.get("next_action").unwrap_or(&Value::Null),
        summary.get("critical_failure").unwrap_or(&Value::Null),
        summary.get("evidence_path").unwrap_or(&Value::Null),
        summary.get("verification_result").unwrap_or(&Value::Null)
    )
}

#[cfg(test)]
mod tests {
    use super::{
        concise_explain_human, explain_node_payload, format_node_inspection_human,
        handle_explain_command, handle_node_command, handle_status_command,
        node_inspection_payload, operator_status_human, operator_status_summary,
        render_cache_policy, status_next_action,
    };
    use crate::commands::{Commands, DagCli};
    use crate::node_execution_explanation::explain_node_execution;
    use crate::read_file;
    use crate::run_data::load_snapshot;
    use crate::ExitCode;
    use serde_json::json;
    use serde_json::Value;
    use std::fs;
    use std::path::Path;

    fn quiet_json_cli() -> DagCli {
        DagCli { json: true, quiet: true, command: Commands::Version }
    }

    fn quiet_human_cli() -> DagCli {
        DagCli { json: false, quiet: true, command: Commands::Version }
    }

    fn write_run_fixture(imported: bool, malformed_manifest: bool) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tmp");
        let run = dir.path();
        fs::create_dir_all(run.join("nodes/extract/outputs")).expect("mkdir outputs");
        fs::create_dir_all(run.join("nodes/extract/inputs")).expect("mkdir inputs");
        if malformed_manifest {
            fs::write(run.join("manifest.json"), b"{not-json").expect("write malformed manifest");
        } else {
            let mut manifest = json!({
                "manifest_version":"run-manifest/v0.1",
                "run_id":"run-1",
                "created_unix_ms":1,
                "started_unix_ms":1,
                "finished_unix_ms":2,
                "graph_snapshot":"graph.snapshot.json",
                "status":"success",
                "spec":"bijux-dag/v0.1",
                "graph_fingerprint":"g1",
                "tool_version":"0.1.0",
                "jobs":1,
                "adapters":[],
                "outputs":[],
                "node_counts":{"success":1,"failed":0,"skipped":0,"cached":0},
                "policy":{"deny_network":true,"deny_env":true,"deny_clock":true,"clean_env":true}
            });
            if imported {
                manifest["import_source"] = json!("bundle");
            }
            fs::write(
                run.join("manifest.json"),
                serde_json::to_vec_pretty(&manifest).expect("manifest"),
            )
            .expect("write manifest");
        }
        fs::write(
            run.join("graph.snapshot.json"),
            serde_json::to_vec_pretty(&json!({
                "graph":{
                    "spec":"bijux-dag/v0.1",
                    "meta":{"name":"x","owners":[],"tags":[]},
                    "inputs":{"dataset_uri":"s3://warehouse/catalog","region":"eu-west-1"},
                    "nodes":[{
                        "id":"extract",
                        "kind":"const",
                        "inputs":[],
                        "outputs":[{"name":"out","path":"extract/out"}],
                        "params":{
                            "request":{
                                "dataset_uri":{"graph_input":"dataset_uri"},
                                "region":{"graph_input":"region"}
                            }
                        },
                        "cache":{"enabled":false,"reason":"fixture keeps node explain cache behavior explicit"},
                        "effects":["env"],
                        "env_allowlist":["REGION_TOKEN"]
                    }],
                    "edges":[]
                },
                "graph_fingerprint":"g1"
            }))
            .expect("snapshot"),
        )
        .expect("write snapshot");
        fs::write(
            run.join("nodes/extract/trace.json"),
            serde_json::to_vec_pretty(&json!({
                "node_id":"extract",
                "status":"success",
                "started_unix_ms": 1u64,
                "finished_unix_ms": 2u64,
                "attempt": 1,
                "fingerprint":"fp-extract",
                "adapter_id":"const",
                "adapter_version":"0.1",
                "adapter_outputs_schema_version":"1",
                "inputs_index":"inputs/index.json",
                "resolved_params":{"request":{"dataset_uri":"s3://warehouse/catalog","region":"eu-west-1"}},
                "exit_code":0,
                "stdout":{
                    "path":"nodes/extract/stdout.log",
                    "size_bytes":28,
                    "tail_lines":["terminal stdout","second line"]
                },
                "stderr":{
                    "path":"nodes/extract/stderr.log",
                    "size_bytes":16,
                    "tail_lines":["terminal stderr"]
                },
                "outputs":[{
                    "name":"out",
                    "path":"extract/out",
                    "kind":"file",
                    "required":true,
                    "present":true,
                    "media_type":"text/plain",
                    "size_bytes":4,
                    "sha256":"abcd"
                }]
            }))
            .expect("trace"),
        )
        .expect("write trace");
        fs::write(
            run.join("nodes/extract/inputs/index.json"),
            serde_json::to_vec_pretty(&json!({
                "files":[{
                    "local_path":"seed/in",
                    "source_sha256":"seed-sha",
                    "source_node_id":"seed",
                    "source_node_fingerprint":"seed-fp",
                    "source_output_name":"out",
                    "materialization_mode":"copy"
                }]
            }))
            .expect("inputs index"),
        )
        .expect("write inputs index");
        fs::write(
            run.join("nodes/extract/outputs/index.json"),
            serde_json::to_vec_pretty(&json!({
                "files":[{
                    "name":"out",
                    "path":"extract/out",
                    "kind":"file",
                    "media_type":"text/plain",
                    "size_bytes":4,
                    "sha256":"abcd",
                    "node_id":"extract",
                    "node_fingerprint":"fp-extract"
                }]
            }))
            .expect("outputs index"),
        )
        .expect("write outputs index");
        fs::write(
            run.join("nodes/extract/resolved_params.json"),
            serde_json::to_vec_pretty(&json!({
                "request":{
                    "dataset_uri":"s3://warehouse/catalog",
                    "region":"eu-west-1"
                }
            }))
            .expect("resolved params"),
        )
        .expect("write resolved params");
        fs::write(
            run.join("nodes/extract/attempts.json"),
            serde_json::to_vec_pretty(&json!([{
                "attempt":1,
                "started_unix_ms":1u64,
                "finished_unix_ms":2u64,
                "status":"Success",
                "stdout_path":"attempts/1/stdout.log",
                "stderr_path":"attempts/1/stderr.log"
            }]))
            .expect("attempts"),
        )
        .expect("write attempts");
        fs::create_dir_all(run.join("nodes/extract/attempts/1")).expect("mkdir attempts");
        fs::write(run.join("nodes/extract/stdout.log"), "terminal stdout\nsecond line\n")
            .expect("stdout");
        fs::write(run.join("nodes/extract/stderr.log"), "terminal stderr\n").expect("stderr");
        fs::write(run.join("nodes/extract/attempts/1/stdout.log"), "attempt stdout\n")
            .expect("attempt stdout");
        fs::write(run.join("nodes/extract/attempts/1/stderr.log"), "attempt stderr\n")
            .expect("attempt stderr");
        dir
    }

    #[test]
    fn inspect_status_success_json_path() {
        let run = write_run_fixture(false, false);
        let cli = quiet_json_cli();
        let code = handle_status_command(&cli, run.path()).expect("status");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn inspect_explain_success_json_path() {
        let run = write_run_fixture(false, false);
        let cli = quiet_json_cli();
        let code = handle_explain_command(&cli, run.path(), &None).expect("explain");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn inspect_routes_handle_imported_run_paths() {
        let run = write_run_fixture(true, false);
        let cli = quiet_json_cli();
        let code = handle_explain_command(&cli, run.path(), &None).expect("imported explain");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn inspect_routes_handle_damaged_bundle_like_manifest_without_panic() {
        let run = write_run_fixture(false, true);
        let cli = quiet_json_cli();
        let result = std::panic::catch_unwind(|| handle_explain_command(&cli, run.path(), &None));
        assert!(result.is_ok(), "inspect explain should not panic");
        assert!(result.expect("result").is_ok());
    }

    #[test]
    fn inspect_node_malformed_run_dir_returns_error() {
        let cli = quiet_json_cli();
        let code = handle_node_command(&cli, Path::new("/missing/run"), "extract").unwrap_err();
        assert_eq!(code, ExitCode::from(3));
    }

    #[test]
    fn inspect_human_paths_do_not_panic() {
        let run = write_run_fixture(false, false);
        let cli = quiet_human_cli();
        let explain = std::panic::catch_unwind(|| handle_explain_command(&cli, run.path(), &None));
        assert!(explain.is_ok());
        assert!(explain.expect("result").is_ok());
        let status = std::panic::catch_unwind(|| handle_status_command(&cli, run.path()));
        assert!(status.is_ok());
        assert!(status.expect("result").is_ok());
    }

    #[test]
    fn inspect_route_entrypoints_do_not_panic_on_missing_run_dir() {
        let cli = quiet_json_cli();
        let explain = std::panic::catch_unwind(|| {
            handle_explain_command(&cli, Path::new("/missing/run"), &None)
        });
        let node = std::panic::catch_unwind(|| {
            handle_node_command(&cli, Path::new("/missing/run"), "extract")
        });
        let status =
            std::panic::catch_unwind(|| handle_status_command(&cli, Path::new("/missing/run")));
        assert!(explain.is_ok());
        assert!(node.is_ok());
        assert!(status.is_ok());
    }

    #[test]
    fn inspect_node_payload_surfaces_graph_contract_details() {
        let run = write_run_fixture(false, false);
        let manifest = read_file(&run.path().join("manifest.json")).expect("manifest");
        let trace = read_file(&run.path().join("nodes/extract/trace.json")).expect("trace");
        let outputs_index =
            read_file(&run.path().join("nodes/extract/outputs/index.json")).expect("index");
        let snapshot = load_snapshot(run.path()).expect("snapshot");
        let node =
            snapshot.graph.nodes.iter().find(|node| node.id == "extract").expect("extract node");
        let trace_json = serde_json::from_str::<Value>(&trace).expect("trace json");
        let execution_explanation =
            explain_node_execution(run.path(), &snapshot.graph, "extract", Some(&trace_json));

        let payload = explain_node_payload(
            &manifest,
            &snapshot.graph,
            node,
            "extract",
            Vec::new(),
            Some(&trace),
            &execution_explanation,
            Some(&outputs_index),
            None,
        );

        assert_eq!(payload["graph_inputs"]["region"], "eu-west-1");
        assert_eq!(payload["graph_input_schema"]["region"]["type"], "string");
        assert_eq!(payload["graph_input_schema"]["region"]["default"], "eu-west-1");
        assert_eq!(payload["cache"]["enabled"], false);
        assert_eq!(
            payload["cache"]["reason"],
            "fixture keeps node explain cache behavior explicit"
        );
        assert_eq!(
            payload["param_bindings"][0]["source"]["GraphInput"]["input_name"],
            "dataset_uri"
        );
        assert_eq!(payload["output_contracts"][0]["path"], "extract/out");
        assert_eq!(payload["env_allowlist"][0], "REGION_TOKEN");
    }

    #[test]
    fn inspect_node_payload_surfaces_attempts_logs_and_artifacts() {
        let run = write_run_fixture(false, false);
        let payload = node_inspection_payload(run.path(), "extract").expect("node inspection");

        assert_eq!(payload.status, "success");
        assert_eq!(payload.planned.id, "extract");
        assert_eq!(payload.resolved_params["request"]["region"], "eu-west-1");
        assert_eq!(payload.input_artifacts.len(), 1);
        assert_eq!(payload.input_artifacts[0].source_node_id, "seed");
        assert_eq!(payload.output_artifacts.len(), 1);
        assert_eq!(payload.output_artifacts[0].name, "out");
        assert_eq!(payload.terminal_attempt, 1);
        assert_eq!(payload.attempts.len(), 1);
        assert_eq!(payload.attempts[0].attempt, 1);
        assert_eq!(
            payload.attempts[0].stdout.as_ref().expect("stdout").path,
            "nodes/extract/attempts/1/stdout.log"
        );
        assert_eq!(
            payload.logs.stdout.as_ref().expect("stdout log").tail,
            vec!["terminal stdout".to_string(), "second line".to_string()]
        );
        assert_eq!(payload.logs.stdout.as_ref().expect("stdout log").size_bytes, 28);
        assert_eq!(payload.failure.exit_code, Some(0));
        assert_eq!(payload.cache.observed_result, "disabled");
        assert!(payload.failure.failure.is_none());
        assert!(payload.evidence_gaps.is_empty());
    }

    #[test]
    fn inspect_node_payload_records_failure_and_missing_evidence_gaps() {
        let run = write_run_fixture(false, false);
        fs::remove_file(run.path().join("nodes/extract/stderr.log")).expect("remove stderr");
        fs::write(
            run.path().join("nodes/extract/trace.json"),
            serde_json::to_vec_pretty(&json!({
                "node_id":"extract",
                "status":"failed",
                "started_unix_ms": 1u64,
                "finished_unix_ms": 2u64,
                "attempt": 2,
                "fingerprint":"fp-extract",
                "adapter_id":"const",
                "adapter_version":"0.1",
                "adapter_outputs_schema_version":"1",
                "failure":{"class":"execution","kind":"Execution","code":"EXEC_FAIL","message":"boom"},
                "transition_cause":"ExecutionFailed",
                "lifecycle_state":"failed"
            }))
            .expect("trace"),
        )
        .expect("write failed trace");

        let payload = node_inspection_payload(run.path(), "extract").expect("node inspection");
        assert_eq!(payload.status, "failed");
        assert_eq!(payload.terminal_attempt, 2);
        assert_eq!(
            payload
                .failure
                .failure
                .as_ref()
                .and_then(|failure| failure.get("code"))
                .and_then(|value| value.as_str()),
            Some("EXEC_FAIL")
        );
        assert_eq!(payload.failure.transition_cause.as_deref(), Some("ExecutionFailed"));
        assert!(payload
            .evidence_gaps
            .iter()
            .any(|gap| gap == "missing stderr log: nodes/extract/stderr.log"));
    }

    #[test]
    fn cache_policy_rendering_is_explicit() {
        let rendered = render_cache_policy(&bijux_dag_core::CacheBehavior {
            enabled: false,
            reason: Some("publishes externally visible state".to_string()),
        });
        assert_eq!(rendered, "disabled (reason: publishes externally visible state)");
    }

    #[test]
    fn inspect_concise_human_snapshot_is_stable() {
        let rendered = concise_explain_human(
            &json!("success"),
            &json!("g1"),
            &json!({"success":1,"failed":0,"skipped":0,"cached":0}),
            &["n1".to_string()],
        );
        let expected = "status: \"success\"\n\
graph_fingerprint: \"g1\"\n\
node_counts: {\"cached\":0,\"failed\":0,\"skipped\":0,\"success\":1}\n\
failed_nodes: [\"n1\"]";
        assert_eq!(rendered, expected);
    }

    #[test]
    fn node_human_output_surfaces_cache_paths_and_attempts() {
        let run = write_run_fixture(false, false);
        let payload = node_inspection_payload(run.path(), "extract").expect("node inspection");
        let rendered = format_node_inspection_human(&payload);

        assert!(rendered.contains("node: extract"));
        assert!(rendered.contains("planned_kind: const"));
        assert!(rendered.contains("input_artifact_count: 1"));
        assert!(rendered.contains("output_artifact_count: 1"));
        assert!(rendered.contains("attempt=1 status=Success"));
        assert!(rendered.contains("cache_status: configured=disabled"));
        assert!(rendered.contains("exit_code: 0"));
        assert!(rendered.contains("stdout_path: nodes/extract/stdout.log"));
        assert!(rendered.contains("stdout_size_bytes: 28"));
        assert!(rendered.contains("stderr_path: nodes/extract/stderr.log"));
        assert!(rendered.contains("terminal stdout"));
    }

    #[test]
    fn status_summary_prioritizes_operator_fields() {
        let run = write_run_fixture(false, false);
        let manifest = serde_json::from_str::<serde_json::Value>(
            &std::fs::read_to_string(run.path().join("manifest.json")).expect("manifest"),
        )
        .expect("parse manifest");
        let traces =
            vec![json!({"node_id":"extract","status":"failed","failure":{"kind":"timeout"}})];
        let summary = operator_status_summary(run.path(), &manifest, &traces);
        assert_eq!(summary["current_state"], "success");
        assert_eq!(summary["verification_result"], "artifact-index-missing");
        assert!(summary["critical_failure"].is_object());
        assert!(summary["next_action"].as_str().expect("next action").contains("dag verify"));
    }

    #[test]
    fn status_human_output_keeps_operator_order() {
        let summary = json!({
            "current_state":"failed",
            "next_action":"run explain",
            "critical_failure":{"node_id":"n1"},
            "evidence_path":"/tmp/run",
            "verification_result":"evidence-present"
        });
        let rendered = operator_status_human(&summary);
        let expected = [
            "current_state:",
            "next_action:",
            "critical_failure:",
            "evidence_path:",
            "verification_result:",
        ];
        let mut cursor = 0usize;
        for marker in expected {
            let index = rendered[cursor..].find(marker).expect("marker order") + cursor;
            cursor = index + marker.len();
        }
    }

    #[test]
    fn status_next_action_requests_verification_when_evidence_is_missing() {
        let action =
            status_next_action("success", "artifact-index-missing", &serde_json::Value::Null);
        assert!(action.contains("dag verify"));
    }
}
