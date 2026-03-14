#![forbid(unsafe_code)]
//! Registry conflict and route resolution tests.

use bijux_cli::api::routing::registry::{RouteError, RouteRegistry, RouteTarget};
use proptest as _;
use serde as _;
use serde_json as _;

use clap as _;
use schemars as _;
use semver as _;
use thiserror as _;

#[test]
fn rejects_reserved_plugin_namespace() {
    let mut registry = RouteRegistry::default();
    let err = registry.register_plugin_namespace("cli").expect_err("must reject reserved");
    assert_eq!(err, RouteError::Reserved("cli".to_string()));
}

#[test]
fn detects_plugin_namespace_conflict() {
    let mut registry = RouteRegistry::default();
    registry.register_plugin_namespace("alpha").expect("first register must succeed");
    let err = registry.register_plugin_namespace("alpha").expect_err("duplicate must fail");
    assert_eq!(err, RouteError::Conflict("alpha".to_string()));
}

#[test]
fn resolves_builtins_and_plugins_deterministically() {
    let mut registry = RouteRegistry::default();
    registry.register_plugin_namespace("alpha").expect("plugin register must succeed");

    let built_in = registry
        .resolve(&["cli".to_string(), "status".to_string()])
        .expect("builtin should resolve");
    assert_eq!(built_in, RouteTarget::BuiltIn);

    let plugin =
        registry.resolve(&["alpha".to_string(), "run".to_string()]).expect("plugin should resolve");
    assert_eq!(plugin, RouteTarget::Plugin("alpha".to_string()));
}

#[test]
fn resolves_registered_plugin_aliases_to_their_namespace() {
    let mut registry = RouteRegistry::default();
    registry
        .register_plugin_namespace_with_aliases(
            "alpha",
            &[String::from("alpha-short"), String::from("alpha-tools")],
        )
        .expect("plugin aliases should register");

    let alias_route = registry
        .resolve(&["alpha-short".to_string(), "run".to_string()])
        .expect("plugin alias should resolve");
    assert_eq!(alias_route, RouteTarget::Plugin("alpha".to_string()));

    let tree = registry.route_tree();
    assert!(tree.iter().any(|row| row.name.0 == "alpha-short" && row.owner == "plugin-alias:alpha"));
}

#[test]
fn suggests_typo_namespace() {
    let registry = RouteRegistry::default();
    let suggestion = registry.suggest_namespace("inspekt");
    assert_eq!(suggestion.as_deref(), Some("inspect"));
}

#[test]
fn suggests_registered_plugin_aliases_for_typoes() {
    let mut registry = RouteRegistry::default();
    registry
        .register_plugin_namespace_with_aliases("alpha", &[String::from("alpha-short")])
        .expect("plugin alias should register");
    let suggestion = registry.suggest_namespace("alph-short");
    assert_eq!(suggestion.as_deref(), Some("alpha-short"));
}

#[test]
fn prevents_plugin_shadowing_builtin_root() {
    let mut registry = RouteRegistry::default();
    let err = registry.register_plugin_namespace("plugins").expect_err("plugins is reserved");
    assert_eq!(err, RouteError::Reserved("plugins".to_string()));
}
