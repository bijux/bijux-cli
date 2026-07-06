use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::Path;

fn workspace_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root")
}

#[test]
fn planner_contract_documents_required_sections_and_tokens() {
    let root = workspace_root();
    let contract =
        fs::read_to_string(root.join("docs/spec/PLANNER_CONTRACT.md")).expect("contract");

    for section in [
        "## Scope",
        "## Lowering pipeline",
        "## Validation Relationship",
        "## Planning diagnostics",
        "## Related tests",
        "## Versioning and change policy",
    ] {
        assert!(contract.contains(section), "planner contract missing section: {section}");
    }

    for token in [
        "parsed graph",
        "validated graph",
        "canonical graph",
        "execution plan",
        "P4021",
        "dag plan-dump",
    ] {
        assert!(contract.contains(token), "planner contract missing token: {token}");
    }
}

#[test]
fn planner_hardening_report_links_contract_and_proof_surfaces() {
    let root = workspace_root();
    let report =
        fs::read_to_string(root.join("docs/reports/foundation/PLANNER_HARDENING_REPORT.md"))
            .expect("report");

    for token in [
        "docs/spec/PLANNER_CONTRACT.md",
        "docs/spec/BATTLE_TRUST_PROPERTIES.md",
        "configs/dag/schema/execution_plan.schema.json",
        "configs/dag/policy/trust_property_test_map.json",
        "configs/dag/policy/battle_trust_properties.json",
        "crates/bijux-dag-core/tests/planner_contract.rs",
        "crates/bijux-dag-runtime/tests/planner_lowering_contracts.rs",
        "crates/bijux-dev/tests/planner_hardening_contracts.rs",
        "dag plan-dump",
        "P4021",
    ] {
        assert!(report.contains(token), "planner hardening report missing: {token}");
    }
}

#[test]
fn battle_trust_properties_document_plan_truth_contract() {
    let root = workspace_root();
    let contract =
        fs::read_to_string(root.join("docs/spec/BATTLE_TRUST_PROPERTIES.md")).expect("contract");

    for token in [
        "configs/dag/policy/battle_trust_properties.json",
        "configs/dag/policy/trust_property_test_map.json",
        "tp_plan_truth",
        "P4021",
        "docs/spec/PLANNER_CONTRACT.md",
        "crates/bijux-dev/tests/battle_suite_concentration_contracts.rs",
    ] {
        assert!(contract.contains(token), "battle trust contract missing: {token}");
    }
}
