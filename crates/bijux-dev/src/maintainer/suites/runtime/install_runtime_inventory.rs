#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn rows() -> Vec<Value> {
    vec![
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-COMPATIBILITY-DEBT-TREND-REPORT",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/compatibility_debt_trend_report.json",
                "artifacts/status/compatibility_debt_trend_report.txt"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-COMPATIBILITY-DEBT-TREND-REPORT",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-HOSTILE-STATE-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/deterministic_hostile_state_report.json",
                "artifacts/status/failure_class_stability_report.json",
                "artifacts/status/deterministic_failure_quality_bar.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-HOSTILE-STATE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-PRECEDENCE-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/precedence_regression_report.json",
                "artifacts/parity/command_precedence_report.json",
                "artifacts/status/precedence_contract.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-PRECEDENCE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-NAMESPACE-RESERVATION-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/namespace_abuse_report.json",
                "artifacts/status/reserved_namespace_inventory.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-NAMESPACE-RESERVATION-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-INSTALL-TRUTH-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/install_source_diagnostics.json",
                "artifacts/status/ambiguous_runtime_diagnostics.json",
                "artifacts/status/install_health_report.json",
                "artifacts/status/install_health_report.txt",
                "artifacts/status/remaining_install_ambiguities.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-INSTALL-TRUTH-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-INSTALL-NEUTRALITY-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/install_neutrality_report.json",
                "artifacts/status/active_runtime_report.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-INSTALL-NEUTRALITY-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-INSTALL-RUNTIME-IDENTITY-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/install_runtime_identity_artifact.json",
                "artifacts/status/install_ambiguity_artifact.json",
                "artifacts/status/package_health_artifact.json",
                "artifacts/status/install_runtime_identity_drift_artifact.json",
                "artifacts/status/install_runtime_identity_contract.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-INSTALL-RUNTIME-IDENTITY-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-CONFIG-CORRUPTION-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/config_corruption_matrix.json",
                "artifacts/status/config_rollback_proof.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-CONFIG-CORRUPTION-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DOCS-DUPLICATION-REPORT",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/docs_duplication_report.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-DOCS-DUPLICATION-REPORT",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-PARSER-ABUSE-REPORT",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/parser_abuse_report.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-PARSER-ABUSE-REPORT",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-REPL-RECOVERY-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/repl_hostile_session_report.json",
                "artifacts/status/repl_recovery_behavior_report.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-REPL-RECOVERY-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-PYTHON-SOVEREIGNTY-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/python_bridge_status_report.json",
                "artifacts/status/python_surface_status_report.json",
                "artifacts/status/python_sovereignty_audit_report.json",
                "artifacts/status/python_desovereignization_report.json",
                "artifacts/status/python_desovereignization_report.txt",
                "artifacts/status/python_drift_report.json",
                "artifacts/status/python_packaging_direction_report.json",
                "artifacts/status/python_surface_direction_contract.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-PYTHON-SOVEREIGNTY-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-RUNTIME-MAINTAINER-LEAKAGE-REPORT",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/runtime_maintainer_leakage_report.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-RUNTIME-MAINTAINER-LEAKAGE-REPORT",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-FLAG-NORMALIZATION-MATRIX",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/flag_normalization_report.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-FLAG-NORMALIZATION-MATRIX",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-PLUGIN-LIFECYCLE-TEST-MATRIX",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/plugin_lifecycle_test_matrix.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-PLUGIN-LIFECYCLE-TEST-MATRIX",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-PLUGIN-FAILURE-ROLLBACK-TEST-MATRIX",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/plugin_failure_rollback_test_matrix.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-PLUGIN-FAILURE-ROLLBACK-TEST-MATRIX",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-RESERVED-NAMESPACE-TEST-MATRIX",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/reserved_namespace_test_matrix.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-RESERVED-NAMESPACE-TEST-MATRIX",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-BRIDGE-DUPLICATE-LAW-REPORT",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/bridge_duplicate_law_report.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-BRIDGE-DUPLICATE-LAW-REPORT",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-PLUGIN-STATE-REPORT",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/plugin_state_report.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-PLUGIN-STATE-REPORT",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-RUNTIME-PACKAGE-DIAGNOSTICS-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/runtime_identity_diagnostics_artifact.json",
                "artifacts/status/package_health_diagnostics_artifact.json",
                "artifacts/status/install_ambiguity_diagnostics_artifact.json",
                "artifacts/status/runtime_package_diagnostics_drift_artifact.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-RUNTIME-PACKAGE-DIAGNOSTICS-REPORTS",
        }),
    ]
}
