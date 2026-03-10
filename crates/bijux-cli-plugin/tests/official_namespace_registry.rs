#![forbid(unsafe_code)]
//! Official product namespace registry alignment tests.

use std::collections::BTreeSet;

use bijux_cli_routing::OFFICIAL_PRODUCT_NAMESPACES;
use bijux_cli_plugin as _;
use semver as _;
use serde::Deserialize;
use sha2 as _;
use thiserror as _;

#[derive(Debug, Deserialize)]
struct RegistryEntry {
    namespace: String,
}

#[derive(Debug, Deserialize)]
struct RegistryFile {
    entries: Vec<RegistryEntry>,
}

#[test]
fn official_product_registry_matches_reserved_namespace_contract() {
    let registry: RegistryFile = serde_json::from_str(include_str!(
        "../../../docs/constitution/official_product_namespace_registry.json"
    ))
    .expect("registry json should parse");

    let file_set: BTreeSet<String> = registry.entries.into_iter().map(|row| row.namespace).collect();
    let contract_set: BTreeSet<String> =
        OFFICIAL_PRODUCT_NAMESPACES.iter().map(|item| (*item).to_string()).collect();

    assert_eq!(file_set, contract_set, "registry and contract reserved namespaces diverged");
}
