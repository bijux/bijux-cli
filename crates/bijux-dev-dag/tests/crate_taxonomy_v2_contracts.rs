use bijux_dag_testkit as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use tempfile as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_json(path: &str) -> Value {
    let payload = fs::read_to_string(repo_root().join(path)).expect("read json file");
    serde_json::from_str(&payload).expect("parse json file")
}

#[test]
fn crate_taxonomy_v2_policy_matches_workspace_metadata() {
    let policy = read_json("configs/policy/crate_taxonomy_v2.json");
    let policy_crates: BTreeSet<String> = policy["workspace_crates"]
        .as_array()
        .expect("workspace_crates array")
        .iter()
        .map(|entry| entry["name"].as_str().expect("crate name").to_string())
        .collect();

    let metadata_payload = std::process::Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .output()
        .expect("run cargo metadata");
    assert!(
        metadata_payload.status.success(),
        "cargo metadata must succeed"
    );
    let metadata: Value = serde_json::from_slice(&metadata_payload.stdout).expect("parse metadata");
    let workspace_crates: BTreeSet<String> = metadata["packages"]
        .as_array()
        .expect("packages array")
        .iter()
        .filter_map(|pkg| pkg["name"].as_str())
        .filter(|name| name.starts_with("bijux-"))
        .map(|name| name.to_string())
        .collect();

    assert_eq!(
        workspace_crates, policy_crates,
        "workspace crate set must match taxonomy policy (new crates are blocked while taxonomy is frozen)"
    );
}

#[test]
fn crate_taxonomy_v2_allowed_edges_cover_workspace_edges() {
    let policy = read_json("configs/policy/crate_taxonomy_v2.json");
    let allowed_edges: BTreeSet<String> = policy["allowed_workspace_edges"]
        .as_array()
        .expect("allowed_workspace_edges array")
        .iter()
        .map(|edge| edge.as_str().expect("edge string").to_string())
        .collect();

    let metadata_payload = std::process::Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .output()
        .expect("run cargo metadata");
    assert!(
        metadata_payload.status.success(),
        "cargo metadata must succeed"
    );
    let metadata: Value = serde_json::from_slice(&metadata_payload.stdout).expect("parse metadata");

    let workspace: BTreeSet<String> = metadata["packages"]
        .as_array()
        .expect("packages array")
        .iter()
        .filter_map(|pkg| pkg["name"].as_str())
        .filter(|name| name.starts_with("bijux-"))
        .map(ToString::to_string)
        .collect();

    let mut observed_edges = BTreeSet::new();
    for pkg in metadata["packages"].as_array().expect("packages array") {
        let from = pkg["name"].as_str().expect("package name");
        if !workspace.contains(from) {
            continue;
        }
        for dep in pkg["dependencies"].as_array().expect("dependencies array") {
            let to = dep["name"].as_str().expect("dependency name");
            if workspace.contains(to) {
                observed_edges.insert(format!("{from}->{to}"));
            }
        }
    }

    for edge in observed_edges {
        assert!(
            allowed_edges.contains(&edge),
            "workspace dependency edge is not allowed by crate taxonomy v2 policy: {edge}"
        );
    }
}

#[test]
fn crate_taxonomy_v2_docs_and_contract_files_are_present_and_named() {
    let policy = read_json("configs/policy/crate_taxonomy_v2.json");
    for entry in policy["workspace_crates"]
        .as_array()
        .expect("workspace_crates array")
    {
        let name = entry["name"].as_str().expect("crate name");
        let path = entry["path"].as_str().expect("crate path");
        let responsibility = entry["responsibility"]
            .as_str()
            .expect("crate responsibility");
        assert!(
            !responsibility.trim().is_empty(),
            "crate responsibility must be non-empty for {name}"
        );
        let readme = repo_root().join(path).join("README.md");
        let contract = repo_root().join(path).join("CONTRACT.md");
        assert!(readme.exists(), "missing README.md for {name}");
        assert!(contract.exists(), "missing CONTRACT.md for {name}");
        let readme_text = fs::read_to_string(readme).expect("read README");
        let contract_text = fs::read_to_string(contract).expect("read CONTRACT");
        assert!(
            readme_text.contains(name),
            "README should contain crate name for {name}"
        );
        assert!(
            contract_text.contains(name),
            "CONTRACT should contain crate name for {name}"
        );
        assert!(
            readme_text.contains(&format!("Responsibility: {responsibility}")),
            "README responsibility drift detected for {name}"
        );
        assert!(
            contract_text.contains(&format!("Responsibility: {responsibility}")),
            "CONTRACT responsibility drift detected for {name}"
        );
    }

    let taxonomy_doc = repo_root().join("docs/spec/CRATE_TAXONOMY_v2.md");
    assert!(taxonomy_doc.exists(), "missing CRATE_TAXONOMY_v2.md");
}

#[test]
fn crate_responsibility_and_taxonomy_docs_cover_all_workspace_crates() {
    let policy = read_json("configs/policy/crate_taxonomy_v2.json");
    let names: Vec<String> = policy["workspace_crates"]
        .as_array()
        .expect("workspace_crates array")
        .iter()
        .map(|entry| entry["name"].as_str().expect("crate name").to_string())
        .collect();
    let taxonomy_doc =
        fs::read_to_string(repo_root().join("docs/spec/CRATE_TAXONOMY_v2.md")).expect("read doc");
    let responsibility_doc =
        fs::read_to_string(repo_root().join("docs/spec/CRATE_RESPONSIBILITY_STATEMENTS.md"))
            .expect("read doc");
    for name in names {
        assert!(
            taxonomy_doc.contains(&format!("`{name}`")),
            "CRATE_TAXONOMY_v2 missing crate {name}"
        );
        assert!(
            responsibility_doc.contains(&format!("`{name}`")),
            "CRATE_RESPONSIBILITY_STATEMENTS missing crate {name}"
        );
    }
}

#[test]
fn crate_graph_snapshot_matches_current_workspace_graph() {
    let snapshot = read_json("docs/reports/foundation/crate_graph_snapshot.json");
    let snapshot_nodes: BTreeSet<String> = snapshot["workspace_crates"]
        .as_array()
        .expect("workspace_crates array")
        .iter()
        .map(|value| value.as_str().expect("node string").to_string())
        .collect();
    let snapshot_edges: BTreeSet<String> = snapshot["workspace_edges"]
        .as_array()
        .expect("workspace_edges array")
        .iter()
        .map(|value| value.as_str().expect("edge string").to_string())
        .collect();

    let metadata_payload = std::process::Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .output()
        .expect("run cargo metadata");
    assert!(
        metadata_payload.status.success(),
        "cargo metadata must succeed"
    );
    let metadata: Value = serde_json::from_slice(&metadata_payload.stdout).expect("parse metadata");

    let workspace_nodes: BTreeSet<String> = metadata["packages"]
        .as_array()
        .expect("packages array")
        .iter()
        .filter_map(|pkg| pkg["name"].as_str())
        .filter(|name| name.starts_with("bijux-"))
        .map(ToString::to_string)
        .collect();
    let mut workspace_edges = BTreeSet::new();
    for pkg in metadata["packages"].as_array().expect("packages array") {
        let from = pkg["name"].as_str().expect("package name");
        if !workspace_nodes.contains(from) {
            continue;
        }
        for dep in pkg["dependencies"].as_array().expect("dependencies array") {
            let to = dep["name"].as_str().expect("dependency name");
            if workspace_nodes.contains(to) {
                workspace_edges.insert(format!("{from}->{to}"));
            }
        }
    }

    assert_eq!(
        snapshot_nodes, workspace_nodes,
        "crate graph snapshot nodes drifted"
    );
    assert_eq!(
        snapshot_edges, workspace_edges,
        "crate graph snapshot edges drifted"
    );
}
