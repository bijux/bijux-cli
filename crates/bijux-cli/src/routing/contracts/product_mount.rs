//! Official product-mount reservation contracts.

use serde::{Deserialize, Serialize};

use super::command::Namespace;

/// Canonical metadata for known Bijux tool projects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KnownBijuxTool {
    /// Canonical tool namespace used in `bijux <tool> ...`.
    pub namespace: &'static str,
    /// Runtime executable used by `bijux <tool> ...`.
    pub runtime_binary: &'static str,
    /// Control-plane executable used by `bijux dev <tool> ...`.
    pub control_binary: &'static str,
    /// Runtime package name used for install workflows.
    pub runtime_package: &'static str,
    /// Control-plane package name used for install workflows.
    pub control_package: &'static str,
    /// Canonical source repository slug under the Bijux GitHub organization.
    pub repository: &'static str,
}

/// Canonical known Bijux tools and their binary/package ownership contracts.
pub const KNOWN_BIJUX_TOOLS: &[KnownBijuxTool] = &[
    KnownBijuxTool {
        namespace: "agent",
        runtime_binary: "bijux-agent",
        control_binary: "bijux-dev-agent",
        runtime_package: "bijux-agent",
        control_package: "bijux-dev-agent",
        repository: "bijux-agent",
    },
    KnownBijuxTool {
        namespace: "atlas",
        runtime_binary: "bijux-atlas",
        control_binary: "bijux-dev-atlas",
        runtime_package: "bijux-atlas",
        control_package: "bijux-dev-atlas",
        repository: "bijux-atlas",
    },
    KnownBijuxTool {
        namespace: "dag",
        runtime_binary: "bijux-dag",
        control_binary: "bijux-dev-dag",
        runtime_package: "bijux-dag",
        control_package: "bijux-dev-dag",
        repository: "bijux-dag",
    },
    KnownBijuxTool {
        namespace: "dna",
        runtime_binary: "bijux-dna",
        control_binary: "bijux-dev-dna",
        runtime_package: "bijux-dna",
        control_package: "bijux-dev-dna",
        repository: "bijux-dna",
    },
    KnownBijuxTool {
        namespace: "gnss",
        runtime_binary: "bijux-gnss",
        control_binary: "bijux-dev-gnss",
        runtime_package: "bijux-gnss",
        control_package: "bijux-dev-gnss",
        repository: "bijux-gnss",
    },
    KnownBijuxTool {
        namespace: "rag",
        runtime_binary: "bijux-rag",
        control_binary: "bijux-dev-rag",
        runtime_package: "bijux-rag",
        control_package: "bijux-dev-rag",
        repository: "bijux-rag",
    },
    KnownBijuxTool {
        namespace: "rar",
        runtime_binary: "bijux-rar",
        control_binary: "bijux-dev-rar",
        runtime_package: "bijux-rar",
        control_package: "bijux-dev-rar",
        repository: "bijux-rar",
    },
    KnownBijuxTool {
        namespace: "vex",
        runtime_binary: "bijux-vex",
        control_binary: "bijux-dev-vex",
        runtime_package: "bijux-vex",
        control_package: "bijux-dev-vex",
        repository: "bijux-vex",
    },
];

/// Canonical reserved namespaces for known Bijux tools.
pub const KNOWN_BIJUX_TOOL_NAMESPACES: &[&str] =
    &["agent", "atlas", "dag", "dna", "gnss", "rag", "rar", "vex"];

/// Canonical reserved namespaces for official product mounts.
pub const OFFICIAL_PRODUCT_NAMESPACES: &[&str] = KNOWN_BIJUX_TOOL_NAMESPACES;

/// Resolve known tool metadata by namespace.
#[must_use]
pub fn known_bijux_tool(namespace: &str) -> Option<&'static KnownBijuxTool> {
    KNOWN_BIJUX_TOOLS.iter().find(|tool| tool.namespace == namespace)
}

/// Smallest metadata contract required for reserved product mounts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductMountMetadata {
    /// Reserved top-level namespace.
    pub namespace: Namespace,
    /// Runtime executable used by `bijux <namespace>`.
    pub runtime_binary: String,
    /// Control-plane executable used by `bijux dev <namespace>`.
    pub control_binary: String,
}
