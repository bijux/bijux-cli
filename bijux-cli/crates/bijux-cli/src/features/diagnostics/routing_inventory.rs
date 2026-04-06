//! Read-only route and registry inventory queries for maintainer tooling.

use serde::Serialize;

use crate::contracts::NamespaceMetadata;
use crate::routing::registry::RouteRegistry;

/// Raw route inventory exposed to maintainer control-plane consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteInventory {
    /// Canonical built-in command paths.
    pub routes: Vec<Vec<String>>,
    /// Alias rewrites represented as `(alias, canonical)` segment lists.
    pub aliases: Vec<(Vec<String>, Vec<String>)>,
}

/// Raw registry inventory exposed to maintainer control-plane consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RegistryInventory {
    /// Namespace metadata rows from the route tree.
    pub namespaces: Vec<NamespaceMetadata>,
}

/// Query raw built-in routes and aliases.
#[must_use]
pub fn route_inventory(registry: &RouteRegistry) -> RouteInventory {
    let routes = registry
        .built_in_paths()
        .into_iter()
        .map(|path| path.segments.into_iter().map(|segment| segment.0).collect())
        .collect();
    let aliases = registry
        .alias_rewrites()
        .into_iter()
        .map(|(alias, canonical)| {
            (
                alias.segments.into_iter().map(|segment| segment.0).collect(),
                canonical.segments.into_iter().map(|segment| segment.0).collect(),
            )
        })
        .collect();
    RouteInventory { routes, aliases }
}

/// Query raw namespace registry metadata.
#[must_use]
pub fn registry_inventory(registry: &RouteRegistry) -> RegistryInventory {
    RegistryInventory { namespaces: registry.route_tree() }
}
