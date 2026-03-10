#![forbid(unsafe_code)]
//! Regression tests for legacy Python command forms.

use bijux_cli_contracts as _;
use bijux_cli_routing::catalog::normalize_command_path;
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

#[test]
fn removed_dev_aliases_no_longer_normalize_to_dev_cli_commands() {
    let cases = [
        (vec!["dev".to_string(), "docs".to_string()], vec!["dev", "docs"]),
        (vec!["dev".to_string(), "env".to_string()], vec!["dev", "env"]),
        (vec!["dev".to_string(), "contracts".to_string()], vec!["dev", "contracts"]),
        (
            vec!["dev".to_string(), "snapshots-audit".to_string()],
            vec!["dev", "snapshots-audit"],
        ),
        (
            vec!["dev".to_string(), "fixture-audit".to_string()],
            vec!["dev", "fixture-audit"],
        ),
    ];
    for (path, expected) in cases {
        let intent = normalize_command_path(&path);
        assert_eq!(
            intent,
            expected.into_iter().map(ToString::to_string).collect::<Vec<_>>()
        );
    }
}

#[test]
fn removed_dev_alias_paths_resolve_as_unknown_and_canonical_path_still_resolves() {
    let registry = RouteRegistry::default();

    let legacy = vec!["dev".to_string(), "docs".to_string()];
    let legacy_err = registry.resolve(&legacy).expect_err("legacy alias should be unknown");
    assert!(matches!(legacy_err, bijux_cli_routing::registry::RouteError::Unknown(_)));

    let canonical = vec!["dev".to_string(), "cli".to_string(), "docs".to_string()];
    let canonical_target = registry.resolve(&canonical).expect("canonical route should resolve");
    assert_eq!(canonical_target, RouteTarget::BuiltIn);
}
