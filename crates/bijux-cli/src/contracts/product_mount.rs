//! Official product-mount reservation contracts.

use std::sync::OnceLock;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::command::Namespace;

const RESERVED_ROOT_NAMESPACES: &[&str] = &[
    "apps",
    "cli",
    "completion",
    "config",
    "doctor",
    "help",
    "history",
    "inspect",
    "install",
    "memory",
    "plugins",
    "repl",
    "self",
    "status",
    "version",
];

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProductRegistryDocument {
    pub schema_version: String,
    pub owner: String,
    pub policy: String,
    pub entries: Vec<ProductRegistryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProductRegistryEntry {
    pub namespace: String,
    pub display_name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub runtime_binary: String,
    pub control_binary: String,
    pub runtime_package: String,
    pub control_package: String,
    pub repository: String,
    pub status: String,
    pub language: String,
    #[serde(default)]
    pub version: Option<String>,
    pub help_summary: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProductEntrypointKind {
    Binary,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProductEntrypoint {
    pub kind: ProductEntrypointKind,
    pub command: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProductHelpMetadata {
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProductMountDescriptor {
    pub namespace: Namespace,
    pub display_name: String,
    #[serde(default)]
    pub aliases: Vec<Namespace>,
    pub entrypoint: ProductEntrypoint,
    pub control_entrypoint: ProductEntrypoint,
    pub help: ProductHelpMetadata,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub version: Option<String>,
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
    /// Human-readable product display name.
    pub display_name: &'static str,
    /// Stable product aliases recognized by contract queries.
    pub aliases: &'static [&'static str],
    /// Owning implementation language.
    pub language: &'static str,
    /// Lightweight manifest version when declared.
    pub version: Option<&'static str>,
    /// Help summary for root inventory surfaces.
    pub help_summary: &'static str,
    /// Declared product capabilities.
    pub capabilities: &'static [&'static str],
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

    /// Stable runtime descriptor for root app inventory surfaces.
    #[must_use]
    pub fn descriptor(&self) -> ProductMountDescriptor {
        ProductMountDescriptor {
            namespace: Namespace(self.namespace.to_string()),
            display_name: self.display_name.to_string(),
            aliases: self.aliases.iter().map(|alias| Namespace((*alias).to_string())).collect(),
            entrypoint: ProductEntrypoint {
                kind: ProductEntrypointKind::Binary,
                command: self.runtime_binary(),
            },
            control_entrypoint: ProductEntrypoint {
                kind: ProductEntrypointKind::Binary,
                command: self.control_binary(),
            },
            help: ProductHelpMetadata { summary: self.help_summary.to_string() },
            capabilities: self.capabilities.iter().map(|value| (*value).to_string()).collect(),
            version: self.version.map(ToOwned::to_owned),
        }
    }
}

fn leak(raw: String) -> &'static str {
    Box::leak(raw.into_boxed_str())
}

fn leak_vec(raw: Vec<String>) -> &'static [&'static str] {
    Box::leak(raw.into_iter().map(leak).collect::<Vec<&'static str>>().into_boxed_slice())
}

fn validate_registry_document(document: &ProductRegistryDocument) -> Result<(), String> {
    if document.schema_version.trim() != "v1" {
        return Err(format!(
            "official product registry schema_version must be `v1`, got `{}`",
            document.schema_version
        ));
    }
    if document.owner.trim().is_empty() {
        return Err("official product registry owner cannot be empty".to_string());
    }
    if document.policy.trim().is_empty() {
        return Err("official product registry policy cannot be empty".to_string());
    }

    let mut seen_namespaces = std::collections::BTreeSet::new();
    let mut seen_aliases = std::collections::BTreeSet::new();

    for entry in &document.entries {
        let namespace = Namespace::new(&entry.namespace).map_err(|error| {
            format!("invalid official namespace `{}`: {error}", entry.namespace)
        })?;
        if RESERVED_ROOT_NAMESPACES.contains(&namespace.as_str()) {
            return Err(format!(
                "official namespace `{}` collides with reserved runtime root",
                namespace.as_str()
            ));
        }
        if !seen_namespaces.insert(namespace.as_str().to_string()) {
            return Err(format!("duplicate official namespace `{}`", namespace.as_str()));
        }
        if entry.display_name.trim().is_empty() {
            return Err(format!(
                "official namespace `{}` is missing display_name",
                namespace.as_str()
            ));
        }
        if entry.runtime_binary.trim().is_empty()
            || entry.control_binary.trim().is_empty()
            || entry.runtime_package.trim().is_empty()
            || entry.control_package.trim().is_empty()
            || entry.repository.trim().is_empty()
            || entry.status.trim().is_empty()
            || entry.language.trim().is_empty()
            || entry.help_summary.trim().is_empty()
        {
            return Err(format!(
                "official namespace `{}` has one or more empty required fields",
                namespace.as_str()
            ));
        }

        let mut capability_set = std::collections::BTreeSet::new();
        for capability in &entry.capabilities {
            let normalized = capability.trim().to_ascii_lowercase().replace(' ', "_");
            if normalized.is_empty() {
                return Err(format!(
                    "official namespace `{}` has an empty capability entry",
                    namespace.as_str()
                ));
            }
            if !capability_set.insert(normalized.clone()) {
                return Err(format!(
                    "official namespace `{}` declares duplicate capability `{}`",
                    namespace.as_str(),
                    normalized
                ));
            }
        }

        let mut local_aliases = std::collections::BTreeSet::new();
        for alias in &entry.aliases {
            let normalized = Namespace::new(alias)
                .map_err(|error| format!("invalid alias `{alias}`: {error}"))?;
            if normalized.as_str() == namespace.as_str() {
                return Err(format!(
                    "official namespace `{}` cannot repeat itself as an alias",
                    namespace.as_str()
                ));
            }
            if RESERVED_ROOT_NAMESPACES.contains(&normalized.as_str()) {
                return Err(format!(
                    "official alias `{}` for `{}` collides with reserved runtime root",
                    normalized.as_str(),
                    namespace.as_str()
                ));
            }
            if seen_namespaces.contains(normalized.as_str()) {
                return Err(format!(
                    "official alias `{}` for `{}` collides with another namespace",
                    normalized.as_str(),
                    namespace.as_str()
                ));
            }
            if !local_aliases.insert(normalized.as_str().to_string()) {
                return Err(format!(
                    "official namespace `{}` declares duplicate alias `{}`",
                    namespace.as_str(),
                    normalized.as_str()
                ));
            }
            if !seen_aliases.insert(normalized.as_str().to_string()) {
                return Err(format!(
                    "official alias `{}` is declared by multiple products",
                    normalized.as_str()
                ));
            }
        }
    }

    Ok(())
}

fn load_known_bijux_tools() -> Vec<KnownBijuxTool> {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/contracts/official_product_namespace_registry.json"
    ));
    let document: ProductRegistryDocument =
        serde_json::from_str(raw).expect("official product registry must stay valid JSON");
    validate_registry_document(&document)
        .expect("official product registry must satisfy namespace, alias, and field contracts");

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
            display_name: leak(entry.display_name),
            aliases: leak_vec(entry.aliases),
            language: leak(entry.language),
            version: entry.version.map(leak),
            help_summary: leak(entry.help_summary),
            capabilities: leak_vec(entry.capabilities),
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

/// Resolve known tool metadata by namespace or declared alias.
#[must_use]
pub fn known_bijux_tool_by_query(query: &str) -> Option<&'static KnownBijuxTool> {
    let normalized = Namespace::normalize(query);
    known_bijux_tools().iter().find(|tool| {
        tool.namespace == normalized
            || tool.aliases.iter().any(|alias| *alias == normalized.as_str())
    })
}

/// Smallest metadata contract required for reserved product mounts.
pub type ProductMountMetadata = ProductMountDescriptor;
