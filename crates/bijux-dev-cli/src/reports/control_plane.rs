//! Maintainer control-plane helper report assembly.

use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::{json, Value};

use crate::schema::command_registry::{command_registry, DevCliCommandGroup};

/// Builds hidden `bijux-dev-cli atlas` report payload.
#[must_use]
pub fn build_atlas_report() -> Value {
    json!({"status": "ok", "mount": "atlas", "entry_surface": "dev-cli"})
}

/// Builds hidden `bijux-dev-cli di` report payload.
#[must_use]
pub fn build_dependency_injection_report() -> Value {
    json!({"status": "ok", "container": "built-in", "entry_surface": "dev-cli"})
}

/// Canonical executable/package row for product command surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductSurfaceRow {
    /// Public command prefix routed by `bijux`.
    pub command_surface: String,
    /// Owned executable name.
    pub binary: String,
    /// Install package that provides the executable.
    pub package: String,
}

/// Canonical product contract row for cross-repository standardization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductContractRow {
    /// Canonical namespace in `bijux <namespace> ...`.
    pub namespace: String,
    /// Repository slug under `github.com/bijux`.
    pub repository: String,
    /// Runtime command surface contract.
    pub runtime: ProductSurfaceRow,
    /// Control-plane command surface contract.
    pub control: ProductSurfaceRow,
}

/// Builds hidden `bijux-dev-cli list-products` report payload.
#[must_use]
pub fn build_product_list_report(products: &[ProductContractRow]) -> Value {
    json!({"status": "ok", "products": products})
}

/// Builds hidden `bijux-dev-cli list-plugins` report payload.
#[must_use]
pub fn build_plugin_list_report(plugins: Vec<Value>) -> Value {
    let mut visible_plugins = Vec::new();
    let mut integrity_issues = Vec::new();

    for plugin in plugins {
        if plugin.get("_integrity_error") == Some(&Value::Bool(true)) {
            integrity_issues.push(plugin);
            continue;
        }
        visible_plugins.push(plugin);
    }

    json!({
        "status": if integrity_issues.is_empty() { "ok" } else { "degraded" },
        "plugins": visible_plugins,
        "integrity_status": if integrity_issues.is_empty() { "ok" } else { "degraded" },
        "integrity_issues": integrity_issues,
    })
}

/// Builds hidden `bijux-dev-cli list-plugins` report payload from structured plugin rows.
#[must_use]
pub fn build_plugin_list_report_from<T: Serialize>(plugins: T) -> Value {
    json!({"status": "ok", "plugins": plugins})
}

/// Builds `bijux-dev-cli docs` report payload.
#[must_use]
pub fn build_docs_inventory_report(docs: Vec<String>) -> Value {
    json!({
        "docs_count": docs.len(),
        "docs": docs,
        "index": "docs/INDEX.md",
    })
}

/// Builds `bijux-dev-cli docs-prune-plan` report payload.
#[must_use]
pub fn build_docs_prune_plan_report(docs_count: usize) -> Value {
    json!({
        "docs_count": docs_count,
        "target_cap": 60,
        "actions": [
            "merge overlapping architecture docs",
            "merge overlapping compatibility docs",
            "move low-value prose detail into generated JSON artifacts",
            "freeze docs rule: every doc explains law or change",
        ],
    })
}

/// Builds `bijux-dev-cli snapshots-audit` report payload.
#[must_use]
pub fn build_snapshots_audit_report(snapshots: Vec<String>) -> Value {
    json!({
        "snapshot_count": snapshots.len(),
        "snapshots": snapshots,
    })
}

/// Builds `bijux-dev-cli fixture-audit` report payload.
#[must_use]
pub fn build_fixture_audit_report(
    parity_fixtures: Vec<String>,
    snapshot_fixtures: Vec<String>,
) -> Value {
    json!({
        "parity_fixtures": parity_fixtures,
        "snapshot_fixtures": snapshot_fixtures,
    })
}

/// Builds `bijux-dev-cli plugin-health` report payload.
#[must_use]
pub fn build_plugin_health_report(machine_report: Value, text_report: String) -> Value {
    json!({
        "machine_report": machine_report,
        "text_report": text_report,
    })
}

/// Builds `bijux-dev-cli doctor` report payload.
#[must_use]
pub fn build_doctor_report(
    config_issues: Vec<Value>,
    path_issues: Vec<Value>,
    plugin_issues: Vec<Value>,
    history_issues: Vec<Value>,
    memory_issues: Vec<Value>,
) -> Value {
    let status = if config_issues.is_empty()
        && path_issues.is_empty()
        && plugin_issues.is_empty()
        && history_issues.is_empty()
        && memory_issues.is_empty()
    {
        "healthy"
    } else {
        "degraded"
    };
    json!({
        "status": status,
        "runtime": "dev-cli",
        "issues": {
            "config": config_issues,
            "paths": path_issues,
            "plugins": plugin_issues,
            "history": history_issues,
            "memory": memory_issues,
        },
    })
}

/// Builds canonical dev-cli command ownership report payload.
#[must_use]
pub fn build_command_ownership_report(generated_at: &str) -> Value {
    let mut grouped = BTreeMap::<String, Vec<&'static str>>::new();
    for entry in command_registry() {
        grouped
            .entry(match entry.group {
                DevCliCommandGroup::Dashboard => "dashboard".to_string(),
                DevCliCommandGroup::Routing => "routing".to_string(),
                DevCliCommandGroup::Runtime => "runtime".to_string(),
                DevCliCommandGroup::Audit => "audit".to_string(),
                DevCliCommandGroup::Internal => "internal".to_string(),
            })
            .or_default()
            .push(entry.command.as_str());
    }
    json!({
        "generated_at": generated_at,
        "owner": "bijux-dev-cli",
        "namespace": "bijux-dev-cli",
        "commands": command_registry()
            .iter()
            .map(|entry| json!({
                "command": entry.command.as_str(),
                "group": entry.group.as_str(),
                "visible": entry.visible,
                "owner": entry.owner,
            }))
            .collect::<Vec<_>>(),
        "grouped_commands": grouped,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::build_plugin_list_report;

    #[test]
    fn plugin_list_report_is_ok_when_no_integrity_errors_exist() {
        let payload = build_plugin_list_report(vec![json!({"manifest":{"namespace":"alpha"}})]);
        assert_eq!(payload["status"], "ok");
        assert_eq!(payload["integrity_status"], "ok");
        assert_eq!(payload["plugins"].as_array().map(Vec::len), Some(1));
        assert_eq!(payload["integrity_issues"].as_array().map(Vec::len), Some(0));
    }

    #[test]
    fn plugin_list_report_is_degraded_when_integrity_errors_exist() {
        let payload = build_plugin_list_report(vec![
            json!({"manifest":{"namespace":"alpha"}}),
            json!({
                "_integrity_error": true,
                "source": "plugin-registry",
                "message": "registry corrupted",
            }),
        ]);
        assert_eq!(payload["status"], "degraded");
        assert_eq!(payload["integrity_status"], "degraded");
        assert_eq!(payload["plugins"].as_array().map(Vec::len), Some(1));
        assert_eq!(payload["integrity_issues"].as_array().map(Vec::len), Some(1));
    }
}
