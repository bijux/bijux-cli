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

#[cfg(test)]
mod tests {
    use super::{
        validate_end_to_end_correlation_ids, verify_event_taxonomy_complete, CorrelationChainV1,
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
}
