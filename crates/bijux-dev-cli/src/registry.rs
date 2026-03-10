//! Maintainer registry report assembly.

use std::collections::BTreeMap;

use bijux_cli_routing::inventory::registry_inventory;
use bijux_cli_routing::registry::RouteRegistry;
use serde_json::{json, Value};

use crate::ReportContext;

/// Builds the maintainer registry report envelope.
#[must_use]
pub fn build_report(registry: &RouteRegistry, _context: &ReportContext) -> Value {
    let inventory = registry_inventory(registry);
    let mut ownership: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for row in &inventory.namespaces {
        ownership.entry(row.owner.clone()).or_default().push(row.name.0.clone());
    }

    json!({
        "registry": inventory.namespaces,
        "ownership": ownership,
        "precedence": ["reserved", "plugin"],
    })
}

#[cfg(test)]
mod tests {
    use super::build_report;
    use crate::ReportContext;
    use bijux_cli_routing::registry::RouteRegistry;

    #[test]
    fn registry_report_shape_is_stable() {
        let mut registry = RouteRegistry::default();
        registry.register_plugin_namespace("community").expect("register");
        let context =
            ReportContext { generated_at: String::new(), data_source: "routing".to_string() };

        let report = build_report(&registry, &context);
        assert!(report.get("registry").is_some(), "registry field must exist");
        assert!(report.get("ownership").is_some(), "ownership field must exist");
        assert!(report.get("precedence").is_some(), "precedence field must exist");
    }
}
