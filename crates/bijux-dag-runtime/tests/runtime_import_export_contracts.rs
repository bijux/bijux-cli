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

use bijux_dag_runtime::{run_manifest_valid, ManifestVerificationInput};

#[test]
fn import_export_requires_manifest_contract_integrity() {
    assert!(run_manifest_valid(&ManifestVerificationInput {
        has_run_header: true,
        has_trace_index: true,
        has_outputs_index: true,
        totals_consistent: true,
    }));
}
