use serde::{Deserialize, Serialize};

/// Nested subgraph execution lineage record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NestedSubgraphExecutionRecordV1 {
    pub parent_run_id: String,
    pub subgraph_run_id: String,
    pub parent_node_id: String,
    pub scoped_node_prefix: String,
    pub failure_propagation_consistent: bool,
}

/// Validate nested subgraph execution contracts.
pub fn validate_nested_subgraph_execution(
    record: NestedSubgraphExecutionRecordV1,
) -> Result<NestedSubgraphExecutionRecordV1, String> {
    for (field_name, field_value) in [
        ("parent_run_id", record.parent_run_id.as_str()),
        ("subgraph_run_id", record.subgraph_run_id.as_str()),
        ("parent_node_id", record.parent_node_id.as_str()),
        ("scoped_node_prefix", record.scoped_node_prefix.as_str()),
    ] {
        if field_value.trim().is_empty() {
            return Err(format!("nested subgraph field {field_name} must not be empty"));
        }
    }
    if !record.failure_propagation_consistent {
        return Err("nested subgraph failure propagation must be consistent".to_string());
    }
    Ok(record)
}

/// Matrix expansion execution record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatrixExecutionRecordV1 {
    pub base_node_id: String,
    pub expanded_node_ids: Vec<String>,
    pub artifact_names: Vec<String>,
    pub deterministic_ids: bool,
    pub replay_bounded: bool,
}

/// Validate matrix workflow execution contracts.
pub fn validate_matrix_execution(
    record: MatrixExecutionRecordV1,
) -> Result<MatrixExecutionRecordV1, String> {
    if record.base_node_id.trim().is_empty() {
        return Err("base_node_id must not be empty".to_string());
    }
    if record.expanded_node_ids.is_empty() {
        return Err("matrix execution must expand at least one node".to_string());
    }
    if record.artifact_names.len() != record.expanded_node_ids.len() {
        return Err("artifact_names must match expanded node count".to_string());
    }
    if !record.deterministic_ids {
        return Err("matrix expanded node ids must be deterministic".to_string());
    }
    if !record.replay_bounded {
        return Err("matrix execution must remain replay-bounded".to_string());
    }
    Ok(record)
}

/// Dataset-partition workflow execution record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionExecutionRecordV1 {
    pub partition_keys: Vec<String>,
    pub producer_node_id: String,
    pub reducer_node_id: String,
    pub lineage_complete: bool,
}

/// Validate dataset partition workflow semantics.
pub fn validate_partition_execution(
    record: PartitionExecutionRecordV1,
) -> Result<PartitionExecutionRecordV1, String> {
    if record.partition_keys.is_empty() {
        return Err("partition workflow must include partition keys".to_string());
    }
    if record.partition_keys.iter().any(|key| key.trim().is_empty()) {
        return Err("partition keys must not be empty".to_string());
    }
    if record.producer_node_id.trim().is_empty() || record.reducer_node_id.trim().is_empty() {
        return Err("producer and reducer node ids must not be empty".to_string());
    }
    if !record.lineage_complete {
        return Err("partition lineage must be complete".to_string());
    }
    Ok(record)
}

/// Quorum trigger execution record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuorumExecutionRecordV1 {
    pub required_successes: usize,
    pub total_candidates: usize,
    pub achieved_successes: usize,
    pub deterministic_outcome: bool,
    pub partial_success_visible: bool,
}

/// Validate quorum trigger workflow semantics.
pub fn validate_quorum_execution(
    record: QuorumExecutionRecordV1,
) -> Result<QuorumExecutionRecordV1, String> {
    if record.required_successes == 0 || record.total_candidates == 0 {
        return Err("quorum counts must be positive".to_string());
    }
    if record.required_successes > record.total_candidates {
        return Err("required successes cannot exceed total candidates".to_string());
    }
    if record.achieved_successes > record.total_candidates {
        return Err("achieved successes cannot exceed total candidates".to_string());
    }
    if !record.deterministic_outcome {
        return Err("quorum outcome must be deterministic".to_string());
    }
    if !record.partial_success_visible {
        return Err("partial success must be explicitly visible".to_string());
    }
    Ok(record)
}

/// Optional-input execution visibility record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionalInputExecutionRecordV1 {
    pub optional_inputs_present: Vec<String>,
    pub optional_inputs_missing: Vec<String>,
    pub policy_requires_presence: bool,
    pub execution_failed: bool,
}

/// Validate optional-input execution semantics.
pub fn validate_optional_input_execution(
    record: OptionalInputExecutionRecordV1,
) -> Result<OptionalInputExecutionRecordV1, String> {
    if record.optional_inputs_present.is_empty() && record.optional_inputs_missing.is_empty() {
        return Err("optional input execution must report present or missing inputs".to_string());
    }
    if !record.policy_requires_presence && record.execution_failed {
        return Err("missing optional inputs must not fail execution without policy requirement"
            .to_string());
    }
    if record.policy_requires_presence
        && !record.optional_inputs_missing.is_empty()
        && !record.execution_failed
    {
        return Err("policy-required optional inputs must fail when missing".to_string());
    }
    Ok(record)
}

/// Service/sensor mock workflow execution record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceSensorExecutionRecordV1 {
    pub lifecycle_events_recorded: bool,
    #[serde(rename = "simulated_outputs")]
    pub synthetic_outputs: bool,
    pub advisory_mode: bool,
}

/// Validate service/sensor mock execution contracts.
pub fn validate_service_sensor_execution(
    record: ServiceSensorExecutionRecordV1,
) -> Result<ServiceSensorExecutionRecordV1, String> {
    if !record.lifecycle_events_recorded {
        return Err("service/sensor execution must record lifecycle events".to_string());
    }
    if !record.synthetic_outputs {
        return Err("service/sensor execution must mark outputs as simulated".to_string());
    }
    if !record.advisory_mode {
        return Err(
            "service/sensor execution must remain advisory until real service support exists"
                .to_string(),
        );
    }
    Ok(record)
}

/// Event-recorded workflow replay record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventRecordedReplayRecordV1 {
    pub recorded_event_ids: Vec<String>,
    pub replay_used_recorded_events_only: bool,
    pub fresh_external_poll_detected: bool,
}

/// Validate event-recorded replay semantics.
pub fn validate_event_recorded_replay(
    record: EventRecordedReplayRecordV1,
) -> Result<EventRecordedReplayRecordV1, String> {
    if record.recorded_event_ids.is_empty() {
        return Err("event-recorded workflow must have recorded event ids".to_string());
    }
    if !record.replay_used_recorded_events_only {
        return Err("replay must use recorded events only".to_string());
    }
    if record.fresh_external_poll_detected {
        return Err("replay must not poll fresh external state".to_string());
    }
    Ok(record)
}

/// Policy overlay execution record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyOverlayExecutionRecordV1 {
    pub base_graph_fingerprint: String,
    pub overlay_profile: String,
    pub overlay_diff_visible: bool,
    pub base_identity_mutated: bool,
}

/// Validate policy overlay workflow semantics.
pub fn validate_policy_overlay_execution(
    record: PolicyOverlayExecutionRecordV1,
) -> Result<PolicyOverlayExecutionRecordV1, String> {
    if record.base_graph_fingerprint.trim().is_empty() {
        return Err("base_graph_fingerprint must not be empty".to_string());
    }
    if record.overlay_profile.trim().is_empty() {
        return Err("overlay_profile must not be empty".to_string());
    }
    if !record.overlay_diff_visible {
        return Err("overlay diff must be visible".to_string());
    }
    if record.base_identity_mutated {
        return Err("overlay must not mutate base graph identity".to_string());
    }
    Ok(record)
}

/// Non-cacheable workflow execution record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonCacheableExecutionRecordV1 {
    pub node_id: String,
    pub non_cacheable_reason: String,
    pub cache_reuse_attempted: bool,
    pub cache_reuse_refused: bool,
}

/// Validate non-cacheable workflow semantics.
pub fn validate_non_cacheable_execution(
    record: NonCacheableExecutionRecordV1,
) -> Result<NonCacheableExecutionRecordV1, String> {
    if record.node_id.trim().is_empty() {
        return Err("node_id must not be empty".to_string());
    }
    if record.non_cacheable_reason.trim().is_empty() {
        return Err("non_cacheable_reason must not be empty".to_string());
    }
    if record.cache_reuse_attempted && !record.cache_reuse_refused {
        return Err("cache reuse must be refused for non-cacheable nodes".to_string());
    }
    Ok(record)
}

/// Graph conformance profile result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphConformanceProfileResultV1 {
    pub profile: String,
    pub behavior_checks_passed: bool,
    pub evidence_checks_passed: bool,
}

/// Validate graph conformance profile execution.
pub fn validate_graph_conformance_profile(
    result: GraphConformanceProfileResultV1,
) -> Result<GraphConformanceProfileResultV1, String> {
    if result.profile.trim().is_empty() {
        return Err("profile must not be empty".to_string());
    }
    match result.profile.as_str() {
        "minimal" | "local-production" | "container-advisory" | "audit" => {}
        _ => return Err("unknown conformance profile".to_string()),
    }
    if !result.behavior_checks_passed {
        return Err("conformance profile failed behavior checks".to_string());
    }
    if !result.evidence_checks_passed {
        return Err("conformance profile failed evidence checks".to_string());
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{
        validate_event_recorded_replay, validate_graph_conformance_profile,
        validate_matrix_execution, validate_nested_subgraph_execution,
        validate_non_cacheable_execution, validate_optional_input_execution,
        validate_partition_execution, validate_policy_overlay_execution, validate_quorum_execution,
        validate_service_sensor_execution, EventRecordedReplayRecordV1,
        GraphConformanceProfileResultV1, MatrixExecutionRecordV1, NestedSubgraphExecutionRecordV1,
        NonCacheableExecutionRecordV1, OptionalInputExecutionRecordV1, PartitionExecutionRecordV1,
        PolicyOverlayExecutionRecordV1, QuorumExecutionRecordV1, ServiceSensorExecutionRecordV1,
    };

    #[test]
    fn nested_subgraph_execution_preserves_parent_child_lineage() {
        let record = validate_nested_subgraph_execution(NestedSubgraphExecutionRecordV1 {
            parent_run_id: "run-100".to_string(),
            subgraph_run_id: "run-100/subgraph-1".to_string(),
            parent_node_id: "call-subgraph".to_string(),
            scoped_node_prefix: "call-subgraph::".to_string(),
            failure_propagation_consistent: true,
        })
        .expect("nested subgraph");
        assert!(record.subgraph_run_id.starts_with("run-100/"));
        assert_eq!(record.scoped_node_prefix, "call-subgraph::");
    }

    #[test]
    fn matrix_execution_uses_deterministic_ids_and_replay_bounds() {
        let record = validate_matrix_execution(MatrixExecutionRecordV1 {
            base_node_id: "align".to_string(),
            expanded_node_ids: vec!["align::sample=a".to_string(), "align::sample=b".to_string()],
            artifact_names: vec!["aligned-a.bam".to_string(), "aligned-b.bam".to_string()],
            deterministic_ids: true,
            replay_bounded: true,
        })
        .expect("matrix execution");
        assert_eq!(record.expanded_node_ids.len(), 2);
        assert!(record.deterministic_ids);
    }

    #[test]
    fn partition_execution_exposes_stable_keys_and_reducer_lineage() {
        let record = validate_partition_execution(PartitionExecutionRecordV1 {
            partition_keys: vec!["sample-a".to_string(), "sample-b".to_string()],
            producer_node_id: "split-samples".to_string(),
            reducer_node_id: "merge-results".to_string(),
            lineage_complete: true,
        })
        .expect("partition execution");
        assert_eq!(record.partition_keys.len(), 2);
        assert_eq!(record.reducer_node_id, "merge-results");
    }

    #[test]
    fn quorum_execution_reports_deterministic_partial_success() {
        let record = validate_quorum_execution(QuorumExecutionRecordV1 {
            required_successes: 2,
            total_candidates: 3,
            achieved_successes: 2,
            deterministic_outcome: true,
            partial_success_visible: true,
        })
        .expect("quorum execution");
        assert_eq!(record.required_successes, 2);
        assert_eq!(record.achieved_successes, 2);
    }

    #[test]
    fn optional_input_execution_keeps_missing_inputs_visible_without_forced_failure() {
        let record = validate_optional_input_execution(OptionalInputExecutionRecordV1 {
            optional_inputs_present: vec!["sample_sheet".to_string()],
            optional_inputs_missing: vec!["annotation_db".to_string()],
            policy_requires_presence: false,
            execution_failed: false,
        })
        .expect("optional input execution");
        assert_eq!(record.optional_inputs_missing, vec!["annotation_db".to_string()]);
        assert!(!record.execution_failed);
    }

    #[test]
    fn service_sensor_execution_stays_simulated_and_advisory() {
        let record = validate_service_sensor_execution(ServiceSensorExecutionRecordV1 {
            lifecycle_events_recorded: true,
            synthetic_outputs: true,
            advisory_mode: true,
        })
        .expect("service sensor execution");
        assert!(record.synthetic_outputs);
        assert!(record.advisory_mode);
    }

    #[test]
    fn event_recorded_replay_refuses_fresh_external_polling() {
        let record = validate_event_recorded_replay(EventRecordedReplayRecordV1 {
            recorded_event_ids: vec!["event-1".to_string(), "event-2".to_string()],
            replay_used_recorded_events_only: true,
            fresh_external_poll_detected: false,
        })
        .expect("event replay");
        assert_eq!(record.recorded_event_ids.len(), 2);
        assert!(record.replay_used_recorded_events_only);
    }

    #[test]
    fn policy_overlay_diff_is_visible_without_mutating_graph_identity() {
        let record = validate_policy_overlay_execution(PolicyOverlayExecutionRecordV1 {
            base_graph_fingerprint: "graph-sha256-123".to_string(),
            overlay_profile: "security".to_string(),
            overlay_diff_visible: true,
            base_identity_mutated: false,
        })
        .expect("overlay execution");
        assert_eq!(record.overlay_profile, "security");
        assert!(!record.base_identity_mutated);
    }

    #[test]
    fn non_cacheable_nodes_refuse_unsafe_cache_reuse() {
        let record = validate_non_cacheable_execution(NonCacheableExecutionRecordV1 {
            node_id: "fetch-clock".to_string(),
            non_cacheable_reason: "external time dependency".to_string(),
            cache_reuse_attempted: true,
            cache_reuse_refused: true,
        })
        .expect("non-cacheable execution");
        assert!(record.cache_reuse_refused);
    }

    #[test]
    fn graph_conformance_profiles_are_behavior_based_not_docs_based() {
        let minimal = validate_graph_conformance_profile(GraphConformanceProfileResultV1 {
            profile: "minimal".to_string(),
            behavior_checks_passed: true,
            evidence_checks_passed: true,
        })
        .expect("minimal profile");
        let audit = validate_graph_conformance_profile(GraphConformanceProfileResultV1 {
            profile: "audit".to_string(),
            behavior_checks_passed: true,
            evidence_checks_passed: true,
        })
        .expect("audit profile");
        assert_eq!(minimal.profile, "minimal");
        assert_eq!(audit.profile, "audit");
    }
}
