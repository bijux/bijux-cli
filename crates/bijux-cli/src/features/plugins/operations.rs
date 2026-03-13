#![forbid(unsafe_code)]
//! Plugin feature operations exposed to command adapters.

use std::fs;
use std::path::Path;

use anyhow::Result;
use serde_json::{json, Value};

use crate::api::version::runtime_semver;
use crate::contracts::plugin_manifest_v2_schema;
use crate::contracts::PluginLifecycleState;
use crate::features::plugins::{
    compatibility_warnings, disable_plugin, enable_plugin, inspect_plugin,
    install_plugin as install_plugin_manifest, is_reserved_namespace, list_plugins,
    load_time_diagnostics, plugin_doctor, scaffold::scaffold_plugin_layout, self_repair_registry,
    uninstall_plugin, validate_manifest, InstallPluginRequest, PluginTrustLevel, CORE_NAMESPACES,
    KNOWN_BIJUX_PROJECT_NAMESPACES, RESERVED_NAMESPACES,
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

pub(crate) fn plugins_overview(plugin_registry_path: &Path, plugins_dir: &Path) -> Value {
    match list_plugins(plugin_registry_path) {
        Ok(plugins) => json!({
            "status": "ok",
            "count": plugins.len(),
            "plugins": plugin_records_payload(&plugins),
            "directory": plugins_dir,
            "integrity_status": "ok",
        }),
        Err(error) => json!({
            "status": "degraded",
            "count": 0,
            "plugins": [],
            "directory": plugins_dir,
            "integrity_status": "degraded",
            "integrity_error": error.to_string(),
        }),
    }
}

pub(crate) fn plugins_list(plugin_registry_path: &Path, plugins_dir: &Path) -> Value {
    match list_plugins(plugin_registry_path) {
        Ok(plugins) => json!({
            "plugins": plugin_records_payload(&plugins),
            "directory": plugins_dir,
            "integrity_status": "ok",
        }),
        Err(error) => json!({
            "plugins": [],
            "directory": plugins_dir,
            "integrity_status": "degraded",
            "integrity_error": error.to_string(),
        }),
    }
}

pub(crate) fn plugins_info(plugin_registry_path: &Path) -> Value {
    let mut integrity_issues = Vec::<Value>::new();
    let plugins = match list_plugins(plugin_registry_path) {
        Ok(plugins) => plugins,
        Err(error) => {
            integrity_issues.push(json!({
                "source": "registry",
                "error": error.to_string(),
            }));
            Vec::new()
        }
    };
    let warnings = match compatibility_warnings(plugin_registry_path, runtime_semver()) {
        Ok(warnings) => warnings,
        Err(error) => {
            integrity_issues.push(json!({
                "source": "compatibility",
                "error": error.to_string(),
            }));
            Vec::new()
        }
    };
    let load_diagnostics = match load_time_diagnostics(plugin_registry_path, runtime_semver()) {
        Ok(diagnostics) => diagnostics
            .into_iter()
            .map(|diag| {
                json!({
                    "namespace": diag.namespace,
                    "severity": diag.severity,
                    "message": diag.message,
                })
            })
            .collect::<Vec<_>>(),
        Err(error) => {
            integrity_issues.push(json!({
                "source": "load-time-diagnostics",
                "error": error.to_string(),
            }));
            Vec::new()
        }
    };

    let has_runtime_issues = !load_diagnostics.is_empty();
    json!({
        "status": if integrity_issues.is_empty() && !has_runtime_issues { "ok" } else { "degraded" },
        "plugins": plugin_records_payload(&plugins),
        "compatibility_warnings": warnings,
        "load_diagnostics": load_diagnostics,
        "registry_file": plugin_registry_path,
        "integrity_status": if integrity_issues.is_empty() && !has_runtime_issues { "ok" } else { "degraded" },
        "integrity_issues": integrity_issues,
    })
}

pub(crate) fn plugins_inspect(plugin_registry_path: &Path, plugin: Option<&str>) -> Result<Value> {
    let mut integrity_issues = Vec::<Value>::new();
    let plugins = if let Some(reference) = plugin {
        vec![inspect_plugin(plugin_registry_path, reference)?]
    } else {
        match list_plugins(plugin_registry_path) {
            Ok(plugins) => plugins,
            Err(error) => {
                integrity_issues.push(json!({
                    "source": "registry",
                    "error": error.to_string(),
                }));
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
            integrity_issues.push(json!({
                "source": "compatibility",
                "error": error.to_string(),
            }));
            Vec::new()
        }
    };
    let load_diagnostics = match load_time_diagnostics(plugin_registry_path, runtime_semver()) {
        Ok(diagnostics) => diagnostics
            .into_iter()
            .filter(|diag| {
                requested_namespace.as_deref().is_none_or(|reference| diag.namespace == reference)
            })
            .map(|diag| {
                json!({
                    "namespace": diag.namespace,
                    "severity": diag.severity,
                    "message": diag.message,
                })
            })
            .collect::<Vec<_>>(),
        Err(error) => {
            integrity_issues.push(json!({
                "source": "load-time-diagnostics",
                "error": error.to_string(),
            }));
            Vec::new()
        }
    };

    let has_runtime_issues = !load_diagnostics.is_empty();
    Ok(json!({
        "plugin": plugin,
        "plugins": plugin_records_payload(&plugins),
        "status": if integrity_issues.is_empty() && !has_runtime_issues { "loaded" } else { "degraded" },
        "compatibility_warnings": compatibility,
        "load_diagnostics": load_diagnostics,
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

    Ok(json!({"plugin": plugin, "status": "healthy", "state": format!("{:?}", record.state)}))
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
    let manifest_text = fs::read_to_string(manifest_path)?;
    let source = source
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| manifest_path.to_string_lossy().into_owned());
    let manifest_path = Some(
        manifest_path
            .canonicalize()
            .unwrap_or_else(|_| manifest_path.to_path_buf())
            .to_string_lossy()
            .into_owned(),
    );

    let installed = install_plugin_manifest(
        plugin_registry_path,
        InstallPluginRequest { manifest_text, source, manifest_path, trust_level },
        runtime_semver(),
    )?;

    Ok(json!({
        "status": "installed",
        "plugin": plugin_record_payload(&installed),
    }))
}

pub(crate) fn uninstall_plugin_namespace(
    plugin_registry_path: &Path,
    namespace: &str,
) -> Result<Value> {
    uninstall_plugin(plugin_registry_path, namespace)?;
    Ok(json!({
        "status": "uninstalled",
        "namespace": namespace,
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
        "namespace": namespace,
        "state": format!("{:?}", record.state),
    }))
}

pub(crate) fn disable_plugin_namespace(
    plugin_registry_path: &Path,
    namespace: &str,
) -> Result<Value> {
    let record = disable_plugin(plugin_registry_path, namespace)?;
    Ok(json!({
        "status": "disabled",
        "namespace": namespace,
        "state": format!("{:?}", record.state),
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
    json!({
        "reserved_namespaces": RESERVED_NAMESPACES,
        "core_namespaces": CORE_NAMESPACES,
        "known_bijux_projects": KNOWN_BIJUX_PROJECT_NAMESPACES,
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
