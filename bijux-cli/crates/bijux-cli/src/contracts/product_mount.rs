//! Official product-mount reservation contracts.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use super::command::Namespace;

#[derive(Debug, Deserialize)]
struct ProductRegistryDocument {
    entries: Vec<ProductRegistryEntry>,
}

#[derive(Debug, Deserialize)]
struct ProductRegistryEntry {
    namespace: String,
    runtime_binary: String,
    control_binary: String,
    runtime_package: String,
    control_package: String,
    repository: String,
    status: String,
}

/// Canonical metadata for known Bijux tool projects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KnownBijuxTool {
    /// Canonical tool namespace used in `bijux <tool> ...`.
    pub namespace: &'static str,
    /// Runtime executable used by `bijux <tool> ...`.
    pub runtime_binary_name: &'static str,
    /// Maintainer executable used by `bijux-dev-<tool> ...`.
    pub control_binary_name: &'static str,
    /// Canonical runtime install package.
    pub runtime_package_name: &'static str,
    /// Canonical control-plane install package.
    pub control_package_name: &'static str,
    /// Canonical repository slug.
    pub repository_name: &'static str,
    /// Declared lifecycle status from the registry contract.
    pub status: &'static str,
}

impl KnownBijuxTool {
    /// Runtime executable used by `bijux <tool> ...`.
    #[must_use]
    pub fn runtime_binary(&self) -> String {
        self.runtime_binary_name.to_string()
    }

    /// Maintainer executable used by `bijux-dev-<tool> ...`.
    #[must_use]
    pub fn control_binary(&self) -> String {
        self.control_binary_name.to_string()
    }

    /// Canonical runtime package to install this product.
    #[must_use]
    pub fn runtime_package(&self) -> String {
        self.runtime_package_name.to_string()
    }

    /// Canonical control-plane package to install this product.
    #[must_use]
    pub fn control_package(&self) -> String {
        self.control_package_name.to_string()
    }

    /// Canonical repository slug for this product.
    #[must_use]
    pub fn repository(&self) -> String {
        self.repository_name.to_string()
    }
}

fn leak(raw: String) -> &'static str {
    Box::leak(raw.into_boxed_str())
}

fn load_known_bijux_tools() -> Vec<KnownBijuxTool> {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/contracts/official_product_namespace_registry.json"
    ));
    let document: ProductRegistryDocument =
        serde_json::from_str(raw).expect("official product registry must stay valid JSON");

    document
        .entries
        .into_iter()
        .map(|entry| KnownBijuxTool {
            namespace: leak(entry.namespace),
            runtime_binary_name: leak(entry.runtime_binary),
            control_binary_name: leak(entry.control_binary),
            runtime_package_name: leak(entry.runtime_package),
            control_package_name: leak(entry.control_package),
            repository_name: leak(entry.repository),
            status: leak(entry.status),
        })
        .collect()
}

fn known_bijux_tools_storage() -> &'static Vec<KnownBijuxTool> {
    static STORAGE: OnceLock<Vec<KnownBijuxTool>> = OnceLock::new();
    STORAGE.get_or_init(load_known_bijux_tools)
}

/// Canonical known Bijux tools and their binary/package ownership contracts.
#[must_use]
pub fn known_bijux_tools() -> &'static [KnownBijuxTool] {
    known_bijux_tools_storage().as_slice()
}

fn load_known_bijux_tool_namespaces() -> Vec<&'static str> {
    known_bijux_tools().iter().map(|tool| tool.namespace).collect()
}

/// Canonical reserved namespaces for known Bijux tools.
#[must_use]
pub fn known_bijux_tool_namespaces() -> &'static [&'static str] {
    static STORAGE: OnceLock<Vec<&'static str>> = OnceLock::new();
    STORAGE.get_or_init(load_known_bijux_tool_namespaces).as_slice()
}

/// Canonical reserved namespaces for official product mounts.
#[must_use]
pub fn official_product_namespaces() -> &'static [&'static str] {
    known_bijux_tool_namespaces()
}

/// Resolve known tool metadata by namespace.
#[must_use]
pub fn known_bijux_tool(namespace: &str) -> Option<&'static KnownBijuxTool> {
    known_bijux_tools().iter().find(|tool| tool.namespace == namespace)
}

/// Smallest metadata contract required for reserved product mounts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductMountMetadata {
    /// Reserved top-level namespace.
    pub namespace: Namespace,
    /// Runtime executable used by `bijux <namespace>`.
    pub runtime_binary: String,
    /// Maintainer executable used by `bijux-dev-<namespace>`.
    pub control_binary: String,
}
