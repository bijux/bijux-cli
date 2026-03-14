#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::contracts::{known_bijux_tool_namespaces, official_product_namespaces};

/// Registry schema version.
pub const REGISTRY_VERSION: &str = "1";

/// Reserved namespaces that plugins cannot claim.
pub const RESERVED_NAMESPACES: &[&str] =
    &[
        "cli",
        "dev",
        "help",
        "version",
        "doctor",
        "repl",
        "plugins",
        "completion",
        "inspect",
    ];

/// Reserved namespaces currently owned by bijux-cli core command graph.
pub const CORE_NAMESPACES: &[&str] = &["cli"];

/// Complete blocked namespace inventory for plugin namespaces and aliases.
#[must_use]
pub fn blocked_namespace_inventory(additional: &[&str]) -> Vec<String> {
    let mut blocked = BTreeSet::new();
    blocked.extend(RESERVED_NAMESPACES.iter().map(|value| (*value).to_string()));
    blocked.extend(CORE_NAMESPACES.iter().map(|value| (*value).to_string()));
    blocked.extend(known_bijux_tool_namespaces().iter().map(|value| (*value).to_string()));
    blocked.extend(official_product_namespaces().iter().map(|value| (*value).to_string()));
    blocked.extend(additional.iter().map(|value| (*value).to_string()));
    blocked.into_iter().collect()
}

/// Return true if namespace is reserved for core or compatibility behavior.
#[must_use]
pub fn is_reserved_namespace(namespace: &str, additional: &[&str]) -> bool {
    blocked_namespace_inventory(additional)
        .iter()
        .any(|blocked| blocked == namespace)
}
