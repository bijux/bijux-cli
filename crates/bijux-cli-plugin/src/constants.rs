#![forbid(unsafe_code)]

use bijux_cli_contracts::OFFICIAL_PRODUCT_NAMESPACES;

/// Registry schema version.
pub const REGISTRY_VERSION: &str = "1";

/// Reserved namespaces that plugins cannot claim.
pub const RESERVED_NAMESPACES: &[&str] =
    &["cli", "dev", "help", "version", "doctor", "repl", "plugins", "completion", "inspect"];

/// Reserved namespaces currently owned by bijux-cli core command graph.
pub const CORE_NAMESPACES: &[&str] = &["cli", "dev"];

/// Reserved namespaces for future official Bijux product mounts.
pub const FUTURE_PRODUCT_NAMESPACES: &[&str] = OFFICIAL_PRODUCT_NAMESPACES;

/// Return true if namespace is reserved for core or compatibility behavior.
#[must_use]
pub fn is_reserved_namespace(namespace: &str, additional: &[&str]) -> bool {
    RESERVED_NAMESPACES.contains(&namespace)
        || CORE_NAMESPACES.contains(&namespace)
        || FUTURE_PRODUCT_NAMESPACES.contains(&namespace)
        || additional.contains(&namespace)
}
