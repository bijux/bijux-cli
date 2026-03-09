#![forbid(unsafe_code)]
//! Regression tests for legacy Python command forms.

use bijux_cli_contracts as _;
use bijux_cli_routing::parser::parse_intent;
use bijux_cli_routing::registry::{RouteRegistry, RouteTarget};
use clap as _;
use proptest as _;
use serde as _;
use serde_json as _;
use strsim as _;
use thiserror as _;

#[test]
fn legacy_status_maps_to_cli_status_and_resolves() {
    let argv = vec!["bijux".to_string(), "status".to_string()];
    let intent = parse_intent(&argv).expect("parse should succeed");
    assert_eq!(intent.normalized_path, vec!["status"]);

    let registry = RouteRegistry::default();
    let target = registry.resolve(&intent.normalized_path).expect("should resolve builtin");
    assert_eq!(target, RouteTarget::BuiltIn);
}

#[test]
fn legacy_plugins_list_maps_to_cli_plugins_list() {
    let argv = vec!["bijux".to_string(), "plugins".to_string(), "list".to_string()];
    let intent = parse_intent(&argv).expect("parse should succeed");
    assert_eq!(intent.normalized_path, vec!["cli", "plugins", "list"]);
}

#[test]
fn legacy_dev_routes_maps_to_dev_cli_routes() {
    let argv = vec!["bijux".to_string(), "dev".to_string(), "routes".to_string()];
    let intent = parse_intent(&argv).expect("parse should succeed");
    assert_eq!(intent.normalized_path, vec!["dev", "cli", "routes"]);
}
