//! Root command handlers.

use std::path::Path;

use serde_json::{json, Value};

use crate::contracts::{known_bijux_tool_by_query, known_bijux_tools};
use crate::features::apps::{
    app_capabilities_report, app_doctor_report, app_manifest_schema_report, app_version_report,
    app_which_report, apps_doctor_report, apps_list_report, scaffold_app_mount,
    validate_app_manifest_report,
};
use crate::features::diagnostics::state_paths::ResolvedStatePaths;
use crate::routing::catalog::normalize_command_path;
use crate::routing::registry::{RouteRegistry, RouteTarget};
use crate::shared::argv::{command_has_flag, command_option_value, command_positionals};

use super::cli::{
    docs_inventory_report, runtime_audit_report, runtime_error_payload, runtime_status_report,
};

pub(crate) fn try_handle(
    normalized_path: &[String],
    argv: &[String],
    paths: &ResolvedStatePaths,
    registry: &RouteRegistry,
    plugin_registry_path: &Path,
) -> Option<Value> {
    match normalized_path {
        [a] if a == "status" => Some(runtime_status_report(paths, plugin_registry_path)),
        [a] if a == "audit" => Some(runtime_audit_report(paths, plugin_registry_path)),
        [a] if a == "docs" => Some(docs_inventory_report()),
        [a] if a == "explain" => Some(explain_route_report(argv, registry)),
        [a, b] if a == "apps" && b == "list" => Some(
            serde_json::to_value(apps_list_report(paths, plugin_registry_path))
                .expect("apps list report"),
        ),
        [a, b] if a == "apps" && b == "doctor" => {
            Some(match command_positionals(argv, &["apps", "doctor"]).first().cloned() {
                Some(namespace) => match app_doctor_report(&namespace, paths) {
                    Ok(report) => serde_json::to_value(report).expect("app doctor report"),
                    Err(error) => runtime_error_payload(error),
                },
                None => serde_json::to_value(apps_doctor_report(paths, plugin_registry_path))
                    .expect("apps doctor report"),
            })
        }
        [a, b] if a == "apps" && b == "which" => {
            let namespace = command_positionals(argv, &["apps", "which"]).first().cloned();
            Some(match namespace.as_deref() {
                Some(namespace) => match app_which_report(namespace, paths) {
                    Ok(report) => serde_json::to_value(report).expect("apps which report"),
                    Err(error) => runtime_error_payload(error),
                },
                None => runtime_error_payload("Missing argument: namespace required".to_string()),
            })
        }
        [a, b] if a == "apps" && b == "version" => {
            let namespace = command_positionals(argv, &["apps", "version"]).first().cloned();
            Some(match namespace.as_deref() {
                Some(namespace) => match app_version_report(namespace, paths) {
                    Ok(report) => serde_json::to_value(report).expect("apps version report"),
                    Err(error) => runtime_error_payload(error),
                },
                None => runtime_error_payload("Missing argument: namespace required".to_string()),
            })
        }
        [a, b] if a == "apps" && b == "capabilities" => {
            let namespace = command_positionals(argv, &["apps", "capabilities"]).first().cloned();
            Some(match namespace.as_deref() {
                Some(namespace) => match app_capabilities_report(namespace, paths) {
                    Ok(report) => serde_json::to_value(report).expect("apps capabilities report"),
                    Err(error) => runtime_error_payload(error),
                },
                None => runtime_error_payload("Missing argument: namespace required".to_string()),
            })
        }
        [a, b] if a == "apps" && b == "schema" => {
            Some(serde_json::to_value(app_manifest_schema_report()).expect("apps schema report"))
        }
        [a, b] if a == "apps" && b == "validate-manifest" => {
            let path = command_positionals(argv, &["apps", "validate-manifest"]).first().cloned();
            Some(match path.as_deref() {
                Some(path) => serde_json::to_value(validate_app_manifest_report(Path::new(path)))
                    .expect("apps validate-manifest report"),
                None => runtime_error_payload("Missing argument: path required".to_string()),
            })
        }
        [a, b] if a == "apps" && b == "scaffold" => {
            let positional = command_positionals(argv, &["apps", "scaffold"]);
            let kind = positional.first().cloned();
            let namespace = positional.get(1).cloned();
            let force = command_has_flag(argv, "--force");
            let target = command_option_value(argv, &["apps", "scaffold"], "--path")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| {
                    let stem = namespace.clone().unwrap_or_else(|| "sample-app".to_string());
                    std::env::current_dir().unwrap_or_default().join(format!("{stem}-app"))
                });
            Some(match (kind.as_deref(), namespace.as_deref()) {
                (Some(kind), Some(namespace)) => {
                    match scaffold_app_mount(kind, namespace, force, &target) {
                        Ok(report) => serde_json::to_value(report).expect("apps scaffold report"),
                        Err(error) => runtime_error_payload(error),
                    }
                }
                _ => runtime_error_payload(
                    "Missing arguments: kind and namespace required".to_string(),
                ),
            })
        }
        _ => None,
    }
}

fn side_effect_class(normalized_path: &[String], target_class: &str) -> &'static str {
    if matches!(target_class, "official_app" | "plugin") {
        return "external-exec";
    }
    match normalized_path {
        [a] if matches!(a.as_str(), "status" | "audit" | "docs" | "version" | "inspect") => {
            "read-only"
        }
        [a] if matches!(a.as_str(), "repl") => "interactive",
        [a] if matches!(a.as_str(), "install") => "state-write",
        [a, b] if a == "apps" && matches!(b.as_str(), "scaffold") => "state-write",
        [a, b] if a == "history" && matches!(b.as_str(), "clear") => "state-write",
        [a] if matches!(a.as_str(), "memory" | "history") => "state-write",
        [a, b] if a == "memory" && matches!(b.as_str(), "set" | "delete" | "clear") => {
            "state-write"
        }
        [a, b] if a == "config" && matches!(b.as_str(), "set" | "unset" | "clear" | "load") => {
            "state-write"
        }
        [a, b]
            if a == "plugins"
                && matches!(
                    b.as_str(),
                    "install" | "uninstall" | "enable" | "disable" | "scaffold"
                ) =>
        {
            "state-write"
        }
        [a, b] if a == "cli" && matches!(b.as_str(), "repl") => "interactive",
        [a, b] if a == "cli" && matches!(b.as_str(), "completion") => "read-only",
        [a, b] if a == "cli" && matches!(b.as_str(), "status" | "paths" | "doctor" | "inspect") => {
            "read-only"
        }
        [a, b, c]
            if a == "cli"
                && b == "config"
                && matches!(c.as_str(), "set" | "unset" | "clear" | "load") =>
        {
            "state-write"
        }
        [a, b, c]
            if a == "cli"
                && b == "plugins"
                && matches!(
                    c.as_str(),
                    "install" | "uninstall" | "enable" | "disable" | "scaffold"
                ) =>
        {
            "state-write"
        }
        _ => "read-only",
    }
}

fn explain_route_report(argv: &[String], registry: &RouteRegistry) -> Value {
    let requested_path = command_positionals(argv, &["explain"]);
    if requested_path.is_empty() {
        return runtime_error_payload("Missing argument: command route required".to_string());
    }

    let normalized_path = normalize_command_path(&requested_path);
    let requested_command = requested_path.join(" ");

    let (target_class, owner, descriptor_source, resolved_namespace) = if let Some(root) =
        requested_path.first()
    {
        if let Some(tool) = known_bijux_tool_by_query(root) {
            (
                "official_app",
                tool.namespace.to_string(),
                "official_product_namespace_registry.json".to_string(),
                Some(tool.namespace.to_string()),
            )
        } else {
            match registry.resolve(&normalized_path) {
                Ok(RouteTarget::BuiltIn) => (
                    "built_in",
                    "bijux-cli".to_string(),
                    "built-in route registry".to_string(),
                    None,
                ),
                Ok(RouteTarget::Plugin(namespace)) => {
                    ("plugin", namespace.clone(), "plugin registry".to_string(), Some(namespace))
                }
                Err(_) => ("unknown", "unknown".to_string(), "unresolved".to_string(), None),
            }
        }
    } else {
        ("unknown", "unknown".to_string(), "unresolved".to_string(), None)
    };

    let side_effect_class = side_effect_class(&normalized_path, target_class);
    let known_official_namespaces =
        known_bijux_tools().iter().map(|tool| tool.namespace.to_string()).collect::<Vec<_>>();

    json!({
        "status": if target_class == "unknown" { "error" } else { "ok" },
        "requested_command": requested_command,
        "requested_path": requested_path,
        "normalized_path": normalized_path,
        "route": {
            "target_class": target_class,
            "owner": owner,
            "resolved_namespace": resolved_namespace,
            "descriptor_source": descriptor_source,
            "side_effect_class": side_effect_class,
        },
        "envelope": {
            "success_schema": "output-envelope-v1",
            "error_schema": "error-envelope-v1",
            "schema_version": "v1",
        },
        "hints": if target_class == "unknown" {
            vec![
                "Run `bijux --help` to inspect root routes.",
                "Run `bijux apps list` to inspect official app namespaces.",
                "Run `bijux plugins list` to inspect installed plugin namespaces.",
            ]
        } else {
            vec![
                "Run `bijux inspect --format json` for route inventory and provenance details.",
            ]
        },
        "known_official_namespaces": known_official_namespaces,
    })
}
