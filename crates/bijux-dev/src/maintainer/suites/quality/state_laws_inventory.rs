#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn rows() -> Vec<Value> {
    vec![
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-STATE-AUDIT-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/state_migration_status.json",
                "artifacts/status/unified_state_behavior_report.json",
                "artifacts/status/unified_state_corruption_report.json",
                "artifacts/status/unified_state_rollback_report.json",
                "artifacts/status/unified_state_path_resolution_report.json",
                "artifacts/status/unified_state_doctor_snapshots.json",
                "artifacts/status/unified_state_audit_payload.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-STATE-AUDIT-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEEP-TEST-QUALITY-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/deep_tests_by_value_report.json",
                "artifacts/status/deep_missing_behavior_cases_report.json",
                "artifacts/status/deep_weak_tests_replacement_report.json",
                "artifacts/status/deep_test_first_domains_contract.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEEP-TEST-QUALITY-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-PERFORMANCE-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/performance_report.json",
                "artifacts/status/performance_regression_budget.json",
                "artifacts/status/performance_benchmark_policy.json",
                "artifacts/status/performance_report.txt"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-PERFORMANCE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MEMORY-SURFACE-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/memory_command_coverage_report.json",
                "artifacts/status/memory_command_coverage_artifact.json",
                "artifacts/status/memory_corruption_matrix_artifact.json",
                "artifacts/status/memory_python_parity_artifact.json",
                "artifacts/status/memory_read_domain_contract.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MEMORY-SURFACE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-STATE-LAW-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/state_file_inventory.json",
                "artifacts/status/state_file_readers.json",
                "artifacts/status/state_file_writers.json",
                "artifacts/status/state_file_mutation_paths.json",
                "artifacts/status/state_write_guarantees.json",
                "artifacts/status/state_recovery_guarantees.json",
                "artifacts/status/state_complexity_report.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-STATE-LAW-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-STREAM-DISCIPLINE-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/stream_discipline_artifact.json",
                "artifacts/status/stream_drift_artifact.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-STREAM-DISCIPLINE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-HISTORY-DEEP-BEHAVIOR-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/history_semantic_artifact.json",
                "artifacts/status/history_determinism_artifact.json",
                "artifacts/status/history_corruption_artifact.json",
                "artifacts/status/history_repl_interop_artifact.json",
                "artifacts/status/history_stream_discipline_artifact.json",
                "artifacts/status/history_failure_class_artifact.json",
                "artifacts/status/history_deep_behavior_drift_artifact.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-HISTORY-DEEP-BEHAVIOR-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MEMORY-DEEP-BEHAVIOR-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/memory_semantic_artifact.json",
                "artifacts/status/memory_determinism_artifact.json",
                "artifacts/status/memory_corruption_artifact.json",
                "artifacts/status/memory_diagnostics_consistency_artifact.json",
                "artifacts/status/memory_failure_class_artifact.json",
                "artifacts/status/memory_path_behavior_artifact.json",
                "artifacts/status/memory_deep_behavior_drift_artifact.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MEMORY-DEEP-BEHAVIOR-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-ROUTE-LAW-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/route_command_owner_mapping.json",
                "artifacts/status/route_command_test_coverage_mapping.json",
                "artifacts/status/route_command_parity_status_mapping.json",
                "artifacts/status/route_special_cases.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-ROUTE-LAW-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-ROOT-COMMAND-SURFACE-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/root_command_coverage_report.json",
                "artifacts/status/root_command_coverage_artifact.json",
                "artifacts/status/root_command_surface_domain_contract.json",
                "artifacts/status/root_command_remaining_inventory.json",
                "artifacts/status/root_command_impact_ranking.json",
                "artifacts/status/root_command_completion_report.json",
                "artifacts/status/root_command_closure_set.json",
                "artifacts/status/root_command_completion_report.txt"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-ROOT-COMMAND-SURFACE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-CLI-COMMAND-SURFACE-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/cli_command_coverage_report.json",
                "artifacts/status/cli_command_coverage_artifact.json",
                "artifacts/status/cli_command_surface_domain_contract.json",
                "artifacts/status/cli_command_remaining_inventory.json",
                "artifacts/status/cli_command_value_ranking.json",
                "artifacts/status/cli_command_completion_report.json",
                "artifacts/status/cli_command_closure_set.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-CLI-COMMAND-SURFACE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-COMPATIBILITY-SHIM-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/compatibility_shim_inventory.json",
                "artifacts/status/compatibility_alias_inventory.json",
                "artifacts/status/hidden_alias_inventory.json",
                "artifacts/status/old_python_path_tolerance_inventory.json",
                "artifacts/status/compatibility_shim_count_delta.json",
                "artifacts/status/compatibility_alias_count_delta.json",
                "artifacts/status/compatibility_shim_count_report.json",
                "artifacts/status/compatibility_alias_count_report.json",
                "artifacts/status/live_compatibility_shims.json",
                "artifacts/status/live_compatibility_aliases.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-COMPATIBILITY-SHIM-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-METADATA-CONSISTENCY-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/command_metadata_artifact.json",
                "artifacts/status/route_metadata_artifact.json",
                "artifacts/status/metadata_drift_artifact.json",
                "artifacts/status/command_ownership_artifact.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-METADATA-CONSISTENCY-REPORTS",
        }),
    ]
}
