//! Official product-mount reservation contracts.

use serde::{Deserialize, Serialize};

use super::command::Namespace;

/// Canonical metadata for known Bijux tool projects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KnownBijuxTool {
    /// Canonical tool namespace used in `bijux <tool> ...`.
    pub namespace: &'static str,
}

impl KnownBijuxTool {
    /// Runtime executable used by `bijux <tool> ...`.
    #[must_use]
    pub fn runtime_binary(&self) -> String {
        format!("bijux-{}", self.namespace)
    }

    /// Control-plane executable used by `bijux dev <tool> ...`.
    #[must_use]
    pub fn control_binary(&self) -> String {
        format!("bijux-dev-{}", self.namespace)
    }
}

/// Canonical known Bijux tools and their binary/package ownership contracts.
pub const KNOWN_BIJUX_TOOLS: &[KnownBijuxTool] = &[
    KnownBijuxTool { namespace: "agent" },
    KnownBijuxTool { namespace: "atlas" },
    KnownBijuxTool { namespace: "dag" },
    KnownBijuxTool { namespace: "dna" },
    KnownBijuxTool { namespace: "gnss" },
    KnownBijuxTool { namespace: "rag" },
    KnownBijuxTool { namespace: "rar" },
    KnownBijuxTool { namespace: "vex" },
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
