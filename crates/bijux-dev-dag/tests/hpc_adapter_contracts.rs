use bijux_dag_testkit as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn hpc_adapter_contract_doc_exists_with_required_sections() {
    let root = repo_root();
    let path = root.join("docs/spec/HPC_ADAPTER_CONTRACT.md");
    assert!(path.exists(), "missing hpc adapter contract doc");
    let text = fs::read_to_string(path).expect("read hpc contract");

    for token in [
        "Queue and partition mapping",
        "Walltime mapping",
        "Retry precedence",
        "Scratch and staging semantics",
        "Failure normalization",
        "Array job and unsupported feature behavior",
        "Environment and scheduler identity capture",
        "Universal vs scheduler-specific semantics",
    ] {
        assert!(
            text.contains(token),
            "hpc contract missing required section: {token}"
        );
    }
}

#[test]
fn hpc_runtime_contract_tests_cover_required_semantics() {
    let root = repo_root();
    let source = fs::read_to_string(
        root.join("crates/bijux-dag-runtime/tests/backend_cluster_contracts.rs"),
    )
    .expect("read backend cluster contract tests");

    for token in [
        "map_node_to_hpc_queue_partition",
        "map_timeout_to_hpc_walltime",
        "effective_hpc_retry_policy",
        "hpc_scratch_staging_semantics",
        "SLURM_QUEUE_REJECTED",
        "SLURM_INVALID_ACCOUNT",
        "SLURM_WALLTIME_EXCEEDED",
        "SLURM_PREEMPTED",
        "hpc_poll_response_recovered",
        "hpc_log_collection_semantics",
        "hpc_array_job_supported",
        "reject_unsupported_hpc_scheduler_features",
        "hpc_environment_fingerprint",
        "capture_hpc_scheduler_version",
        "hpc_resource_fingerprint",
        "hpc_replay_fidelity_from_module_fingerprints",
    ] {
        assert!(
            source.contains(token),
            "backend cluster contracts missing hpc token: {token}"
        );
    }
}
