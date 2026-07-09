#![forbid(unsafe_code)]
//! Route registry stability checks for deterministic behavior and crash resistance.
//! test_type: route-registry-stability

use bijux_cli::api::diagnostics::{registry_inventory, route_inventory};
use bijux_cli::api::routing::registry::{RouteError, RouteRegistry};
use clap as _;
use proptest as _;
use schemars as _;
use semver as _;
use serde as _;
use serde_json as _;
use thiserror as _;

fn shuffled(values: &[&str], seed: u64) -> Vec<String> {
    let mut out: Vec<String> = values.iter().map(|v| (*v).to_string()).collect();
    let mut state = seed;
    for i in 0..out.len() {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let j = (state as usize) % out.len();
        out.swap(i, j);
    }
    out
}

#[test]
fn fuzz_route_registration_order_is_deterministic() {
    let source = ["community", "alpha", "zeta", "ops", "cloud"];
    let mut left = RouteRegistry::default();
    let mut right = RouteRegistry::default();

    for name in shuffled(&source, 7) {
        left.register_plugin_namespace(&name).expect("left registration");
    }
    for name in shuffled(&source, 99) {
        right.register_plugin_namespace(&name).expect("right registration");
    }

    assert_eq!(left.route_tree(), right.route_tree());
    assert_eq!(left.render_command_tree(), right.render_command_tree());
}

#[test]
fn fuzz_randomized_plugin_namespace_registration_is_safe_and_deterministic() {
    let corpus = [
        "Community",
        "community",
        "my_plugin",
        "my-plugin",
        "registry",
        "config",
        "help",
        "atlas",
        "alpha",
        "Alpha",
        "ops-team",
        "ops_team",
    ];

    let mut a = RouteRegistry::default();
    let mut b = RouteRegistry::default();
    for raw in shuffled(&corpus, 123) {
        let _ = a.register_plugin_namespace(&raw);
    }
    for raw in shuffled(&corpus, 456) {
        let _ = b.register_plugin_namespace(&raw);
    }

    assert_eq!(a.route_tree(), b.route_tree());
}

#[test]
fn fuzz_normalized_collision_registration_rejects_equivalent_namespaces() {
    let mut registry = RouteRegistry::default();
    registry.register_plugin_namespace("my-plugin").expect("baseline register");

    let err_a = registry
        .register_plugin_namespace("MY_PLUGIN")
        .expect_err("normalized collision must fail");
    let err_b = registry
        .register_plugin_namespace("my_plugin")
        .expect_err("normalized collision must fail");

    assert!(matches!(err_a, RouteError::Conflict(_)));
    assert!(matches!(err_b, RouteError::Conflict(_)));
}

#[test]
fn fuzz_hidden_alias_collision_registration_rejects_alias_roots() {
    let mut registry = RouteRegistry::default();

    for blocked in ["apps", "config", "doctor", "inspect", "plugins", "cli", "version"] {
        let err = registry
            .register_plugin_namespace(blocked)
            .expect_err("alias/root collision must be rejected");
        assert!(matches!(err, RouteError::Conflict(_) | RouteError::Reserved(_)));
    }
}

#[test]
fn fuzz_command_tree_export_under_randomized_registration_is_stable() {
    let source = ["ops", "community", "acme", "build", "alpha", "omega"];

    let mut outputs = Vec::new();
    for seed in [1_u64, 2, 3, 4, 5, 6] {
        let mut registry = RouteRegistry::default();
        for name in shuffled(&source, seed) {
            registry.register_plugin_namespace(&name).expect("register");
        }
        outputs.push(registry.render_command_tree());
    }

    for output in &outputs[1..] {
        assert_eq!(output, &outputs[0]);
    }
}

#[test]
fn fuzz_route_inspection_payload_generation_is_json_stable() {
    let mut registry = RouteRegistry::default();
    for name in ["community", "inspector", "builder"] {
        registry.register_plugin_namespace(name).expect("register");
    }

    let rows = registry.route_tree();
    let first = serde_json::to_string(&rows).expect("serialize route tree");
    let second = serde_json::to_string(&rows).expect("serialize route tree");
    assert_eq!(first, second);
}

#[test]
fn fuzz_unknown_command_suggestion_generation_is_stable() {
    let mut registry = RouteRegistry::default();
    for ns in ["community", "commander", "compare"] {
        registry.register_plugin_namespace(ns).expect("register");
    }

    let typos = ["commnad", "inspekt", "verison", "plguins", "doctro", "cmmunity"];
    for typo in typos {
        let a = registry.suggest_namespace(typo);
        let b = registry.suggest_namespace(typo);
        assert_eq!(a, b);
        assert!(a.is_some());
    }
}

#[test]
fn fuzz_command_metadata_rendering_is_stable() {
    let mut registry = RouteRegistry::default();
    registry.register_plugin_namespace("community").expect("register");

    let routes = route_inventory(&registry);
    let registry_meta = registry_inventory(&registry);

    let routes_json_a = serde_json::to_string(&routes).expect("serialize routes");
    let routes_json_b = serde_json::to_string(&routes).expect("serialize routes");
    assert_eq!(routes_json_a, routes_json_b);

    let registry_json_a = serde_json::to_string(&registry_meta).expect("serialize registry report");
    let registry_json_b = serde_json::to_string(&registry_meta).expect("serialize registry report");
    assert_eq!(registry_json_a, registry_json_b);
}

#[test]
fn fuzz_route_tree_serialization_is_stable() {
    let mut registry = RouteRegistry::default();
    for ns in ["community", "alpha", "omega"] {
        registry.register_plugin_namespace(ns).expect("register");
    }

    let tree = registry.route_tree();
    let json = serde_json::to_string(&tree).expect("json");
    let reparsed: serde_json::Value = serde_json::from_str(&json).expect("reparse");
    assert!(reparsed.is_array());
}

#[test]
fn fuzz_route_tree_text_rendering_is_stable() {
    let mut registry = RouteRegistry::default();
    for ns in ["community", "alpha", "omega"] {
        registry.register_plugin_namespace(ns).expect("register");
    }

    let a = registry.render_command_tree();
    let b = registry.render_command_tree();
    assert_eq!(a, b);
    assert!(a.contains("cli"));
    assert!(!a.contains("dev"));
}
