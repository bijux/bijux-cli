#![forbid(unsafe_code)]
//! Registry precedence and namespace-policy tests.

use bijux_cli_contracts as _;
use bijux_cli_routing::registry::{RouteError, RouteRegistry};
use clap as _;
use proptest as _;
use serde as _;
use serde_json as _;
use strsim as _;
use thiserror as _;

#[test]
fn official_reserved_namespaces_take_precedence() {
    let mut registry = RouteRegistry::default();
    for ns in
        ["cli", "dev", "help", "version", "doctor", "repl", "plugins", "completion", "inspect"]
    {
        let result = registry.register_plugin_namespace(ns);
        assert!(
            matches!(result, Err(RouteError::Reserved(_))),
            "expected reserved rejection for {ns}"
        );
    }
}

#[test]
fn user_plugin_namespace_rejection_rules_apply() {
    let mut registry = RouteRegistry::default();
    assert!(
        registry.register_plugin_namespace("status").is_err(),
        "builtin root collision must fail"
    );
    assert!(registry.register_plugin_namespace("plugins").is_err(), "reserved namespace must fail");

    registry.register_plugin_namespace("community").expect("first plugin register should succeed");
    assert!(
        registry.register_plugin_namespace("community").is_err(),
        "duplicate plugin namespace must fail"
    );
}

#[test]
fn plugin_name_collision_with_builtin_command_root_is_rejected() {
    let mut registry = RouteRegistry::default();
    let err =
        registry.register_plugin_namespace("config").expect_err("config root must be protected");
    assert!(matches!(err, RouteError::Conflict(_)));
}
