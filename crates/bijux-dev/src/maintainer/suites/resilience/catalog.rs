#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(crate) fn rows() -> Vec<Value> {
    vec![
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DETERMINISTIC-OUTPUT-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/deterministic_output_report.json",
                "artifacts/status/determinism_dashboard.json",
                "artifacts/status/determinism_expectations.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-DETERMINISTIC-OUTPUT-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-OUTPUT-BRIDGE-FUZZ-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/output_crash_triage_artifact.json",
                "artifacts/status/bridge_conversion_crash_triage_artifact.json",
                "artifacts/status/output_fuzz_regression_artifact.json",
                "artifacts/status/bridge_conversion_fuzz_regression_artifact.json",
                "artifacts/status/output_envelope_fuzz_contract.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-OUTPUT-BRIDGE-FUZZ-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-PARSER-FUZZ-HARDENING-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/parser_crash_triage_artifact.json",
                "artifacts/status/parser_fuzz_regression_artifact.json",
                "artifacts/status/parser_fuzz_campaign_artifact.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-PARSER-FUZZ-HARDENING-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-CLEANUP-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/docs_unreferenced_candidates.json",
                "artifacts/status/stale_snapshot_candidates.json",
                "artifacts/status/dead_generated_artifact_candidates.json",
                "artifacts/status/cleanup_report.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-CLEANUP-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MIGRATION-NOTES",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/migration_notes_commands.json",
                "artifacts/status/migration_notes_packaging.json",
                "artifacts/status/migration_notes_plugin_lifecycle.json",
                "artifacts/status/migration_notes_state_behavior.json",
                "artifacts/status/migration_notes.txt"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MIGRATION-NOTES",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-PRODUCT-MOUNT-READINESS-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/official_product_mount_registry.json",
                "artifacts/status/product_mount_readiness_report.json",
                "artifacts/status/product_mount_support_report.json",
                "artifacts/status/product_mount_gap_report.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-PRODUCT-MOUNT-READINESS-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-CONFIG-FUZZ-HARDENING-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/config_parser_crash_triage_artifact.json",
                "artifacts/status/config_serializer_crash_triage_artifact.json",
                "artifacts/status/config_fuzz_regression_artifact.json",
                "artifacts/status/config_fuzz_contract.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-CONFIG-FUZZ-HARDENING-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-ADVERSARIAL-FS-PROCESS-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/adversarial_fs_process_matrix.json",
                "artifacts/status/adversarial_fs_process_artifact.json",
                "artifacts/status/adversarial_fs_process_contract.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-ADVERSARIAL-FS-PROCESS-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-STATE-CORRUPTION-HARNESS-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/state_corruption_campaign_artifact.json",
                "artifacts/status/state_corruption_reproducer_retention_artifact.json",
                "artifacts/status/state_corruption_harness_contract.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-STATE-CORRUPTION-HARNESS-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-COMMAND-SURFACE-INVENTORY",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/documented_python_commands_not_proven_in_rust.json",
                "artifacts/status/public_python_paths_still_reachable.json",
                "artifacts/status/legacy_alias_paths_still_accepted.json",
                "artifacts/status/compatibility_shims_still_active.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-COMMAND-SURFACE-INVENTORY",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-COMMAND-FAMILY-CLOSURE-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/config_closure_report.json",
                "artifacts/status/plugins_closure_report.json",
                "artifacts/status/history_closure_report.json",
                "artifacts/status/memory_closure_report.json",
                "artifacts/status/diagnostics_closure_report.json",
                "artifacts/status/repl_shared_law_closure_report.json",
                "artifacts/status/command_family_closure_report.json",
                "artifacts/status/command_family_closure_report.txt",
                "artifacts/status/command_family_partial_area_acceptance.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-COMMAND-FAMILY-CLOSURE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-COMMAND-MIGRATION-MATRIX",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/command_migration_matrix.json",
                "artifacts/status/command_migration_rust_partial.json",
                "artifacts/status/command_migration_python_only.json",
                "artifacts/status/command_migration_intentional_differences.json",
                "artifacts/status/command_migration_matrix.txt",
                "artifacts/status/command_migration_repl_paths.json",
                "artifacts/status/command_migration_python_bridge_entrypoints.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-COMMAND-MIGRATION-MATRIX",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-EVIDENCE-INTEGRITY-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/evidence_coverage_report.json",
                "artifacts/status/evidence_integrity_artifact.json",
                "artifacts/status/orphan_evidence_report.json",
                "artifacts/status/orphan_evidence_artifact.json",
                "artifacts/status/claim_without_evidence_report.json",
                "artifacts/status/evidence_command_map_report.json",
                "artifacts/status/evidence_parity_map_report.json",
                "artifacts/status/config_owners_by_layer_report.json",
                "artifacts/status/config_file_schema_owners_report.json",
                "artifacts/status/config_python_compatibility_shims_report.json",
                "artifacts/status/config_rust_sources_report.json",
                "artifacts/status/config_precedence_proofs_report.json",
                "artifacts/status/config_mutation_rollback_proofs_report.json",
                "artifacts/status/config_corruption_evidence_report.json",
                "artifacts/status/config_owner_drift_report.json",
                "artifacts/status/config_evidence_link_report.json",
                "artifacts/status/config_ownership_truth.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-EVIDENCE-INTEGRITY-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-HISTORY-SURFACE-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/history_command_coverage_report.json",
                "artifacts/status/history_command_coverage_artifact.json",
                "artifacts/status/history_corruption_matrix_artifact.json",
                "artifacts/status/history_read_domain_contract.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-HISTORY-SURFACE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DIAGNOSTICS-SURFACE-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/diagnostics_command_coverage_report.json",
                "artifacts/status/diagnostics_matrix_artifact.json",
                "artifacts/status/diagnostics_shape_drift_artifact.json",
                "artifacts/status/diagnostics_operator_truth_contract.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-DIAGNOSTICS-SURFACE-REPORTS",
        }),
    ]
}
