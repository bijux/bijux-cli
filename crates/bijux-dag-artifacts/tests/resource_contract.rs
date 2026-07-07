use bijux_dag_artifacts::{Manifest, NodeTrace, RunSummary};
use bijux_dag_testkit as _;
use hex as _;
use serde as _;
use serde_json::json;
use sha2 as _;
use tempfile as _;
use thiserror as _;

#[test]
fn resource_manifest_does_not_duplicate_output_summaries_pathologically() {
    let manifest = Manifest {
        manifest_version: "run-manifest/v0.1".to_string(),
        run_id: "run-1".to_string(),
        created_unix_ms: 1,
        started_unix_ms: 1,
        finished_unix_ms: 2,
        graph_snapshot: "graph.snapshot.json".to_string(),
        status: "succeeded".to_string(),
        spec: "dag/v0.1".to_string(),
        graph_fingerprint: "fp".to_string(),
        planner_contract_version: "bijux-dag-planner/v1".to_string(),
        planner_fingerprint: None,
        execution_fingerprint: None,
        evidence_fingerprint: None,
        tool_version: "0.1.0".to_string(),
        jobs: 1,
        adapters: vec![],
        outputs: (0..200)
            .map(|idx| bijux_dag_artifacts::OutputSummary {
                node_id: format!("n{idx}"),
                node_fingerprint: format!("fp-{idx}"),
                name: format!("out-{idx}"),
                path: format!("node-{idx}/out-{idx}.bin"),
                kind: "file".to_string(),
                media_type: "application/octet-stream".to_string(),
                size_bytes: idx as u64,
                sha256: "hash".to_string(),
                promotable: false,
            })
            .collect(),
        node_counts: bijux_dag_artifacts::NodeCounts {
            success: 200,
            failed: 0,
            skipped: 0,
            cached: 0,
            cancelled: 0,
        },
        policy: bijux_dag_artifacts::PolicyInfo {
            deny_network: false,
            deny_env: false,
            deny_clock: false,
            clean_env: false,
            container_image_reference_policy:
                bijux_dag_artifacts::ContainerImageReferencePolicy::RequireDigest,
        },
        cache_mode: None,
        cache_dir: None,
        run_timeout_ms: None,
        run_timeout_behavior: None,
        run_cancellation_cause: None,
        run_metadata: None,
        run_summary: Some(RunSummary {
            total_nodes: 200,
            success: 200,
            failed: 0,
            skipped: 0,
            cached: 0,
            cancelled: 0,
            promoted_outputs: Vec::new(),
        }),
    };

    let serialized = serde_json::to_vec(&manifest).expect("serialize");
    assert!(serialized.len() < 200_000, "manifest unexpectedly large");
}

#[test]
fn resource_retry_trace_event_volume_stays_bounded() {
    let trace = NodeTrace {
        node_id: "retry-node".to_string(),
        status: "failed".to_string(),
        started_unix_ms: 1,
        finished_unix_ms: 2,
        attempt: 250,
        fingerprint: "fp".to_string(),
        planner_contract_version: Some("bijux-dag-planner/v1".to_string()),
        execution_fingerprint: Some("execution-fp".to_string()),
        evidence_fingerprint: Some("evidence-fp".to_string()),
        adapter_id: "shell".to_string(),
        adapter_version: "v1".to_string(),
        adapter_outputs_schema_version: "v0.1".to_string(),
        adapter_binary_sha256: None,
        resources: None,
        inputs_index: None,
        resolved_params: Some(json!({"argv":["/bin/sh","-c","exit 1"]})),
        exit_code: None,
        stdout: None,
        stderr: None,
        outputs: Vec::new(),
        container: None,
        cache_proof: None,
        cache_identity: None,
        branch_decision: None,
        trigger_evaluation: Some(bijux_dag_artifacts::TriggerEvaluation {
            trigger_rule: "all_success".to_string(),
            satisfied: false,
            reason: "requires every upstream to complete in success or cached status".to_string(),
            parent_statuses: vec![bijux_dag_artifacts::TriggerParentStatus {
                node_id: "extract".to_string(),
                status: "failed".to_string(),
            }],
        }),
        skip_reason: None,
        failure: Some(bijux_dag_artifacts::FailureInfo {
            class: Some(bijux_dag_artifacts::FailureClass::Execution),
            kind: "execution".to_string(),
            code: "EXEC_NON_ZERO".to_string(),
            message: "failed".to_string(),
            details: None,
        }),
        transition_cause: Some("retry_exhausted".to_string()),
        lifecycle_state: Some("failed".to_string()),
        lifecycle_transitions: vec![
            bijux_dag_artifacts::NodeLifecycleTransition {
                from_state: "pending".to_string(),
                to_state: "eligible".to_string(),
                cause: "scheduler_eligible".to_string(),
                unix_ms: 1,
            },
            bijux_dag_artifacts::NodeLifecycleTransition {
                from_state: "eligible".to_string(),
                to_state: "queued".to_string(),
                cause: "scheduler_queued".to_string(),
                unix_ms: 1,
            },
            bijux_dag_artifacts::NodeLifecycleTransition {
                from_state: "queued".to_string(),
                to_state: "failed".to_string(),
                cause: "execution_failed".to_string(),
                unix_ms: 2,
            },
        ],
        replay_provenance: None,
    };

    let serialized = serde_json::to_vec(&trace).expect("serialize trace");
    assert!(serialized.len() < 120_000, "trace unexpectedly bloated");
}
