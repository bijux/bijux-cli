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

#[cfg(test)]
mod tests {
    use super::{validate_nested_subgraph_execution, NestedSubgraphExecutionRecordV1};

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
}
