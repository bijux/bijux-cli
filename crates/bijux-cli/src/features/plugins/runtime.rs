#![forbid(unsafe_code)]

use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use serde_json::Value;

use crate::contracts::{PluginKind, PluginLifecycleState};

use super::entrypoint::{installed_manifest_root, resolve_delegated_entrypoint};
use super::errors::PluginError;
use super::inspect_plugin;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PluginProcessResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PluginRouteOutput {
    Structured(Value),
    Process(PluginProcessResult),
}

#[derive(Debug, Deserialize)]
struct PythonBridgeEnvelope {
    result: Value,
}

const PYTHON_PLUGIN_BRIDGE: &str = r#"
import importlib
import json
import sys

manifest_root = sys.argv[1]
module_name = sys.argv[2]
callable_name = sys.argv[3]
argv = sys.argv[4:]

if manifest_root:
    sys.path.insert(0, manifest_root)

module = importlib.import_module(module_name)
target = module
for part in callable_name.split('.'):
    target = getattr(target, part)

result = target(argv)
json.dump({"result": result}, sys.stdout)
"#;

fn ensure_plugin_is_routable(namespace: &str, state: PluginLifecycleState) -> Result<()> {
    match state {
        PluginLifecycleState::Disabled => {
            anyhow::bail!("plugin `{namespace}` is disabled");
        }
        PluginLifecycleState::Broken => {
            anyhow::bail!("plugin `{namespace}` is broken");
        }
        PluginLifecycleState::Incompatible => {
            anyhow::bail!("plugin `{namespace}` is incompatible with current runtime");
        }
        _ => {}
    }

    Ok(())
}

fn plugin_forwarded_args(argv: &[String], route_root: &str) -> Vec<String> {
    argv.iter()
        .enumerate()
        .skip(1)
        .find(|(_, token)| token.eq_ignore_ascii_case(route_root))
        .map(|(index, _)| argv[index + 1..].to_vec())
        .unwrap_or_default()
}

fn delegated_module_and_callable(entrypoint: &str, kind: PluginKind) -> Result<(&str, &str)> {
    let (module_name, callable_name) =
        entrypoint.split_once(':').ok_or(PluginError::InvalidEntrypoint { kind })?;
    if module_name.trim().is_empty() || callable_name.trim().is_empty() {
        return Err(PluginError::InvalidEntrypoint { kind }.into());
    }
    Ok((module_name, callable_name))
}

fn resolve_python_interpreter() -> Option<&'static str> {
    ["python3.11", "python3", "python"]
        .into_iter()
        .find(|candidate| Command::new(candidate).arg("--version").output().is_ok())
}

fn render_process_result(output: std::process::Output) -> PluginProcessResult {
    PluginProcessResult {
        exit_code: output.status.code().unwrap_or(1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn run_python_plugin(
    namespace: &str,
    manifest_root: &Path,
    module_name: &str,
    callable_name: &str,
    forwarded_args: &[String],
) -> Result<PluginRouteOutput> {
    let python = resolve_python_interpreter()
        .ok_or_else(|| anyhow!("python interpreter is required to run plugin `{namespace}`"))?;
    let output = Command::new(python)
        .current_dir(manifest_root)
        .arg("-c")
        .arg(PYTHON_PLUGIN_BRIDGE)
        .arg(manifest_root)
        .arg(module_name)
        .arg(callable_name)
        .args(forwarded_args)
        .output()
        .with_context(|| format!("failed to launch python runtime for plugin `{namespace}`"))?;
    if !output.status.success() {
        return Ok(PluginRouteOutput::Process(render_process_result(output)));
    }

    let envelope: PythonBridgeEnvelope =
        serde_json::from_slice(&output.stdout).with_context(|| {
            format!("plugin `{namespace}` returned an invalid structured payload")
        })?;
    Ok(PluginRouteOutput::Structured(envelope.result))
}

fn run_external_plugin(
    namespace: &str,
    entrypoint_path: &Path,
    manifest_root: Option<&Path>,
    forwarded_args: &[String],
) -> Result<PluginRouteOutput> {
    let mut command = Command::new(entrypoint_path);
    if let Some(root) = manifest_root {
        command.current_dir(root);
    }
    let output = command
        .args(forwarded_args)
        .output()
        .with_context(|| format!("failed to launch external plugin `{namespace}`"))?;
    Ok(PluginRouteOutput::Process(render_process_result(output)))
}

pub(crate) fn execute_plugin_route(
    plugin_registry_path: &Path,
    namespace: &str,
    route_root: &str,
    argv: &[String],
) -> Result<PluginRouteOutput> {
    let record = inspect_plugin(plugin_registry_path, namespace)?;
    ensure_plugin_is_routable(namespace, record.state)?;
    let forwarded_args = plugin_forwarded_args(argv, route_root);

    match record.manifest.kind {
        PluginKind::Delegated | PluginKind::Python => {
            let _ = resolve_delegated_entrypoint(
                record.manifest_path.as_deref(),
                &record.source,
                &record.manifest.entrypoint,
            )
            .ok_or_else(|| {
                let manifest_root =
                    installed_manifest_root(record.manifest_path.as_deref(), &record.source);
                let fallback_path =
                    manifest_root.unwrap_or_default().join(record.manifest.entrypoint.clone());
                PluginError::MissingEntrypointPath {
                    kind: record.manifest.kind,
                    path: fallback_path,
                }
            })?;
            let manifest_root =
                installed_manifest_root(record.manifest_path.as_deref(), &record.source)
                    .ok_or(PluginError::InvalidEntrypoint { kind: record.manifest.kind })?;
            let (module_name, callable_name) =
                delegated_module_and_callable(&record.manifest.entrypoint, record.manifest.kind)?;
            run_python_plugin(
                namespace,
                &manifest_root,
                module_name,
                callable_name,
                &forwarded_args,
            )
        }
        PluginKind::ExternalExec => {
            let manifest_root =
                installed_manifest_root(record.manifest_path.as_deref(), &record.source);
            let entrypoint_path = super::resolve_external_exec_entrypoint(
                record.manifest_path.as_deref(),
                &record.source,
                &record.manifest.entrypoint,
            );
            run_external_plugin(
                namespace,
                &entrypoint_path,
                manifest_root.as_deref(),
                &forwarded_args,
            )
        }
        PluginKind::Native => Err(PluginError::UnsupportedKind(PluginKind::Native).into()),
    }
}
