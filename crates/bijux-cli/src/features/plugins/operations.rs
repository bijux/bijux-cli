#![forbid(unsafe_code)]
//! Plugin feature operations exposed to command adapters.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde_json::{json, Value};

use crate::contracts::{PluginKind, PluginLifecycleState};
use crate::features::plugins::{
    compatibility_warnings, disable_plugin, enable_plugin, inspect_plugin,
    install_plugin as install_plugin_manifest, is_reserved_namespace, list_plugins,
    load_time_diagnostics, plugin_doctor, scaffold::scaffold_plugin_layout, self_repair_registry,
    uninstall_plugin, validate_manifest, InstallPluginRequest, PluginTrustLevel, CORE_NAMESPACES,
    KNOWN_BIJUX_PROJECT_NAMESPACES, RESERVED_NAMESPACES,
};

pub(crate) fn plugins_overview(plugin_registry_path: &Path, plugins_dir: &Path) -> Value {
    let plugins = list_plugins(plugin_registry_path).unwrap_or_default();
    json!({
        "status": "ok",
        "count": plugins.len(),
        "plugins": plugins,
        "directory": plugins_dir,
    })
}

pub(crate) fn plugins_list(plugin_registry_path: &Path, plugins_dir: &Path) -> Value {
    json!({
        "plugins": list_plugins(plugin_registry_path).unwrap_or_default(),
        "directory": plugins_dir,
    })
}

pub(crate) fn plugins_info(plugin_registry_path: &Path) -> Value {
    let plugins = list_plugins(plugin_registry_path).unwrap_or_default();
    let warnings =
        compatibility_warnings(plugin_registry_path, env!("CARGO_PKG_VERSION")).unwrap_or_default();

    json!({
        "status": "ok",
        "plugins": plugins,
        "compatibility_warnings": warnings,
        "registry_file": plugin_registry_path,
    })
}

pub(crate) fn plugins_inspect(plugin_registry_path: &Path) -> Value {
    json!({
        "plugins": list_plugins(plugin_registry_path).unwrap_or_default(),
        "status": "loaded",
        "compatibility_warnings": compatibility_warnings(plugin_registry_path, env!("CARGO_PKG_VERSION")).unwrap_or_default(),
    })
}

pub(crate) fn check_plugin_health(plugin_registry_path: &Path, plugin: &str) -> Result<Value> {
    let record = inspect_plugin(plugin_registry_path, plugin)?;
    let _ =
        validate_manifest(record.manifest.clone(), env!("CARGO_PKG_VERSION"), RESERVED_NAMESPACES)?;

    if matches!(record.state, PluginLifecycleState::Disabled) {
        anyhow::bail!("Invalid argument: plugin {plugin} is disabled");
    }

    if matches!(record.manifest.kind, PluginKind::ExternalExec) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let path = PathBuf::from(&record.manifest.entrypoint);
            if !path.exists() {
                anyhow::bail!("Invalid argument: plugin entrypoint was not found");
            }
            let mode = fs::metadata(&path)?.permissions().mode();
            if mode & 0o111 == 0 {
                anyhow::bail!("Invalid argument: plugin entrypoint is not executable");
            }
        }
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
        env!("CARGO_PKG_VERSION"),
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
    let diagnostics =
        load_time_diagnostics(plugin_registry_path, env!("CARGO_PKG_VERSION")).unwrap_or_default();
    let report = plugin_doctor(plugin_registry_path).ok();

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
