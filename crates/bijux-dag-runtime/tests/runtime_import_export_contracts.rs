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
