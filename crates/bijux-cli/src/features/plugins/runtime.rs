#![forbid(unsafe_code)]

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PythonInterpreterStatus {
    pub command: String,
    pub version: String,
    pub supported: bool,
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

const DEFAULT_PLUGIN_TIMEOUT_SECONDS: u64 = 30;
const MAX_PLUGIN_TIMEOUT_SECONDS: u64 = 600;

fn plugin_timeout() -> Duration {
    let raw = std::env::var("BIJUX_PLUGIN_TIMEOUT_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(DEFAULT_PLUGIN_TIMEOUT_SECONDS)
        .clamp(1, MAX_PLUGIN_TIMEOUT_SECONDS);
    Duration::from_secs(raw)
}

fn apply_plugin_env_policy(command: &mut Command) {
    const PASSTHROUGH_KEYS: &[&str] = &[
        "PATH",
        "HOME",
        "USER",
        "SHELL",
        "TMPDIR",
        "TEMP",
        "TMP",
        "SYSTEMROOT",
        "COMSPEC",
        "WINDIR",
        "LANG",
        "LC_ALL",
        "LC_CTYPE",
        "LC_MESSAGES",
    ];

    command.env_clear();
    for key in PASSTHROUGH_KEYS {
        if let Ok(value) = std::env::var(key) {
            command.env(key, value);
        }
    }
    for (key, value) in std::env::vars() {
        if key.starts_with("BIJUX_") || key.starts_with("PYTHON") {
            command.env(key, value);
        }
    }
}

fn run_command_with_timeout(
    command: &mut Command,
    timeout: Duration,
    timeout_message: String,
) -> Result<PluginProcessResult> {
    let mut child =
        command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    let start = Instant::now();

    loop {
        if child.try_wait()?.is_some() {
            let output = child.wait_with_output()?;
            return Ok(render_process_result(output));
        }
        if start.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(PluginProcessResult {
                exit_code: 124,
                stdout: String::new(),
                stderr: timeout_message,
            });
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

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

fn parse_python_version(text: &str) -> Option<(u64, u64)> {
    let raw = text.trim().strip_prefix("Python ")?;
    let mut parts = raw.split('.');
    let major = parts.next()?.trim().parse().ok()?;
    let minor = parts.next()?.trim().parse().ok()?;
    Some((major, minor))
}

pub(crate) fn detected_python_interpreters() -> Vec<PythonInterpreterStatus> {
    ["python3.11", "python3", "python"]
        .into_iter()
        .filter_map(|candidate| {
            let output = Command::new(candidate).arg("--version").output().ok()?;
            let version = if output.stdout.is_empty() {
                String::from_utf8_lossy(&output.stderr).trim().to_string()
            } else {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            };
            let supported = parse_python_version(&version)
                .is_some_and(|(major, minor)| major > 3 || (major == 3 && minor >= 11));
            Some(PythonInterpreterStatus { command: candidate.to_string(), version, supported })
        })
        .collect()
}

pub(crate) fn resolve_python_interpreter() -> Option<PythonInterpreterStatus> {
    detected_python_interpreters().into_iter().find(|candidate| candidate.supported)
}

fn python_runtime_error(namespace: &str) -> anyhow::Error {
    if let Some(found) = detected_python_interpreters().into_iter().next() {
        return anyhow!(
            "python 3.11 or newer is required to run plugin `{namespace}`; found {} ({})",
            found.command,
            found.version
        );
    }

    anyhow!("python 3.11 or newer is required to run plugin `{namespace}`")
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
    let python = resolve_python_interpreter().ok_or_else(|| python_runtime_error(namespace))?;
    let timeout = plugin_timeout();
    let mut command = Command::new(&python.command);
    command
        .current_dir(manifest_root)
        .arg("-c")
        .arg(PYTHON_PLUGIN_BRIDGE)
        .arg(manifest_root)
        .arg(module_name)
        .arg(callable_name)
        .args(forwarded_args);
    apply_plugin_env_policy(&mut command);
    let output = run_command_with_timeout(
        &mut command,
        timeout,
        format!("plugin `{namespace}` timed out after {} seconds", timeout.as_secs()),
    )
    .with_context(|| format!("failed to launch python runtime for plugin `{namespace}`"))?;
    if output.exit_code != 0 {
        return Ok(PluginRouteOutput::Process(output));
    }

    let envelope: PythonBridgeEnvelope = serde_json::from_str(&output.stdout)
        .with_context(|| format!("plugin `{namespace}` returned an invalid structured payload"))?;
    Ok(PluginRouteOutput::Structured(envelope.result))
}

fn run_external_plugin(
    namespace: &str,
    entrypoint_path: &Path,
    manifest_root: Option<&Path>,
    forwarded_args: &[String],
) -> Result<PluginRouteOutput> {
    let timeout = plugin_timeout();
    let mut command = Command::new(entrypoint_path);
    if let Some(root) = manifest_root {
        command.current_dir(root);
    }
    command.args(forwarded_args);
    apply_plugin_env_policy(&mut command);
    let output = run_command_with_timeout(
        &mut command,
        timeout,
        format!("plugin `{namespace}` timed out after {} seconds", timeout.as_secs()),
    )
    .with_context(|| format!("failed to launch external plugin `{namespace}`"))?;
    Ok(PluginRouteOutput::Process(output))
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

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

    use super::{plugin_timeout, DEFAULT_PLUGIN_TIMEOUT_SECONDS, MAX_PLUGIN_TIMEOUT_SECONDS};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn plugin_timeout_uses_default_when_env_is_absent() {
        let _guard = env_lock().lock().expect("lock");
        std::env::remove_var("BIJUX_PLUGIN_TIMEOUT_SECONDS");
        assert_eq!(plugin_timeout().as_secs(), DEFAULT_PLUGIN_TIMEOUT_SECONDS);
    }

    #[test]
    fn plugin_timeout_clamps_invalid_or_large_values() {
        let _guard = env_lock().lock().expect("lock");
        std::env::set_var("BIJUX_PLUGIN_TIMEOUT_SECONDS", "0");
        assert_eq!(plugin_timeout().as_secs(), 1);
        std::env::set_var("BIJUX_PLUGIN_TIMEOUT_SECONDS", "999999");
        assert_eq!(plugin_timeout().as_secs(), MAX_PLUGIN_TIMEOUT_SECONDS);
        std::env::remove_var("BIJUX_PLUGIN_TIMEOUT_SECONDS");
    }
}
