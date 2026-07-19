#![forbid(unsafe_code)]
//! Registry precedence and namespace-policy tests.

use std::collections::BTreeMap;
use std::fs;
use std::sync::{Arc, Barrier, Mutex};

use bijux_cli::api::routing::registry::{RouteError, RouteRegistry};
use bijux_cli::contracts::{
    known_bijux_tool_by_query, known_bijux_tools, official_product_namespaces,
    official_product_registry_schema,
};
use proptest as _;
use serde as _;
use serde::Deserialize;
use serde_json as _;

use clap as _;
use schemars as _;
use semver as _;
use thiserror as _;

fn assert_runtime_package_contract(package_name: &str, binary_name: &str) {
    let alternate_cli_package = format!("{binary_name}-cli");
    assert!(
        package_name == binary_name || package_name == alternate_cli_package,
        "runtime package must match the binary name or the durable `-cli` crate pattern"
    );
}

#[test]
fn official_reserved_namespaces_take_precedence() {
    let mut registry = RouteRegistry::default();
    for ns in
        ["apps", "cli", "help", "version", "doctor", "repl", "plugins", "completion", "inspect"]
    {
        let result = registry.register_plugin_namespace(ns);
        assert!(
            matches!(result, Err(RouteError::Reserved(_))),
            "expected reserved rejection for {ns}"
        );
    }
    for ns in official_product_namespaces() {
        let result = registry.register_plugin_namespace(ns);
        assert!(
            matches!(result, Err(RouteError::Reserved(_))),
            "expected reserved rejection for official namespace {ns}"
        );
    }
}

#[test]
fn plugin_aliases_cannot_shadow_official_namespaces_or_aliases() {
    let mut registry = RouteRegistry::default();
    let err = registry
        .register_plugin_namespace_with_aliases("community", &["dag".to_string()])
        .expect_err("official namespace alias must be reserved");
    assert!(matches!(err, RouteError::Reserved(_)));

    let err = registry
        .register_plugin_namespace_with_aliases("community", &["workflow".to_string()])
        .expect_err("official alias must be reserved");
    assert!(matches!(err, RouteError::Reserved(_)));
}

#[test]
fn known_bijux_tool_registry_matches_expected_namespaces() {
    let expected = ["agent", "atlas", "dag", "dna", "gnss", "rag", "rar", "vex"];
    let official: Vec<&str> = official_product_namespaces().to_vec();
    let known: Vec<&str> = known_bijux_tools().iter().map(|tool| tool.namespace).collect();

    assert_eq!(official, expected);
    assert_eq!(known, expected);
}

#[test]
fn known_bijux_tools_follow_standard_binary_and_package_patterns() {
    for tool in known_bijux_tools() {
        let runtime_binary = tool.runtime_binary();
        let control_binary = tool.control_binary();
        let runtime_package = tool.runtime_package();
        let control_package = tool.control_package();
        assert_eq!(tool.runtime_binary(), format!("bijux-{}", tool.namespace));
        assert_eq!(tool.control_binary(), format!("bijux-dev-{}", tool.namespace));
        assert_eq!(runtime_binary, format!("bijux-{}", tool.namespace));
        assert_eq!(control_binary, format!("bijux-dev-{}", tool.namespace));
        assert_runtime_package_contract(&runtime_package, &runtime_binary);
        assert_eq!(control_package, control_binary);
        assert_eq!(tool.repository(), runtime_binary);
        assert_eq!(tool.status, "declared");
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct OfficialProductRegistry {
    entries: Vec<OfficialProductRegistryEntry>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct OfficialProductRegistryEntry {
    namespace: String,
    display_name: String,
    aliases: Vec<String>,
    runtime_binary: String,
    control_binary: String,
    runtime_package: String,
    control_package: String,
    repository: String,
    status: String,
    language: String,
    version: Option<String>,
    help_summary: String,
    capabilities: Vec<String>,
}

#[test]
fn official_product_registry_doc_stays_in_sync_with_known_tool_contracts() {
    let registry_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/official_product_namespace_registry.json");
    let registry_text = fs::read_to_string(&registry_path).expect("read official product registry");
    let registry: OfficialProductRegistry =
        serde_json::from_str(&registry_text).expect("parse official product registry json");

    let expected: BTreeMap<String, OfficialProductRegistryEntry> = known_bijux_tools()
        .iter()
        .map(|tool| {
            (
                tool.namespace.to_string(),
                OfficialProductRegistryEntry {
                    namespace: tool.namespace.to_string(),
                    display_name: tool.display_name.to_string(),
                    aliases: tool.aliases.iter().map(|value| (*value).to_string()).collect(),
                    runtime_binary: tool.runtime_binary(),
                    control_binary: tool.control_binary(),
                    runtime_package: tool.runtime_package(),
                    control_package: tool.control_package(),
                    repository: tool.repository(),
                    status: tool.status.to_string(),
                    language: tool.language.to_string(),
                    version: tool.version.map(ToOwned::to_owned),
                    help_summary: tool.help_summary.to_string(),
                    capabilities: tool
                        .capabilities
                        .iter()
                        .map(|value| (*value).to_string())
                        .collect(),
                },
            )
        })
        .collect();

    let actual: BTreeMap<String, OfficialProductRegistryEntry> =
        registry.entries.into_iter().map(|entry| (entry.namespace.clone(), entry)).collect();

    assert_eq!(actual, expected);
}

#[test]
fn official_product_registry_schema_exposes_descriptor_fields() {
    let schema = serde_json::to_value(official_product_registry_schema()).expect("schema json");
    let properties = schema["definitions"]["ProductRegistryEntry"]["properties"]
        .as_object()
        .expect("product registry entry properties");
    for field in [
        "namespace",
        "display_name",
        "aliases",
        "runtime_binary",
        "control_binary",
        "help_summary",
        "capabilities",
    ] {
        assert!(
            properties.contains_key(field),
            "official product registry schema should expose field `{field}`"
        );
    }
}

#[test]
fn official_product_alias_queries_resolve_through_contract_registry() {
    let tool = known_bijux_tool_by_query("workflow").expect("dag alias should resolve");
    assert_eq!(tool.namespace, "dag");
    assert!(tool.capabilities.contains(&"run"));
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
