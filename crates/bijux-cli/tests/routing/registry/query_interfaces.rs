#![forbid(unsafe_code)]
//! Query interface shape and determinism checks for routing-owned dev bridge data.

use bijux_cli::api::diagnostics::{registry_inventory, route_inventory};
use bijux_cli::api::routing::registry::RouteRegistry;
use bijux_cli::contracts::contracts_schema_query;

#[test]
fn contracts_schema_query_shape_is_stable() {
    let query = contracts_schema_query();
    assert_eq!(query.schema_version, "v2");
    assert_eq!(
        query.schema_ids,
        vec![
            "output-envelope-v1".to_string(),
            "error-envelope-v1".to_string(),
            "plugin-manifest-v2".to_string(),
        ]
    );
}

#[test]
fn route_and_registry_queries_are_stable_across_repeated_runs() {
    let mut registry = RouteRegistry::default();
    registry.register_plugin_namespace("community").expect("register");

    let first_routes = route_inventory(&registry);
    let second_routes = route_inventory(&registry);
    assert_eq!(first_routes, second_routes);

    let first_registry = registry_inventory(&registry);
    let second_registry = registry_inventory(&registry);
    assert_eq!(first_registry, second_registry);
}
