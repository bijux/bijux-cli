#[allow(clippy::wildcard_imports)]
use crate::domain::automation_contracts::*;

pub(super) fn rows() -> Vec<Value> {
    vec![
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-CROSS-SURFACE-STATE-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/cross_surface_state_consistency_artifact.json",
                "artifacts/status/cross_surface_state_drift_artifact.json",
                "artifacts/status/cross_surface_state_contract.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-CONTRACT-GENERATE-CROSS-SURFACE-STATE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-PLUGIN-DISCOVERY-DETERMINISM-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/plugin_discovery_determinism_report.json",
                "artifacts/status/plugin_ordering_law.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-CONTRACT-GENERATE-PLUGIN-DISCOVERY-DETERMINISM-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-PLUGIN-LIFECYCLE-FAILURE-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/plugin_lifecycle_failure_injection_report.json",
                "artifacts/status/plugin_rollback_proof_report.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-CONTRACT-GENERATE-PLUGIN-LIFECYCLE-FAILURE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-PACKAGING-AMBIGUITY-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/packaging_ambiguity_report.json",
                "artifacts/status/install_state_assumptions_report.json",
                "artifacts/status/package_health_report.json",
                "artifacts/status/package_health_report.txt"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-CONTRACT-GENERATE-PACKAGING-AMBIGUITY-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-STATE-RESILIENCE-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/history_corruption_matrix.json",
                "artifacts/status/memory_corruption_matrix.json",
                "artifacts/status/state_recovery_guidance.json",
                "artifacts/status/state_recovery_guidance.txt",
                "artifacts/status/state_resilience_summary.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-CONTRACT-GENERATE-STATE-RESILIENCE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-COMMAND-SURFACE-CONSISTENCY-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/command_surface_consistency_artifact.json",
                "artifacts/status/command_surface_consistency_drift_artifact.json",
                "artifacts/status/command_surface_consistency_summary.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-CONTRACT-GENERATE-COMMAND-SURFACE-CONSISTENCY-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-COMMAND-FAMILY-CONSISTENCY-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/command_family_consistency_artifact.json",
                "artifacts/status/cross_family_drift_artifact.json",
                "artifacts/status/shared_law_proof_artifact.json",
                "artifacts/status/command_family_consistency_requirement.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-CONTRACT-GENERATE-COMMAND-FAMILY-CONSISTENCY-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-CROSS-SURFACE-CONSISTENCY-LAW-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/cross_surface_consistency_artifact.json",
                "artifacts/status/cross_surface_drift_artifact.json",
                "artifacts/status/cross_surface_consistency_contract.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-CONTRACT-GENERATE-CROSS-SURFACE-CONSISTENCY-LAW-REPORTS",
        }),
    ]
}
