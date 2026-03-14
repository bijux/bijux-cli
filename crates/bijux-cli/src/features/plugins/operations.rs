#![forbid(unsafe_code)]
//! Plugin feature operations exposed to command adapters.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::Result;
use serde_json::{json, Value};

use crate::api::version::runtime_semver;
use crate::contracts::{
    known_bijux_tool_namespaces, official_product_namespaces, plugin_manifest_v2_schema,
    PluginLifecycleState,
};
use crate::features::plugins::{
    blocked_namespace_inventory, compatibility_warnings, disable_plugin, enable_plugin, inspect_plugin,
    install_plugin as install_plugin_manifest, is_reserved_namespace, list_plugins,
    load_time_diagnostics, plugin_doctor, scaffold::scaffold_plugin_layout, self_repair_registry,
    uninstall_plugin, validate_manifest, InstallPluginRequest, PluginTrustLevel, CORE_NAMESPACES,
    RESERVED_NAMESPACES,
};

fn plugin_record_payload(record: &crate::features::plugins::PluginRecord) -> Value {
    json!({
        "manifest": record.manifest,
        "state": record.state,
        "source": record.source,
        "trust_level": record.trust_level,
        "manifest_checksum_sha256": record.manifest_checksum_sha256,
    })
}

fn plugin_records_payload(records: &[crate::features::plugins::PluginRecord]) -> Vec<Value> {
    records.iter().map(plugin_record_payload).collect()
}

fn lifecycle_state_name(state: PluginLifecycleState) -> &'static str {
    match state {
        PluginLifecycleState::Discovered => "discovered",
        PluginLifecycleState::Validated => "validated",
        PluginLifecycleState::Installed => "installed",
        PluginLifecycleState::Enabled => "enabled",
        PluginLifecycleState::Disabled => "disabled",
        PluginLifecycleState::Broken => "broken",
        PluginLifecycleState::Incompatible => "incompatible",
    }
}

fn namespace_reference_payload(requested: &str, canonical: &str) -> Value {
    json!({
        "requested_reference": requested,
        "namespace": canonical,
        "matched_via": if requested == canonical { "namespace" } else { "alias" },
    })
}

fn resolve_install_manifest_path(input: &Path) -> Result<std::path::PathBuf> {
    if input.is_dir() {
        let manifest_path = input.join("plugin.manifest.json");
        if !manifest_path.exists() {
            anyhow::bail!(
                "Invalid argument: plugin directory does not contain plugin.manifest.json: {}",
                input.display()
            );
        }
        return Ok(manifest_path);
    }
    Ok(input.to_path_buf())
}

fn integrity_issue(source: &str, error: impl ToString) -> Value {
    json!({
        "source": source,
        "error": error.to_string(),
    })
}

fn render_load_diagnostics(
    diagnostics: impl IntoIterator<Item = crate::features::plugins::PluginLoadDiagnostic>,
) -> Vec<Value> {
    diagnostics
        .into_iter()
        .map(|diag| {
            json!({
                "namespace": diag.namespace,
                "severity": diag.severity,
                "message": diag.message,
            })
        })
        .collect()
}

fn state_counts(records: &[crate::features::plugins::PluginRecord]) -> Value {
    let mut counts = BTreeMap::<&'static str, usize>::new();
    for record in records {
        let key = match record.state {
            PluginLifecycleState::Discovered => "discovered",
            PluginLifecycleState::Validated => "validated",
            PluginLifecycleState::Installed => "installed",
            PluginLifecycleState::Enabled => "enabled",
            PluginLifecycleState::Disabled => "disabled",
            PluginLifecycleState::Broken => "broken",
            PluginLifecycleState::Incompatible => "incompatible",
        };
        *counts.entry(key).or_insert(0) += 1;
    }
    json!(counts)
}

fn inventory_report(plugin_registry_path: &Path, plugins_dir: &Path) -> Value {
    let mut integrity_issues = Vec::<Value>::new();
    let plugins = match list_plugins(plugin_registry_path) {
        Ok(plugins) => plugins,
        Err(error) => {
            integrity_issues.push(integrity_issue("registry", error));
            Vec::new()
        }
    };
    let compatibility = match compatibility_warnings(plugin_registry_path, runtime_semver()) {
        Ok(warnings) => warnings,
        Err(error) => {
            integrity_issues.push(integrity_issue("compatibility", error));
            Vec::new()
        }
    };
    let load_diagnostics = match load_time_diagnostics(plugin_registry_path, runtime_semver()) {
        Ok(diagnostics) => render_load_diagnostics(diagnostics),
        Err(error) => {
            integrity_issues.push(integrity_issue("load-time-diagnostics", error));
            Vec::new()
        }
    };
    let degraded = !integrity_issues.is_empty() || !load_diagnostics.is_empty();
    let integrity_error =
        integrity_issues.first().and_then(|issue| issue["error"].as_str()).map(str::to_string);

    json!({
        "status": if degraded { "degraded" } else { "ok" },
        "count": plugins.len(),
        "plugins": plugin_records_payload(&plugins),
        "directory": plugins_dir,
        "registry_file": plugin_registry_path,
        "state_counts": state_counts(&plugins),
        "compatibility_warnings": compatibility,
        "compatibility_warning_count": compatibility.len(),
        "load_diagnostics": load_diagnostics,
        "load_diagnostic_count": load_diagnostics.len(),
        "integrity_status": if degraded { "degraded" } else { "ok" },
        "integrity_error": integrity_error,
        "integrity_issues": integrity_issues,
    })
}

pub(crate) fn plugins_overview(plugin_registry_path: &Path, plugins_dir: &Path) -> Value {
    inventory_report(plugin_registry_path, plugins_dir)
}

pub(crate) fn plugins_list(plugin_registry_path: &Path, plugins_dir: &Path) -> Value {
    inventory_report(plugin_registry_path, plugins_dir)
}

pub(crate) fn plugins_info(plugin_registry_path: &Path, plugins_dir: &Path) -> Value {
    inventory_report(plugin_registry_path, plugins_dir)
}

pub(crate) fn plugins_inspect(plugin_registry_path: &Path, plugin: Option<&str>) -> Result<Value> {
    let mut integrity_issues = Vec::<Value>::new();
    let plugins = if let Some(reference) = plugin {
        vec![inspect_plugin(plugin_registry_path, reference)?]
    } else {
        match list_plugins(plugin_registry_path) {
            Ok(plugins) => plugins,
            Err(error) => {
                integrity_issues.push(integrity_issue("registry", error));
                Vec::new()
            }
        }
    };
    let requested_namespace =
        plugin.and_then(|_| plugins.first().map(|record| record.manifest.namespace.0.clone()));
    let compatibility = match compatibility_warnings(plugin_registry_path, runtime_semver()) {
        Ok(warnings) => warnings
            .into_iter()
            .filter(|warning| {
                requested_namespace
                    .as_deref()
                    .is_none_or(|namespace| warning.starts_with(&format!("{namespace}:")))
            })
            .collect::<Vec<_>>(),
        Err(error) => {
            integrity_issues.push(integrity_issue("compatibility", error));
            Vec::new()
        }
    };
    let load_diagnostics = match load_time_diagnostics(plugin_registry_path, runtime_semver()) {
        Ok(diagnostics) => render_load_diagnostics(diagnostics.into_iter().filter(|diag| {
            requested_namespace.as_deref().is_none_or(|reference| diag.namespace == reference)
        })),
        Err(error) => {
            integrity_issues.push(integrity_issue("load-time-diagnostics", error));
            Vec::new()
        }
    };

    let has_runtime_issues = !load_diagnostics.is_empty();
    Ok(json!({
        "plugin": plugin,
        "count": plugins.len(),
        "plugins": plugin_records_payload(&plugins),
        "status": if integrity_issues.is_empty() && !has_runtime_issues { "loaded" } else { "degraded" },
        "registry_file": plugin_registry_path,
        "state_counts": state_counts(&plugins),
        "compatibility_warnings": compatibility,
        "compatibility_warning_count": compatibility.len(),
        "load_diagnostics": load_diagnostics,
        "load_diagnostic_count": load_diagnostics.len(),
        "integrity_status": if integrity_issues.is_empty() && !has_runtime_issues { "ok" } else { "degraded" },
        "integrity_issues": integrity_issues,
    }))
}

pub(crate) fn check_plugin_health(plugin_registry_path: &Path, plugin: &str) -> Result<Value> {
    let record = inspect_plugin(plugin_registry_path, plugin)?;
    let _ = validate_manifest(record.manifest.clone(), runtime_semver(), RESERVED_NAMESPACES)?;

    if matches!(record.state, PluginLifecycleState::Disabled) {
        anyhow::bail!("Invalid argument: plugin {plugin} is disabled");
    }
    let current_diagnostics = load_time_diagnostics(plugin_registry_path, runtime_semver())?;
    if let Some(diag) =
        current_diagnostics.into_iter().find(|diag| diag.namespace == record.manifest.namespace.0)
    {
        anyhow::bail!("Invalid argument: {}", diag.message);
    }

    Ok(json!({
        "status": "healthy",
        "state": lifecycle_state_name(record.state),
        "trust_level": record.trust_level,
        "source": record.source,
        "aliases": record.manifest.aliases,
        "manifest_path": record.manifest_path,
        "reference": namespace_reference_payload(plugin, &record.manifest.namespace.0),
    }))
}

pub(crate) fn scaffold_plugin(
    kind: &str,
    namespace: &str,
    force: bool,
    target: &Path,
) -> Result<Value> {
    let manifest = scaffold_plugin_layout(target, kind, namespace, force)?;
    Ok(json!({
        "status": "scaffolded",
        "kind": kind,
        "namespace": namespace,
        "path": target,
        "manifest": manifest,
    }))
}

pub(crate) fn install_plugin_from_manifest(
    plugin_registry_path: &Path,
    manifest_path: &Path,
    source: Option<&str>,
    trust_level: PluginTrustLevel,
) -> Result<Value> {
    let manifest_path = resolve_install_manifest_path(manifest_path)?;
    let manifest_text = fs::read_to_string(&manifest_path)?;
    let source = source
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| manifest_path.to_string_lossy().into_owned());
    let manifest_path_text = Some(
        manifest_path
            .canonicalize()
            .unwrap_or_else(|_| manifest_path.to_path_buf())
            .to_string_lossy()
            .into_owned(),
    );

    let installed = install_plugin_manifest(
        plugin_registry_path,
        InstallPluginRequest {
            manifest_text,
            source,
            manifest_path: manifest_path_text.clone(),
            trust_level,
        },
        runtime_semver(),
    )?;

    Ok(json!({
        "status": "installed",
        "manifest_path": manifest_path_text,
        "plugin": plugin_record_payload(&installed),
    }))
}

pub(crate) fn uninstall_plugin_namespace(
    plugin_registry_path: &Path,
    namespace: &str,
) -> Result<Value> {
    let record = inspect_plugin(plugin_registry_path, namespace)?;
    uninstall_plugin(plugin_registry_path, namespace)?;
    Ok(json!({
        "status": "uninstalled",
        "reference": namespace_reference_payload(namespace, &record.manifest.namespace.0),
    }))
}

pub(crate) fn enable_plugin_namespace(
    plugin_registry_path: &Path,
    namespace: &str,
) -> Result<Value> {
    let record = inspect_plugin(plugin_registry_path, namespace)?;
    let _ = validate_manifest(record.manifest.clone(), runtime_semver(), RESERVED_NAMESPACES)?;
    let current_diagnostics = load_time_diagnostics(plugin_registry_path, runtime_semver())?;
    if let Some(diag) =
        current_diagnostics.into_iter().find(|diag| diag.namespace == record.manifest.namespace.0)
    {
        anyhow::bail!(
            "Invalid argument: cannot enable plugin with current runtime issue: {}",
            diag.message
        );
    }
    let record = enable_plugin(plugin_registry_path, namespace)?;
    Ok(json!({
        "status": "enabled",
        "state": lifecycle_state_name(record.state),
        "reference": namespace_reference_payload(namespace, &record.manifest.namespace.0),
    }))
}

pub(crate) fn disable_plugin_namespace(
    plugin_registry_path: &Path,
    namespace: &str,
) -> Result<Value> {
    let record = disable_plugin(plugin_registry_path, namespace)?;
    Ok(json!({
        "status": "disabled",
        "state": lifecycle_state_name(record.state),
        "reference": namespace_reference_payload(namespace, &record.manifest.namespace.0),
    }))
}

pub(crate) fn plugin_doctor_report(plugin_registry_path: &Path) -> Result<Value> {
    let repaired = self_repair_registry(plugin_registry_path).is_ok();
    let report = plugin_doctor(plugin_registry_path)?;
    let diagnostics = load_time_diagnostics(plugin_registry_path, runtime_semver())?;

    Ok(json!({
        "status": if report.broken.is_empty() && report.incompatible.is_empty() { "ok" } else { "degraded" },
        "doctor": {
            "installed": report.installed,
            "broken": report.broken,
            "incompatible": report.incompatible,
        },
        "load_diagnostics": diagnostics.into_iter().map(|diag| {
            json!({
                "namespace": diag.namespace,
                "severity": diag.severity,
                "message": diag.message,
            })
        }).collect::<Vec<_>>(),
        "self_repair_attempted": true,
        "self_repair_success": repaired,
    }))
}

pub(crate) fn reserved_namespaces_report() -> Value {
    let blocked_namespaces = blocked_namespace_inventory(&[]);
    json!({
        "status": "ok",
        "blocked_namespaces": blocked_namespaces,
        "reserved_namespaces": RESERVED_NAMESPACES,
        "core_namespaces": CORE_NAMESPACES,
        "official_product_namespaces": official_product_namespaces(),
        "known_bijux_projects": known_bijux_tool_namespaces(),
        "alias_policy": {
            "namespace_rules_apply_to_aliases": true,
            "notes": [
                "plugins cannot claim built-in runtime command namespaces",
                "plugins cannot claim official product namespaces",
                "plugins cannot claim namespaces already owned by repository-managed Bijux tools"
            ]
        }
    })
}

pub(crate) fn plugin_locations_report(plugins_dir: &Path, plugin_registry_path: &Path) -> Value {
    json!({
        "plugins_dir": plugins_dir,
        "registry_file": plugin_registry_path,
    })
}

pub(crate) fn explain_plugin_report(
    plugin_registry_path: &Path,
    plugin: Option<&str>,
) -> Result<Value> {
    let mut integrity_issues = Vec::<Value>::new();
    let resolved_namespace = match plugin {
        Some(requested) if !is_reserved_namespace(requested, &[]) => {
            Some(inspect_plugin(plugin_registry_path, requested)?.manifest.namespace.0)
        }
        Some(requested) => Some(requested.to_string()),
        None => None,
    };
    let diagnostics = match load_time_diagnostics(plugin_registry_path, runtime_semver()) {
        Ok(diagnostics) => diagnostics,
        Err(error) => {
            integrity_issues.push(json!({
                "source": "load-time-diagnostics",
                "error": error.to_string(),
            }));
            Vec::new()
        }
    };
    let report = match plugin_doctor(plugin_registry_path) {
        Ok(report) => Some(report),
        Err(error) => {
            integrity_issues.push(json!({
                "source": "plugin-doctor",
                "error": error.to_string(),
            }));
            None
        }
    };

    let mut filtered: Vec<Value> = diagnostics
        .into_iter()
        .filter(|d| resolved_namespace.as_deref().is_none_or(|wanted| d.namespace == wanted))
        .map(|diag| {
            json!({
                "namespace": diag.namespace,
                "severity": diag.severity,
                "message": diag.message,
            })
        })
        .collect();

    if let Some(requested) = resolved_namespace.as_deref() {
        if is_reserved_namespace(requested, &[]) {
            filtered.push(json!({
                "namespace": requested,
                "severity": "error",
                "message": format!("namespace is reserved: {requested}"),
            }));
        }
    }

    let summary = report
        .map(|value| {
            json!({
                "installed": value.installed,
                "broken": value.broken,
                "incompatible": value.incompatible,
            })
        })
        .unwrap_or_else(|| json!({"installed": 0, "broken": [], "incompatible": []}));

    Ok(json!({
        "plugin": resolved_namespace,
        "diagnostics": filtered,
        "summary": summary,
        "integrity_status": if integrity_issues.is_empty() { "ok" } else { "degraded" },
        "integrity_issues": integrity_issues,
    }))
}

pub(crate) fn plugin_schema_report() -> Value {
    let schema = plugin_manifest_v2_schema();
    json!({
        "schema": "plugin-manifest-v2",
        "schema_json": schema,
    })
}
