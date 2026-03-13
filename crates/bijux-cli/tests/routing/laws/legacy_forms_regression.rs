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
fn external_runtime_mounts_are_preserved_without_alias_rewrite() {
    let cases = [
        (vec!["atlas".to_string(), "status".to_string()], vec!["atlas", "status"]),
        (vec!["rag".to_string(), "doctor".to_string()], vec!["rag", "doctor"]),
        (vec!["vex".to_string(), "config".to_string()], vec!["vex", "config"]),
    ];
    for (path, expected) in cases {
        let intent = normalize_command_path(&path);
        assert_eq!(intent, expected.into_iter().map(ToString::to_string).collect::<Vec<_>>());
    }
}

#[test]
fn external_runtime_mounts_stay_outside_runtime_registry() {
    let registry = RouteRegistry::default();

    let direct = vec!["atlas".to_string(), "status".to_string()];
    let direct_err =
        registry.resolve(&direct).expect_err("external product runtime should be unknown locally");
    assert!(matches!(direct_err, bijux_cli::api::routing::registry::RouteError::Unknown(_)));
}

#[test]
fn maintainer_only_namespaces_are_reserved_but_not_routed_locally() {
    let registry = RouteRegistry::default();
    let err = registry
        .resolve(&["dev".to_string(), "doctor".to_string()])
        .expect_err("maintainer command namespace must stay outside runtime routing");
    assert!(matches!(err, bijux_cli::api::routing::registry::RouteError::Unknown(_)));
}
