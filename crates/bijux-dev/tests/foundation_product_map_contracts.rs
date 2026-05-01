use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct ProductMap {
    schema_version: String,
    owner: String,
    products: Vec<ProductEntry>,
}

#[derive(Debug, Deserialize)]
struct ProductEntry {
    #[serde(rename = "crate")]
    crate_name: String,
    lane: String,
    owns: Vec<String>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_product_map() -> ProductMap {
    let path = repo_root().join("contracts/foundation/workspace_product_map.v1.json");
    let raw = fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!("failed to read workspace product map contract {}: {err}", path.display())
    });
    serde_json::from_str(&raw).expect("workspace product map contract must be valid JSON")
}

fn workspace_crates_from_metadata() -> BTreeSet<String> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(repo_root())
        .output()
        .expect("run cargo metadata");
    assert!(output.status.success(), "cargo metadata failed");
    let payload: Value = serde_json::from_slice(&output.stdout).expect("parse metadata json");
    payload["packages"]
        .as_array()
        .expect("packages array")
        .iter()
        .filter_map(|pkg| pkg["name"].as_str())
        .filter(|name| name.starts_with("bijux-"))
        .map(ToString::to_string)
        .collect()
}

#[test]
fn workspace_product_map_contract_covers_every_workspace_product_crate() {
    let map = read_product_map();
    assert_eq!(map.schema_version, "foundation-workspace-product-map/v1");
    assert_eq!(map.owner, "bijux-core");

    let declared: BTreeSet<String> =
        map.products.iter().map(|entry| entry.crate_name.clone()).collect();
    let observed = workspace_crates_from_metadata();

    assert_eq!(
        declared, observed,
        "workspace product map must list each workspace product crate exactly once"
    );
}

#[test]
fn workspace_product_map_entries_define_lane_and_owned_responsibilities() {
    let map = read_product_map();

    for entry in map.products {
        assert!(
            matches!(entry.lane.as_str(), "released-product" | "maintainer-control-plane"),
            "unknown lane `{}` for crate `{}`",
            entry.lane,
            entry.crate_name
        );
        assert!(
            !entry.owns.is_empty(),
            "crate `{}` must declare at least one owned responsibility",
            entry.crate_name
        );
        assert!(
            entry.owns.iter().all(|item| !item.trim().is_empty()),
            "crate `{}` has an empty ownership entry",
            entry.crate_name
        );
    }
}
