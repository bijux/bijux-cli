//! Official product-mount reservation contracts.

use serde::{Deserialize, Serialize};

use super::command::Namespace;

/// Canonical reserved namespaces for official product mounts.
pub const OFFICIAL_PRODUCT_NAMESPACES: &[&str] = &["atlas", "dag"];

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
