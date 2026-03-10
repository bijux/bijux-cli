//! Maintainer route inventory report assembly.

use bijux_cli_routing::inventory::route_inventory;
use bijux_cli_routing::registry::RouteRegistry;
use serde_json::{json, Value};

use crate::ReportContext;

/// Builds the maintainer route inventory report envelope.
#[must_use]
pub fn build_report(registry: &RouteRegistry, _context: &ReportContext) -> Value {
    let inventory = route_inventory(registry);
    let routes: Vec<Value> = inventory
        .routes
        .into_iter()
        .map(|segments| json!({"segments": segments, "owner": "bijux-cli", "source": "built-in"}))
        .collect();
    let aliases: Vec<Value> = inventory
        .aliases
        .into_iter()
        .map(|(alias, canonical)| {
            json!({"alias": alias, "canonical": canonical, "source": "compatibility-alias"})
        })
        .collect();

    json!({ "routes": routes, "aliases": aliases })
}

#[cfg(test)]
mod tests {
    use super::build_report;
    use crate::ReportContext;
    use bijux_cli_routing::registry::RouteRegistry;

    #[test]
    fn routes_report_shape_is_stable() {
        let mut registry = RouteRegistry::default();
        registry.register_plugin_namespace("community").expect("register");
        let context =
            ReportContext { generated_at: String::new(), data_source: "routing".to_string() };

        let report = build_report(&registry, &context);
        assert!(report.get("routes").is_some(), "routes field must exist");
        assert!(report.get("aliases").is_some(), "aliases field must exist");
    }
}
