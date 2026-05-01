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
        return Err("missing optional inputs must not fail execution without policy requirement".to_string());
    }
    if record.policy_requires_presence && !record.optional_inputs_missing.is_empty() && !record.execution_failed {
        return Err("policy-required optional inputs must fail when missing".to_string());
    }
    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::{
        validate_matrix_execution, validate_nested_subgraph_execution, validate_partition_execution,
        validate_optional_input_execution, validate_quorum_execution, MatrixExecutionRecordV1,
        NestedSubgraphExecutionRecordV1, OptionalInputExecutionRecordV1, PartitionExecutionRecordV1,
        QuorumExecutionRecordV1,
    };

    #[test]
    fn g111_nested_subgraph_execution_preserves_parent_child_lineage() {
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
    fn g112_matrix_execution_uses_deterministic_ids_and_replay_bounds() {
        let record = validate_matrix_execution(MatrixExecutionRecordV1 {
            base_node_id: "align".to_string(),
            expanded_node_ids: vec![
                "align::sample=a".to_string(),
                "align::sample=b".to_string(),
            ],
            artifact_names: vec![
                "aligned-a.bam".to_string(),
                "aligned-b.bam".to_string(),
            ],
            deterministic_ids: true,
            replay_bounded: true,
        })
        .expect("matrix execution");
        assert_eq!(record.expanded_node_ids.len(), 2);
        assert!(record.deterministic_ids);
    }

    #[test]
    fn g113_partition_execution_exposes_stable_keys_and_reducer_lineage() {
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
    fn g114_quorum_execution_reports_deterministic_partial_success() {
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
    fn g115_optional_input_execution_keeps_missing_inputs_visible_without_forced_failure() {
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
}
