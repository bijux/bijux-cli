#![forbid(unsafe_code)]

/// Registry schema version.
pub const REGISTRY_VERSION: &str = "1";

/// Reserved namespaces that plugins cannot claim.
pub const RESERVED_NAMESPACES: &[&str] =
    &["cli", "dev", "help", "version", "doctor", "repl", "plugins", "completion", "inspect"];

/// Reserved namespaces currently owned by bijux-cli core command graph.
pub const CORE_NAMESPACES: &[&str] = &["cli", "dev"];

/// Reserved namespaces for future official Bijux product mounts.
pub const FUTURE_PRODUCT_NAMESPACES: &[&str] = &["atlas", "cloud", "ops", "security"];
