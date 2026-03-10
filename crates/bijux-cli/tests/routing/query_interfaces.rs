#![forbid(unsafe_code)]
//! Query interface shape and determinism checks for routing-owned dev bridge data.

use bijux_cli::routing::query::contracts_schema_query;
use bijux_cli::routing::registry::RouteRegistry;

#[test]
fn contracts_schema_query_shape_is_stable() {
    let query = contracts_schema_query();
    assert_eq!(query.schema_version, "v1");
    assert_eq!(
        query.schema_ids,
        vec![
            "output-envelope-v1".to_string(),
            "error-envelope-v1".to_string(),
            "plugin-manifest-v1".to_string(),
        ]
    );
}

#[test]
fn route_and_registry_queries_are_stable_across_repeated_runs() {
    let mut registry = RouteRegistry::default();
    registry.register_plugin_namespace("community").expect("register");

    let first_routes = bijux_cli::routing::inventory::route_inventory(&registry);
    let second_routes = bijux_cli::routing::inventory::route_inventory(&registry);
    assert_eq!(first_routes, second_routes);

    let first_registry = bijux_cli::routing::inventory::registry_inventory(&registry);
    let second_registry = bijux_cli::routing::inventory::registry_inventory(&registry);
    assert_eq!(first_registry, second_registry);
}
