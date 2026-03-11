#[allow(clippy::wildcard_imports)]
use crate::domain::automation_contracts::*;

pub(super) fn rows() -> Vec<Value> {
    vec![
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-RELEASE-BUILD-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/release_binary_size_report.json",
                "artifacts/status/debug_binary_size_report.json",
                "artifacts/status/release_binary_size_contributors.json",
                "artifacts/status/release_dependency_inventory.json",
                "artifacts/status/license_inventory.json",
                "artifacts/status/reproducible_build_assumptions.json",
                "artifacts/status/release_artifact_manifest.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-CONTRACT-GENERATE-RELEASE-BUILD-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-RELEASE-EVIDENCE-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/release_evidence_bundle.json",
                "artifacts/status/release_status_manifest.json",
                "artifacts/status/release_truth_report.json",
                "artifacts/status/release_truth_report.txt"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-CONTRACT-GENERATE-RELEASE-EVIDENCE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-PLUGIN-SCAFFOLD-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/plugin_scaffold_python_inventory.json",
                "artifacts/status/plugin_scaffold_rust_inventory.json",
                "artifacts/status/plugin_scaffold_diff.json",
                "artifacts/status/plugin_scaffold_non_behavioral_files.json",
                "artifacts/status/plugin_scaffold_file_justification.json",
                "artifacts/status/plugin_scaffold_minimalism_summary.txt"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-CONTRACT-GENERATE-PLUGIN-SCAFFOLD-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-PLUGIN-MIGRATION-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/plugin_lifecycle_ownership_report.json",
                "artifacts/status/plugin_scaffold_efficiency_report.json",
                "artifacts/status/plugin_scaffold_lifecycle_proof_report.json",
                "artifacts/status/plugin_namespace_abuse_proof_report.json",
                "artifacts/status/plugin_doctor_clarity_report.json",
                "artifacts/status/plugin_explain_clarity_report.json",
                "artifacts/status/plugin_where_ownership_report.json",
                "artifacts/status/plugin_command_set_status.json",
                "artifacts/status/plugin_migration_report.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-CONTRACT-GENERATE-PLUGIN-MIGRATION-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-PLUGIN-MANIFEST-SCAFFOLD-FUZZ-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/plugin_manifest_crash_triage_artifact.json",
                "artifacts/status/plugin_scaffold_crash_triage_artifact.json",
                "artifacts/status/plugin_manifest_fuzz_regression_artifact.json",
                "artifacts/status/plugin_scaffold_fuzz_regression_artifact.json",
                "artifacts/status/plugin_manifest_scaffold_fuzz_contract.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-CONTRACT-GENERATE-PLUGIN-MANIFEST-SCAFFOLD-FUZZ-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-PLUGIN-STATE-CORRUPTION-CAMPAIGN-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/plugin_state_corruption_campaign_artifact.json",
                "artifacts/status/plugin_state_corruption_corpus_retention_artifact.json",
                "artifacts/status/plugin_state_corruption_triage_artifact.json",
                "artifacts/status/plugin_state_corruption_regression_artifact.json",
                "artifacts/status/plugin_state_corruption_severity_classification.json",
                "artifacts/status/plugin_state_corruption_contract.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-CONTRACT-GENERATE-PLUGIN-STATE-CORRUPTION-CAMPAIGN-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-CONFIG-DEEP-BEHAVIOR-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/config_semantic_roundtrip_artifact.json",
                "artifacts/status/config_precedence_artifact.json",
                "artifacts/status/config_determinism_artifact.json",
                "artifacts/status/config_corruption_recovery_artifact.json",
                "artifacts/status/config_deep_behavior_drift_artifact.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-CONTRACT-GENERATE-CONFIG-DEEP-BEHAVIOR-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-CONFIG-CORRUPTION-CAMPAIGN-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/config_corruption_campaign_artifact.json",
                "artifacts/status/config_corruption_invariants_artifact.json",
                "artifacts/status/config_corruption_corpus_retention_artifact.json",
                "artifacts/status/config_corruption_triage_artifact.json",
                "artifacts/status/config_corruption_regression_artifact.json",
                "artifacts/status/config_corruption_severity_classification.json",
                "artifacts/status/config_corruption_recovery_classification.json",
                "artifacts/status/config_corruption_determinism_artifact.json",
                "artifacts/status/config_corruption_release_blocking_contract.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-CONTRACT-GENERATE-CONFIG-CORRUPTION-CAMPAIGN-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DIAGNOSTICS-DEEP-BEHAVIOR-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/diagnostics_consistency_artifact.json",
                "artifacts/status/doctor_determinism_artifact.json",
                "artifacts/status/diagnostics_schema_drift_artifact.json",
                "artifacts/status/diagnostics_source_of_truth_artifact.json",
                "artifacts/status/findings_order_artifact.json",
                "artifacts/status/diagnostics_contract_artifact.json",
                "artifacts/status/diagnostics_deep_behavior_drift_artifact.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-CONTRACT-GENERATE-DIAGNOSTICS-DEEP-BEHAVIOR-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DIAGNOSTICS-TRUST-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/diagnostics_trust_artifact.json",
                "artifacts/status/actionable_diagnostics_artifact.json",
                "artifacts/status/diagnostics_minimalism_artifact.json",
                "artifacts/status/diagnostics_trust_schema_drift_artifact.json",
                "artifacts/status/diagnostics_trust_contract.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-CONTRACT-GENERATE-DIAGNOSTICS-TRUST-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-STATUS-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/status.json",
                "artifacts/status/status_root_commands.json",
                "artifacts/status/status_cli_subcommands.json",
                "artifacts/status/status_dev_cli_subcommands.json",
                "artifacts/status/status_plugin_commands.json",
                "artifacts/status/status_repl_parity_coverage.json",
                "artifacts/status/status_python_bridge_parity_coverage.json",
                "artifacts/status/status_install_packaging_parity_coverage.json",
                "artifacts/status/status_state_behavior_coverage.json",
                "artifacts/status/status_state_paths_report.json",
                "artifacts/status/status_state_corruption_health_report.json",
                "artifacts/status/status_snapshot_coverage.json",
                "artifacts/status/status_stream_coverage.json",
                "artifacts/status/status_exit_code_coverage.json",
                "artifacts/status/status_failure_path_coverage.json",
                "artifacts/status/status_compatibility_aliases.json",
                "artifacts/status/status_known_parity_gaps.json",
                "artifacts/status/status_intentional_differences.json",
                "artifacts/status/status_unowned_scripts.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-CONTRACT-GENERATE-STATUS-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-MAINTAINER-CONTROL-PLANE-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/maintainer_scripts_outside_dev_cli.json",
                "artifacts/status/maintainer_control_plane_commands.json",
                "artifacts/status/maintainer_control_plane_text_report.txt",
                "artifacts/status/maintainer_control_plane_report.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-CONTRACT-GENERATE-MAINTAINER-CONTROL-PLANE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-CRATE-BOUNDARY-METRICS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/crate_boundary_metrics.json",
                "artifacts/status/crate_boundary_report.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-CONTRACT-GENERATE-CRATE-BOUNDARY-METRICS",
        }),
    ]
}
