use bijux_dag_runtime::{run_manifest_valid, ManifestVerificationInput};

#[test]
fn state_machine_terminal_manifest_requires_consistent_totals() {
    assert!(!run_manifest_valid(&ManifestVerificationInput {
        has_run_header: true,
        has_trace_index: true,
        has_outputs_index: true,
        totals_consistent: false,
    }));
}
