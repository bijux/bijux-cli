#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn rows() -> Vec<Value> {
    vec![
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-CONFIG-READ-SURFACE-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/config_read_coverage_artifact.json",
                "artifacts/status/config_read_domain_contract.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-CONFIG-READ-SURFACE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-CONFIG-MUTATION-SURFACE-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/config_mutation_coverage_artifact.json",
                "artifacts/status/config_mutation_domain_contract.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-CONFIG-MUTATION-SURFACE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-CONFIG-SOURCE-SURFACE-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/config_source_parity_artifact.json",
                "artifacts/status/config_source_drift_artifact.json",
                "artifacts/status/config_source_precedence_contract.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-CONFIG-SOURCE-SURFACE-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-PYTHON-BRIDGE-EXECUTION-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/python_bridge_execution_artifact.json",
                "artifacts/status/python_bridge_drift_artifact.json",
                "artifacts/status/python_bridge_execution_contract.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-PYTHON-BRIDGE-EXECUTION-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-PYTHON-BRIDGE-CONVERSION-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/bridge_conversion_artifact.json",
                "artifacts/status/bridge_exception_mapping_artifact.json",
                "artifacts/status/bridge_envelope_integrity_artifact.json",
                "artifacts/status/bridge_conversion_drift_artifact.json",
                "artifacts/status/bridge_conversion_contract.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-PYTHON-BRIDGE-CONVERSION-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-REPL-COMPLETION-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/repl_completion_artifact.json",
                "artifacts/status/repl_completion_ordering_artifact.json",
                "artifacts/status/repl_completion_drift_artifact.json",
                "artifacts/status/repl_completion_contract.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-REPL-COMPLETION-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-REPL-BEHAVIOR-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/repl_only_behaviors.json",
                "artifacts/parity/repl_cli_output_diff.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-REPL-BEHAVIOR-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-REPL-EXECUTION-LAW-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/repl_shared_law_artifact.json",
                "artifacts/status/repl_cli_diff_artifact.json",
                "artifacts/status/repl_shared_law_drift_artifact.json",
                "artifacts/status/repl_shared_law_contract.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-REPL-EXECUTION-LAW-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-REPL-HOSTILE-SESSION-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/repl_hostile_session_artifact.json",
                "artifacts/status/repl_recovery_artifact.json",
                "artifacts/status/repl_startup_resilience_artifact.json",
                "artifacts/status/repl_command_loop_failure_class_artifact.json",
                "artifacts/status/repl_hostile_session_contract.json",
                "artifacts/status/repl_hostile_session_drift_artifact.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-REPL-HOSTILE-SESSION-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-KERNEL-INVARIANTS-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/kernel_invariants_report.json",
                "artifacts/status/kernel_invariants_diff.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-KERNEL-INVARIANTS-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-HELP-TREE-LAW-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/help_law_artifact.json",
                "artifacts/status/command_tree_help_consistency_artifact.json",
                "artifacts/status/help_drift_artifact.json",
                "artifacts/status/help_tree_contract.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-HELP-TREE-LAW-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-DIAGNOSTICS-LAW-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/diagnostics_taxonomy.json",
                "artifacts/status/diagnostics_usefulness_review.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-DIAGNOSTICS-LAW-REPORTS",
        }),
        json!({
            "contract_id": "STATUS-CONTRACT-GENERATE-CROSS-SURFACE-REPORTS",
            "kind": "generate",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/cross_surface_equivalence_report.json",
                "artifacts/status/cross_surface_drift_report.json",
                "artifacts/status/cross_surface_duality_contract.json"
            ],
            "command": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-GENERATE-CROSS-SURFACE-REPORTS",
        }),
    ]
}
