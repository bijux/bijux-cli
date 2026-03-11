#![forbid(unsafe_code)]
//! Command metadata and inspection consistency matrix.
//! test_type: command-metadata-consistency

use std::collections::BTreeSet;
use std::process::{Command, Output};

use bijux_cli as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .output()
        .expect("binary should execute")
}

fn run_json(args: &[&str]) -> Value {
    let out = run(args);
    assert_eq!(out.status.code(), Some(0), "expected success for {args:?}");
    serde_json::from_slice(&out.stdout).expect("stdout should be valid json")
}

fn route_key(row: &Value) -> String {
    row["segments"]
        .as_array()
        .expect("segments array")
        .iter()
        .map(|segment| segment.as_str().expect("segment should be string"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn top_level_roots(routes: &[Value]) -> BTreeSet<String> {
    routes
        .iter()
        .filter_map(|row| row.get("segments").and_then(Value::as_array))
        .filter_map(|segments| segments.first())
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect()
}

fn parse_help_commands(help_text: &str) -> BTreeSet<String> {
    let mut in_commands = false;
    let mut names = BTreeSet::new();
    for line in help_text.lines() {
        let trimmed = line.trim_end();
        if trimmed == "Commands:" {
            in_commands = true;
            continue;
        }
        if in_commands {
            if trimmed.is_empty() {
                continue;
            }
            if !line.starts_with("  ") {
                break;
            }
            let first = trimmed.split_whitespace().next().unwrap_or_default();
            if !first.is_empty() {
                names.insert(first.to_string());
            }
        }
    }
    names
}

#[test]
fn every_routable_command_has_inspectable_metadata_and_stable_route_identity() {
    let inspect = run_json(&["inspect", "--format", "json", "--no-pretty"]);
    let routes = run_json(&["dev", "cli", "routes", "--format", "json", "--no-pretty"]);

    let inspect_routes: BTreeSet<String> = inspect["route_sources"]
        .as_array()
        .expect("route_sources")
        .iter()
        .map(route_key)
        .collect();
    let routed_routes: BTreeSet<String> = routes["routes"]
        .as_array()
        .expect("routes")
        .iter()
        .map(route_key)
        .collect();

    assert!(
        !inspect_routes.is_empty(),
        "inspect route metadata should not be empty"
    );
    assert_eq!(
        inspect_routes, routed_routes,
        "inspect route identity must match dev cli routes"
    );
}

#[test]
fn inspect_exposes_builtin_and_plugin_metadata_consistently() {
    let inspect = run_json(&["inspect", "--format", "json", "--no-pretty"]);
    assert!(
        inspect["builtins"].is_array(),
        "builtins should be an array"
    );
    assert!(
        inspect["plugin_origins"].is_array(),
        "plugin_origins should be an array"
    );

    for row in inspect["builtins"].as_array().expect("builtins array") {
        assert!(
            row["segments"].is_array(),
            "built-in row should expose segments array"
        );
    }
    for row in inspect["plugin_origins"]
        .as_array()
        .expect("plugin origins array")
    {
        let has_path_identity = row.get("segments").is_some_and(Value::is_array)
            || row.get("namespace").is_some_and(Value::is_string);
        assert!(
            has_path_identity,
            "plugin row should expose route segments or namespace"
        );
        assert!(
            row["owner"].is_string() || row["source"].is_string(),
            "plugin row should expose source metadata"
        );
    }
}

#[test]
fn inspect_routes_and_registry_agree_on_namespace_ownership_and_plugin_source_metadata() {
    let inspect = run_json(&["inspect", "--format", "json", "--no-pretty"]);
    let routes = run_json(&["dev", "cli", "routes", "--format", "json", "--no-pretty"]);
    let registry = run_json(&["dev", "cli", "registry", "--format", "json", "--no-pretty"]);

    let inspect_roots =
        top_level_roots(inspect["route_sources"].as_array().expect("route_sources"));
    let route_roots = top_level_roots(routes["routes"].as_array().expect("routes"));
    assert_eq!(
        inspect_roots, route_roots,
        "namespace ownership should agree across inspect and routes"
    );

    let inspect_plugin_owners: BTreeSet<String> = inspect["plugin_origins"]
        .as_array()
        .expect("plugin_origins")
        .iter()
        .filter_map(|row| row.get("owner").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect();
    let registry_owners: BTreeSet<String> = registry["registry"]
        .as_array()
        .expect("registry")
        .iter()
        .filter_map(|row| row.get("owner").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect();
    assert!(
        inspect_plugin_owners.is_subset(&registry_owners),
        "inspect plugin owner set should be a subset of registry owners"
    );
}

#[test]
fn route_metadata_is_stable_and_json_serializable_for_covered_commands() {
    let first = run_json(&["inspect", "--format", "json", "--no-pretty"]);
    let second = run_json(&["inspect", "--format", "json", "--no-pretty"]);
    assert_eq!(
        first["route_sources"], second["route_sources"],
        "route metadata should be stable"
    );
}

#[test]
fn command_metadata_fields_do_not_disappear_or_rename_silently() {
    let first = run_json(&["inspect", "--format", "json", "--no-pretty"]);
    let second = run_json(&["inspect", "--format", "json", "--no-pretty"]);

    let required = [
        "status",
        "builtins",
        "route_sources",
        "reserved_namespaces",
        "plugin_origins",
        "alias_rewrites",
        "contracts",
    ];
    for key in required {
        assert!(
            first.get(key).is_some(),
            "missing required key in first payload: {key}"
        );
        assert!(
            second.get(key).is_some(),
            "missing required key in second payload: {key}"
        );
    }

    let first_keys: BTreeSet<String> = first
        .as_object()
        .expect("first payload should be object")
        .keys()
        .cloned()
        .collect();
    let second_keys: BTreeSet<String> = second
        .as_object()
        .expect("second payload should be object")
        .keys()
        .cloned()
        .collect();
    assert_eq!(
        first_keys, second_keys,
        "top-level metadata keys should not drift silently"
    );
}

#[test]
fn reserved_namespaces_and_alias_metadata_are_consistent_and_non_canonical() {
    let inspect = run_json(&["inspect", "--format", "json", "--no-pretty"]);
    let registry = run_json(&["dev", "cli", "registry", "--format", "json", "--no-pretty"]);

    let reserved_from_inspect: BTreeSet<String> = inspect["reserved_namespaces"]
        .as_array()
        .expect("reserved namespaces")
        .iter()
        .filter(|row| row["reserved"] == true)
        .filter_map(|row| row["name"].as_str())
        .map(ToString::to_string)
        .collect();
    let reserved_from_registry: BTreeSet<String> = registry["registry"]
        .as_array()
        .expect("registry")
        .iter()
        .filter(|row| row["reserved"] == true)
        .filter_map(|row| row["name"].as_str())
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        reserved_from_inspect, reserved_from_registry,
        "reserved namespace metadata should match inspect and registry"
    );

    let aliases = inspect["alias_rewrites"]
        .as_array()
        .expect("alias rewrites");
    assert!(
        !aliases.is_empty(),
        "compatibility alias metadata should be present"
    );
    for row in aliases {
        assert_eq!(row["source"], "compatibility-alias");
        let alias = row["alias"].as_array().expect("alias");
        let canonical = row["canonical"].as_array().expect("canonical");
        assert_ne!(
            alias, canonical,
            "compatibility alias must not be canonical"
        );
    }
}

#[test]
fn help_output_and_inspect_metadata_agree_on_command_names_and_grouping() {
    let inspect = run_json(&["inspect", "--format", "json", "--no-pretty"]);
    let help = run(&["--help"]);
    assert_eq!(help.status.code(), Some(0));
    let help_text = String::from_utf8(help.stdout).expect("utf-8");
    let help_commands = parse_help_commands(&help_text);
    assert!(
        !help_commands.is_empty(),
        "root help command list should be non-empty"
    );

    let inspect_roots =
        top_level_roots(inspect["route_sources"].as_array().expect("route_sources"));
    for must_exist in [
        "status", "cli", "dev", "config", "plugins", "history", "memory",
    ] {
        assert!(
            help_commands.contains(must_exist),
            "help missing command {must_exist}"
        );
        assert!(
            inspect_roots.contains(must_exist),
            "inspect missing root command {must_exist}"
        );
    }

    for root in &inspect_roots {
        assert!(
            help_commands.contains(root),
            "inspect root not present in help tree: {root}"
        );
    }
}
