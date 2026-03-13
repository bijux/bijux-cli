#![forbid(unsafe_code)]
//! Plugin feature operations exposed to command adapters.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde_json::{json, Value};

use crate::api::version::runtime_semver;
use crate::contracts::{PluginKind, PluginLifecycleState};
use crate::features::plugins::{
    compatibility_warnings, disable_plugin, enable_plugin, inspect_plugin,
    install_plugin as install_plugin_manifest, is_reserved_namespace, list_plugins,
    load_time_diagnostics, plugin_doctor, resolve_delegated_entrypoint,
    resolve_external_exec_entrypoint, scaffold::scaffold_plugin_layout, self_repair_registry,
    uninstall_plugin, validate_manifest, InstallPluginRequest, PluginTrustLevel, CORE_NAMESPACES,
    KNOWN_BIJUX_PROJECT_NAMESPACES, RESERVED_NAMESPACES,
};

fn missing_delegated_entrypoint(
    record: &crate::features::plugins::PluginRecord,
) -> Option<PathBuf> {
    if resolve_delegated_entrypoint(&record.source, &record.manifest.entrypoint).is_some() {
        return None;
    }
    crate::features::plugins::delegated_entrypoint_candidates(
        crate::features::plugins::installed_manifest_root(&record.source)?,
        &record.manifest.entrypoint,
    )
    .into_iter()
    .next()
}

pub(crate) fn plugins_overview(plugin_registry_path: &Path, plugins_dir: &Path) -> Value {
    match list_plugins(plugin_registry_path) {
        Ok(plugins) => json!({
            "status": "ok",
            "count": plugins.len(),
            "plugins": plugins,
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
            "plugins": plugins,
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

    json!({
        "status": if integrity_issues.is_empty() { "ok" } else { "degraded" },
        "plugins": plugins,
        "compatibility_warnings": warnings,
        "registry_file": plugin_registry_path,
        "integrity_status": if integrity_issues.is_empty() { "ok" } else { "degraded" },
        "integrity_issues": integrity_issues,
    })
}

pub(crate) fn plugins_inspect(plugin_registry_path: &Path) -> Value {
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
    let compatibility = match compatibility_warnings(plugin_registry_path, runtime_semver()) {
        Ok(warnings) => warnings,
        Err(error) => {
            integrity_issues.push(json!({
                "source": "compatibility",
                "error": error.to_string(),
            }));
            Vec::new()
        }
    };

    json!({
        "plugins": plugins,
        "status": if integrity_issues.is_empty() { "loaded" } else { "degraded" },
        "compatibility_warnings": compatibility,
        "integrity_status": if integrity_issues.is_empty() { "ok" } else { "degraded" },
        "integrity_issues": integrity_issues,
    })
}

pub(crate) fn check_plugin_health(plugin_registry_path: &Path, plugin: &str) -> Result<Value> {
    let record = inspect_plugin(plugin_registry_path, plugin)?;
    let _ = validate_manifest(record.manifest.clone(), runtime_semver(), RESERVED_NAMESPACES)?;

    if matches!(record.state, PluginLifecycleState::Disabled) {
        anyhow::bail!("Invalid argument: plugin {plugin} is disabled");
    }

    if matches!(record.manifest.kind, PluginKind::ExternalExec) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let path =
                resolve_external_exec_entrypoint(&record.source, &record.manifest.entrypoint);
            if !path.exists() {
                anyhow::bail!("Invalid argument: plugin entrypoint was not found");
            }
            let mode = fs::metadata(&path)?.permissions().mode();
            if mode & 0o111 == 0 {
                anyhow::bail!("Invalid argument: plugin entrypoint is not executable");
            }
        }
    }

    if matches!(record.manifest.kind, PluginKind::Delegated | PluginKind::Python)
        && missing_delegated_entrypoint(&record).is_some()
    {
        anyhow::bail!("Invalid argument: plugin entrypoint was not found");
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

    let installed = install_plugin_manifest(
        plugin_registry_path,
        InstallPluginRequest { manifest_text, source, trust_level },
        runtime_semver(),
    )?;

    Ok(json!({
        "status": "installed",
        "plugin": installed,
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

    Ok(json!({
        "status": "ok",
        "doctor": {
            "installed": report.installed,
            "broken": report.broken,
            "incompatible": report.incompatible,
        },
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

pub(crate) fn explain_plugin_report(plugin_registry_path: &Path, plugin: Option<&str>) -> Value {
    let mut integrity_issues = Vec::<Value>::new();
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
        .filter(|d| plugin.is_none_or(|wanted| d.namespace == wanted))
        .map(|diag| {
            json!({
                "namespace": diag.namespace,
                "severity": diag.severity,
                "message": diag.message,
            })
        })
        .collect();

    if let Some(requested) = plugin {
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

    json!({
        "plugin": plugin,
        "diagnostics": filtered,
        "summary": summary,
        "integrity_status": if integrity_issues.is_empty() { "ok" } else { "degraded" },
        "integrity_issues": integrity_issues,
    })
}

pub(crate) fn plugin_schema_report() -> Value {
    json!({
        "schema": "plugin-manifest-v1",
        "required_fields": [
            "name",
            "version",
            "schema_version",
            "manifest_version",
            "compatibility",
            "namespace",
            "kind",
            "entrypoint",
        ],
        "optional_fields": ["aliases", "capabilities"],
    })
}
