#![forbid(unsafe_code)]

use crate::contracts::known_bijux_tool_namespaces;

/// Registry schema version.
pub const REGISTRY_VERSION: &str = "1";

/// Reserved namespaces that plugins cannot claim.
pub const RESERVED_NAMESPACES: &[&str] =
    &["cli", "help", "version", "doctor", "repl", "plugins", "completion", "inspect"];

/// Reserved namespaces currently owned by bijux-cli core command graph.
pub const CORE_NAMESPACES: &[&str] = &["cli"];

/// Return true if namespace is reserved for core or compatibility behavior.
#[must_use]
pub fn is_reserved_namespace(namespace: &str, additional: &[&str]) -> bool {
    RESERVED_NAMESPACES.contains(&namespace)
        || CORE_NAMESPACES.contains(&namespace)
        || known_bijux_tool_namespaces().contains(&namespace)
        || additional.contains(&namespace)
}
