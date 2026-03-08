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

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn advanced_semantics_inventory_and_reports_exist() {
    let root = repo_root();
    for rel in [
        "configs/policy/advanced_semantics_governance.json",
        "docs/reports/foundation/advanced_semantics_inventory.md",
        "docs/reports/foundation/advanced_semantics_no_user_path_report.md",
        "docs/reports/foundation/advanced_semantics_no_direct_tests_report.md",
        "docs/reports/foundation/advanced_semantics_no_examples_report.md",
        "docs/spec/ADVANCED_SEMANTICS_SCOPE.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing advanced semantics governance artifact: {rel}"
        );
    }
}

#[test]
fn advanced_semantics_modules_are_classified_with_allowed_categories() {
    let root = repo_root();
    let policy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/advanced_semantics_governance.json"))
            .expect("read advanced semantics policy"),
    )
    .expect("parse advanced semantics policy");

    let allowed = BTreeSet::from([
        "kernel-relevant",
        "runtime-relevant",
        "adapter-relevant",
        "speculative",
    ]);

    for entry in policy["advanced_semantics_modules"]
        .as_array()
        .expect("advanced semantics modules array")
    {
        let module = entry["module"].as_str().expect("module path");
        let category = entry["category"].as_str().expect("module category");
        assert!(
            allowed.contains(category),
            "invalid advanced semantics category for {module}: {category}"
        );
        assert!(
            root.join("crates/bijux-dag-runtime/src")
                .join(module)
                .exists(),
            "advanced semantics module path does not exist: {module}"
        );
    }
}

#[test]
fn speculative_advanced_semantics_stay_quarantined_under_expected_namespaces() {
    let root = repo_root();
    let policy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/advanced_semantics_governance.json"))
            .expect("read advanced semantics policy"),
    )
    .expect("parse advanced semantics policy");

    let prefixes: Vec<String> = policy["quarantined_prefixes"]
        .as_array()
        .expect("quarantined_prefixes array")
        .iter()
        .map(|entry| entry.as_str().expect("prefix").to_string())
        .collect();

    for entry in policy["advanced_semantics_modules"]
        .as_array()
        .expect("advanced semantics modules array")
    {
        let module = entry["module"].as_str().expect("module path");
        let category = entry["category"].as_str().expect("module category");
        if category == "speculative" {
            assert!(
                prefixes.iter().any(|prefix| module.starts_with(prefix)),
                "speculative module must be quarantined under governed namespace: {module}"
            );
        }
    }
}

#[test]
fn advanced_semantics_do_not_appear_in_graph_identity_surfaces() {
    let root = repo_root();
    let advanced_tokens = [
        "federated",
        "geo_federation",
        "ai_operator_assist",
        "workflow_product",
        "dataset_semantics",
        "cost_optimization",
    ];

    for rel in [
        "crates/bijux-dag-core/src/graph/canonical.rs",
        "crates/bijux-dag-core/src/graph/edge.rs",
        "crates/bijux-dag-core/src/graph/topology.rs",
    ] {
        let source = fs::read_to_string(root.join(rel)).expect("read graph identity source");
        for token in advanced_tokens {
            assert!(
                !source.contains(token),
                "advanced semantics token leaked into graph identity surface `{rel}`: {token}"
            );
        }
    }
}

#[test]
fn advanced_semantics_do_not_appear_in_replay_proof_surfaces() {
    let root = repo_root();
    let advanced_tokens = [
        "federated",
        "geo_federation",
        "ai_operator_assist",
        "workflow_product",
        "dataset_semantics",
        "cost_optimization",
    ];

    for rel in [
        "crates/bijux-dag-runtime/src/replay/mod.rs",
        "crates/bijux-dag-runtime/src/replay/verifier.rs",
        "crates/bijux-dag-runtime/src/replay/diff.rs",
        "crates/bijux-dag-runtime/src/cache/proof.rs",
    ] {
        let source = fs::read_to_string(root.join(rel)).expect("read replay/proof source");
        for token in advanced_tokens {
            assert!(
                !source.contains(token),
                "advanced semantics token leaked into replay/proof surface `{rel}`: {token}"
            );
        }
    }
}

#[test]
fn advanced_semantics_do_not_leak_into_default_cli_surfaces() {
    let root = repo_root();
    let cli = fs::read_to_string(root.join("crates/bijux-dev-dag/src/commands/cli.rs"))
        .expect("read dev-dag cli source");
    for leaked in [
        "federated",
        "geo-federation",
        "geo federation",
        "ai-assist",
        "workflow product",
        "dataset semantics",
        "cost optimization",
    ] {
        assert!(
            !cli.contains(leaked),
            "advanced semantics leaked into default CLI surface: {leaked}"
        );
    }
}
