#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

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
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-REPO-HEALTH-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-EVIDENCE-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/maintainer_evidence_list_report.json",
                "artifacts/status/maintainer_evidence_audit_report.json",
                "artifacts/status/maintainer_evidence_stale_report.json",
                "artifacts/status/maintainer_evidence_matrix_report.json",
                "artifacts/status/maintainer_evidence_website_export_report.json",
                "artifacts/status/maintainer_evidence_ci_export_report.json",
                "artifacts/status/maintainer_evidence_release_export_report.json",
                "artifacts/status/maintainer_evidence_command_map_report.json",
                "artifacts/status/maintainer_evidence_parity_map_report.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-EVIDENCE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-COCKPIT-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/maintainer_status_report.json",
                "artifacts/status/maintainer_dashboard_report.json",
                "artifacts/status/maintainer_quickcheck_report.json",
                "artifacts/status/maintainer_truth_report.json",
                "artifacts/status/maintainer_blockers_report.json",
                "artifacts/status/maintainer_next_report.json",
                "artifacts/status/maintainer_cockpit_text_heads.json",
                "artifacts/status/maintainer_summary_surface_artifact.json",
                "artifacts/status/maintainer_summary_surface_drift_artifact.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-COCKPIT-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-RELEASE-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/maintainer_release_status_report.json",
                "artifacts/status/maintainer_release_evidence_report.json",
                "artifacts/status/maintainer_release_readiness_report.json",
                "artifacts/status/maintainer_release_diff_report.json",
                "artifacts/status/maintainer_release_gaps_report.json",
                "artifacts/status/maintainer_release_summary_report.json",
                "artifacts/status/maintainer_release_manifest_report.json",
                "artifacts/status/maintainer_release_notes_report.json",
                "artifacts/status/maintainer_release_behavior_changes_report.json",
                "artifacts/status/maintainer_release_intentional_differences_report.json",
                "artifacts/status/maintainer_release_unresolved_gaps_report.json",
                "artifacts/status/maintainer_release_compatibility_leftovers_report.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-RELEASE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-MAINTENANCE-MIGRATION-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/maintainer_maintenance_remaining_report.json",
                "artifacts/status/maintainer_maintenance_migrated_report.json",
                "artifacts/status/maintainer_maintenance_diff_report.json",
                "artifacts/status/maintainer_maintenance_value_ranking.json",
                "artifacts/status/maintainer_make_target_inventory.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-MAINTENANCE-MIGRATION-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-REPO-DOCS-MAINTENANCE-CRATE-HEALTH-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/repo_docs_maintenance_crate_health_artifact.json",
                "artifacts/status/repo_docs_maintenance_crate_health_drift_artifact.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-REPO-DOCS-MAINTENANCE-CRATE-HEALTH-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-ROUTE-REGISTRY-ENV-CONTRACTS-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/route_registry_env_contracts_artifact.json",
                "artifacts/status/route_registry_env_contracts_drift_artifact.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-ROUTE-REGISTRY-ENV-CONTRACTS-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-RUSTDOC-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/rustdoc_audit_report.json",
                "artifacts/status/rustdoc_public_api_coverage_report.json",
                "artifacts/status/rustdoc_audit_report.txt"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-RUSTDOC-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-RELEASE-TRUTH-BUNDLE",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/maintainer_release_truth_bundle.json"],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-RELEASE-TRUTH-BUNDLE",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-CONTROL-PLANE-BUNDLE",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/maintainer_control_plane_bundle.json"],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-CONTROL-PLANE-BUNDLE",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-REPORT-IO-MAP",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/maintainer_report_io_map.json"],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-REPORT-IO-MAP",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-PARITY-CONSISTENCY-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/migration_truth_artifact.json",
                "artifacts/status/parity_evidence_consistency_artifact.json",
                "artifacts/status/parity_drift_artifact.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-PARITY-CONSISTENCY-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-INVARIANTS-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/maintainer_invariants_artifact.json",
                "artifacts/status/maintainer_invariants_drift_artifact.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-INVARIANTS-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-ROUTE-REGISTRY-OWNERSHIP-DIFF",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/maintainer_route_registry_ownership_diff.json"],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-ROUTE-REGISTRY-OWNERSHIP-DIFF",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-DIAGNOSTICS-SOURCE-MAP",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/maintainer_diagnostics_source_map.json"],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-DIAGNOSTICS-SOURCE-MAP",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-INTERFACE-BRIDGE-REPORT",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/maintainer_interface_bridge_report.json"],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-INTERFACE-BRIDGE-REPORT",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-OWNERSHIP-REPORT",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/maintainer_ownership_report.json",
                "artifacts/status/maintainer_ownership_report.txt"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-OWNERSHIP-REPORT",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-STALE-ARTIFACT-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/stale_artifact_artifact.json",
                "artifacts/status/stale_evidence_artifact.json",
                "artifacts/status/stale_report_artifact.json",
                "artifacts/status/stale_detection_regression_suite.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-STALE-ARTIFACT-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-STATE-DIAGNOSTICS-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/state_audit_truth_artifact.json",
                "artifacts/status/state_doctor_truth_artifact.json",
                "artifacts/status/corrupted_state_truth_artifact.json",
                "artifacts/status/state_diagnostics_drift_artifact.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-STATE-DIAGNOSTICS-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-BOUNDARY-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/maintainer_owned_behaviors_inventory.json",
                "artifacts/status/runtime_owned_behaviors_inventory.json",
                "artifacts/status/misplaced_maintainer_behaviors_report.json",
                "artifacts/status/maintainer_command_ownership_report.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-BOUNDARY-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-COMMAND-SURFACE-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/maintainer_command_coverage_report.json",
                "artifacts/status/maintainer_command_matrix_artifact.json",
                "artifacts/status/maintainer_command_surface_domain_contract.json",
                "artifacts/status/maintainer_command_remaining_inventory.json",
                "artifacts/status/maintainer_command_value_ranking.json",
                "artifacts/status/maintainer_command_completion_report.json",
                "artifacts/status/maintainer_command_closure_set.json",
                "artifacts/status/cli_maintainer_command_closure_report.json",
                "artifacts/status/cli_maintainer_command_closure_report.txt"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-COMMAND-SURFACE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-DISPATCH-OWNERSHIP-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/maintainer_dispatch_ownership_report.json",
                "artifacts/status/bin_entrypoint_responsibility_diff.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-DISPATCH-OWNERSHIP-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-RESILIENCE-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/maintainer_control_plane_resilience_artifact.json",
                "artifacts/status/maintainer_determinism_artifact.json",
                "artifacts/status/maintainer_side_effect_audit_artifact.json",
                "artifacts/status/maintainer_resilience_drift_artifact.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-RESILIENCE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-SCOPE-REASSESSMENT",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/runtime_responsibility_reassessment.json"],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-SCOPE-REASSESSMENT",
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
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-BRIDGE-WRAPPER-ONLY-REPORTS",
        }),
    ]
}
