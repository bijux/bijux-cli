use crate::commands::{DagCli, RunsCommands};
use crate::inspect_service;
use crate::routes::run_lookup::{read_manifest_json, RunWorkspacePaths};
use crate::routes::selector_grammar::parse_selector_expressions;
use crate::run_views::RunTimelineQuery;
use crate::{
    emit_json, format_inspect_human, format_show_human, list_runs, print_human_diff, read_file,
    replay_service, resolve_run_dir, runs_compare, runs_failures, runs_flakes, runs_summary,
    runs_trend, verify_run, ExitCode,
};
use bijux_dag_artifacts::{write_json_atomic_durable, RunStopRequest};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

fn read_optional_json(path: &Path) -> Value {
    read_file(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .unwrap_or(Value::Null)
}

fn redact_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut redacted = serde_json::Map::with_capacity(map.len());
            for (key, child) in map {
                let key_lc = key.to_ascii_lowercase();
                if key_lc.contains("secret")
                    || key_lc.contains("token")
                    || key_lc.contains("password")
                    || key_lc.contains("api_key")
                {
                    redacted.insert(key.clone(), Value::String("[REDACTED]".to_string()));
                } else {
                    redacted.insert(key.clone(), redact_value(child));
                }
            }
            Value::Object(redacted)
        }
        Value::Array(items) => Value::Array(items.iter().map(redact_value).collect()),
        _ => value.clone(),
    }
}

fn collect_node_traces(run_dir: &Path) -> Result<Value, ExitCode> {
    let nodes_dir = run_dir.join("nodes");
    if !nodes_dir.exists() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    let mut traces = BTreeMap::<String, Value>::new();
    for entry in fs::read_dir(nodes_dir).map_err(|_| ExitCode::from(3))? {
        let entry = entry.map_err(|_| ExitCode::from(3))?;
        let node_id = entry.file_name().to_string_lossy().to_string();
        let trace = read_optional_json(&entry.path().join("trace.json"));
        traces.insert(node_id, trace);
    }
    Ok(serde_json::to_value(traces).unwrap_or(Value::Null))
}

fn format_runs_history_human(report: &Value) -> String {
    let mut lines = Vec::new();
    let rows = report.get("runs").and_then(Value::as_array).cloned().unwrap_or_default();
    for row in rows {
        let run_id = row.get("run_id").and_then(Value::as_str).unwrap_or("unknown");
        let lifecycle_state =
            row.get("lifecycle_state").and_then(Value::as_str).unwrap_or("historical");
        let status = row.get("status").and_then(Value::as_str).unwrap_or("unknown");
        let graph_name = row.get("graph_name").and_then(Value::as_str).unwrap_or("-");
        let output_location = row.get("output_location").and_then(Value::as_str).unwrap_or("-");
        let parent_run_id = row.get("parent_run_id").and_then(Value::as_str).unwrap_or("-");
        let child_count = row
            .get("lineage")
            .and_then(|value| value.get("child_run_ids"))
            .and_then(Value::as_array)
            .map_or(0, std::vec::Vec::len);
        lines.push(format!(
            "{run_id} status={status} lifecycle={lifecycle_state} graph={graph_name} output={output_location} parent={parent_run_id} children={child_count}"
        ));
    }
    if let Some(page) = report.get("page") {
        lines.push(format!(
            "page offset={} limit={} total={}",
            page.get("offset").unwrap_or(&Value::Null),
            page.get("limit").unwrap_or(&Value::Null),
            page.get("total").unwrap_or(&Value::Null),
        ));
    }
    lines.join("\n")
}

fn format_run_timeline_human(report: &Value) -> String {
    let mut lines = vec![
        format!("source: {}", report.get("source").and_then(Value::as_str).unwrap_or("unknown")),
        format!(
            "matched: {}/{} events",
            report.get("matched_event_count").and_then(Value::as_u64).unwrap_or(0),
            report.get("total_event_count").and_then(Value::as_u64).unwrap_or(0),
        ),
    ];

    if let Some(filters) = report.get("filters").and_then(Value::as_object) {
        let mut active_filters = Vec::new();
        if let Some(node) = filters.get("node").and_then(Value::as_str) {
            active_filters.push(format!("node={node}"));
        }
        if let Some(event) = filters.get("event").and_then(Value::as_str) {
            active_filters.push(format!("event={event}"));
        }
        if let Some(since_unix_ms) = filters.get("since_unix_ms").and_then(Value::as_u64) {
            active_filters.push(format!("since_unix_ms={since_unix_ms}"));
        }
        if let Some(until_unix_ms) = filters.get("until_unix_ms").and_then(Value::as_u64) {
            active_filters.push(format!("until_unix_ms={until_unix_ms}"));
        }
        if !active_filters.is_empty() {
            lines.push(format!("filters: {}", active_filters.join(" ")));
        }
    }

    let events = report.get("events").and_then(Value::as_array).cloned().unwrap_or_default();
    if events.is_empty() {
        lines.push("no matching events".to_string());
        return lines.join("\n");
    }

    for event in events {
        let mut segments = Vec::new();
        segments.push(format!(
            "timestamp_unix_ms={}",
            event.get("unix_ms").cloned().unwrap_or(Value::Null)
        ));
        segments.push(format!(
            "event={}",
            event.get("label").and_then(Value::as_str).unwrap_or("unknown")
        ));
        segments.push(format!(
            "category={}",
            event.get("category").and_then(Value::as_str).unwrap_or("unknown")
        ));
        if let Some(node_id) = event.get("node_id").and_then(Value::as_str) {
            segments.push(format!("node={node_id}"));
        }
        if let Some(status) = event.get("status").and_then(Value::as_str) {
            segments.push(format!("status={status}"));
        }
        if let Some(reason) = event.get("reason").and_then(Value::as_str) {
            segments.push(format!("cause={reason}"));
        }
        if let Some(source_event) = event.get("source_event").and_then(Value::as_str) {
            segments.push(format!("source_event={source_event}"));
        }
        lines.push(segments.join(" "));
    }

    lines.join("\n")
}

fn format_scheduler_checkpoint_human(report: &Value) -> String {
    let mut lines = vec![
        format!(
            "inspection_state: {}",
            report.get("inspection_state").and_then(Value::as_str).unwrap_or("unknown")
        ),
        format!(
            "checkpoint_path: {}",
            report.get("checkpoint_path").and_then(Value::as_str).unwrap_or("-")
        ),
    ];

    match report.get("inspection_state").and_then(Value::as_str).unwrap_or("unknown") {
        "present" => {
            lines.push(format!(
                "decision_reason: {}",
                report.get("decision_reason").and_then(Value::as_str).unwrap_or("unknown")
            ));
            lines.push(format!(
                "ready_queue_depth: {}",
                report.get("ready_queue_depth").cloned().unwrap_or(Value::Null)
            ));
            lines.push(format!("ready_queue: {}", render_string_list(report.get("ready_queue"))));
            lines.push(format!(
                "scheduled_batch: {}",
                render_string_list(report.get("scheduled_batch"))
            ));
            lines.push(format!(
                "inflight_nodes: {}",
                render_string_list(report.get("inflight_nodes"))
            ));
            lines.push(format!(
                "resource_blocked_nodes: {}",
                render_string_list(report.get("resource_blocked_nodes"))
            ));
            lines.push(format!(
                "completed_statuses: {}",
                render_string_map(report.get("completed_statuses"))
            ));
            lines.push(format!(
                "blocked_reasons: {}",
                render_string_map(report.get("blocked_reasons"))
            ));
        }
        _ => {
            lines.push(format!(
                "detail: {}",
                report
                    .get("detail")
                    .and_then(Value::as_str)
                    .unwrap_or("checkpoint details unavailable")
            ));
        }
    }

    lines.join("\n")
}

fn render_string_list(value: Option<&Value>) -> String {
    let Some(items) = value.and_then(Value::as_array) else {
        return "[]".to_string();
    };
    if items.is_empty() {
        return "[]".to_string();
    }
    let rendered =
        items.iter().filter_map(Value::as_str).map(ToOwned::to_owned).collect::<Vec<_>>();
    if rendered.is_empty() {
        "[]".to_string()
    } else {
        rendered.join(", ")
    }
}

fn render_string_map(value: Option<&Value>) -> String {
    let Some(map) = value.and_then(Value::as_object) else {
        return "{}".to_string();
    };
    if map.is_empty() {
        return "{}".to_string();
    }
    map.iter()
        .map(|(key, value)| format!("{key}={}", value.as_str().unwrap_or("unknown")))
        .collect::<Vec<_>>()
        .join(", ")
}

fn diagnostics_bundle_payload(run_dir: &Path, redact: bool) -> Result<Value, ExitCode> {
    let manifest = read_optional_json(&run_dir.join("manifest.json"));
    let graph_snapshot = if run_dir.join("graph.snapshot.json").exists() {
        read_optional_json(&run_dir.join("graph.snapshot.json"))
    } else {
        read_optional_json(&run_dir.join("snapshot.json"))
    };
    let plan = read_optional_json(&run_dir.join("plan.json"));
    let artifact_inventory = if run_dir.join("outputs/index.json").exists() {
        read_optional_json(&run_dir.join("outputs/index.json"))
    } else {
        read_optional_json(&run_dir.join("outputs.index.json"))
    };
    let payload = serde_json::json!({
        "bundle_version": "dag-diagnostics-bundle/v0.1",
        "run_id": manifest.get("run_id").cloned().unwrap_or(Value::Null),
        "run_dir": run_dir.display().to_string(),
        "manifest": manifest,
        "graph": graph_snapshot,
        "plan": plan,
        "config": {
            "policy": manifest.get("policy").cloned().unwrap_or(Value::Null),
            "runtime": manifest.get("runtime").cloned().unwrap_or(Value::Null)
        },
        "traces": collect_node_traces(run_dir)?,
        "logs": {
            "events": read_optional_json(&run_dir.join("observability.events.json")),
            "timeline": read_optional_json(&run_dir.join("observability.timeline.json")),
            "root_causes": read_optional_json(&run_dir.join("observability.root-causes.json"))
        },
        "artifact_inventory": artifact_inventory,
        "cache_proof": read_optional_json(&run_dir.join("cache.proof.json")),
        "command_context": {
            "submission_source": manifest
                .get("run_metadata")
                .and_then(|m| m.get("submission_source"))
                .cloned()
                .unwrap_or(Value::Null),
            "trigger_source": manifest
                .get("run_metadata")
                .and_then(|m| m.get("trigger_source"))
                .cloned()
                .unwrap_or(Value::Null),
            "operator": manifest
                .get("run_metadata")
                .and_then(|m| m.get("operator"))
                .cloned()
                .unwrap_or(Value::Null)
        }
    });
    Ok(if redact { redact_value(&payload) } else { payload })
}

fn now_unix_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_millis())
}

fn read_stop_request(path: &Path) -> Result<RunStopRequest, ExitCode> {
    let raw = read_file(path)?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

fn record_stop_request(root: &Path, run_id: &str) -> Result<Value, ExitCode> {
    let paths = RunWorkspacePaths::for_run(root, run_id)?;
    if let Some(active_run_dir) = paths.active_run_path() {
        let request_path = active_run_dir.join("run.stop-request.json");
        if request_path.exists() {
            let request = read_stop_request(&request_path)?;
            return Ok(serde_json::json!({
                "run_id": paths.normalized_run_id,
                "requested": false,
                "state": "already_requested",
                "run_dir": active_run_dir,
                "request_path": request_path,
                "request": request,
            }));
        }

        let request = RunStopRequest {
            schema_version: "run-stop-request/v0.1".to_string(),
            run_id: paths.normalized_run_id.clone(),
            requested_unix_ms: now_unix_ms(),
            source: "cli".to_string(),
            reason: None,
        };
        let request_value = serde_json::to_value(&request).map_err(|_| ExitCode::from(3))?;
        write_json_atomic_durable(&request_path, &request_value).map_err(|_| ExitCode::from(3))?;
        return Ok(serde_json::json!({
            "run_id": paths.normalized_run_id,
            "requested": true,
            "state": "requested",
            "run_dir": active_run_dir,
            "request_path": request_path,
            "request": request,
        }));
    }

    if let Some(run_dir) = paths.stable_run_path() {
        let manifest = read_manifest_json(&run_dir)?;
        let status = manifest.get("status").and_then(Value::as_str).unwrap_or("unknown");
        return Ok(serde_json::json!({
            "run_id": paths.normalized_run_id,
            "requested": false,
            "state": "already_finished",
            "status": status,
            "run_dir": run_dir,
        }));
    }

    Err(ExitCode::from(3))
}

pub(crate) fn handle_runs_command(
    cli: &DagCli,
    command: &RunsCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        RunsCommands::List { root } => {
            let runs = list_runs(root).map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.list",
                    true,
                    serde_json::json!({"runs": runs}),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            for run in runs {
                println!("{run}");
            }
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Show { run_id, root } => {
            let summary = inspect_service::run_summary_for_id(root, run_id)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.show",
                    true,
                    summary,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", format_show_human(&summary));
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Inspect { run_id, root } => {
            let summary = inspect_service::run_summary_for_id(root, run_id)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.inspect",
                    true,
                    summary,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", format_inspect_human(&summary));
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::History { root, status, graph, source, offset, limit, select } => {
            let selectors = parse_selector_expressions(select)?;
            let pagination = limit.map(|value| (offset.unwrap_or(0), value));
            let report = inspect_service::run_history_query_for_root(
                root,
                status.as_deref(),
                source.as_deref(),
                graph.as_deref(),
                pagination,
                Some(selectors.as_slice()),
            )?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.history",
                    true,
                    report,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", format_runs_history_human(&report));
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::IdExplain { run_id, root } => {
            let report = inspect_service::run_id_explain_for_root(root, run_id)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.id-explain",
                    true,
                    report,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Tree { run_id, root } => {
            let tree = inspect_service::run_tree_for_id(root, run_id)?;
            if cli.json {
                return emit_json(cli, "dag.runs.tree", true, tree, Vec::new(), ExitCode::SUCCESS);
            }
            println!("{}", serde_json::to_string_pretty(&tree).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Timeline { run_id, root, node, event, since_unix_ms, until_unix_ms } => {
            let query = RunTimelineQuery {
                node: node.clone(),
                event: event.clone(),
                since_unix_ms: *since_unix_ms,
                until_unix_ms: *until_unix_ms,
            };
            let timeline = inspect_service::run_timeline_for_id_with_query(root, run_id, &query)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.timeline",
                    true,
                    timeline,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", format_run_timeline_human(&timeline));
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::SchedulerCheckpoint { run_id, root } => {
            let report = inspect_service::scheduler_checkpoint_for_run_id(root, run_id)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.scheduler-checkpoint",
                    true,
                    report,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", format_scheduler_checkpoint_human(&report));
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Stop { run_id, root } => {
            let report = record_stop_request(root, run_id)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.stop",
                    true,
                    report,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            match report.get("state").and_then(Value::as_str).unwrap_or("requested") {
                "requested" => {
                    println!(
                        "stop request recorded for run {}",
                        report.get("run_id").and_then(Value::as_str).unwrap_or(run_id)
                    );
                    println!(
                        "request file: {}",
                        report.get("request_path").and_then(Value::as_str).unwrap_or("-")
                    );
                }
                "already_requested" => {
                    println!(
                        "stop request already recorded for run {}",
                        report.get("run_id").and_then(Value::as_str).unwrap_or(run_id)
                    );
                    println!(
                        "request file: {}",
                        report.get("request_path").and_then(Value::as_str).unwrap_or("-")
                    );
                }
                _ => {
                    println!(
                        "run {} is already finished with status {}",
                        report.get("run_id").and_then(Value::as_str).unwrap_or(run_id),
                        report.get("status").and_then(Value::as_str).unwrap_or("unknown")
                    );
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Diff { run_a, run_b, mode, node, explain } => {
            let payload =
                replay_service::run_diff_mode_payload(run_a, run_b, *mode, node.as_deref())?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.diff",
                    true,
                    payload.clone(),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            if matches!(mode, crate::commands::DiffModeArg::Semantic) {
                print_human_diff(&payload);
            } else {
                println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            }
            if *explain {
                if let Some(summary) = payload
                    .get("root_cause_summary")
                    .or_else(|| {
                        payload
                            .get("replay_equivalence")
                            .and_then(|v| v.get("reason_report"))
                            .and_then(|v| v.get("summary"))
                    })
                    .and_then(serde_json::Value::as_str)
                {
                    println!("replay_reason: {summary}");
                }
                if let Some(cause_groups) = payload.get("cause_groups").or_else(|| {
                    payload.get("replay_equivalence").and_then(|v| v.get("cause_groups"))
                }) {
                    println!(
                        "replay_cause_groups: {}",
                        serde_json::to_string(cause_groups).unwrap()
                    );
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Verify { run_id, root, deep, strict } => {
            let run_dir = resolve_run_dir(root, run_id);
            let report = verify_run(&run_dir, *deep, *strict)?;
            let ok =
                report.get("status").and_then(|v| v.as_str()).map(|v| v == "ok").unwrap_or(false);
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.verify",
                    ok,
                    report,
                    Vec::new(),
                    if ok { ExitCode::SUCCESS } else { ExitCode::from(3) },
                );
            }
            println!("status: {}", if ok { "ok" } else { "invalid" });
            if !ok {
                return Err(ExitCode::from(3));
            }
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Doctor { run_id, root } => {
            let report = inspect_service::doctor_for_run_id(root, run_id);
            let ok = report.get("status").and_then(|v| v.as_str()) == Some("ok");
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.doctor",
                    ok,
                    report,
                    Vec::new(),
                    if ok { ExitCode::SUCCESS } else { ExitCode::from(3) },
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            if !ok {
                return Err(ExitCode::from(3));
            }
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::ExplainFailure { run_id, root } => {
            let report = inspect_service::explain_failure_for_run_id(root, run_id)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.explain-failure",
                    true,
                    report,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Summary { root } => {
            let report = runs_summary(root).map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.summary",
                    true,
                    report,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Compare { run_a, run_b, root } => {
            let report = runs_compare(root, run_a, run_b).map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.compare",
                    true,
                    report,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Trend { root } => {
            let report = runs_trend(root).map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.trend",
                    true,
                    report,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Failures { root } => {
            let report = runs_failures(root).map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.failures",
                    true,
                    report,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Flakes { root } => {
            let report = runs_flakes(root).map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.flakes",
                    true,
                    report,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::DiagnosticsBundle { run_id, root, out, redact } => {
            let run_dir = resolve_run_dir(root, run_id);
            let payload = diagnostics_bundle_payload(&run_dir, *redact)?;
            if let Some(parent) = out.parent() {
                fs::create_dir_all(parent).map_err(|_| ExitCode::from(3))?;
            }
            fs::write(out, serde_json::to_vec_pretty(&payload).map_err(|_| ExitCode::from(3))?)
                .map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.diagnostics-bundle",
                    true,
                    serde_json::json!({
                        "run_id": run_id,
                        "run_dir": run_dir,
                        "bundle_path": out
                    }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("wrote diagnostics bundle: {}", out.display());
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Index { root } => {
            let path = inspect_service::rebuild_run_history_index_for_root(root)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.index",
                    true,
                    serde_json::json!({
                        "index_path": path
                    }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("wrote run history index: {}", path.display());
            Ok(ExitCode::SUCCESS)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::handle_runs_command;
    use crate::commands::{Commands, DagCli, RunsCommands};
    use crate::inspect_service;
    use crate::run_views::RunTimelineQuery;
    use crate::ExitCode;
    use bijux_dag_artifacts::RunStopRequest;
    use bijux_dag_runtime::ExecutionCheckpoint;
    use serde_json::json;
    use std::fs;
    use std::path::Path;

    fn quiet_json_cli() -> DagCli {
        DagCli { json: true, quiet: true, command: Commands::Version }
    }

    fn write_run(root: &Path, run_id: &str, imported: bool) {
        let run = root.join(run_id);
        fs::create_dir_all(run.join("nodes/n1")).expect("mkdir nodes");
        let mut manifest = json!({
            "run_id": run_id,
            "status": "success",
            "run_dir_format": "run-dir/v0.1",
            "graph_fingerprint": "g1",
            "created_unix_ms": 1,
            "started_unix_ms": 1,
            "finished_unix_ms": 2,
            "node_counts": {"success": 1, "failed": 0, "skipped": 0, "cached": 0},
            "run_metadata": {"submission_source": "manual", "trigger_source": "manual", "labels": ["etl"]}
        });
        if imported {
            manifest["run_metadata"]["submission_source"] = json!("imported");
            manifest["run_metadata"]["labels"] = json!(["etl", "imported"]);
        }
        fs::write(
            run.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest).expect("manifest"),
        )
        .expect("write manifest");
        fs::write(
            run.join("snapshot.json"),
            serde_json::to_vec_pretty(&json!({
                "graph": {
                    "nodes": [{"id":"n1"}],
                    "edges": []
                }
            }))
            .expect("snapshot"),
        )
        .expect("write snapshot");
        fs::write(run.join("outputs.index.json"), b"[]").expect("write outputs index");
        fs::write(
            run.join("nodes/n1/trace.json"),
            serde_json::to_vec_pretty(&json!({
                "status":"success","started_unix_ms":1,"finished_unix_ms":2,"attempt":1
            }))
            .expect("trace"),
        )
        .expect("write trace");
    }

    fn write_timeline_run(root: &Path, run_id: &str) {
        write_run(root, run_id, false);
        fs::write(
            root.join(run_id).join("observability.timeline.json"),
            serde_json::to_vec_pretty(&json!({
                "schema_version": "v0.1",
                "entries": [
                    {
                        "unix_ms": 100u64,
                        "category": "run",
                        "label": "run_started",
                        "node_id": null,
                        "source_event": "run_started"
                    },
                    {
                        "unix_ms": 130u64,
                        "category": "failure",
                        "label": "node_failed",
                        "node_id": "n1",
                        "status": "failed",
                        "reason": "execution_failed",
                        "source_event": "node_finished"
                    }
                ]
            }))
            .expect("timeline"),
        )
        .expect("write timeline");
    }

    fn write_scheduler_checkpoint_run(root: &Path, run_id: &str) {
        write_run(root, run_id, false);
        let checkpoint = ExecutionCheckpoint {
            loop_index: 4,
            ready_queue_depth: 1,
            ready_queue: vec!["publish".to_string()],
            inflight: vec!["package".to_string()],
            scheduled: vec!["package".to_string()],
            blocked_by_budget: vec!["notify".to_string()],
            blocked_reasons: std::collections::BTreeMap::from([(
                "notify".to_string(),
                "memory budget exhausted".to_string(),
            )]),
            completed_statuses: std::collections::BTreeMap::from([(
                "build".to_string(),
                "success".to_string(),
            )]),
            decision_reason: "ready_batch".to_string(),
            failure_propagation_mode: "continue_independent".to_string(),
            dependency_closure_enabled: false,
            generated_unix_ms: 420,
        };
        fs::write(
            root.join(run_id).join("scheduler.checkpoint.json"),
            serde_json::to_vec_pretty(&checkpoint).expect("checkpoint"),
        )
        .expect("write checkpoint");
    }

    fn write_active_run(root: &Path, run_id: &str) -> std::path::PathBuf {
        let normalized = run_id.strip_prefix("run-").unwrap_or(run_id);
        let run = root.join(format!("run.tmp-{normalized}"));
        fs::create_dir_all(run.join("nodes/prepare")).expect("mkdir nodes");
        fs::write(
            run.join("manifest.json"),
            serde_json::to_vec_pretty(&json!({
                "run_id": normalized,
                "status": "running",
                "run_dir_format": "run-dir/v0.1",
                "graph_fingerprint": "g-active",
                "created_unix_ms": 1,
                "started_unix_ms": 1,
                "finished_unix_ms": 1,
                "node_counts": {"success": 0, "failed": 0, "skipped": 0, "cached": 0, "cancelled": 0},
                "run_metadata": {"submission_source": "manual", "trigger_source": "cli", "operator": "ops"}
            }))
            .expect("manifest"),
        )
        .expect("write manifest");
        run
    }

    #[test]
    fn runs_routes_support_listing_and_summary_flows() {
        let tmp = tempfile::tempdir().expect("tmp");
        write_run(tmp.path(), "run-a", false);
        let cli = quiet_json_cli();
        let list =
            handle_runs_command(&cli, &RunsCommands::List { root: tmp.path().to_path_buf() })
                .expect("list");
        assert_eq!(list, ExitCode::SUCCESS);
        let summary =
            handle_runs_command(&cli, &RunsCommands::Summary { root: tmp.path().to_path_buf() })
                .expect("summary");
        assert_eq!(summary, ExitCode::SUCCESS);
    }

    #[test]
    fn runs_routes_support_timeline_and_tree_flows() {
        let tmp = tempfile::tempdir().expect("tmp");
        write_run(tmp.path(), "run-tree", false);
        let cli = quiet_json_cli();
        let tree = handle_runs_command(
            &cli,
            &RunsCommands::Tree { run_id: "run-tree".to_string(), root: tmp.path().to_path_buf() },
        )
        .expect("tree");
        assert_eq!(tree, ExitCode::SUCCESS);
        let timeline = handle_runs_command(
            &cli,
            &RunsCommands::Timeline {
                run_id: "run-tree".to_string(),
                root: tmp.path().to_path_buf(),
                node: None,
                event: None,
                since_unix_ms: None,
                until_unix_ms: None,
            },
        )
        .expect("timeline");
        assert_eq!(timeline, ExitCode::SUCCESS);
    }

    #[test]
    fn runs_routes_support_imported_bundle_like_flows() {
        let tmp = tempfile::tempdir().expect("tmp");
        write_run(tmp.path(), "run-imported", true);
        let cli = quiet_json_cli();
        let inspect = handle_runs_command(
            &cli,
            &RunsCommands::Inspect {
                run_id: "run-imported".to_string(),
                root: tmp.path().to_path_buf(),
            },
        )
        .expect("inspect");
        assert_eq!(inspect, ExitCode::SUCCESS);
    }

    #[test]
    fn runs_stop_records_request_for_active_run() {
        let tmp = tempfile::tempdir().expect("tmp");
        let active_run = write_active_run(tmp.path(), "run-stoppable");
        let cli = quiet_json_cli();

        let result = handle_runs_command(
            &cli,
            &RunsCommands::Stop {
                run_id: "run-stoppable".to_string(),
                root: tmp.path().to_path_buf(),
            },
        )
        .expect("stop");
        assert_eq!(result, ExitCode::SUCCESS);

        let request: RunStopRequest = serde_json::from_str(
            &fs::read_to_string(active_run.join("run.stop-request.json")).expect("read request"),
        )
        .expect("parse request");
        assert_eq!(request.run_id, "stoppable");
        assert_eq!(request.source, "cli");
        assert!(request.reason.is_none());
    }

    #[test]
    fn runs_stop_is_idempotent_for_active_run() {
        let tmp = tempfile::tempdir().expect("tmp");
        let active_run = write_active_run(tmp.path(), "run-stoppable");
        let cli = quiet_json_cli();

        handle_runs_command(
            &cli,
            &RunsCommands::Stop {
                run_id: "run-stoppable".to_string(),
                root: tmp.path().to_path_buf(),
            },
        )
        .expect("initial stop");
        let first_request =
            fs::read_to_string(active_run.join("run.stop-request.json")).expect("first request");

        let result = handle_runs_command(
            &cli,
            &RunsCommands::Stop { run_id: "stoppable".to_string(), root: tmp.path().to_path_buf() },
        )
        .expect("repeated stop");
        assert_eq!(result, ExitCode::SUCCESS);

        let second_request =
            fs::read_to_string(active_run.join("run.stop-request.json")).expect("second request");
        assert_eq!(first_request, second_request);
    }

    #[test]
    fn runs_history_supports_filter_and_pagination_flags() {
        let tmp = tempfile::tempdir().expect("tmp");
        write_run(tmp.path(), "run-a", false);
        write_run(tmp.path(), "run-b", true);
        let cli = quiet_json_cli();
        let history = handle_runs_command(
            &cli,
            &RunsCommands::History {
                root: tmp.path().to_path_buf(),
                status: Some("success".to_string()),
                graph: None,
                source: Some("imported".to_string()),
                offset: Some(0),
                limit: Some(10),
                select: vec!["tag:etl".to_string(), "run:run-b".to_string()],
            },
        )
        .expect("history");
        assert_eq!(history, ExitCode::SUCCESS);
    }

    #[test]
    fn runs_routes_tolerate_corrupted_run_dir_without_panic() {
        let tmp = tempfile::tempdir().expect("tmp");
        let run = tmp.path().join("run-bad");
        fs::create_dir_all(&run).expect("mkdir");
        fs::write(run.join("manifest.json"), b"{bad-json").expect("manifest");
        let cli = quiet_json_cli();
        let result = std::panic::catch_unwind(|| {
            handle_runs_command(
                &cli,
                &RunsCommands::Timeline {
                    run_id: "run-bad".to_string(),
                    root: tmp.path().to_path_buf(),
                    node: None,
                    event: None,
                    since_unix_ms: None,
                    until_unix_ms: None,
                },
            )
        });
        assert!(result.is_ok(), "timeline flow should not panic");
        assert!(result.expect("result").is_ok());
    }

    #[test]
    fn runs_timeline_human_output_surfaces_filters_timestamp_and_cause() {
        let tmp = tempfile::tempdir().expect("tmp");
        write_timeline_run(tmp.path(), "run-timeline");
        let report = inspect_service::run_timeline_for_id_with_query(
            tmp.path(),
            "run-timeline",
            &RunTimelineQuery {
                node: Some("n1".to_string()),
                event: Some("node_failed".to_string()),
                since_unix_ms: Some(120),
                until_unix_ms: Some(140),
            },
        )
        .expect("timeline");

        let rendered = super::format_run_timeline_human(&report);
        assert!(rendered.contains("matched: 1/2 events"));
        assert!(rendered
            .contains("filters: node=n1 event=node_failed since_unix_ms=120 until_unix_ms=140"));
        assert!(rendered.contains("timestamp_unix_ms=130"));
        assert!(rendered.contains("cause=execution_failed"));
        assert!(rendered.contains("source_event=node_finished"));
    }

    #[test]
    fn runs_scheduler_checkpoint_reports_decisions_and_blocked_nodes() {
        let tmp = tempfile::tempdir().expect("tmp");
        write_scheduler_checkpoint_run(tmp.path(), "run-scheduler");

        let report = inspect_service::scheduler_checkpoint_for_run_id(tmp.path(), "run-scheduler")
            .expect("scheduler checkpoint");

        assert_eq!(report["inspection_state"], "present");
        assert_eq!(report["decision_reason"], "ready_batch");
        assert_eq!(report["ready_queue"], json!(["publish"]));
        assert_eq!(report["scheduled_batch"], json!(["package"]));
        assert_eq!(report["inflight_nodes"], json!(["package"]));
        assert_eq!(report["resource_blocked_nodes"], json!(["notify"]));
        assert_eq!(report["blocked_reasons"]["notify"], "memory budget exhausted");
    }

    #[test]
    fn scheduler_checkpoint_human_output_surfaces_decision_reason_and_budget_blocks() {
        let report = json!({
            "inspection_state": "present",
            "checkpoint_path": "/tmp/run-scheduler/scheduler.checkpoint.json",
            "decision_reason": "ready_batch",
            "ready_queue_depth": 1,
            "ready_queue": ["publish"],
            "scheduled_batch": ["package"],
            "inflight_nodes": ["package"],
            "resource_blocked_nodes": ["notify"],
            "completed_statuses": {"build": "success"},
            "blocked_reasons": {"notify": "memory budget exhausted"},
        });

        let rendered = super::format_scheduler_checkpoint_human(&report);
        assert!(rendered.contains("decision_reason: ready_batch"));
        assert!(rendered.contains("resource_blocked_nodes: notify"));
        assert!(rendered.contains("blocked_reasons: notify=memory budget exhausted"));
    }

    #[test]
    fn runs_diagnostics_bundle_exports_redacted_payload() {
        let tmp = tempfile::tempdir().expect("tmp");
        write_run(tmp.path(), "run-secret", true);
        let run_manifest_path = tmp.path().join("run-secret").join("manifest.json");
        let mut manifest: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&run_manifest_path).expect("read manifest"))
                .expect("parse manifest");
        manifest["run_metadata"]["api_token"] = serde_json::json!("secret-token-value");
        fs::write(
            &run_manifest_path,
            serde_json::to_vec_pretty(&manifest).expect("encode manifest"),
        )
        .expect("write manifest");
        fs::write(
            tmp.path().join("run-secret").join("observability.events.json"),
            b"{\"status\":\"ok\"}",
        )
        .expect("write events");

        let bundle_path = tmp.path().join("bundle.json");
        let cli = quiet_json_cli();
        let result = handle_runs_command(
            &cli,
            &RunsCommands::DiagnosticsBundle {
                run_id: "run-secret".to_string(),
                root: tmp.path().to_path_buf(),
                out: bundle_path.clone(),
                redact: true,
            },
        )
        .expect("bundle");
        assert_eq!(result, ExitCode::SUCCESS);

        let bundle: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(bundle_path).expect("read bundle"))
                .expect("parse bundle");
        assert_eq!(bundle["bundle_version"], "dag-diagnostics-bundle/v0.1");
        assert_eq!(bundle["command_context"]["submission_source"], "imported");
        assert_eq!(bundle["command_context"]["operator"], serde_json::Value::Null);
        assert_eq!(bundle["run_dir"], tmp.path().join("run-secret").display().to_string());
        assert_eq!(
            bundle["manifest"]["run_metadata"]["api_token"], "[REDACTED]",
            "secret metadata must be redacted in exported bundle"
        );
    }

    #[test]
    fn runs_index_writes_history_index_file() {
        let tmp = tempfile::tempdir().expect("tmp");
        write_run(tmp.path(), "run-indexed", false);
        let cli = quiet_json_cli();
        let result =
            handle_runs_command(&cli, &RunsCommands::Index { root: tmp.path().to_path_buf() })
                .expect("index");
        assert_eq!(result, ExitCode::SUCCESS);
        assert!(tmp.path().join(".bijux-run-history-index.json").exists());
    }
}
