use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Event domains required for runtime evidence taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservabilityEventDomainV1 {
    Planner,
    Scheduler,
    Adapter,
    Artifact,
    Cache,
    Replay,
    Policy,
    Security,
    Operator,
}

/// Structured runtime event record used for evidence reconstruction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservabilityEventRecordV1 {
    pub event_id: String,
    pub unix_ms: u64,
    pub domain: ObservabilityEventDomainV1,
    pub name: String,
    pub run_id: String,
    pub node_attempt_id: Option<String>,
}

/// Taxonomy completeness report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservabilityTaxonomyReportV1 {
    pub total_events: usize,
    pub domains_present: Vec<ObservabilityEventDomainV1>,
    pub missing_domains: Vec<ObservabilityEventDomainV1>,
}

/// Verify event taxonomy coverage for planner/scheduler/adapter/artifact/cache/replay/policy/security/operator.
pub fn verify_event_taxonomy_complete(
    events: &[ObservabilityEventRecordV1],
) -> Result<ObservabilityTaxonomyReportV1, String> {
    if events.is_empty() {
        return Err("event taxonomy verification requires at least one event".to_string());
    }
    let mut present = BTreeSet::new();
    for event in events {
        if event.event_id.trim().is_empty() {
            return Err("event taxonomy contains empty event_id".to_string());
        }
        if event.name.trim().is_empty() {
            return Err(format!("event '{}' has empty name", event.event_id));
        }
        if event.run_id.trim().is_empty() {
            return Err(format!("event '{}' has empty run_id", event.event_id));
        }
        present.insert(event.domain);
    }

    let all = [
        ObservabilityEventDomainV1::Planner,
        ObservabilityEventDomainV1::Scheduler,
        ObservabilityEventDomainV1::Adapter,
        ObservabilityEventDomainV1::Artifact,
        ObservabilityEventDomainV1::Cache,
        ObservabilityEventDomainV1::Replay,
        ObservabilityEventDomainV1::Policy,
        ObservabilityEventDomainV1::Security,
        ObservabilityEventDomainV1::Operator,
    ];

    let mut missing_domains = Vec::new();
    for domain in all {
        if !present.contains(&domain) {
            missing_domains.push(domain);
        }
    }

    let domains_present = present.into_iter().collect::<Vec<_>>();
    let report = ObservabilityTaxonomyReportV1 {
        total_events: events.len(),
        domains_present,
        missing_domains,
    };
    if !report.missing_domains.is_empty() {
        return Err(format!(
            "event taxonomy is incomplete: missing domains {:?}",
            report.missing_domains
        ));
    }
    Ok(report)
}

/// End-to-end correlation chain across command, run, node, artifact, and support evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrelationChainV1 {
    pub root_command_id: String,
    pub dag_command_id: String,
    pub run_id: String,
    pub node_attempt_id: String,
    pub artifact_id: String,
    pub support_bundle_id: String,
}

/// Verify correlation IDs are present across all required runtime surfaces.
pub fn validate_end_to_end_correlation_ids(chain: &CorrelationChainV1) -> Result<(), String> {
    for (field, value) in [
        ("root_command_id", chain.root_command_id.as_str()),
        ("dag_command_id", chain.dag_command_id.as_str()),
        ("run_id", chain.run_id.as_str()),
        ("node_attempt_id", chain.node_attempt_id.as_str()),
        ("artifact_id", chain.artifact_id.as_str()),
        ("support_bundle_id", chain.support_bundle_id.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!("correlation chain must include {field}"));
        }
    }
    if !chain.node_attempt_id.starts_with(&chain.run_id) {
        return Err("node_attempt_id must be namespaced by run_id".to_string());
    }
    Ok(())
}

/// Deterministic timeline entry reconstructed from runtime events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconstructedTimelineEntryV1 {
    pub event_id: String,
    pub run_id: String,
    pub node_attempt_id: Option<String>,
    pub domain: ObservabilityEventDomainV1,
    pub name: String,
    pub unix_ms: u64,
    pub duration_ms_since_previous: Option<u64>,
    pub cause_event_id: Option<String>,
}

/// Timeline reconstruction report with machine and compact human representations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelineReconstructionReportV1 {
    pub entries: Vec<ReconstructedTimelineEntryV1>,
    pub total_duration_ms: u64,
    pub human_timeline: Vec<String>,
}

/// Reconstruct an ordered causal timeline and durations from runtime events.
pub fn reconstruct_useful_timeline(
    events: &[ObservabilityEventRecordV1],
) -> Result<TimelineReconstructionReportV1, String> {
    if events.is_empty() {
        return Err("timeline reconstruction requires at least one event".to_string());
    }
    let mut sorted = events.to_vec();
    sorted.sort_by_key(|event| (event.unix_ms, event.event_id.clone()));

    let mut seen_ids = BTreeSet::new();
    let mut entries = Vec::with_capacity(sorted.len());
    let mut human_timeline = Vec::with_capacity(sorted.len());
    let mut previous_unix_ms: Option<u64> = None;
    let mut previous_event_id: Option<String> = None;

    for event in sorted {
        if !seen_ids.insert(event.event_id.clone()) {
            return Err(format!(
                "timeline reconstruction requires unique event_id, duplicate '{}'",
                event.event_id
            ));
        }
        let duration_ms_since_previous =
            previous_unix_ms.map(|prev| event.unix_ms.saturating_sub(prev));
        let cause_event_id = previous_event_id.clone();
        human_timeline.push(format!(
            "{} {} {} +{}ms",
            event.unix_ms,
            serde_json::to_string(&event.domain).unwrap_or_else(|_| "\"unknown\"".to_string()),
            event.name,
            duration_ms_since_previous.unwrap_or(0)
        ));
        entries.push(ReconstructedTimelineEntryV1 {
            event_id: event.event_id.clone(),
            run_id: event.run_id.clone(),
            node_attempt_id: event.node_attempt_id.clone(),
            domain: event.domain,
            name: event.name.clone(),
            unix_ms: event.unix_ms,
            duration_ms_since_previous,
            cause_event_id,
        });
        previous_unix_ms = Some(event.unix_ms);
        previous_event_id = Some(event.event_id);
    }

    let total_duration_ms = entries
        .first()
        .zip(entries.last())
        .map_or(0, |(first, last)| last.unix_ms.saturating_sub(first.unix_ms));

    Ok(TimelineReconstructionReportV1 { entries, total_duration_ms, human_timeline })
}

/// Input metrics captured for a run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunMetricsSampleV1 {
    pub run_id: String,
    pub queue_time_ms: u64,
    pub run_time_ms: u64,
    pub retry_count: u32,
    pub io_bytes: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub failure_count: u32,
    pub verification_state: String,
}

/// Actionable metrics report for operators.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionableRunMetricsV1 {
    pub run_id: String,
    pub queue_time_ms: u64,
    pub run_time_ms: u64,
    pub retry_count: u32,
    pub io_bytes: u64,
    pub cache_hit_ratio: f64,
    pub failure_count: u32,
    pub verification_state: String,
    pub diagnostics: Vec<String>,
}

/// Build actionable run metrics diagnostics for performance and failure debugging.
pub fn build_actionable_run_metrics(
    sample: &RunMetricsSampleV1,
) -> Result<ActionableRunMetricsV1, String> {
    if sample.run_id.trim().is_empty() {
        return Err("run metrics require run_id".to_string());
    }
    if sample.verification_state.trim().is_empty() {
        return Err("run metrics require verification_state".to_string());
    }
    let cache_total = sample.cache_hits + sample.cache_misses;
    let cache_hit_ratio =
        if cache_total == 0 { 0.0 } else { sample.cache_hits as f64 / cache_total as f64 };

    let mut diagnostics = Vec::new();
    if sample.queue_time_ms > sample.run_time_ms {
        diagnostics.push("queue_time dominates run_time; inspect scheduler capacity".to_string());
    }
    if sample.retry_count > 0 {
        diagnostics
            .push(format!("run observed {} retries; inspect unstable nodes", sample.retry_count));
    }
    if sample.failure_count > 0 {
        diagnostics.push(format!(
            "run observed {} failures; inspect node failure root causes",
            sample.failure_count
        ));
    }
    if cache_total > 0 && cache_hit_ratio < 0.2 {
        diagnostics
            .push("low cache hit ratio; inspect cache keys and materialized outputs".to_string());
    }
    if sample.verification_state != "verified" {
        diagnostics.push(format!(
            "verification state is '{}'; inspect missing or invalid evidence",
            sample.verification_state
        ));
    }

    Ok(ActionableRunMetricsV1 {
        run_id: sample.run_id.clone(),
        queue_time_ms: sample.queue_time_ms,
        run_time_ms: sample.run_time_ms,
        retry_count: sample.retry_count,
        io_bytes: sample.io_bytes,
        cache_hit_ratio,
        failure_count: sample.failure_count,
        verification_state: sample.verification_state.clone(),
        diagnostics,
    })
}

/// Node outcome row used for compact large-run summaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunNodeOutcomeV1 {
    pub node_id: String,
    pub state: String,
    pub warning_count: u32,
    pub changed_artifact_count: u32,
    pub cache_reused: bool,
}

/// Compact summary for large runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LargeRunSummaryV1 {
    pub node_count: usize,
    pub state_counts: BTreeMap<String, u64>,
    pub warning_count: u64,
    pub cache_reuse_count: u64,
    pub changed_artifact_count: u64,
    pub failure_nodes: Vec<String>,
    pub verification_state: String,
    pub compact_overview: String,
}

/// Summarize large run outcomes into a compact and operator-focused report.
pub fn summarize_large_run_compact(
    nodes: &[RunNodeOutcomeV1],
    verification_state: &str,
) -> Result<LargeRunSummaryV1, String> {
    if nodes.is_empty() {
        return Err("large run summary requires at least one node outcome".to_string());
    }
    if verification_state.trim().is_empty() {
        return Err("large run summary requires verification_state".to_string());
    }
    let mut state_counts = BTreeMap::new();
    let mut warning_count = 0u64;
    let mut cache_reuse_count = 0u64;
    let mut changed_artifact_count = 0u64;
    let mut failure_nodes = Vec::new();

    for node in nodes {
        if node.node_id.trim().is_empty() {
            return Err("large run summary contains empty node_id".to_string());
        }
        if node.state.trim().is_empty() {
            return Err(format!("node '{}' has empty state", node.node_id));
        }
        *state_counts.entry(node.state.clone()).or_insert(0) += 1;
        warning_count += u64::from(node.warning_count);
        changed_artifact_count += u64::from(node.changed_artifact_count);
        if node.cache_reused {
            cache_reuse_count += 1;
        }
        if node.state == "failed" {
            failure_nodes.push(node.node_id.clone());
        }
    }

    failure_nodes.sort();
    if failure_nodes.len() > 10 {
        failure_nodes.truncate(10);
    }

    let compact_overview = format!(
        "nodes={} failed={} warnings={} cache_reused={} changed_artifacts={} verification={}",
        nodes.len(),
        state_counts.get("failed").copied().unwrap_or(0),
        warning_count,
        cache_reuse_count,
        changed_artifact_count,
        verification_state
    );

    Ok(LargeRunSummaryV1 {
        node_count: nodes.len(),
        state_counts,
        warning_count,
        cache_reuse_count,
        changed_artifact_count,
        failure_nodes,
        verification_state: verification_state.to_string(),
        compact_overview,
    })
}

/// Comparable run surface fingerprints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunComparisonInputV1 {
    pub run_id: String,
    pub graph_fingerprint: String,
    pub config_fingerprint: String,
    pub input_fingerprint: String,
    pub output_fingerprint: String,
    pub runtime_fingerprint: String,
    pub environment_fingerprint: String,
    pub evidence_fingerprint: String,
}

/// Cross-run comparison report with explicit drift dimensions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossRunComparisonReportV1 {
    pub baseline_run_id: String,
    pub candidate_run_id: String,
    pub drift_dimensions: Vec<String>,
    pub replay_safe: bool,
}

/// Compare two runs across graph, config, inputs, outputs, runtime, environment, and evidence.
pub fn compare_runs_complete(
    baseline: &RunComparisonInputV1,
    candidate: &RunComparisonInputV1,
) -> Result<CrossRunComparisonReportV1, String> {
    for (field, value) in [
        ("baseline.run_id", baseline.run_id.as_str()),
        ("candidate.run_id", candidate.run_id.as_str()),
        ("baseline.graph_fingerprint", baseline.graph_fingerprint.as_str()),
        ("candidate.graph_fingerprint", candidate.graph_fingerprint.as_str()),
        ("baseline.config_fingerprint", baseline.config_fingerprint.as_str()),
        ("candidate.config_fingerprint", candidate.config_fingerprint.as_str()),
        ("baseline.input_fingerprint", baseline.input_fingerprint.as_str()),
        ("candidate.input_fingerprint", candidate.input_fingerprint.as_str()),
        ("baseline.output_fingerprint", baseline.output_fingerprint.as_str()),
        ("candidate.output_fingerprint", candidate.output_fingerprint.as_str()),
        ("baseline.runtime_fingerprint", baseline.runtime_fingerprint.as_str()),
        ("candidate.runtime_fingerprint", candidate.runtime_fingerprint.as_str()),
        ("baseline.environment_fingerprint", baseline.environment_fingerprint.as_str()),
        ("candidate.environment_fingerprint", candidate.environment_fingerprint.as_str()),
        ("baseline.evidence_fingerprint", baseline.evidence_fingerprint.as_str()),
        ("candidate.evidence_fingerprint", candidate.evidence_fingerprint.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!("cross-run comparison requires non-empty {field}"));
        }
    }

    let mut drift_dimensions = Vec::new();
    if baseline.graph_fingerprint != candidate.graph_fingerprint {
        drift_dimensions.push("graph".to_string());
    }
    if baseline.config_fingerprint != candidate.config_fingerprint {
        drift_dimensions.push("config".to_string());
    }
    if baseline.input_fingerprint != candidate.input_fingerprint {
        drift_dimensions.push("inputs".to_string());
    }
    if baseline.output_fingerprint != candidate.output_fingerprint {
        drift_dimensions.push("outputs".to_string());
    }
    if baseline.runtime_fingerprint != candidate.runtime_fingerprint {
        drift_dimensions.push("runtime".to_string());
    }
    if baseline.environment_fingerprint != candidate.environment_fingerprint {
        drift_dimensions.push("environment".to_string());
    }
    if baseline.evidence_fingerprint != candidate.evidence_fingerprint {
        drift_dimensions.push("evidence".to_string());
    }

    let replay_safe = drift_dimensions.is_empty();
    Ok(CrossRunComparisonReportV1 {
        baseline_run_id: baseline.run_id.clone(),
        candidate_run_id: candidate.run_id.clone(),
        drift_dimensions,
        replay_safe,
    })
}

/// Historical node execution row used for flake analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunHistoryEntryV1 {
    pub run_id: String,
    pub node_id: String,
    pub adapter_id: String,
    pub retries: u32,
    pub transient_failure: bool,
}

/// Flake analysis report with unstable surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlakeAnalysisReportV1 {
    pub transient_failure_count: u64,
    pub flaky_nodes: Vec<String>,
    pub retry_storm_nodes: Vec<String>,
    pub unstable_adapters: Vec<String>,
}

/// Analyze run history for transient failures, retry storms, and unstable adapters.
pub fn analyze_flake_history(
    history: &[RunHistoryEntryV1],
) -> Result<FlakeAnalysisReportV1, String> {
    if history.is_empty() {
        return Err("flake analysis requires non-empty run history".to_string());
    }
    let mut transient_failure_count = 0u64;
    let mut node_transient_counts = BTreeMap::<String, u64>::new();
    let mut adapter_transient_counts = BTreeMap::<String, u64>::new();
    let mut retry_storm_nodes = BTreeSet::<String>::new();

    for entry in history {
        for (field, value) in [
            ("run_id", entry.run_id.as_str()),
            ("node_id", entry.node_id.as_str()),
            ("adapter_id", entry.adapter_id.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(format!("flake analysis history entry has empty {field}"));
            }
        }
        if entry.transient_failure {
            transient_failure_count += 1;
            *node_transient_counts.entry(entry.node_id.clone()).or_insert(0) += 1;
            *adapter_transient_counts.entry(entry.adapter_id.clone()).or_insert(0) += 1;
        }
        if entry.retries >= 3 {
            retry_storm_nodes.insert(entry.node_id.clone());
        }
    }

    let flaky_nodes = node_transient_counts
        .into_iter()
        .filter(|(_, count)| *count >= 2)
        .map(|(node_id, _)| node_id)
        .collect::<Vec<_>>();
    let unstable_adapters = adapter_transient_counts
        .into_iter()
        .filter(|(_, count)| *count >= 2)
        .map(|(adapter_id, _)| adapter_id)
        .collect::<Vec<_>>();
    let retry_storm_nodes = retry_storm_nodes.into_iter().collect::<Vec<_>>();

    Ok(FlakeAnalysisReportV1 {
        transient_failure_count,
        flaky_nodes,
        retry_storm_nodes,
        unstable_adapters,
    })
}

/// Stable evidence record that mounted apps can attach to a run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppEvidenceAttachmentV1 {
    pub app_namespace: String,
    pub evidence_id: String,
    pub evidence_kind: String,
    pub payload_sha256: String,
    pub produced_at_unix_ms: u64,
}

/// Run-scoped evidence envelope consumable by runtime and mounted apps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunEvidenceEnvelopeV1 {
    pub run_id: String,
    pub attachments: Vec<AppEvidenceAttachmentV1>,
}

/// Attach app evidence into a run envelope using stable validation rules.
pub fn attach_app_evidence(
    envelope: &mut RunEvidenceEnvelopeV1,
    attachment: AppEvidenceAttachmentV1,
) -> Result<(), String> {
    if envelope.run_id.trim().is_empty() {
        return Err("run evidence envelope requires run_id".to_string());
    }
    for (field, value) in [
        ("app_namespace", attachment.app_namespace.as_str()),
        ("evidence_id", attachment.evidence_id.as_str()),
        ("evidence_kind", attachment.evidence_kind.as_str()),
        ("payload_sha256", attachment.payload_sha256.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!("app evidence attachment requires {field}"));
        }
    }
    if !attachment.payload_sha256.starts_with("sha256:") {
        return Err("app evidence attachment payload_sha256 must use sha256: prefix".to_string());
    }
    if envelope.attachments.iter().any(|existing| existing.evidence_id == attachment.evidence_id) {
        return Err(format!(
            "duplicate app evidence_id '{}' for run '{}'",
            attachment.evidence_id, envelope.run_id
        ));
    }
    envelope.attachments.push(attachment);
    envelope.attachments.sort_by(|left, right| left.evidence_id.cmp(&right.evidence_id));
    Ok(())
}

/// Directed evidence relationship edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceGraphEdgeV1 {
    pub from_id: String,
    pub to_id: String,
    pub relation: String,
}

/// Evidence relationship graph over runs, plans, nodes, artifacts, cache, replay, and events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceGraphV1 {
    pub nodes: Vec<String>,
    pub edges: Vec<EvidenceGraphEdgeV1>,
}

/// Query result for graph traversal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceGraphQueryResultV1 {
    pub start_id: String,
    pub visited_nodes: Vec<String>,
    pub traversed_edges: Vec<EvidenceGraphEdgeV1>,
}

/// Query evidence graph relationships from a starting node with bounded traversal depth.
pub fn query_evidence_graph(
    graph: &EvidenceGraphV1,
    start_id: &str,
    max_depth: usize,
) -> Result<EvidenceGraphQueryResultV1, String> {
    if start_id.trim().is_empty() {
        return Err("evidence graph query requires start_id".to_string());
    }
    if max_depth == 0 {
        return Err("evidence graph query requires max_depth >= 1".to_string());
    }
    if !graph.nodes.iter().any(|node| node == start_id) {
        return Err(format!("evidence graph query start_id '{}' does not exist", start_id));
    }

    let mut visited = BTreeSet::<String>::new();
    let mut traversed_edges = Vec::<EvidenceGraphEdgeV1>::new();
    let mut queue = VecDeque::<(String, usize)>::new();
    visited.insert(start_id.to_string());
    queue.push_back((start_id.to_string(), 0));

    while let Some((current, depth)) = queue.pop_front() {
        if depth >= max_depth {
            continue;
        }
        for edge in graph.edges.iter().filter(|edge| edge.from_id == current) {
            if edge.to_id.trim().is_empty() || edge.relation.trim().is_empty() {
                return Err("evidence graph contains empty edge fields".to_string());
            }
            traversed_edges.push(edge.clone());
            if visited.insert(edge.to_id.clone()) {
                queue.push_back((edge.to_id.clone(), depth + 1));
            }
        }
    }

    let visited_nodes = visited.into_iter().collect::<Vec<_>>();
    Ok(EvidenceGraphQueryResultV1 {
        start_id: start_id.to_string(),
        visited_nodes,
        traversed_edges,
    })
}

/// Completeness profile for evidence promotion gates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceCompletenessProfileV1 {
    Draft,
    Operational,
    Audit,
    Release,
    Scientific,
}

/// Profile verification result for evidence completeness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceCompletenessReportV1 {
    pub profile: EvidenceCompletenessProfileV1,
    pub required_fields: Vec<String>,
    pub missing_fields: Vec<String>,
    pub promotable: bool,
}

fn required_fields_for_profile(profile: EvidenceCompletenessProfileV1) -> Vec<String> {
    let mut required = vec!["run_manifest".to_string(), "event_log".to_string()];
    if matches!(
        profile,
        EvidenceCompletenessProfileV1::Operational
            | EvidenceCompletenessProfileV1::Audit
            | EvidenceCompletenessProfileV1::Release
            | EvidenceCompletenessProfileV1::Scientific
    ) {
        required.extend([
            "timeline".to_string(),
            "node_traces".to_string(),
            "outputs_index".to_string(),
        ]);
    }
    if matches!(
        profile,
        EvidenceCompletenessProfileV1::Audit
            | EvidenceCompletenessProfileV1::Release
            | EvidenceCompletenessProfileV1::Scientific
    ) {
        required.extend([
            "policy_trace".to_string(),
            "security_audit".to_string(),
            "correlation_chain".to_string(),
        ]);
    }
    if matches!(
        profile,
        EvidenceCompletenessProfileV1::Release | EvidenceCompletenessProfileV1::Scientific
    ) {
        required.extend(["verification_report".to_string(), "signature_bundle".to_string()]);
    }
    if matches!(profile, EvidenceCompletenessProfileV1::Scientific) {
        required.extend([
            "sample_identity".to_string(),
            "reference_identity".to_string(),
            "provenance_bundle".to_string(),
        ]);
    }
    required
}

/// Verify run evidence against a completeness profile and return missing gates.
pub fn verify_evidence_completeness_profile(
    profile: EvidenceCompletenessProfileV1,
    present_fields: &[String],
) -> EvidenceCompletenessReportV1 {
    let required_fields = required_fields_for_profile(profile);
    let present = present_fields
        .iter()
        .map(|field| field.trim())
        .filter(|field| !field.is_empty())
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    let missing_fields = required_fields
        .iter()
        .filter(|field| !present.contains(*field))
        .cloned()
        .collect::<Vec<_>>();
    EvidenceCompletenessReportV1 {
        profile,
        required_fields,
        promotable: missing_fields.is_empty(),
        missing_fields,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        analyze_flake_history, attach_app_evidence, build_actionable_run_metrics,
        compare_runs_complete, query_evidence_graph, reconstruct_useful_timeline,
        summarize_large_run_compact, validate_end_to_end_correlation_ids,
        verify_event_taxonomy_complete, verify_evidence_completeness_profile,
        AppEvidenceAttachmentV1, CorrelationChainV1, EvidenceCompletenessProfileV1,
        EvidenceGraphEdgeV1, EvidenceGraphV1, ObservabilityEventDomainV1,
        ObservabilityEventRecordV1, RunComparisonInputV1, RunEvidenceEnvelopeV1, RunHistoryEntryV1,
        RunMetricsSampleV1, RunNodeOutcomeV1,
    };

    fn event(domain: ObservabilityEventDomainV1, name: &str) -> ObservabilityEventRecordV1 {
        ObservabilityEventRecordV1 {
            event_id: format!("evt-{name}"),
            unix_ms: 10,
            domain,
            name: name.to_string(),
            run_id: "run-100".to_string(),
            node_attempt_id: None,
        }
    }

    #[test]
    fn event_taxonomy_requires_all_runtime_observability_domains() {
        let events = vec![
            event(ObservabilityEventDomainV1::Planner, "plan-built"),
            event(ObservabilityEventDomainV1::Scheduler, "node-ready"),
            event(ObservabilityEventDomainV1::Adapter, "adapter-start"),
            event(ObservabilityEventDomainV1::Artifact, "artifact-written"),
            event(ObservabilityEventDomainV1::Cache, "cache-miss"),
            event(ObservabilityEventDomainV1::Replay, "replay-scan"),
            event(ObservabilityEventDomainV1::Policy, "policy-allow"),
            event(ObservabilityEventDomainV1::Security, "redaction-applied"),
            event(ObservabilityEventDomainV1::Operator, "support-bundle-export"),
        ];
        let report = verify_event_taxonomy_complete(&events).expect("taxonomy should be complete");
        assert_eq!(report.total_events, 9);
        assert!(report.missing_domains.is_empty());

        let incomplete = events[..8].to_vec();
        let error =
            verify_event_taxonomy_complete(&incomplete).expect_err("taxonomy must refuse gaps");
        assert!(error.contains("incomplete"));
        assert!(error.contains("Operator"));
    }

    #[test]
    fn correlation_ids_link_root_command_to_support_bundle() {
        let chain = CorrelationChainV1 {
            root_command_id: "cmd-root-17".to_string(),
            dag_command_id: "cmd-dag-17".to_string(),
            run_id: "run-17".to_string(),
            node_attempt_id: "run-17/node-a/attempt-1".to_string(),
            artifact_id: "artifact-run-17-node-a-output".to_string(),
            support_bundle_id: "support-run-17".to_string(),
        };
        validate_end_to_end_correlation_ids(&chain).expect("correlation chain should validate");

        let mut broken = chain;
        broken.support_bundle_id.clear();
        let error = validate_end_to_end_correlation_ids(&broken)
            .expect_err("missing support bundle id must fail");
        assert!(error.contains("support_bundle_id"));
    }

    #[test]
    fn timeline_reconstruction_is_deterministic_and_causal() {
        let events = vec![
            ObservabilityEventRecordV1 {
                event_id: "evt-2".to_string(),
                unix_ms: 1_025,
                domain: ObservabilityEventDomainV1::Scheduler,
                name: "node-scheduled".to_string(),
                run_id: "run-17".to_string(),
                node_attempt_id: Some("run-17/node-a/attempt-1".to_string()),
            },
            ObservabilityEventRecordV1 {
                event_id: "evt-1".to_string(),
                unix_ms: 1_000,
                domain: ObservabilityEventDomainV1::Planner,
                name: "plan-built".to_string(),
                run_id: "run-17".to_string(),
                node_attempt_id: None,
            },
            ObservabilityEventRecordV1 {
                event_id: "evt-3".to_string(),
                unix_ms: 1_055,
                domain: ObservabilityEventDomainV1::Adapter,
                name: "node-finished".to_string(),
                run_id: "run-17".to_string(),
                node_attempt_id: Some("run-17/node-a/attempt-1".to_string()),
            },
        ];

        let report =
            reconstruct_useful_timeline(&events).expect("timeline reconstruction should work");
        assert_eq!(report.entries.len(), 3);
        assert_eq!(report.entries[0].event_id, "evt-1");
        assert_eq!(report.entries[1].cause_event_id.as_deref(), Some("evt-1"));
        assert_eq!(report.entries[1].duration_ms_since_previous, Some(25));
        assert_eq!(report.entries[2].duration_ms_since_previous, Some(30));
        assert_eq!(report.total_duration_ms, 55);
    }

    #[test]
    fn run_metrics_report_queue_runtime_retries_cache_failures_and_verification() {
        let report = build_actionable_run_metrics(&RunMetricsSampleV1 {
            run_id: "run-17".to_string(),
            queue_time_ms: 15_000,
            run_time_ms: 9_000,
            retry_count: 3,
            io_bytes: 4_096_000,
            cache_hits: 1,
            cache_misses: 9,
            failure_count: 2,
            verification_state: "incomplete".to_string(),
        })
        .expect("metrics report should build");
        assert_eq!(report.queue_time_ms, 15_000);
        assert_eq!(report.run_time_ms, 9_000);
        assert_eq!(report.retry_count, 3);
        assert_eq!(report.failure_count, 2);
        assert!(report.cache_hit_ratio < 0.2);
        assert!(report
            .diagnostics
            .iter()
            .any(|line| line.contains("queue_time dominates run_time")));
        assert!(report
            .diagnostics
            .iter()
            .any(|line| line.contains("verification state is 'incomplete'")));
    }

    #[test]
    fn large_run_summary_is_compact_and_operator_focused() {
        let summary = summarize_large_run_compact(
            &[
                RunNodeOutcomeV1 {
                    node_id: "align-a".to_string(),
                    state: "failed".to_string(),
                    warning_count: 1,
                    changed_artifact_count: 2,
                    cache_reused: false,
                },
                RunNodeOutcomeV1 {
                    node_id: "align-b".to_string(),
                    state: "success".to_string(),
                    warning_count: 0,
                    changed_artifact_count: 1,
                    cache_reused: true,
                },
                RunNodeOutcomeV1 {
                    node_id: "qc".to_string(),
                    state: "cached".to_string(),
                    warning_count: 1,
                    changed_artifact_count: 0,
                    cache_reused: true,
                },
            ],
            "verified",
        )
        .expect("large run summary should build");
        assert_eq!(summary.node_count, 3);
        assert_eq!(summary.state_counts.get("failed"), Some(&1));
        assert_eq!(summary.warning_count, 2);
        assert_eq!(summary.cache_reuse_count, 2);
        assert_eq!(summary.changed_artifact_count, 3);
        assert_eq!(summary.failure_nodes, vec!["align-a".to_string()]);
        assert!(summary.compact_overview.contains("verification=verified"));
    }

    #[test]
    fn cross_run_comparison_reports_all_drift_dimensions() {
        let baseline = RunComparisonInputV1 {
            run_id: "run-a".to_string(),
            graph_fingerprint: "graph-1".to_string(),
            config_fingerprint: "cfg-1".to_string(),
            input_fingerprint: "in-1".to_string(),
            output_fingerprint: "out-1".to_string(),
            runtime_fingerprint: "rt-1".to_string(),
            environment_fingerprint: "env-1".to_string(),
            evidence_fingerprint: "ev-1".to_string(),
        };
        let candidate = RunComparisonInputV1 {
            run_id: "run-b".to_string(),
            graph_fingerprint: "graph-1".to_string(),
            config_fingerprint: "cfg-2".to_string(),
            input_fingerprint: "in-1".to_string(),
            output_fingerprint: "out-2".to_string(),
            runtime_fingerprint: "rt-2".to_string(),
            environment_fingerprint: "env-1".to_string(),
            evidence_fingerprint: "ev-2".to_string(),
        };
        let report =
            compare_runs_complete(&baseline, &candidate).expect("comparison should succeed");
        assert_eq!(
            report.drift_dimensions,
            vec![
                "config".to_string(),
                "outputs".to_string(),
                "runtime".to_string(),
                "evidence".to_string(),
            ]
        );
        assert!(!report.replay_safe);
    }

    #[test]
    fn flake_analysis_identifies_transient_failures_retry_storms_and_unstable_adapters() {
        let report = analyze_flake_history(&[
            RunHistoryEntryV1 {
                run_id: "run-1".to_string(),
                node_id: "align".to_string(),
                adapter_id: "shell".to_string(),
                retries: 4,
                transient_failure: true,
            },
            RunHistoryEntryV1 {
                run_id: "run-2".to_string(),
                node_id: "align".to_string(),
                adapter_id: "shell".to_string(),
                retries: 1,
                transient_failure: true,
            },
            RunHistoryEntryV1 {
                run_id: "run-3".to_string(),
                node_id: "qc".to_string(),
                adapter_id: "python".to_string(),
                retries: 0,
                transient_failure: false,
            },
        ])
        .expect("flake analysis should build");
        assert_eq!(report.transient_failure_count, 2);
        assert_eq!(report.flaky_nodes, vec!["align".to_string()]);
        assert_eq!(report.retry_storm_nodes, vec!["align".to_string()]);
        assert_eq!(report.unstable_adapters, vec!["shell".to_string()]);
    }

    #[test]
    fn app_evidence_api_supports_stable_attachment_contract() {
        let mut envelope =
            RunEvidenceEnvelopeV1 { run_id: "run-17".to_string(), attachments: Vec::new() };
        attach_app_evidence(
            &mut envelope,
            AppEvidenceAttachmentV1 {
                app_namespace: "dag.app.quality".to_string(),
                evidence_id: "evidence-qc-1".to_string(),
                evidence_kind: "quality-report".to_string(),
                payload_sha256: "sha256:abc123".to_string(),
                produced_at_unix_ms: 42,
            },
        )
        .expect("evidence attach should work");

        assert_eq!(envelope.attachments.len(), 1);
        assert_eq!(envelope.attachments[0].evidence_kind, "quality-report");

        let duplicate = attach_app_evidence(
            &mut envelope,
            AppEvidenceAttachmentV1 {
                app_namespace: "dag.app.quality".to_string(),
                evidence_id: "evidence-qc-1".to_string(),
                evidence_kind: "quality-report".to_string(),
                payload_sha256: "sha256:def456".to_string(),
                produced_at_unix_ms: 43,
            },
        )
        .expect_err("duplicate evidence ids must fail");
        assert!(duplicate.contains("duplicate"));
    }

    #[test]
    fn evidence_graph_query_returns_related_entities_without_file_scans() {
        let graph = EvidenceGraphV1 {
            nodes: vec![
                "run-17".to_string(),
                "plan-17".to_string(),
                "node-a".to_string(),
                "artifact-1".to_string(),
            ],
            edges: vec![
                EvidenceGraphEdgeV1 {
                    from_id: "run-17".to_string(),
                    to_id: "plan-17".to_string(),
                    relation: "run_has_plan".to_string(),
                },
                EvidenceGraphEdgeV1 {
                    from_id: "plan-17".to_string(),
                    to_id: "node-a".to_string(),
                    relation: "plan_contains_node".to_string(),
                },
                EvidenceGraphEdgeV1 {
                    from_id: "node-a".to_string(),
                    to_id: "artifact-1".to_string(),
                    relation: "node_produces_artifact".to_string(),
                },
            ],
        };
        let result = query_evidence_graph(&graph, "run-17", 3).expect("graph query should work");
        assert_eq!(
            result.visited_nodes,
            vec![
                "artifact-1".to_string(),
                "node-a".to_string(),
                "plan-17".to_string(),
                "run-17".to_string(),
            ]
        );
        assert_eq!(result.traversed_edges.len(), 3);
    }

    #[test]
    fn evidence_completeness_is_profile_driven_for_promotion_gates() {
        let audit = verify_evidence_completeness_profile(
            EvidenceCompletenessProfileV1::Audit,
            &[
                "run_manifest".to_string(),
                "event_log".to_string(),
                "timeline".to_string(),
                "node_traces".to_string(),
                "outputs_index".to_string(),
                "policy_trace".to_string(),
                "correlation_chain".to_string(),
            ],
        );
        assert!(!audit.promotable);
        assert_eq!(audit.missing_fields, vec!["security_audit".to_string()]);

        let scientific = verify_evidence_completeness_profile(
            EvidenceCompletenessProfileV1::Scientific,
            &[
                "run_manifest".to_string(),
                "event_log".to_string(),
                "timeline".to_string(),
                "node_traces".to_string(),
                "outputs_index".to_string(),
                "policy_trace".to_string(),
                "security_audit".to_string(),
                "correlation_chain".to_string(),
                "verification_report".to_string(),
                "signature_bundle".to_string(),
                "sample_identity".to_string(),
                "reference_identity".to_string(),
                "provenance_bundle".to_string(),
            ],
        );
        assert!(scientific.promotable);
        assert!(scientific.missing_fields.is_empty());
    }
}
