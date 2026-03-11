#[allow(clippy::wildcard_imports)]
use crate::contract_engine::maintenance::*;

pub(crate) fn rows() -> Vec<Value> {
    vec![
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-REPO-HEALTH-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/repo_health_report.json",
                "artifacts/status/repo_drift_report.json",
                "artifacts/status/repo_inventories_report.json",
                "artifacts/status/repo_generated_report.json",
                "artifacts/status/repo_stale_report.json"
            ],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-REPO-HEALTH-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-EVIDENCE-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/dev_cli_evidence_list_report.json",
                "artifacts/status/dev_cli_evidence_audit_report.json",
                "artifacts/status/dev_cli_evidence_stale_report.json",
                "artifacts/status/dev_cli_evidence_matrix_report.json",
                "artifacts/status/dev_cli_evidence_website_export_report.json",
                "artifacts/status/dev_cli_evidence_ci_export_report.json",
                "artifacts/status/dev_cli_evidence_release_export_report.json",
                "artifacts/status/dev_cli_evidence_command_map_report.json",
                "artifacts/status/dev_cli_evidence_parity_map_report.json"
            ],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-EVIDENCE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-COCKPIT-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/dev_cli_status_report.json",
                "artifacts/status/dev_cli_dashboard_report.json",
                "artifacts/status/dev_cli_quickcheck_report.json",
                "artifacts/status/dev_cli_truth_report.json",
                "artifacts/status/dev_cli_blockers_report.json",
                "artifacts/status/dev_cli_next_report.json",
                "artifacts/status/dev_cli_cockpit_text_heads.json",
                "artifacts/status/dev_cli_summary_surface_artifact.json",
                "artifacts/status/dev_cli_summary_surface_drift_artifact.json"
            ],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-COCKPIT-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-RELEASE-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/dev_cli_release_status_report.json",
                "artifacts/status/dev_cli_release_evidence_report.json",
                "artifacts/status/dev_cli_release_readiness_report.json",
                "artifacts/status/dev_cli_release_diff_report.json",
                "artifacts/status/dev_cli_release_gaps_report.json",
                "artifacts/status/dev_cli_release_summary_report.json",
                "artifacts/status/dev_cli_release_manifest_report.json",
                "artifacts/status/dev_cli_release_notes_report.json",
                "artifacts/status/dev_cli_release_behavior_changes_report.json",
                "artifacts/status/dev_cli_release_intentional_differences_report.json",
                "artifacts/status/dev_cli_release_unresolved_gaps_report.json",
                "artifacts/status/dev_cli_release_compatibility_leftovers_report.json"
            ],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-RELEASE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-MAINTENANCE-MIGRATION-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/dev_cli_maintenance_remaining_report.json",
                "artifacts/status/dev_cli_maintenance_migrated_report.json",
                "artifacts/status/dev_cli_maintenance_diff_report.json",
                "artifacts/status/dev_cli_maintenance_value_ranking.json",
                "artifacts/status/dev_cli_make_target_inventory.json"
            ],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-MAINTENANCE-MIGRATION-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-REPO-DOCS-MAINTENANCE-CRATE-HEALTH-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/repo_docs_maintenance_crate_health_artifact.json",
                "artifacts/status/repo_docs_maintenance_crate_health_drift_artifact.json"
            ],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-REPO-DOCS-MAINTENANCE-CRATE-HEALTH-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-ROUTE-REGISTRY-ENV-CONTRACTS-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/route_registry_env_contracts_artifact.json",
                "artifacts/status/route_registry_env_contracts_drift_artifact.json"
            ],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-ROUTE-REGISTRY-ENV-CONTRACTS-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-RUSTDOC-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/rustdoc_audit_report.json",
                "artifacts/status/rustdoc_public_api_coverage_report.json",
                "artifacts/status/rustdoc_audit_report.txt"
            ],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-RUSTDOC-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-RELEASE-TRUTH-BUNDLE",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/dev_cli_release_truth_bundle.json"],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-RELEASE-TRUTH-BUNDLE",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-CONTROL-PLANE-BUNDLE",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/dev_cli_control_plane_bundle.json"],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-CONTROL-PLANE-BUNDLE",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-MAINTAINER-REPORT-IO-MAP",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/dev_cli_maintainer_report_io_map.json"],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-MAINTAINER-REPORT-IO-MAP",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-PARITY-CONSISTENCY-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/migration_truth_artifact.json",
                "artifacts/status/parity_evidence_consistency_artifact.json",
                "artifacts/status/parity_drift_artifact.json"
            ],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-PARITY-CONSISTENCY-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-INVARIANTS-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/dev_cli_invariants_artifact.json",
                "artifacts/status/dev_cli_invariants_drift_artifact.json"
            ],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-INVARIANTS-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-ROUTE-REGISTRY-OWNERSHIP-DIFF",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/dev_cli_route_registry_ownership_diff.json"],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-ROUTE-REGISTRY-OWNERSHIP-DIFF",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-DIAGNOSTICS-SOURCE-MAP",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/dev_cli_diagnostics_source_map.json"],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-DIAGNOSTICS-SOURCE-MAP",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-INTERFACE-BRIDGE-REPORT",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/dev_cli_interface_bridge_report.json"],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-INTERFACE-BRIDGE-REPORT",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-OWNERSHIP-REPORT",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/dev_cli_ownership_report.json",
                "artifacts/status/dev_cli_ownership_report.txt"
            ],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-OWNERSHIP-REPORT",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-STALE-ARTIFACT-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/stale_artifact_artifact.json",
                "artifacts/status/stale_evidence_artifact.json",
                "artifacts/status/stale_report_artifact.json",
                "artifacts/status/stale_detection_regression_suite.json"
            ],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-STALE-ARTIFACT-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-STATE-DIAGNOSTICS-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/state_audit_truth_artifact.json",
                "artifacts/status/state_doctor_truth_artifact.json",
                "artifacts/status/corrupted_state_truth_artifact.json",
                "artifacts/status/state_diagnostics_drift_artifact.json"
            ],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-STATE-DIAGNOSTICS-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-BOUNDARY-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/dev_cli_owned_behaviors_inventory.json",
                "artifacts/status/runtime_owned_behaviors_inventory.json",
                "artifacts/status/misplaced_dev_behaviors_report.json",
                "artifacts/status/dev_cli_maintainer_command_ownership_report.json"
            ],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-BOUNDARY-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-COMMAND-SURFACE-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/dev_cli_command_coverage_report.json",
                "artifacts/status/dev_cli_command_matrix_artifact.json",
                "artifacts/status/dev_cli_command_surface_domain_contract.json",
                "artifacts/status/dev_cli_command_remaining_inventory.json",
                "artifacts/status/dev_cli_command_value_ranking.json",
                "artifacts/status/dev_cli_command_completion_report.json",
                "artifacts/status/dev_cli_command_closure_set.json",
                "artifacts/status/cli_dev_command_closure_report.json",
                "artifacts/status/cli_dev_command_closure_report.txt"
            ],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-COMMAND-SURFACE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-DISPATCH-OWNERSHIP-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/dev_cli_dispatch_ownership_report.json",
                "artifacts/status/bin_entrypoint_responsibility_diff.json"
            ],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-DISPATCH-OWNERSHIP-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-RESILIENCE-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/dev_cli_control_plane_resilience_artifact.json",
                "artifacts/status/dev_cli_determinism_artifact.json",
                "artifacts/status/dev_cli_side_effect_audit_artifact.json",
                "artifacts/status/dev_cli_resilience_drift_artifact.json"
            ],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-RESILIENCE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DEV-CLI-SCOPE-REASSESSMENT",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/runtime_responsibility_reassessment.json"],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-DEV-CLI-SCOPE-REASSESSMENT",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-BRIDGE-WRAPPER-ONLY-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/bridge_wrapper_only_closure_report.json",
                "artifacts/status/bridge_wrapper_only_closure_report.txt"
            ],
            "command": "bijux dev cli maintenance status run --id STATUS-CONTRACT-GENERATE-BRIDGE-WRAPPER-ONLY-REPORTS",
        }),
    ]
}
