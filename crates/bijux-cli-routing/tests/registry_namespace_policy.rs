#![forbid(unsafe_code)]
//! Registry precedence and namespace-policy tests.

use bijux_cli_routing as _;
use bijux_cli_routing::registry::{RouteError, RouteRegistry};
use bijux_cli_routing::OFFICIAL_PRODUCT_NAMESPACES;
use proptest as _;
use serde as _;
use serde_json as _;
use std::sync::{Arc, Barrier, Mutex};

use clap as _;
use schemars as _;
use semver as _;
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
    for ns in OFFICIAL_PRODUCT_NAMESPACES {
        let result = registry.register_plugin_namespace(ns);
        assert!(
            matches!(result, Err(RouteError::Reserved(_))),
            "expected reserved rejection for official namespace {ns}"
        );
    }
}

#[test]
fn normalized_and_case_folded_namespace_collisions_are_rejected() {
    let mut registry = RouteRegistry::default();
    registry.register_plugin_namespace("my-plugin").expect("baseline namespace should register");

    let normalized_collision = registry
        .register_plugin_namespace("my_plugin")
        .expect_err("normalized collision must fail");
    assert!(matches!(normalized_collision, RouteError::Conflict(_)));

    let case_collision = registry
        .register_plugin_namespace("MY-PLUGIN")
        .expect_err("case-folding collision must fail");
    assert!(matches!(case_collision, RouteError::Conflict(_)));
}

#[test]
fn hidden_alias_paths_remain_builtin_when_namespace_resembles_alias_tail() {
    let mut registry = RouteRegistry::default();
    registry.register_plugin_namespace("registry").expect("namespace is allowed");

    let resolved = registry
        .resolve(&["dev".to_string(), "cli".to_string(), "registry".to_string()])
        .expect("canonical dev cli registry path must stay builtin");
    assert!(matches!(resolved, bijux_cli_routing::registry::RouteTarget::BuiltIn));
}

#[test]
fn concurrent_registration_on_normalized_equivalent_namespaces_yields_single_winner() {
    let registry = Arc::new(Mutex::new(RouteRegistry::default()));
    let barrier = Arc::new(Barrier::new(2));
    let mut handles = Vec::new();

    for namespace in ["My_Plugin", "my-plugin"] {
        let shared = Arc::clone(&registry);
        let sync = Arc::clone(&barrier);
        handles.push(std::thread::spawn(move || {
            sync.wait();
            shared.lock().expect("lock registry").register_plugin_namespace(namespace)
        }));
    }

    let mut success = 0_u8;
    let mut conflicts = 0_u8;
    for handle in handles {
        match handle.join().expect("thread join") {
            Ok(()) => success += 1,
            Err(RouteError::Conflict(_)) => conflicts += 1,
            Err(other) => panic!("unexpected namespace registration error: {other}"),
        }
    }

    assert_eq!(success, 1);
    assert_eq!(conflicts, 1);
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
