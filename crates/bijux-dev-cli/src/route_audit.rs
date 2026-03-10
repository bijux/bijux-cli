//! Maintainer route-audit report assembly.

use bijux_cli_routing::inventory::route_inventory;
use bijux_cli_routing::registry::RouteRegistry;
use serde_json::{json, Value};

/// Builds the maintainer route-audit report envelope.
#[must_use]
pub fn build_report(registry: &RouteRegistry) -> Value {
    let inventory = route_inventory(registry);
    build_report_from_query(&inventory.routes, &inventory.aliases)
}

/// Builds the maintainer route-audit report envelope from routing query rows.
#[must_use]
pub fn build_report_from_query(
    routes: &[Vec<String>],
    aliases: &[(Vec<String>, Vec<String>)],
) -> Value {
    let routes: Vec<Value> = routes
        .iter()
        .cloned()
        .map(|segments| json!({"segments": segments, "owner": "bijux-cli", "source": "built-in"}))
        .collect();
    let aliases: Vec<Value> = aliases
        .iter()
        .cloned()
        .map(|(alias, canonical)| {
            json!({"alias": alias, "canonical": canonical, "source": "compatibility-alias"})
        })
        .collect();
    let summary = json!({
        "route_count": routes.len(),
        "alias_count": aliases.len(),
    });

    json!({
        "routes": routes,
        "aliases": aliases,
        "summary": summary,
    })
}

#[cfg(test)]
mod tests {
    use super::build_report;
    use bijux_cli_routing::registry::RouteRegistry;

    #[test]
    fn route_audit_report_shape_is_stable() {
        let report = build_report(&RouteRegistry::default());
        assert!(report.get("routes").is_some());
        assert!(report.get("aliases").is_some());
        assert!(report.get("summary").is_some());
    }
}
