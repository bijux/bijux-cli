#![forbid(unsafe_code)]
//! Route-law consistency checks across root, cli/dev cli, and plugin dispatch.

use bijux_cli_contracts as _;
use bijux_cli_routing::catalog::normalize_command_path;
use bijux_cli_routing::registry::{RouteRegistry, RouteTarget};
use clap as _;
use proptest as _;
use serde as _;
use serde_json as _;
use strsim as _;
use thiserror as _;

#[test]
fn root_cli_and_dev_cli_paths_follow_one_route_law() {
    let registry = RouteRegistry::default();
    let cases = [
        (vec!["status".to_string()], vec!["status".to_string()]),
        (
            vec!["cli".to_string(), "status".to_string()],
            vec!["cli".to_string(), "status".to_string()],
        ),
        (
            vec!["dev".to_string(), "cli".to_string(), "routes".to_string()],
            vec!["dev".to_string(), "cli".to_string(), "routes".to_string()],
        ),
    ];

    for (input, expected) in cases {
        let normalized = normalize_command_path(&input);
        assert_eq!(normalized, expected);
        let resolved = registry.resolve(&normalized).expect("normalized route should resolve");
        assert!(matches!(resolved, RouteTarget::BuiltIn));
    }
}

#[test]
fn plugin_namespace_dispatch_stays_predictable_with_builtin_roots() {
    let mut registry = RouteRegistry::default();
    registry.register_plugin_namespace("community").expect("plugin register");
    registry.register_plugin_namespace("atlas").expect("plugin register");

    let plugin = registry
        .resolve(&["community".to_string(), "status".to_string()])
        .expect("plugin route should resolve");
    assert!(matches!(plugin, RouteTarget::Plugin(ns) if ns == "community"));

    let builtin = registry
        .resolve(&["dev".to_string(), "cli".to_string(), "routes".to_string()])
        .expect("builtin route should resolve");
    assert!(matches!(builtin, RouteTarget::BuiltIn));
}

#[test]
fn removed_legacy_special_cases_are_unknown_while_canonical_paths_still_resolve() {
    let registry = RouteRegistry::default();

    let legacy_routes = registry.resolve(&["dev".to_string(), "routes".to_string()]);
    assert!(legacy_routes.is_err());

    let legacy_registry = registry.resolve(&["dev".to_string(), "registry".to_string()]);
    assert!(legacy_registry.is_err());

    assert!(registry.resolve(&["dev".to_string(), "cli".to_string(), "routes".to_string()]).is_ok());
    assert!(registry.resolve(&["dev".to_string(), "cli".to_string(), "registry".to_string()]).is_ok());
}
