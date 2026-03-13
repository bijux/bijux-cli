#![forbid(unsafe_code)]
//! Route-law consistency checks across root, cli, and plugin dispatch.

use bijux_cli::api::routing::catalog::{is_known_route, normalize_command_path};
use bijux_cli::api::routing::registry::{RouteError, RouteRegistry, RouteTarget};
use bijux_cli::contracts::OFFICIAL_PRODUCT_NAMESPACES;
use proptest as _;
use serde as _;
use serde_json as _;

use clap as _;
use schemars as _;
use semver as _;
use thiserror as _;

#[test]
fn root_and_cli_paths_follow_one_route_law() {
    let registry = RouteRegistry::default();
    let cases = [
        (vec!["status".to_string()], vec!["status".to_string()]),
        (
            vec!["cli".to_string(), "status".to_string()],
            vec!["cli".to_string(), "status".to_string()],
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

    let plugin = registry
        .resolve(&["community".to_string(), "status".to_string()])
        .expect("plugin route should resolve");
    assert!(matches!(plugin, RouteTarget::Plugin(ns) if ns == "community"));

    let external = registry
        .resolve(&["atlas".to_string(), "status".to_string()])
        .expect_err("external product routes must stay outside runtime registry");
    assert!(matches!(external, RouteError::Unknown(_)));
}

#[test]
fn route_tree_marks_official_product_namespaces_as_reserved() {
    let registry = RouteRegistry::default();
    let tree = registry.route_tree();
    for namespace in OFFICIAL_PRODUCT_NAMESPACES {
        assert!(tree.iter().any(|item| {
            item.name.0 == *namespace
                && item.reserved
                && item.owner == format!("bijux-{namespace}")
        }));
    }
}

#[test]
fn official_product_namespace_registry_drives_routing_rejections() {
    let mut registry = RouteRegistry::default();
    for namespace in OFFICIAL_PRODUCT_NAMESPACES {
        let err = registry
            .register_plugin_namespace(namespace)
            .expect_err("official product namespace must stay reserved");
        assert!(matches!(err, RouteError::Reserved(_)));
    }
}

#[test]
fn maintainer_namespace_is_reserved_but_not_known_as_runtime_route() {
    let registry = RouteRegistry::default();

    let maintainer = registry.resolve(&["dev".to_string(), "routes".to_string()]);
    assert!(maintainer.is_err());
    assert!(!is_known_route(&["dev".to_string(), "routes".to_string()]));
}
