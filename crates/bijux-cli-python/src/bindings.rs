#![forbid(unsafe_code)]
//! Python bridge API bindings into the canonical Rust application entrypoint.

use std::collections::HashMap;
use std::fs;
use std::hash::BuildHasher;
use std::io;
use std::path::Path;

use bijux_cli::api::runtime::{run_app, AppRunResult};
use bijux_cli::contracts::ContractMarker;
use serde::Serialize;
use serde_json::{json, Value};

use crate::compatibility::{
    default_compatibility_paths, discover_compatibility_paths, load_compatibility_config,
    CompatibilityConfig, CompatibilityError, PathOverrides,
};
use crate::conversions::{classify_core_error, classify_failure, python_exception_tag};

#[derive(Serialize)]
struct CommandTreePayload {
    root: &'static str,
    namespaces: Vec<String>,
    source: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    warning: Option<String>,
}

#[derive(Serialize)]
struct ExecutionOutcomePayload {
    exit_code: i32,
    stdout: String,
    stderr: String,
    error_kind: Option<&'static str>,
}

#[derive(Serialize)]
struct CompatibilityPathsPayload {
    config_file: std::path::PathBuf,
    history_file: std::path::PathBuf,
    plugins_dir: std::path::PathBuf,
}

#[derive(Serialize)]
struct MissingRegistryPayload {
    version: &'static str,
    plugins: std::collections::BTreeMap<String, Value>,
}

fn json_string<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("bridge payload serialization should not fail")
}

fn normalized_argv(argv: &[String]) -> Vec<String> {
    if matches!(argv.first().map(String::as_str), Some("bijux")) {
        return argv.to_vec();
    }

    let mut normalized = Vec::with_capacity(argv.len() + 1);
    normalized.push("bijux".to_string());
    normalized.extend(argv.iter().cloned());
    normalized
}

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
                return json_string(&CommandTreePayload {
                    root: "bijux",
                    namespaces,
                    source: "runtime-inspect",
                    warning: None,
                });
            }
            return json_string(&CommandTreePayload {
                root: "bijux",
                namespaces: Vec::new(),
                source: "fallback-empty",
                warning: Some("inspect output was not valid json".to_string()),
            });
        }
        return json_string(&CommandTreePayload {
            root: "bijux",
            namespaces: Vec::new(),
            source: "fallback-empty",
            warning: Some(format!(
                "inspect command failed: {}",
                if result.stderr.trim().is_empty() {
                    format!("exit {}", result.exit_code)
                } else {
                    result.stderr.trim().to_string()
                }
            )),
        });
    }
    json_string(&CommandTreePayload {
        root: "bijux",
        namespaces: Vec::new(),
        source: "fallback-empty",
        warning: Some("inspect command failed to execute".to_string()),
    })
}

/// Execute the Rust-backed CLI facade through the canonical runtime entrypoint.
pub fn execution_facade_api(argv: &[String]) -> Result<String, CompatibilityError> {
    let argv = normalized_argv(argv);
    let result = run_app(&argv).map_err(|error| {
        CompatibilityError::Io(io::Error::other(format!(
            "{}: {}",
            python_exception_tag(classify_core_error(&error)),
            error
        )))
    })?;
    if result.exit_code != 0 {
        return Err(command_execution_error(&result));
    }
    Ok(result.stdout)
}

/// Return execution outcome with full stream context.
pub fn execution_outcome_api(argv: &[String]) -> Result<String, CompatibilityError> {
    let argv = normalized_argv(argv);
    match run_app(&argv) {
        Ok(result) => {
            let error_kind = if result.exit_code == 0 {
                None
            } else {
                Some(python_exception_tag(classify_failure(result.exit_code, &result.stderr)))
            };
            Ok(json_string(&ExecutionOutcomePayload {
                exit_code: result.exit_code,
                stdout: result.stdout,
                stderr: result.stderr,
                error_kind,
            }))
        }
        Err(error) => Ok(json_string(&ExecutionOutcomePayload {
            exit_code: 1,
            stdout: String::new(),
            stderr: error.to_string(),
            error_kind: Some(python_exception_tag(classify_core_error(&error))),
        })),
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
    Ok(json_string(&CompatibilityPathsPayload {
        config_file: resolved.config_file,
        history_file: resolved.history_file,
        plugins_dir: resolved.plugins_dir,
    }))
}

/// Return install-path helpers as JSON.
#[must_use]
pub fn install_path_helpers_api(home_dir: &Path) -> String {
    let defaults = default_compatibility_paths(home_dir);
    json_string(&CompatibilityPathsPayload {
        config_file: defaults.config_file,
        history_file: defaults.history_file,
        plugins_dir: defaults.plugins_dir,
    })
}

/// Resolve config/history/plugins using runtime precedence without CLI overrides.
pub fn config_resolution_helpers_api(home_dir: &Path) -> Result<String, CompatibilityError> {
    let defaults = default_compatibility_paths(home_dir);
    let file_config = load_compatibility_config(&defaults.config_file)?;
    let env_map: HashMap<String, String> = std::env::vars().collect();
    let resolved = discover_compatibility_paths(
        Some(home_dir),
        &PathOverrides::default(),
        &env_map,
        &file_config,
    )?;
    Ok(json_string(&CompatibilityPathsPayload {
        config_file: resolved.config_file,
        history_file: resolved.history_file,
        plugins_dir: resolved.plugins_dir,
    }))
}

/// Return plugin registry inspection payload as JSON.
pub fn plugin_registry_inspection_api(registry_path: &Path) -> Result<String, CompatibilityError> {
    if !registry_path.exists() {
        return Ok(json_string(&MissingRegistryPayload {
            version: "1",
            plugins: std::collections::BTreeMap::new(),
        }));
    }
    let text = fs::read_to_string(registry_path)?;
    let parsed: Value = serde_json::from_str(&text).map_err(|error| {
        CompatibilityError::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid plugin registry json at {}: {error}", registry_path.display()),
        ))
    })?;
    Ok(json_string(&parsed))
}

fn command_execution_error(result: &AppRunResult) -> CompatibilityError {
    let exception_tag = python_exception_tag(classify_failure(result.exit_code, &result.stderr));
    let details = if !result.stderr.trim().is_empty() {
        result.stderr.trim().to_string()
    } else if !result.stdout.trim().is_empty() {
        result.stdout.trim().to_string()
    } else {
        format!("command exited with status {}", result.exit_code)
    };
    CompatibilityError::Io(io::Error::other(format!(
        "{exception_tag}: exit_code={}; {details}",
        result.exit_code
    )))
}
