//! Route diagnostics reports shared by dev CLI command handlers.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::registry::RouteRegistry;
use crate::NamespaceMetadata;

/// Built-in command route entry exposed by route diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteEntry {
    /// Canonical command path segments.
    pub segments: Vec<String>,
    /// Crate or runtime owner for the route.
    pub owner: String,
    /// Origin kind for the route declaration.
    pub source: String,
}

/// Compatibility alias entry exposed by route diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AliasEntry {
    /// Alias path segments accepted by the router.
    pub alias: Vec<String>,
    /// Canonical path segments the alias resolves to.
    pub canonical: Vec<String>,
    /// Origin kind for the alias declaration.
    pub source: String,
}

/// Full route listing with alias rewrites.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RoutesReport {
    /// Built-in command paths.
    pub routes: Vec<RouteEntry>,
    /// Compatibility aliases currently live in routing.
    pub aliases: Vec<AliasEntry>,
}

/// Summary counters for route and alias inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteAuditSummary {
    /// Total number of built-in routes.
    pub route_count: usize,
    /// Total number of compatibility aliases.
    pub alias_count: usize,
}

/// Route inventory plus summary counters for audit views.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteAuditReport {
    /// Built-in command paths.
    pub routes: Vec<RouteEntry>,
    /// Compatibility aliases currently live in routing.
    pub aliases: Vec<AliasEntry>,
    /// Aggregate route and alias counts.
    pub summary: RouteAuditSummary,
}

/// Namespace ownership and precedence view for registry diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RegistryReport {
    /// Raw namespace metadata rows from the route tree.
    pub registry: Vec<NamespaceMetadata>,
    /// Grouped namespace names by owning actor.
    pub ownership: BTreeMap<String, Vec<String>>,
    /// Resolution precedence used by dispatch.
    pub precedence: Vec<String>,
}

fn route_entries(registry: &RouteRegistry) -> Vec<RouteEntry> {
    registry
        .built_in_paths()
        .into_iter()
        .map(|path| RouteEntry {
            segments: path.segments.into_iter().map(|s| s.0).collect(),
            owner: "bijux-cli".to_string(),
            source: "built-in".to_string(),
        })
        .collect()
}

fn alias_entries(registry: &RouteRegistry) -> Vec<AliasEntry> {
    registry
        .alias_rewrites()
        .into_iter()
        .map(|(alias, canonical)| AliasEntry {
            alias: alias.segments.into_iter().map(|s| s.0).collect(),
            canonical: canonical.segments.into_iter().map(|s| s.0).collect(),
            source: "compatibility-alias".to_string(),
        })
        .collect()
}

#[must_use]
/// Build a complete route + alias listing for `dev cli routes`.
pub fn routes_report(registry: &RouteRegistry) -> RoutesReport {
    RoutesReport { routes: route_entries(registry), aliases: alias_entries(registry) }
}

#[must_use]
/// Build a route audit report with summary counters for `dev cli route-audit`.
pub fn route_audit_report(registry: &RouteRegistry) -> RouteAuditReport {
    let routes = route_entries(registry);
    let aliases = alias_entries(registry);
    let summary = RouteAuditSummary { route_count: routes.len(), alias_count: aliases.len() };
    RouteAuditReport { routes, aliases, summary }
}

#[must_use]
/// Build a namespace ownership report for `dev cli registry`.
pub fn registry_report(registry: &RouteRegistry) -> RegistryReport {
    let registry_rows = registry.route_tree();
    let mut ownership: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for row in &registry_rows {
        ownership.entry(row.owner.clone()).or_default().push(row.name.0.clone());
    }
    RegistryReport {
        registry: registry_rows,
        ownership,
        precedence: vec!["reserved".to_string(), "plugin".to_string()],
    }
}
