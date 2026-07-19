use bijux_dag_artifacts::{
    FailureAffectedGroups, FailureCauseRecord, FailurePropagationRecord, RunFailureSummary,
};
use serde as _;
use tempfile as _;
use thiserror as _;

#[test]
fn run_failure_summary_roundtrip_preserves_root_and_propagation_boundaries() {
    let summary = RunFailureSummary {
        roots: vec!["train:execution_failed".to_string()],
        primary_failure: Some(FailureCauseRecord {
            node_id: "train".to_string(),
            failure_class: Some("execution".to_string()),
            failure_code: Some("EXEC_FAIL".to_string()),
            message: Some("command exited with status 7".to_string()),
            reason: Some("execution_failed".to_string()),
            finished_unix_ms: Some(1234),
        }),
        propagated_failures: vec![FailurePropagationRecord {
            node_id: "report".to_string(),
            status: "failed".to_string(),
            reason: "upstream_failed".to_string(),
            propagation_mode: Some("continue_independent".to_string()),
            blocking_nodes: vec!["train".to_string()],
        }],
        propagated_skips: vec![FailurePropagationRecord {
            node_id: "publish".to_string(),
            status: "skipped".to_string(),
            reason: "isolated_branch_failure".to_string(),
            propagation_mode: Some("isolate_branch".to_string()),
            blocking_nodes: vec!["report".to_string()],
        }],
        downstream_affected_nodes: vec!["publish".to_string(), "report".to_string()],
        downstream_affected_groups: FailureAffectedGroups {
            failed: vec!["report".to_string()],
            skipped: vec!["publish".to_string()],
            cancelled: Vec::new(),
        },
    };

    let encoded = serde_json::to_vec_pretty(&summary).expect("encode");
    let decoded: RunFailureSummary = serde_json::from_slice(&encoded).expect("decode");
    assert_eq!(decoded, summary);
}
