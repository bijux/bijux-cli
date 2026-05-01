use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

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
        let duration_ms_since_previous = previous_unix_ms.map(|prev| event.unix_ms.saturating_sub(prev));
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

    Ok(TimelineReconstructionReportV1 {
        entries,
        total_duration_ms,
        human_timeline,
    })
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
pub fn build_actionable_run_metrics(sample: &RunMetricsSampleV1) -> Result<ActionableRunMetricsV1, String> {
    if sample.run_id.trim().is_empty() {
        return Err("run metrics require run_id".to_string());
    }
    if sample.verification_state.trim().is_empty() {
        return Err("run metrics require verification_state".to_string());
    }
    let cache_total = sample.cache_hits + sample.cache_misses;
    let cache_hit_ratio = if cache_total == 0 {
        0.0
    } else {
        sample.cache_hits as f64 / cache_total as f64
    };

    let mut diagnostics = Vec::new();
    if sample.queue_time_ms > sample.run_time_ms {
        diagnostics.push("queue_time dominates run_time; inspect scheduler capacity".to_string());
    }
    if sample.retry_count > 0 {
        diagnostics.push(format!("run observed {} retries; inspect unstable nodes", sample.retry_count));
    }
    if sample.failure_count > 0 {
        diagnostics.push(format!(
            "run observed {} failures; inspect node failure root causes",
            sample.failure_count
        ));
    }
    if cache_total > 0 && cache_hit_ratio < 0.2 {
        diagnostics.push("low cache hit ratio; inspect cache keys and materialized outputs".to_string());
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

#[cfg(test)]
mod tests {
    use super::{
        build_actionable_run_metrics, reconstruct_useful_timeline, validate_end_to_end_correlation_ids,
        verify_event_taxonomy_complete, CorrelationChainV1, RunMetricsSampleV1,
        ObservabilityEventDomainV1, ObservabilityEventRecordV1,
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
    fn g161_event_taxonomy_requires_all_runtime_observability_domains() {
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
        let error = verify_event_taxonomy_complete(&incomplete).expect_err("taxonomy must refuse gaps");
        assert!(error.contains("incomplete"));
        assert!(error.contains("Operator"));
    }

    #[test]
    fn g162_correlation_ids_link_root_command_to_support_bundle() {
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
        let error =
            validate_end_to_end_correlation_ids(&broken).expect_err("missing support bundle id must fail");
        assert!(error.contains("support_bundle_id"));
    }

    #[test]
    fn g163_timeline_reconstruction_is_deterministic_and_causal() {
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

        let report = reconstruct_useful_timeline(&events).expect("timeline reconstruction should work");
        assert_eq!(report.entries.len(), 3);
        assert_eq!(report.entries[0].event_id, "evt-1");
        assert_eq!(report.entries[1].cause_event_id.as_deref(), Some("evt-1"));
        assert_eq!(report.entries[1].duration_ms_since_previous, Some(25));
        assert_eq!(report.entries[2].duration_ms_since_previous, Some(30));
        assert_eq!(report.total_duration_ms, 55);
    }

    #[test]
    fn g164_run_metrics_report_queue_runtime_retries_cache_failures_and_verification() {
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
}
