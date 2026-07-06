use crate::run_data::read_node_traces;
use bijux_dag_artifacts::TriggerParentStatus;
use bijux_dag_core::Graph;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum NodeExecutionClassification {
    Executed,
    DependencyBlocked,
    TriggerRuleBlocked,
    BranchSkipped,
    ResourceBlocked,
    SelectorExcluded,
    CacheReused,
    PolicyDenied,
    Unknown,
}

impl NodeExecutionClassification {
    fn as_str(self) -> &'static str {
        match self {
            Self::Executed => "executed",
            Self::DependencyBlocked => "dependency_blocked",
            Self::TriggerRuleBlocked => "trigger_rule_blocked",
            Self::BranchSkipped => "branch_skipped",
            Self::ResourceBlocked => "resource_blocked",
            Self::SelectorExcluded => "selector_excluded",
            Self::CacheReused => "cache_reused",
            Self::PolicyDenied => "policy_denied",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct NodeExecutionExplanation {
    pub classification: NodeExecutionClassification,
    pub executed: bool,
    pub reason: String,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocking_nodes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_rule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parent_statuses: Vec<TriggerParentStatus>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scheduler_reasons: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_sources: Vec<String>,
}

#[derive(Debug, Default)]
struct NodeEventEvidence {
    details: Vec<Value>,
    source: Option<&'static str>,
}

pub(crate) fn explain_node_execution(
    run_dir: &Path,
    graph: &Graph,
    node_id: &str,
    trace: Option<&Value>,
) -> NodeExecutionExplanation {
    let node_events = read_node_event_evidence(run_dir, node_id);
    let latest_blocked =
        node_events.details.iter().rev().find(|event| event_name(event) == Some("node_blocked"));
    let latest_block_reason = latest_blocked.and_then(event_reason);
    let scheduler_reasons = node_events
        .details
        .iter()
        .filter(|event| event_name(event) == Some("node_blocked"))
        .filter_map(event_reason)
        .collect::<Vec<_>>();
    let trigger_rule =
        trace_trigger_rule(trace).or_else(|| latest_blocked.and_then(event_trigger_rule));
    let trigger_reason = trace_trigger_reason(trace);
    let parent_statuses = trace_parent_statuses(trace);
    let all_traces = read_node_traces(run_dir).unwrap_or_default();
    let dependencies = dependency_node_ids(graph, node_id);
    let blocking_nodes = latest_blocked
        .and_then(event_blocking_nodes)
        .or_else(|| parent_blocking_nodes(&parent_statuses))
        .or_else(|| dependency_blocking_nodes(&dependencies, &all_traces))
        .unwrap_or_default();
    let trace_status = trace_status(trace);
    let trace_transition_cause = trace_transition_cause(trace);
    let trace_failure_code = trace_failure_code(trace);
    let trace_failure_message = trace_failure_message(trace);
    let trace_skip_reason = trace_skip_reason(trace);
    let mut evidence_sources = trace_evidence_sources(trace);
    if let Some(source) = node_events.source {
        evidence_sources.push(source.to_string());
    }

    let classification =
        if trace_status == Some("cached") || trace_transition_cause == Some("CachedReuse") {
            NodeExecutionClassification::CacheReused
        } else if trace_failure_code == Some("POLICY_DENIED")
            || trace_transition_cause == Some("PolicyDenied")
        {
            NodeExecutionClassification::PolicyDenied
        } else if trace_trigger_satisfied(trace) == Some(false)
            || latest_block_reason.as_deref() == Some("blocked_by_trigger_rule")
        {
            NodeExecutionClassification::TriggerRuleBlocked
        } else if matches!(latest_block_reason.as_deref(), Some("branch_decision_not_selected"))
            || matches!(trace_skip_reason, Some("branch_decision_not_selected"))
            || trace_transition_cause == Some("BranchDecisionFiltered")
        {
            NodeExecutionClassification::BranchSkipped
        } else if matches!(
            latest_block_reason.as_deref(),
            Some(
                "filtered"
                    | "not_selected_by_include_selector"
                    | "excluded_by_selector"
                    | "not_selected_by_dependency_closure"
            )
        ) || matches!(
            trace_skip_reason,
            Some(
                "filtered"
                    | "not_selected_by_include_selector"
                    | "excluded_by_selector"
                    | "not_selected_by_dependency_closure"
            )
        ) || trace_transition_cause == Some("SelectionFiltered")
        {
            NodeExecutionClassification::SelectorExcluded
        } else if latest_block_reason.as_deref() == Some("upstream_failed")
            || trace_failure_code == Some("UPSTREAM_FAILED")
            || trace_skip_reason == Some("upstream_failed")
            || trace_transition_cause == Some("DependencyFailed")
            || (!is_terminal_status(trace_status)
                && has_non_success_dependencies(&dependencies, &all_traces))
        {
            NodeExecutionClassification::DependencyBlocked
        } else if !is_terminal_status(trace_status)
            && scheduler_reasons.iter().any(|reason| {
                reason.starts_with("blocked_by_") && reason != "blocked_by_trigger_rule"
            })
        {
            NodeExecutionClassification::ResourceBlocked
        } else if did_node_execute(trace_status) {
            NodeExecutionClassification::Executed
        } else {
            NodeExecutionClassification::Unknown
        };

    let executed = did_node_execute(trace_status);
    let reason = explanation_reason(
        classification,
        trace_status,
        trace_transition_cause,
        trace_failure_code,
        trace_skip_reason,
        latest_block_reason.as_deref().or_else(|| scheduler_reasons.first().map(String::as_str)),
    );
    let summary = explanation_summary(
        classification,
        &reason,
        &blocking_nodes,
        trigger_rule.as_deref(),
        trigger_reason.as_deref(),
        trace_failure_message,
        trace_status,
    );

    NodeExecutionExplanation {
        classification,
        executed,
        reason,
        summary,
        blocking_nodes,
        trigger_rule,
        trigger_reason,
        parent_statuses,
        scheduler_reasons,
        evidence_sources: dedup_strings(evidence_sources),
    }
}

pub(crate) fn format_node_execution_explanation_human(
    explanation: &NodeExecutionExplanation,
) -> String {
    let blocking_nodes = if explanation.blocking_nodes.is_empty() {
        "-".to_string()
    } else {
        explanation.blocking_nodes.join(", ")
    };
    let scheduler_reasons = if explanation.scheduler_reasons.is_empty() {
        "-".to_string()
    } else {
        explanation.scheduler_reasons.join(", ")
    };
    let evidence_sources = if explanation.evidence_sources.is_empty() {
        "-".to_string()
    } else {
        explanation.evidence_sources.join(", ")
    };
    let trigger_rule = explanation.trigger_rule.as_deref().unwrap_or("-");
    let trigger_reason = explanation.trigger_reason.as_deref().unwrap_or("-");

    format!(
        "executed={} classification={} reason={} summary={} blocking_nodes={} trigger_rule={} trigger_reason={} scheduler_reasons={} evidence_sources={}",
        explanation.executed,
        explanation.classification.as_str(),
        explanation.reason,
        explanation.summary,
        blocking_nodes,
        trigger_rule,
        trigger_reason,
        scheduler_reasons,
        evidence_sources
    )
}

fn read_node_event_evidence(run_dir: &Path, node_id: &str) -> NodeEventEvidence {
    for (path, source) in [
        (run_dir.join("observability.events.json"), "observability.events.json"),
        (run_dir.join("run-log.index.json"), "run-log.index.json"),
    ] {
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(events) = serde_json::from_str::<Vec<Value>>(&raw) else {
            continue;
        };
        let details = events
            .into_iter()
            .map(|event| event.get("details").cloned().unwrap_or(event))
            .filter(|detail| detail.get("node_id").and_then(Value::as_str) == Some(node_id))
            .collect::<Vec<_>>();
        if !details.is_empty() {
            return NodeEventEvidence { details, source: Some(source) };
        }
    }
    NodeEventEvidence::default()
}

fn dependency_node_ids(graph: &Graph, node_id: &str) -> Vec<String> {
    graph
        .edges
        .iter()
        .filter(|edge| edge.to.node_id == node_id)
        .map(|edge| edge.from.node_id.clone())
        .collect::<Vec<_>>()
}

fn dependency_blocking_nodes(
    dependencies: &[String],
    traces: &HashMap<String, Value>,
) -> Option<Vec<String>> {
    let blocking = dependencies
        .iter()
        .filter(|dependency| {
            traces.get(*dependency).is_some_and(|trace| {
                !matches!(trace.get("status").and_then(Value::as_str), Some("success" | "cached"))
            })
        })
        .cloned()
        .collect::<Vec<_>>();
    (!blocking.is_empty()).then_some(blocking)
}

fn has_non_success_dependencies(dependencies: &[String], traces: &HashMap<String, Value>) -> bool {
    dependency_blocking_nodes(dependencies, traces).into_iter().flatten().next().is_some()
}

fn event_name(event: &Value) -> Option<&str> {
    event.get("event").and_then(Value::as_str)
}

fn event_reason(event: &Value) -> Option<String> {
    event.get("reason").and_then(Value::as_str).map(ToString::to_string)
}

fn event_blocking_nodes(event: &Value) -> Option<Vec<String>> {
    event.get("blocking_nodes").and_then(Value::as_array).map(|nodes| {
        nodes.iter().filter_map(Value::as_str).map(ToString::to_string).collect::<Vec<_>>()
    })
}

fn event_trigger_rule(event: &Value) -> Option<String> {
    event.get("trigger_rule").and_then(Value::as_str).map(ToString::to_string)
}

fn trace_status(trace: Option<&Value>) -> Option<&str> {
    trace.and_then(|trace| trace.get("status")).and_then(Value::as_str)
}

fn trace_transition_cause(trace: Option<&Value>) -> Option<&str> {
    trace.and_then(|trace| trace.get("transition_cause")).and_then(Value::as_str)
}

fn trace_failure_code(trace: Option<&Value>) -> Option<&str> {
    trace
        .and_then(|trace| trace.get("failure"))
        .and_then(|failure| failure.get("code"))
        .and_then(Value::as_str)
}

fn trace_failure_message(trace: Option<&Value>) -> Option<&str> {
    trace
        .and_then(|trace| trace.get("failure"))
        .and_then(|failure| failure.get("message"))
        .and_then(Value::as_str)
}

fn trace_skip_reason(trace: Option<&Value>) -> Option<&str> {
    trace
        .and_then(|trace| trace.get("skip_reason"))
        .and_then(|skip_reason| skip_reason.get("reason"))
        .and_then(Value::as_str)
}

fn trace_trigger_rule(trace: Option<&Value>) -> Option<String> {
    trace
        .and_then(|trace| trace.get("trigger_evaluation"))
        .and_then(|trigger| trigger.get("trigger_rule"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn trace_trigger_reason(trace: Option<&Value>) -> Option<String> {
    trace
        .and_then(|trace| trace.get("trigger_evaluation"))
        .and_then(|trigger| trigger.get("reason"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn trace_trigger_satisfied(trace: Option<&Value>) -> Option<bool> {
    trace
        .and_then(|trace| trace.get("trigger_evaluation"))
        .and_then(|trigger| trigger.get("satisfied"))
        .and_then(Value::as_bool)
}

fn trace_parent_statuses(trace: Option<&Value>) -> Vec<TriggerParentStatus> {
    trace
        .and_then(|trace| trace.get("trigger_evaluation"))
        .and_then(|trigger| trigger.get("parent_statuses"))
        .cloned()
        .and_then(|value| serde_json::from_value::<Vec<TriggerParentStatus>>(value).ok())
        .unwrap_or_default()
}

fn parent_blocking_nodes(parent_statuses: &[TriggerParentStatus]) -> Option<Vec<String>> {
    let blocking = parent_statuses
        .iter()
        .filter(|status| !matches!(status.status.as_str(), "success" | "cached"))
        .map(|status| status.node_id.clone())
        .collect::<Vec<_>>();
    (!blocking.is_empty()).then_some(blocking)
}

fn trace_evidence_sources(trace: Option<&Value>) -> Vec<String> {
    let mut sources = Vec::new();
    if trace.is_some() {
        sources.push("trace.status".to_string());
    }
    if trace_failure_code(trace).is_some() {
        sources.push("trace.failure".to_string());
    }
    if trace_skip_reason(trace).is_some() {
        sources.push("trace.skip_reason".to_string());
    }
    if trace_transition_cause(trace).is_some() {
        sources.push("trace.transition_cause".to_string());
    }
    if trace_trigger_rule(trace).is_some() {
        sources.push("trace.trigger_evaluation".to_string());
    }
    sources
}

fn is_terminal_status(status: Option<&str>) -> bool {
    matches!(status, Some("success" | "failed" | "skipped" | "cached" | "cancelled"))
}

fn did_node_execute(status: Option<&str>) -> bool {
    matches!(status, Some("success" | "failed"))
}

fn explanation_reason(
    classification: NodeExecutionClassification,
    trace_status: Option<&str>,
    transition_cause: Option<&str>,
    failure_code: Option<&str>,
    skip_reason: Option<&str>,
    scheduler_reason: Option<&str>,
) -> String {
    match classification {
        NodeExecutionClassification::CacheReused => "cache_reused".to_string(),
        NodeExecutionClassification::PolicyDenied => "policy_denied".to_string(),
        NodeExecutionClassification::TriggerRuleBlocked => "blocked_by_trigger_rule".to_string(),
        NodeExecutionClassification::BranchSkipped => "branch_decision_not_selected".to_string(),
        NodeExecutionClassification::SelectorExcluded => {
            skip_reason.unwrap_or("selector_excluded").to_string()
        }
        NodeExecutionClassification::DependencyBlocked => "upstream_failed".to_string(),
        NodeExecutionClassification::ResourceBlocked => {
            scheduler_reason.unwrap_or("blocked_by_resource").to_string()
        }
        NodeExecutionClassification::Executed => failure_code
            .map(ToString::to_string)
            .or_else(|| transition_cause.map(snake_case_transition_cause))
            .or_else(|| trace_status.map(ToString::to_string))
            .unwrap_or_else(|| "executed".to_string()),
        NodeExecutionClassification::Unknown => "unknown".to_string(),
    }
}

fn explanation_summary(
    classification: NodeExecutionClassification,
    reason: &str,
    blocking_nodes: &[String],
    trigger_rule: Option<&str>,
    trigger_reason: Option<&str>,
    failure_message: Option<&str>,
    trace_status: Option<&str>,
) -> String {
    match classification {
        NodeExecutionClassification::CacheReused => {
            "node outputs were reused from cache instead of re-executing".to_string()
        }
        NodeExecutionClassification::PolicyDenied => format!(
            "node did not execute because policy denied it{}",
            failure_message.map(|message| format!(": {message}")).unwrap_or_default()
        ),
        NodeExecutionClassification::TriggerRuleBlocked => format!(
            "node did not execute because trigger rule `{}` was unsatisfied{}",
            trigger_rule.unwrap_or("unknown"),
            trigger_reason.map(|detail| format!(" ({detail})")).unwrap_or_default()
        ),
        NodeExecutionClassification::BranchSkipped => {
            "node was skipped because the selected branch did not include it".to_string()
        }
        NodeExecutionClassification::SelectorExcluded => {
            format!("node was excluded before execution ({reason})")
        }
        NodeExecutionClassification::DependencyBlocked => {
            if blocking_nodes.is_empty() {
                "node did not execute because upstream dependencies were not ready".to_string()
            } else {
                format!(
                    "node did not execute because upstream dependencies blocked it: {}",
                    blocking_nodes.join(", ")
                )
            }
        }
        NodeExecutionClassification::ResourceBlocked => {
            format!("node has not executed; scheduler last reported `{reason}`")
        }
        NodeExecutionClassification::Executed => format!(
            "node executed and reached terminal status `{}`",
            trace_status.unwrap_or("unknown")
        ),
        NodeExecutionClassification::Unknown => format!(
            "insufficient persisted evidence to explain the node execution state{}",
            trace_status.map(|status| format!(" (current status `{status}`)")).unwrap_or_default()
        ),
    }
}

fn snake_case_transition_cause(cause: &str) -> String {
    let mut result = String::with_capacity(cause.len());
    for (index, ch) in cause.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if index > 0 {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }
    result
}

fn dedup_strings(values: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    values.into_iter().filter(|value| seen.insert(value.clone())).collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::{
        explain_node_execution, format_node_execution_explanation_human,
        NodeExecutionClassification,
    };
    use bijux_dag_core::Graph;
    use serde_json::json;
    use std::fs;

    fn graph_fixture() -> Graph {
        serde_json::from_value(json!({
            "spec":"bijux-dag/v0.1",
            "meta":{"name":"node-explain","owners":[],"tags":[]},
            "nodes":[
                {"id":"seed","kind":"const","outputs":[{"name":"out","path":"seed/out"}]},
                {"id":"branch_left","kind":"const","outputs":[{"name":"out","path":"left/out"}]},
                {"id":"publish","kind":"shell","inputs":["seed_in"],"outputs":[{"name":"out","path":"publish/out"}]}
            ],
            "edges":[
                {"from":{"node_id":"seed","port":"out"},"to":{"node_id":"publish","port":"seed_in"}}
            ]
        }))
        .expect("graph fixture")
    }

    #[test]
    fn classifies_cache_reuse_from_trace_transition() {
        let tmp = tempfile::tempdir().expect("tmp");
        let explanation = explain_node_execution(
            tmp.path(),
            &graph_fixture(),
            "publish",
            Some(&json!({
                "status":"cached",
                "transition_cause":"CachedReuse"
            })),
        );

        assert_eq!(explanation.classification, NodeExecutionClassification::CacheReused);
        assert!(!explanation.executed);
        assert_eq!(explanation.reason, "cache_reused");
    }

    #[test]
    fn classifies_trigger_rule_block_from_trace_and_event() {
        let tmp = tempfile::tempdir().expect("tmp");
        fs::write(
            tmp.path().join("observability.events.json"),
            serde_json::to_vec_pretty(&vec![json!({
                "details": {
                    "event":"node_blocked",
                    "node_id":"publish",
                    "reason":"blocked_by_trigger_rule",
                    "blocking_nodes":["seed"],
                    "trigger_rule":"all_success"
                }
            })])
            .expect("events"),
        )
        .expect("write events");

        let explanation = explain_node_execution(
            tmp.path(),
            &graph_fixture(),
            "publish",
            Some(&json!({
                "status":"skipped",
                "trigger_evaluation":{
                    "trigger_rule":"all_success",
                    "satisfied":false,
                    "reason":"upstream dependency failed",
                    "parent_statuses":[{"node_id":"seed","status":"failed"}]
                },
                "skip_reason":{"reason":"upstream_failed"},
                "transition_cause":"DependencyFailed"
            })),
        );

        assert_eq!(explanation.classification, NodeExecutionClassification::TriggerRuleBlocked);
        assert_eq!(explanation.blocking_nodes, vec!["seed".to_string()]);
        assert_eq!(explanation.trigger_rule.as_deref(), Some("all_success"));
    }

    #[test]
    fn classifies_resource_block_from_scheduler_event_without_trace() {
        let tmp = tempfile::tempdir().expect("tmp");
        fs::write(
            tmp.path().join("run-log.index.json"),
            serde_json::to_vec_pretty(&vec![json!({
                "event":"node_blocked",
                "node_id":"publish",
                "reason":"blocked_by_cpu"
            })])
            .expect("events"),
        )
        .expect("write events");

        let explanation = explain_node_execution(tmp.path(), &graph_fixture(), "publish", None);

        assert_eq!(explanation.classification, NodeExecutionClassification::ResourceBlocked);
        assert!(!explanation.executed);
        assert_eq!(explanation.reason, "blocked_by_cpu");
        assert_eq!(explanation.evidence_sources, vec!["run-log.index.json".to_string()]);
    }

    #[test]
    fn classifies_dependency_block_from_upstream_trace_when_trace_is_missing() {
        let tmp = tempfile::tempdir().expect("tmp");
        let publish_dir = tmp.path().join("nodes").join("seed");
        fs::create_dir_all(&publish_dir).expect("mkdir nodes");
        fs::write(
            publish_dir.join("trace.json"),
            serde_json::to_vec_pretty(&json!({
                "node_id":"seed",
                "status":"failed"
            }))
            .expect("trace"),
        )
        .expect("write trace");

        let explanation = explain_node_execution(tmp.path(), &graph_fixture(), "publish", None);

        assert_eq!(explanation.classification, NodeExecutionClassification::DependencyBlocked);
        assert_eq!(explanation.blocking_nodes, vec!["seed".to_string()]);
    }

    #[test]
    fn human_format_uses_stable_classification_labels() {
        let tmp = tempfile::tempdir().expect("tmp");
        fs::write(
            tmp.path().join("run-log.index.json"),
            serde_json::to_vec_pretty(&vec![json!({
                "event":"node_blocked",
                "node_id":"publish",
                "reason":"blocked_by_parallelism"
            })])
            .expect("events"),
        )
        .expect("write events");
        let explanation = explain_node_execution(tmp.path(), &graph_fixture(), "publish", None);

        let rendered = format_node_execution_explanation_human(&explanation);
        assert!(rendered.contains("classification=resource_blocked"));
        assert!(rendered.contains("reason=blocked_by_parallelism"));
    }
}
