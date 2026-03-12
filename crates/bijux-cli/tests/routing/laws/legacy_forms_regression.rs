#![forbid(unsafe_code)]
//! Regression tests for legacy Python command forms.

use bijux_cli::api::routing::catalog::normalize_command_path;
use bijux_cli::api::routing::parser::parse_intent;
use bijux_cli::api::routing::registry::{RouteRegistry, RouteTarget};
use proptest as _;
use serde as _;
use serde_json as _;

use clap as _;
use schemars as _;
use semver as _;
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
fn dev_routes_are_preserved_without_legacy_alias_rewrite() {
    let argv = vec!["bijux".to_string(), "dev".to_string(), "routes".to_string()];
    let intent = parse_intent(&argv).expect("parse should succeed");
    assert_eq!(intent.normalized_path, vec!["dev", "routes"]);
}

#[test]
fn removed_dev_aliases_no_longer_normalize_to_dev_cli_commands() {
    let cases = [
        (vec!["dev".to_string(), "docs".to_string()], vec!["dev", "docs"]),
        (vec!["dev".to_string(), "env".to_string()], vec!["dev", "env"]),
        (vec!["dev".to_string(), "contracts".to_string()], vec!["dev", "contracts"]),
        (vec!["dev".to_string(), "snapshots-audit".to_string()], vec!["dev", "snapshots-audit"]),
        (vec!["dev".to_string(), "fixture-audit".to_string()], vec!["dev", "fixture-audit"]),
    ];
    for (path, expected) in cases {
        let intent = normalize_command_path(&path);
        assert_eq!(intent, expected.into_iter().map(ToString::to_string).collect::<Vec<_>>());
    }
}

#[test]
fn dev_routes_are_known_for_delegation_but_unknown_to_runtime_registry() {
    let registry = RouteRegistry::default();

    let direct = vec!["dev".to_string(), "docs".to_string()];
    let direct_err = registry.resolve(&direct).expect_err("dev route should be unknown to runtime");
    assert!(matches!(direct_err, bijux_cli::api::routing::registry::RouteError::Unknown(_)));

    let canonical = vec!["dev".to_string(), "cli".to_string(), "routes".to_string()];
    assert!(
        bijux_cli::api::routing::catalog::is_known_route(&canonical),
        "canonical dev cli command should remain known for runtime delegation"
    );
    let canonical_err = registry
        .resolve(&canonical)
        .expect_err("canonical dev cli command should remain outside runtime registry");
    assert!(matches!(canonical_err, bijux_cli::api::routing::registry::RouteError::Unknown(_)));
}

#[test]
fn removed_dev_aliases_for_atlas_di_and_list_products_are_unknown() {
    let registry = RouteRegistry::default();
    let delegated = [
        vec!["dev".to_string(), "atlas".to_string()],
        vec!["dev".to_string(), "di".to_string()],
        vec!["dev".to_string(), "list-products".to_string()],
        vec!["dev".to_string(), "crate-health".to_string()],
        vec!["dev".to_string(), "docs-prune-plan".to_string()],
        vec!["dev".to_string(), "maintenance-audit".to_string()],
    ];

    for path in delegated {
        assert!(
            bijux_cli::api::routing::catalog::is_known_route(&path),
            "delegated dev route should stay known for runtime delegation"
        );
        let err =
            registry.resolve(&path).expect_err("delegated dev route should resolve as unknown");
        assert!(matches!(err, bijux_cli::api::routing::registry::RouteError::Unknown(_)));
    }
}
