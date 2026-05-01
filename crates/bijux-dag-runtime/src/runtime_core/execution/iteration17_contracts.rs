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

#[cfg(test)]
mod tests {
    use super::{
        verify_event_taxonomy_complete, ObservabilityEventDomainV1, ObservabilityEventRecordV1,
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
}
