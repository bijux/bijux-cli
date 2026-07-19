use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::{
    fake_batch_backend_reference, fake_batch_executor_contract, kubernetes_adapter_contract,
    reject_unsupported_k8s_fields, slurm_adapter_design_contract, FakeBatchExecutor,
    FakeBatchJobStatus,
};

#[test]
fn fake_batch_executor_supports_queued_running_failed_completed_and_cancelled_states() {
    let mut executor = FakeBatchExecutor::default();
    let job_a = executor.submit("run-1", "node-a");
    assert_eq!(executor.snapshot(&job_a).expect("queued").status, FakeBatchJobStatus::Queued);
    executor.transition(&job_a, FakeBatchJobStatus::Running).expect("running");
    executor.complete_failure(&job_a, 17, "remote worker crashed").expect("failed");
    let failed = executor.snapshot(&job_a).expect("failed snapshot");
    assert_eq!(failed.status, FakeBatchJobStatus::Failed);
    assert_eq!(failed.exit_code, Some(17));

    let job_b = executor.submit("run-1", "node-b");
    executor.transition(&job_b, FakeBatchJobStatus::Running).expect("running");
    executor.transition(&job_b, FakeBatchJobStatus::Completed).expect("completed");
    assert_eq!(
        executor.snapshot(&job_b).expect("completed snapshot").status,
        FakeBatchJobStatus::Completed
    );

    let job_c = executor.submit("run-1", "node-c");
    executor.cancel(&job_c, "operator request").expect("cancelled");
    let cancelled = executor.snapshot(&job_c).expect("cancelled snapshot");
    assert_eq!(cancelled.status, FakeBatchJobStatus::Cancelled);
    assert!(cancelled.diagnostics.iter().any(|value| value.contains("operator request")));
}

#[test]
fn slurm_design_contract_documents_submit_poll_cancel_and_failure_mapping() {
    let report = slurm_adapter_design_contract();
    assert!(report.submit_status_cancel_documented);
    assert_eq!(report.contract.submit_command, "sbatch");
    assert_eq!(report.contract.poll_command, "sacct");
    assert_eq!(report.contract.cancel_command, "scancel");
    assert_eq!(
        report.failure_mapping_examples.get("SLURM_WALLTIME_EXCEEDED").map(String::as_str),
        Some("timeout")
    );
    assert_eq!(
        report.failure_mapping_examples.get("SLURM_PREEMPTED").map(String::as_str),
        Some("infrastructure")
    );
}

#[test]
fn kubernetes_contract_documents_mapping_and_rejects_unsupported_fields() {
    let report = kubernetes_adapter_contract();
    assert!(report.job_spec_mapping.contains("Job"));
    assert!(report.pod_status_mapping.contains("runtime"));
    assert!(report.timeout_cancel_behavior.contains("deadline"));
    assert!(report.unsupported_field_rejection.contains(&"hostNetwork".to_string()));
    assert!(reject_unsupported_k8s_fields(&["hostNetwork".to_string()]).is_err());
}

#[test]
fn fake_batch_contract_matches_generic_backend_reference() {
    let fake = fake_batch_executor_contract();
    let generic = fake_batch_backend_reference();
    assert_eq!(generic.platform_name, "fake-batch");
    assert_eq!(fake.submit_api, format!("{}(run_id,node_id) -> job_id", generic.submit_api));
    assert_eq!(fake.poll_api, format!("{}(job_id) -> status", generic.poll_api));
    assert_eq!(fake.cancel_api, format!("{}(job_id, diagnostic)", generic.cancel_api));
}
