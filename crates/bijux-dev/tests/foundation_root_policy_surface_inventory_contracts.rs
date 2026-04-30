use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct WorkspaceProductMapContract {
    products: Vec<WorkspaceProduct>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceProduct {
    #[serde(rename = "crate")]
    crate_name: String,
}

#[derive(Debug, Deserialize)]
struct RootPolicySurfaceContract {
    schema_version: String,
    policy_files: Vec<RootPolicyFile>,
}

#[derive(Debug, Deserialize)]
struct RootPolicyFile {
    path: String,
    gates_behavior: String,
    owning_crate: String,
    enforced_by: Vec<String>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> T {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|err| panic!("invalid json {}: {err}", path.display()))
}

fn read_workspace_product_crates() -> BTreeSet<String> {
    let path = repo_root().join("contracts/foundation/workspace_product_map.v1.json");
    let contract: WorkspaceProductMapContract = read_json(&path);
    contract
        .products
        .into_iter()
        .map(|product| product.crate_name)
        .collect()
}

fn read_root_policy_surface_contract() -> RootPolicySurfaceContract {
    let path = repo_root().join("contracts/foundation/root_policy_surface_inventory.v1.json");
    read_json(&path)
}

fn collect_policy_files_under(root: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }

            let include = path
                .extension()
                .is_some_and(|ext| ext == "json")
                && (path.starts_with(repo_root().join("contracts"))
                    || path.starts_with(repo_root().join("configs/status")));
            if !include {
                continue;
            }

            let relative = path
                .strip_prefix(repo_root())
                .expect("policy path must stay under repo root")
                .to_string_lossy()
                .replace('\\', "/");
            out.insert(relative);
        }
    }
    out
}

#[test]
fn root_policy_surface_contract_schema_is_current() {
    let contract = read_root_policy_surface_contract();
    assert_eq!(
        contract.schema_version,
        "foundation-root-policy-surface-inventory/v1"
    );
}

#[test]
fn root_policy_surface_inventory_covers_all_contract_and_status_policy_files() {
    let contract = read_root_policy_surface_contract();
    let listed = contract
        .policy_files
        .iter()
        .map(|entry| entry.path.clone())
        .collect::<BTreeSet<_>>();

    let observed_contract_files = collect_policy_files_under(&repo_root().join("contracts"));
    let observed_status_files = collect_policy_files_under(&repo_root().join("configs/status"));
    let observed = observed_contract_files
        .union(&observed_status_files)
        .cloned()
        .collect::<BTreeSet<_>>();

    assert_eq!(
        listed, observed,
        "root policy surface inventory drifted from contracts/ and configs/status/"
    );
}

#[test]
fn root_policy_surface_entries_have_known_owners_and_enforcement_paths() {
    let product_crates = read_workspace_product_crates();
    let contract = read_root_policy_surface_contract();

    let mut seen = BTreeSet::new();
    for entry in contract.policy_files {
        assert!(
            seen.insert(entry.path.clone()),
            "duplicate root policy path entry: {}",
            entry.path
        );
        assert!(
            !entry.gates_behavior.trim().is_empty(),
            "gates_behavior must be non-empty: {}",
            entry.path
        );
        assert!(
            product_crates.contains(&entry.owning_crate),
            "unknown owning crate {} for {}",
            entry.owning_crate,
            entry.path
        );

        let policy_path = repo_root().join(&entry.path);
        assert!(
            policy_path.is_file(),
            "policy file does not exist: {}",
            policy_path.display()
        );

        assert!(
            !entry.enforced_by.is_empty(),
            "policy file must declare at least one enforcement path: {}",
            entry.path
        );
        for enforcement in entry.enforced_by {
            let enforcement_path = repo_root().join(&enforcement);
            assert!(
                enforcement_path.exists(),
                "enforcement path does not exist for {}: {}",
                entry.path,
                enforcement_path.display()
            );
        }
    }
}
