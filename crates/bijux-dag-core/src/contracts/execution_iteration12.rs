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

#[cfg(test)]
mod tests {
    use super::{
        validate_matrix_execution, validate_nested_subgraph_execution, MatrixExecutionRecordV1,
        NestedSubgraphExecutionRecordV1,
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
}
