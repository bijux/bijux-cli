use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use tempfile as _;

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn policy() -> serde_json::Value {
    serde_json::from_str(
        &fs::read_to_string(root().join("configs/policy/release_gate_governance.json"))
            .expect("read release gate governance policy"),
    )
    .expect("parse release gate governance policy")
}

#[test]
fn release_gate_policy_declares_owner_purpose_failure_action_rule() {
    let policy = policy();
    assert_eq!(
        policy["governance_rules"]["new_gate_requires_owner_purpose_failure_action_docs"],
        true
    );
    assert_eq!(
        policy["governance_rules"]["docs_and_workflows_must_remain_aligned"],
        true
    );

    for gate in policy["gates"].as_array().expect("gates array") {
        for field in [
            "gate_id",
            "make_target",
            "owner",
            "purpose",
            "failure_action",
            "docs_page",
        ] {
            let value = gate[field].as_str().expect("required gate field");
            assert!(
                !value.trim().is_empty(),
                "gate field {field} must not be empty"
            );
        }
        let docs_page = gate["docs_page"].as_str().expect("docs_page");
        assert!(
            root().join(docs_page).exists(),
            "missing gate docs page: {docs_page}"
        );
    }
}

#[test]
fn release_gate_inventory_reports_exist() {
    for rel in [
        "docs/reports/foundation/release_gate_inventory_report.md",
        "docs/reports/foundation/release_gate_blocking_vs_advisory_report.md",
        "docs/reports/foundation/release_gate_overlap_report.md",
        "docs/reports/foundation/release_gate_redundancy_decisions_report.md",
        "docs/reports/foundation/release_gate_troubleshooting_guide.md",
        "docs/reports/foundation/release_gate_outputs_missing_human_summary_report.md",
        "docs/reports/foundation/release_gate_human_summaries.md",
        "docs/reports/foundation/release_gate_machine_summaries.json",
        "docs/reports/foundation/release_gate_docs_missing_report.md",
        "docs/reports/foundation/stale_gate_docs_report.md",
        "docs/reports/foundation/release_gate_suite_claim_matrix.md",
        "docs/reports/foundation/release_gate_owner_escalation_matrix.md",
        "docs/reports/foundation/release_gate_runtime_budget_trend_report.md",
        "docs/reports/foundation/release_review_pack.md",
        "docs/reference/RELEASE_GATE_CONTRIBUTOR_QUICKSTART.md",
        "docs/reference/RELEASE_GATE_MAINTAINER_TRIAGE_QUICKSTART.md",
    ] {
        assert!(
            root().join(rel).exists(),
            "missing release gate clarity artifact: {rel}"
        );
    }
}

#[test]
fn release_gate_machine_and_human_summaries_cover_major_gates() {
    let policy = policy();
    let governed: BTreeSet<String> = policy["gates"]
        .as_array()
        .expect("gates array")
        .iter()
        .map(|g| g["gate_id"].as_str().expect("gate_id").to_string())
        .collect();

    let human =
        fs::read_to_string(root().join("docs/reports/foundation/release_gate_human_summaries.md"))
            .expect("read human summaries");
    let machine: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            root().join("docs/reports/foundation/release_gate_machine_summaries.json"),
        )
        .expect("read machine summaries"),
    )
    .expect("parse machine summaries");

    let machine_gates: BTreeSet<String> = machine["gates"]
        .as_array()
        .expect("machine gates array")
        .iter()
        .map(|g| g["gate"].as_str().expect("machine gate").to_string())
        .collect();
    assert_eq!(
        machine_gates, governed,
        "machine summaries must cover governed gates"
    );

    for gate in governed {
        assert!(
            human.contains(&format!("`{}`", gate)),
            "human summary missing gate {gate}"
        );
    }
}

#[test]
fn release_gate_policy_aligns_with_make_targets() {
    let make_root = fs::read_to_string(root().join("make/root.mk")).expect("read make/root.mk");
    let make_cargo = fs::read_to_string(root().join("make/cargo.mk")).expect("read make/cargo.mk");
    let make_evidence =
        fs::read_to_string(root().join("make/evidence.mk")).expect("read make/evidence.mk");

    let make_all = format!("{}\n{}\n{}", make_root, make_cargo, make_evidence);
    for gate in policy()["gates"].as_array().expect("gates array") {
        let target = gate["make_target"].as_str().expect("make_target");
        assert!(
            make_all.contains(&format!("{}:", target)),
            "governed gate target missing in make files: {target}"
        );
    }
}

#[test]
fn release_gate_runtime_budgets_cover_each_governed_gate() {
    let policy = policy();
    let budgets = policy["runtime_budget_targets_minutes"]
        .as_object()
        .expect("runtime_budget_targets_minutes object");
    for gate in policy["gates"].as_array().expect("gates array") {
        let gate_id = gate["gate_id"].as_str().expect("gate_id");
        let budget = budgets
            .get(gate_id)
            .and_then(|v| v.as_i64())
            .expect("gate budget minutes");
        assert!(budget > 0, "gate budget must be positive: {gate_id}");
    }
}
