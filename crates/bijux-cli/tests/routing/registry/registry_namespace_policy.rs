#![forbid(unsafe_code)]
//! Registry precedence and namespace-policy tests.

use bijux_cli::api::routing::registry::{RouteError, RouteRegistry};
use bijux_cli::contracts::{KNOWN_BIJUX_TOOLS, OFFICIAL_PRODUCT_NAMESPACES};
use proptest as _;
use serde as _;
use serde::Deserialize;
use serde_json as _;
use std::collections::BTreeMap;
use std::fs;
use std::sync::{Arc, Barrier, Mutex};

use clap as _;
use schemars as _;
use semver as _;
use thiserror as _;

#[test]
fn official_reserved_namespaces_take_precedence() {
    let mut registry = RouteRegistry::default();
    for ns in ["cli", "help", "version", "doctor", "repl", "plugins", "completion", "inspect"] {
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
fn known_bijux_tool_registry_matches_expected_namespaces() {
    let expected = ["agent", "atlas", "dag", "dna", "gnss", "rag", "rar", "vex"];
    let official: Vec<&str> = OFFICIAL_PRODUCT_NAMESPACES.to_vec();
    let known: Vec<&str> = KNOWN_BIJUX_TOOLS.iter().map(|tool| tool.namespace).collect();

    assert_eq!(official, expected);
    assert_eq!(known, expected);
}

#[test]
fn known_bijux_tools_follow_standard_binary_and_package_patterns() {
    for tool in KNOWN_BIJUX_TOOLS {
        let runtime_binary = tool.runtime_binary();
        let control_binary = tool.control_binary();
        assert_eq!(tool.runtime_binary(), format!("bijux-{}", tool.namespace));
        assert_eq!(tool.control_binary(), format!("bijux-dev-{}", tool.namespace));
        assert_eq!(runtime_binary, format!("bijux-{}", tool.namespace));
        assert_eq!(control_binary, format!("bijux-dev-{}", tool.namespace));
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct OfficialProductRegistry {
    entries: Vec<OfficialProductRegistryEntry>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct OfficialProductRegistryEntry {
    namespace: String,
    runtime_binary: String,
    control_binary: String,
    runtime_package: String,
    control_package: String,
    repository: String,
}

#[test]
fn official_product_registry_doc_stays_in_sync_with_known_tool_contracts() {
    let registry_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/official_product_namespace_registry.json");
    let registry_text = fs::read_to_string(&registry_path).expect("read official product registry");
    let registry: OfficialProductRegistry =
        serde_json::from_str(&registry_text).expect("parse official product registry json");

    let expected: BTreeMap<String, OfficialProductRegistryEntry> = KNOWN_BIJUX_TOOLS
        .iter()
        .map(|tool| {
            (
                tool.namespace.to_string(),
                OfficialProductRegistryEntry {
                    namespace: tool.namespace.to_string(),
                    runtime_binary: tool.runtime_binary(),
                    control_binary: tool.control_binary(),
                    runtime_package: tool.runtime_binary(),
                    control_package: tool.control_binary(),
                    repository: tool.runtime_binary(),
                },
            )
        })
        .collect();

    let actual: BTreeMap<String, OfficialProductRegistryEntry> =
        registry.entries.into_iter().map(|entry| (entry.namespace.clone(), entry)).collect();

    assert_eq!(actual, expected);
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
        .resolve(&["atlas".to_string(), "registry".to_string()])
        .expect_err("external product runtime remains outside the local registry");
    assert!(matches!(resolved, RouteError::Unknown(_)));
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
