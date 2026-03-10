#![forbid(unsafe_code)]
//! Python bridge API bindings into the canonical Rust application entrypoint.

use std::collections::HashMap;
use std::fs;
use std::hash::BuildHasher;
use std::path::Path;

use bijux_cli::app::{run_app, AppRunResult};
use bijux_cli::routing::ContractMarker;
use serde_json::{json, Value};

use crate::compatibility::{
    default_compatibility_paths, discover_compatibility_paths, CompatibilityConfig,
    CompatibilityError, PathOverrides,
};
use crate::conversions::{classify_core_error, classify_failure, python_exception_tag};

/// Build python-bridge marker.
#[must_use]
pub fn python_bridge_marker() -> ContractMarker {
    ContractMarker { namespace: "python-bridge".to_string() }
}

/// Return command tree introspection payload as JSON.
#[must_use]
pub fn command_tree_introspection_api() -> String {
    let argv = vec![
        "bijux".to_string(),
        "inspect".to_string(),
        "--format".to_string(),
        "json".to_string(),
        "--no-pretty".to_string(),
    ];
    if let Ok(result) = run_app(&argv) {
        if result.exit_code == 0 {
            if let Ok(payload) = serde_json::from_str::<Value>(&result.stdout) {
                let mut namespaces = payload
                    .get("builtins")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|entry| entry.get("segments"))
                    .filter_map(Value::as_array)
                    .filter_map(|segments| segments.first())
                    .filter_map(Value::as_str)
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();
                namespaces.sort();
                namespaces.dedup();
                return json!({
                    "root": "bijux",
                    "namespaces": namespaces,
                })
                .to_string();
            }
        }
    }
    json!({
        "root": "bijux",
        "namespaces": ["cli", "dev", "help", "version", "doctor", "repl", "plugins", "completion", "inspect"],
    })
    .to_string()
}

/// Execute the Rust-backed CLI facade through the canonical runtime entrypoint.
pub fn execution_facade_api(argv: &[String]) -> Result<String, CompatibilityError> {
    match run_app(argv) {
        Ok(result) => Ok(select_primary_stream(&result)),
        Err(error) => Ok(json!({
            "status": "error",
            "error": {
                "kind": python_exception_tag(classify_core_error(&error)),
                "message": error.to_string()
            }
        })
        .to_string()),
    }
}

/// Return execution outcome with full stream context.
pub fn execution_outcome_api(argv: &[String]) -> Result<String, CompatibilityError> {
    match run_app(argv) {
        Ok(result) => Ok(json!({
            "exit_code": result.exit_code,
            "stdout": result.stdout,
            "stderr": result.stderr,
            "error_kind": python_exception_tag(classify_failure(result.exit_code, &result.stderr)),
        })
        .to_string()),
        Err(error) => Ok(json!({
            "exit_code": 1,
            "stdout": "",
            "stderr": error.to_string(),
            "error_kind": python_exception_tag(classify_core_error(&error)),
        })
        .to_string()),
    }
}

/// Execute `version` through the bridge.
pub fn version_binding_api() -> Result<String, CompatibilityError> {
    execution_facade_api(&["bijux".to_string(), "version".to_string()])
}

/// Execute `doctor` through the bridge.
pub fn doctor_binding_api() -> Result<String, CompatibilityError> {
    execution_facade_api(&["bijux".to_string(), "doctor".to_string()])
}

/// Execute `status` through the bridge.
pub fn status_binding_api() -> Result<String, CompatibilityError> {
    execution_facade_api(&["bijux".to_string(), "status".to_string()])
}

/// Execute `cli status` through the bridge.
pub fn cli_status_binding_api() -> Result<String, CompatibilityError> {
    execution_facade_api(&["bijux".to_string(), "cli".to_string(), "status".to_string()])
}

/// Execute `plugins list` through the bridge.
pub fn plugins_list_binding_api() -> Result<String, CompatibilityError> {
    execution_facade_api(&[
        "bijux".to_string(),
        "cli".to_string(),
        "plugins".to_string(),
        "list".to_string(),
    ])
}

/// Execute `repl --help` through the bridge.
pub fn repl_bootstrap_binding_api() -> Result<String, CompatibilityError> {
    execution_facade_api(&["bijux".to_string(), "repl".to_string(), "--help".to_string()])
}

/// Export known schema helpers for Python wrappers.
#[must_use]
pub fn schema_export_helpers_api() -> String {
    json!({
        "schemas": ["output-envelope-v1", "error-envelope-v1", "plugin-manifest-v1"],
    })
    .to_string()
}

/// Resolve compatibility paths and return JSON payload for Python consumers.
pub fn config_resolution_api(
    home_dir: Option<&Path>,
    cli_overrides: &PathOverrides,
    env_map: &HashMap<String, String, impl BuildHasher>,
    file_config: &CompatibilityConfig,
) -> Result<String, CompatibilityError> {
    let resolved = discover_compatibility_paths(home_dir, cli_overrides, env_map, file_config)?;
    Ok(json!({
        "config_file": resolved.config_file,
        "history_file": resolved.history_file,
        "plugins_dir": resolved.plugins_dir,
    })
    .to_string())
}

/// Return install-path helpers as JSON.
#[must_use]
pub fn install_path_helpers_api(home_dir: &Path) -> String {
    let defaults = default_compatibility_paths(home_dir);
    json!({
        "config_file": defaults.config_file,
        "history_file": defaults.history_file,
        "plugins_dir": defaults.plugins_dir,
    })
    .to_string()
}

/// Return plugin registry inspection payload as JSON.
pub fn plugin_registry_inspection_api(registry_path: &Path) -> Result<String, CompatibilityError> {
    if !registry_path.exists() {
        return Ok("{\"version\":\"1\",\"plugins\":{}}".to_string());
    }
    let text = fs::read_to_string(registry_path)?;
    Ok(text)
}

fn select_primary_stream(result: &AppRunResult) -> String {
    if result.exit_code == 0 {
        result.stdout.clone()
    } else if !result.stderr.is_empty() {
        result.stderr.clone()
    } else {
        result.stdout.clone()
    }
}
