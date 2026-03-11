#![forbid(unsafe_code)]
//! Route-inspection JSON output coverage.

use bijux_cli::routing::registry::RouteRegistry;
use proptest as _;
use serde as _;
use serde_json as _;

use clap as _;
use schemars as _;
use semver as _;
use thiserror as _;

#[test]
fn route_tree_is_json_serializable_with_expected_fields() {
    let mut registry = RouteRegistry::default();
    registry.register_plugin_namespace("community").expect("plugin namespace should register");

    let rows = registry.route_tree();
    let value = serde_json::to_value(&rows).expect("route tree should serialize");
    let list = value.as_array().expect("serialized route tree should be an array");

    assert!(!list.is_empty());
    let first = &list[0];
    assert!(first.get("name").is_some());
    assert!(first.get("reserved").is_some());
    assert!(first.get("owner").is_some());
}

#[test]
fn built_in_paths_are_json_serializable_for_inspect_payloads() {
    let registry = RouteRegistry::default();
    let paths = registry.built_in_paths();
    let value = serde_json::to_value(&paths).expect("built-in paths should serialize");
    let list = value.as_array().expect("serialized paths should be an array");
    assert!(!list.is_empty());

    for row in list {
        let segments = row
            .get("segments")
            .and_then(serde_json::Value::as_array)
            .expect("command path should contain segments");
        assert!(!segments.is_empty());
    }
}
