//! Root command handlers.

use std::path::Path;

use serde_json::Value;

use crate::features::apps::{
    app_capabilities_report, app_version_report, app_which_report, apps_doctor_report,
    apps_list_report,
};
use crate::features::diagnostics::state_paths::ResolvedStatePaths;
use crate::shared::argv::command_positionals;

use super::cli::{
    docs_inventory_report, runtime_audit_report, runtime_error_payload, runtime_status_report,
};

pub(crate) fn try_handle(
    normalized_path: &[String],
    argv: &[String],
    paths: &ResolvedStatePaths,
    plugin_registry_path: &Path,
) -> Option<Value> {
    match normalized_path {
        [a] if a == "status" => Some(runtime_status_report(paths, plugin_registry_path)),
        [a] if a == "audit" => Some(runtime_audit_report(paths, plugin_registry_path)),
        [a] if a == "docs" => Some(docs_inventory_report()),
        [a, b] if a == "apps" && b == "list" => Some(
            serde_json::to_value(apps_list_report(paths, plugin_registry_path))
                .expect("apps list report"),
        ),
        [a, b] if a == "apps" && b == "doctor" => Some(
            serde_json::to_value(apps_doctor_report(paths, plugin_registry_path))
                .expect("apps doctor report"),
        ),
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
        _ => None,
    }
}
