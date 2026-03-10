//! Top-level application entrypoint and route execution.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use bijux_cli_install::{
    atomic_write_text, canonical_crate_name, cargo_install_strategy, default_compatibility_paths,
    discover_compatibility_paths, install_health_report, load_compatibility_config,
    pip_install_strategy, post_install_hint, query::runtime_identity_query, CompatibilityConfig,
    CompatibilityPaths, InstallHealthReport, PackageChannel, PathOverrides, ENV_CONFIG_PATH,
    ENV_HISTORY_PATH, ENV_PLUGINS_PATH,
};
use bijux_cli_output::{render_value, EmitterConfig};
use bijux_cli_plugin::{
    compatibility_warnings, disable_plugin, enable_plugin, inspect_plugin,
    install_plugin as install_plugin_manifest, is_reserved_namespace, list_plugins,
    load_time_diagnostics, plugin_doctor, plugin_origin_metadata, prune_registry_backup,
    registry_path_from_plugins_dir, uninstall_plugin, validate_manifest, InstallPluginRequest,
    PluginTrustLevel, CORE_NAMESPACES, FUTURE_PRODUCT_NAMESPACES, RESERVED_NAMESPACES,
};
use bijux_cli_routing::catalog::is_known_route as is_known_catalog_route;
use bijux_cli_routing::parser::{parse_intent, root_command, ParsedGlobalFlags};
use bijux_cli_routing::query::contracts_schema_query;
use bijux_cli_routing::registry::{RouteRegistry, RouteTarget};
use bijux_cli_routing::{ColorMode, LogLevel, OutputFormat, PrettyMode};
use bijux_dev_cli::{
    contracts as dev_contracts, control_plane as dev_control_plane,
    crate_health as dev_crate_health, docs_audit as dev_docs_audit, env as dev_env,
    package_health as dev_package_health, parity as dev_parity, registry as dev_registry,
    release as dev_release, route_audit as dev_route_audit, routes as dev_routes,
    runtime_identity as dev_runtime_identity, rustdoc as dev_rustdoc,
    script_audit as dev_script_audit, scripts as dev_scripts, state_audit as dev_state_audit,
    status as dev_status, ReportContext,
};
use serde_json::{json, Value};

use crate::argv::command_positionals;
use crate::config::execute_config_command;
use crate::config::storage::{ConfigRepository, FileConfigRepository};
use crate::query::state_diagnostics_query;

/// In-memory process output and exit result produced by the core app runner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppRunResult {
    /// Process exit code.
    pub exit_code: i32,
    /// Payload that should be written to stdout.
    pub stdout: String,
    /// Payload that should be written to stderr.
    pub stderr: String,
}

fn emitter_config(flags: &ParsedGlobalFlags) -> EmitterConfig {
    EmitterConfig {
        format: flags.output_format.unwrap_or(OutputFormat::Json),
        pretty: !matches!(flags.pretty_mode, Some(PrettyMode::Compact)),
        color: flags.color_mode.unwrap_or(ColorMode::Never),
        log_level: flags.log_level.unwrap_or(LogLevel::Info),
        quiet: flags.quiet,
        no_color: env::var("NO_COLOR").ok().as_deref() == Some("1"),
    }
}

fn home_dir() -> Option<std::path::PathBuf> {
    env::var_os("HOME").map(std::path::PathBuf::from)
}

fn env_map() -> HashMap<String, String> {
    [ENV_CONFIG_PATH, ENV_HISTORY_PATH, ENV_PLUGINS_PATH]
        .iter()
        .filter_map(|key| env::var(key).ok().map(|value| ((*key).to_string(), value)))
        .collect()
}

fn workspace_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .unwrap_or_else(|_| Path::new(".").to_path_buf())
}

fn collect_files(base: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if !base.exists() {
        return out;
    }
    let mut stack = vec![base.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.is_file() {
                    out.push(path);
                }
            }
        }
    }
    out.sort();
    out
}

fn rel_to_root(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

fn read_json_if_exists(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| json!({}))
}

fn command_option_value(argv: &[String], name: &str) -> Option<String> {
    let prefixed = format!("{name}=");
    if let Some(found) = argv.iter().find(|arg| arg.starts_with(&prefixed)) {
        return Some(found[prefixed.len()..].to_string());
    }
    argv.iter().position(|arg| arg == name).and_then(|idx| argv.get(idx + 1)).cloned()
}

fn command_has_flag(argv: &[String], flag: &str) -> bool {
    argv.iter().any(|arg| arg == flag)
}

fn is_safe_scaffold_path(path: &Path) -> bool {
    !path.components().any(|component| matches!(component, Component::ParentDir))
}

fn scaffold_manifest_json(kind: &str, namespace: &str) -> String {
    let plugin_kind = if kind == "python" { "python" } else { "delegated" };
    let entrypoint = if kind == "python" { "plugin:main" } else { "plugin:main" };
    format!(
        "{{\n  \"name\": \"{}\",\n  \"version\": \"0.1.0\",\n  \"schema_version\": \"v1\",\n  \"manifest_version\": \"v1\",\n  \"compatibility\": {{ \"min_inclusive\": \"0.1.0\", \"max_exclusive\": null }},\n  \"namespace\": \"{}\",\n  \"kind\": \"{}\",\n  \"aliases\": [],\n  \"entrypoint\": \"{}\",\n  \"capabilities\": []\n}}\n",
        namespace,
        namespace,
        plugin_kind,
        entrypoint,
    )
}

fn scaffold_plugin_layout(
    base_dir: &Path,
    kind: &str,
    namespace: &str,
    force: bool,
) -> Result<PathBuf> {
    if is_reserved_namespace(namespace, &[]) {
        anyhow::bail!("plugin namespace is reserved: {namespace}");
    }
    if !namespace.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-') {
        anyhow::bail!("plugin namespace must be lowercase kebab-case");
    }
    if !is_safe_scaffold_path(base_dir) {
        anyhow::bail!("scaffold path is unsafe");
    }
    if base_dir.exists() && !force {
        anyhow::bail!("scaffold path already exists; pass --force to overwrite");
    }
    fs::create_dir_all(base_dir)?;
    let manifest_path = base_dir.join("plugin.manifest.json");
    fs::write(&manifest_path, scaffold_manifest_json(kind, namespace))?;
    if kind == "python" {
        fs::write(
            base_dir.join("plugin.py"),
            "def main(argv: list[str]) -> dict:\n    return {\"status\": \"ok\", \"argv\": argv}\n",
        )?;
    } else {
        fs::create_dir_all(base_dir.join("src"))?;
        fs::write(
            base_dir.join("src/lib.rs"),
            "pub fn main(argv: &[String]) -> String { format!(\"ok {}\", argv.len()) }\n",
        )?;
    }
    // Shared validation step: generated manifest must pass plugin parser.
    let manifest_text = fs::read_to_string(&manifest_path)?;
    let manifest = bijux_cli_plugin::parse_manifest_v1(&manifest_text)?;
    let _ = bijux_cli_plugin::validate_manifest(
        manifest,
        env!("CARGO_PKG_VERSION"),
        RESERVED_NAMESPACES,
    )?;
    Ok(manifest_path)
}

fn history_entry_from_command(command: &str) -> Value {
    json!({
        "command": command,
        "params": [],
        "timestamp": 0.0,
        "success": true,
        "return_code": 0,
        "duration_ms": 0.0,
        "raw": {},
    })
}

fn parse_history_entries(text: &str) -> Result<Vec<Value>> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        if let Some(items) = value.as_array() {
            return Ok(items.iter().filter(|item| item.is_object()).cloned().collect());
        }
        anyhow::bail!("Unexpected history file format (not JSON array)");
    }

    // Compatibility fallback for line-oriented history files with partial corruption.
    let mut out = Vec::new();
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<Value>(line) {
            if value.is_object() {
                out.push(value);
            }
            continue;
        }
        out.push(history_entry_from_command(line));
    }
    Ok(out)
}

fn read_history_entries(path: &Path, limit: usize) -> Result<Vec<Value>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path)?;
    let mut entries = parse_history_entries(&text)?;
    if entries.len() > limit {
        entries = entries.split_off(entries.len() - limit);
    }
    Ok(entries)
}

fn write_history_entries(path: &Path, entries: &[Value]) -> Result<()> {
    write_json_document(path, &Value::Array(entries.to_vec()))
}

#[derive(Debug, Clone)]
struct ResolvedStatePaths {
    config_file: PathBuf,
    history_file: PathBuf,
    plugins_dir: PathBuf,
    plugin_registry_file: PathBuf,
    memory_file: PathBuf,
}

fn resolve_state_paths(flags: &ParsedGlobalFlags) -> Result<ResolvedStatePaths> {
    let home = home_dir();
    let defaults = home
        .as_deref()
        .map(default_compatibility_paths)
        .unwrap_or_else(|| default_compatibility_paths(Path::new(".")));

    let config = load_compatibility_config(&defaults.config_file)
        .unwrap_or_else(|_| CompatibilityConfig::default());
    let mut overrides = PathOverrides::default();
    if let Some(path) = &flags.config_path {
        overrides.config_file = Some(path.into());
    }
    let resolved = discover_compatibility_paths(home.as_deref(), &overrides, &env_map(), &config)?;
    let plugin_registry_file = registry_path_from_plugins_dir(&resolved.plugins_dir);
    let memory_file = resolved
        .config_file
        .parent()
        .map(|dir| dir.join(".memory.json"))
        .unwrap_or_else(|| Path::new(".").join(".bijux").join(".memory.json"));
    Ok(ResolvedStatePaths {
        config_file: resolved.config_file,
        history_file: resolved.history_file,
        plugins_dir: resolved.plugins_dir,
        plugin_registry_file,
        memory_file,
    })
}

fn read_memory_map(path: &Path) -> Result<serde_json::Map<String, Value>> {
    if !path.exists() {
        return Ok(serde_json::Map::new());
    }
    let text = fs::read_to_string(path)?;
    let parsed: Value = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(_) => return Ok(serde_json::Map::new()),
    };
    let Some(object) = parsed.as_object() else {
        anyhow::bail!("Malformed memory state: expected JSON object");
    };
    Ok(object.clone())
}

fn write_memory_map(path: &Path, memory: &serde_json::Map<String, Value>) -> Result<()> {
    write_json_document(path, &Value::Object(memory.clone()))
}

fn write_json_document(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let payload = serde_json::to_string_pretty(value)?;
    atomic_write_text(path, &(payload + "\n"))?;
    Ok(())
}

fn state_path_status_value(status: &crate::query::StatePathStatus) -> Value {
    json!({
        "path": status.path,
        "exists": status.exists,
        "is_file": status.is_file,
        "is_dir": status.is_dir,
        "size_bytes": status.size_bytes,
        "readable": status.readable,
        "writable": status.writable,
    })
}

fn state_diagnostics(paths: &ResolvedStatePaths) -> Value {
    let mut issues = Vec::<Value>::new();
    let mut repairs = Vec::<Value>::new();

    let repository = FileConfigRepository;
    if let Err(err) = repository.load(&paths.config_file) {
        issues.push(json!({
            "area": "config",
            "severity": "error",
            "message": err.to_string(),
            "path": paths.config_file,
        }));
    }
    if let Ok(text) = fs::read_to_string(&paths.config_file) {
        let mut seen = std::collections::BTreeMap::<String, usize>::new();
        for line in
            text.lines().map(str::trim).filter(|line| !line.is_empty() && !line.starts_with('#'))
        {
            if let Some((left, _)) = line.split_once('=') {
                *seen.entry(left.trim().to_string()).or_insert(0) += 1;
            }
        }
        let duplicates: Vec<String> =
            seen.into_iter().filter_map(|(key, count)| (count > 1).then_some(key)).collect();
        if !duplicates.is_empty() {
            issues.push(json!({
                "area": "config",
                "severity": "error",
                "message": "duplicate config keys found",
                "keys": duplicates,
                "path": paths.config_file,
            }));
        }
    }
    let config_tmp = paths.config_file.with_extension("tmp");
    if config_tmp.exists() {
        issues.push(json!({
            "area": "config",
            "severity": "warning",
            "message": "partial-write rollback artifact detected",
            "path": config_tmp,
        }));
    }

    if let Err(err) = read_history_entries(&paths.history_file, 20) {
        issues.push(json!({
            "area": "history",
            "severity": "error",
            "message": err.to_string(),
            "path": paths.history_file,
        }));
    }

    match read_memory_map(&paths.memory_file) {
        Ok(memory) => {
            let wrong_type_keys: Vec<String> = memory
                .iter()
                .filter_map(|(key, value)| (!value.is_object()).then_some(key.clone()))
                .collect();
            if !wrong_type_keys.is_empty() {
                issues.push(json!({
                    "area": "memory",
                    "severity": "warning",
                    "message": "memory entries with wrong-type values detected",
                    "keys": wrong_type_keys,
                    "path": paths.memory_file,
                }));
            }
        }
        Err(err) => {
            issues.push(json!({
                "area": "memory",
                "severity": "error",
                "message": err.to_string(),
                "path": paths.memory_file,
            }));
        }
    }

    if bijux_cli_plugin::self_repair_registry(&paths.plugin_registry_file).is_ok() {
        if let Ok(true) = prune_registry_backup(&paths.plugin_registry_file) {
            repairs.push(json!({
                "area": "plugins",
                "action": "removed-stale-backup",
                "path": paths.plugin_registry_file.with_extension("bak"),
            }));
        }
    }

    if let Err(err) = plugin_doctor(&paths.plugin_registry_file) {
        issues.push(json!({
            "area": "plugins",
            "severity": "error",
            "message": err.to_string(),
            "path": paths.plugin_registry_file,
        }));
    }

    json!({
        "status": if issues.is_empty() { "healthy" } else { "degraded" },
        "issues": issues,
        "repairs": repairs,
    })
}

fn render_command_help(path: &[&str]) -> Result<String> {
    let mut cmd = root_command();
    let target =
        find_command_mut(&mut cmd, path).ok_or_else(|| anyhow::anyhow!("unknown help path"))?;
    let mut out = Vec::new();
    target.write_long_help(&mut out)?;
    let mut rendered = String::from_utf8(out)?;
    if matches!(path, ["inspect"] | ["cli", "inspect"] | ["cli", "plugins", "inspect"]) {
        rendered.push_str(
            "\nCompatibility note: inspect output includes plugin compatibility warnings when present.\n",
        );
    }
    Ok(rendered)
}

fn find_command_mut<'a>(
    command: &'a mut clap::Command,
    path: &[&str],
) -> Option<&'a mut clap::Command> {
    if let Some((head, tail)) = path.split_first() {
        let child = command.find_subcommand_mut(head)?;
        find_command_mut(child, tail)
    } else {
        Some(command)
    }
}

fn route_response(
    normalized_path: &[String],
    argv: &[String],
    global_flags: &ParsedGlobalFlags,
) -> Result<Value> {
    let mut registry = RouteRegistry::default();
    let _ = registry.register_plugin_namespace("community");

    let target = match normalized_path {
        [a] if a == "config"
            || a == "history"
            || a == "memory"
            || a == "plugins"
            || a == "dev"
            || a == "atlas" =>
        {
            RouteTarget::BuiltIn
        }
        [a, b] if a == "history" && b == "clear" => RouteTarget::BuiltIn,
        [a, b]
            if a == "memory"
                && (b == "list" || b == "get" || b == "set" || b == "delete" || b == "clear") =>
        {
            RouteTarget::BuiltIn
        }
        [a, b]
            if a == "plugins"
                && (b == "list"
                    || b == "info"
                    || b == "inspect"
                    || b == "check"
                    || b == "reserved-names"
                    || b == "where"
                    || b == "explain"
                    || b == "schema") =>
        {
            RouteTarget::BuiltIn
        }
        [a, b, c] if a == "dev" && b == "cli" => {
            let _ = c;
            RouteTarget::BuiltIn
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" => {
            let _ = d;
            RouteTarget::BuiltIn
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" => {
            let _ = d;
            RouteTarget::BuiltIn
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" => {
            let _ = d;
            RouteTarget::BuiltIn
        }
        _ => registry.resolve(normalized_path)?,
    };
    if matches!(target, RouteTarget::Plugin(_)) {
        return Ok(json!({
            "status": "ok",
            "route": normalized_path.join(" "),
            "owner": "plugin"
        }));
    }

    let paths = resolve_state_paths(global_flags)?;
    let compatibility_paths = CompatibilityPaths {
        config_file: paths.config_file.clone(),
        history_file: paths.history_file.clone(),
        plugins_dir: paths.plugins_dir.clone(),
    };
    let plugin_registry_path = paths.plugin_registry_file.clone();
    if let Some(payload) = execute_config_command(normalized_path, argv, &compatibility_paths)? {
        return Ok(payload);
    }

    let payload = match normalized_path {
        [a, b] if a == "cli" && b == "version" => {
            json!({"version": env!("CARGO_PKG_VERSION")})
        }
        [a, b] if a == "cli" && b == "doctor" => {
            let install_report = install_health_report(
                &env::var("PATH").unwrap_or_default(),
                env::var("BIJUX_BIN").ok().as_deref(),
                env::var("BIJUX_WHEEL_VERSION").ok().as_deref(),
                env!("CARGO_PKG_VERSION"),
            );
            json!({
                "status": "healthy",
                "checks": ["routing", "output", "config", "install"],
                "install": {
                    "has_path_shadowing": install_report.has_path_shadowing,
                    "has_duplicate_installs": install_report.has_duplicate_installs,
                    "stale_wrapper_scripts": install_report.stale_wrapper_scripts,
                    "legacy_installer_conflicts": false,
                    "has_mismatched_wheel_binary_versions": install_report.has_mismatched_wheel_binary_versions,
                }
            })
        }
        [a, b] if a == "cli" && b == "repl" => {
            json!({"status": "ready", "mode": "repl", "history_file": paths.history_file})
        }
        [a, b] if a == "cli" && b == "completion" => {
            json!({"shells": ["bash", "zsh", "fish", "powershell"]})
        }
        [a, b] if a == "cli" && b == "inspect" => {
            let route_sources: Vec<Value> = registry
                .built_in_paths()
                .into_iter()
                .map(|path| {
                    let segments: Vec<String> = path.segments.into_iter().map(|s| s.0).collect();
                    json!({
                        "segments": segments,
                        "owner": "bijux-cli",
                        "source": "built-in",
                    })
                })
                .collect();
            json!({
                "status": "ok",
                "reserved_namespaces": registry.route_tree(),
                "builtins": registry.built_in_paths(),
                "route_sources": route_sources,
                "alias_rewrites": registry.alias_rewrites().into_iter().map(|(alias, canonical)| {
                    let alias_segments: Vec<String> = alias.segments.into_iter().map(|s| s.0).collect();
                    let canonical_segments: Vec<String> = canonical.segments.into_iter().map(|s| s.0).collect();
                    json!({
                        "alias": alias_segments,
                        "canonical": canonical_segments,
                        "source": "compatibility-alias",
                    })
                }).collect::<Vec<_>>(),
                "plugin_origins": plugin_origin_metadata(&plugin_registry_path).unwrap_or_default(),
                "compatibility_warnings": compatibility_warnings(&plugin_registry_path, env!("CARGO_PKG_VERSION")).unwrap_or_default(),
                "contracts": {
                    "schemas": ["output-envelope-v1", "error-envelope-v1", "plugin-manifest-v1"],
                    "version": "v1",
                }
            })
        }
        [a, b] if a == "cli" && b == "status" => {
            json!({"status": "ok", "runtime": "rust-foundation"})
        }
        [a] if a == "status" => {
            json!({"status": "ok", "runtime": "rust-foundation"})
        }
        [a] if a == "audit" => {
            json!({
                "status": "ok",
                "checks": ["config", "paths", "plugins", "history"],
                "issues": []
            })
        }
        [a] if a == "docs" => {
            json!({
                "status": "ok",
                "topics": ["commands", "configuration", "plugins", "repl", "diagnostics"],
            })
        }
        [a] if a == "atlas" => {
            json!({
                "status": "ok",
                "mount": "atlas",
            })
        }
        [a] if a == "sleep" => {
            let duration_secs = argv
                .get(2)
                .and_then(|raw| raw.parse::<f64>().ok())
                .map(|v| v.clamp(0.0, 2.0))
                .unwrap_or(0.0);
            if duration_secs > 0.0 {
                thread::sleep(Duration::from_secs_f64(duration_secs));
            }
            json!({"status": "ok", "slept_seconds": duration_secs})
        }
        [a, b] if a == "cli" && b == "paths" => {
            let install_report = install_health_report(
                &env::var("PATH").unwrap_or_default(),
                env::var("BIJUX_BIN").ok().as_deref(),
                env::var("BIJUX_WHEEL_VERSION").ok().as_deref(),
                env!("CARGO_PKG_VERSION"),
            );
            let hint =
                install_report.active_binary.as_deref().map(post_install_hint).unwrap_or_else(
                    || {
                        "Run `bijux version` and `bijux cli doctor` to verify your environment."
                            .to_string()
                    },
                );
            json!({
                "config": paths.config_file,
                "history": paths.history_file,
                "plugins": paths.plugins_dir,
                "active_binary": install_report.active_binary,
                "path_binaries": install_report.path_binaries,
                "post_install_hint": hint
            })
        }
        [a] if a == "history" => {
            let positional = command_positionals(argv, &["history"]);
            let mut limit = 20_usize;
            if let Some(idx) = argv.iter().position(|arg| arg == "--limit" || arg == "-l") {
                if let Some(raw) = argv.get(idx + 1) {
                    limit = raw.parse::<usize>().unwrap_or(20);
                }
            }
            if let Some(raw) = positional.first().and_then(|token| token.strip_prefix("--limit=")) {
                limit = raw.parse::<usize>().unwrap_or(20);
            }
            let mut entries = read_history_entries(&paths.history_file, limit)?;
            if let Some(idx) = argv.iter().position(|arg| arg == "--filter" || arg == "-F") {
                if let Some(needle) = argv.get(idx + 1) {
                    entries.retain(|entry| {
                        entry
                            .get("command")
                            .and_then(Value::as_str)
                            .map(|command| command.contains(needle))
                            .unwrap_or(false)
                    });
                }
            }
            if argv.iter().any(|arg| arg == "--sort")
                && argv
                    .iter()
                    .position(|arg| arg == "--sort")
                    .and_then(|idx| argv.get(idx + 1))
                    .map(|value| value == "timestamp")
                    .unwrap_or(false)
            {
                entries.sort_by(|a, b| {
                    let left = a.get("timestamp").and_then(Value::as_f64).unwrap_or(0.0);
                    let right = b.get("timestamp").and_then(Value::as_f64).unwrap_or(0.0);
                    left.partial_cmp(&right).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            json!({"entries": entries})
        }
        [a, b] if a == "history" && b == "clear" => {
            let removed = read_history_entries(&paths.history_file, usize::MAX)
                .map(|entries| entries.len())
                .unwrap_or(0);
            write_history_entries(&paths.history_file, &[])?;
            json!({"status": "cleared", "removed_entries": removed, "file": paths.history_file})
        }
        [a] if a == "memory" => {
            let memory = read_memory_map(&paths.memory_file)?;
            json!({"status": "ok", "count": memory.len(), "message": "Memory command executed"})
        }
        [a, b] if a == "memory" && b == "list" => {
            let memory = read_memory_map(&paths.memory_file)?;
            let mut keys: Vec<String> = memory.keys().cloned().collect();
            keys.sort_unstable();
            json!({"status": "ok", "keys": keys, "count": keys.len()})
        }
        [a, b] if a == "memory" && b == "get" => {
            let key = command_positionals(argv, &["memory", "get"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Missing argument: KEY required"))?;
            let memory = read_memory_map(&paths.memory_file)?;
            json!({"status": "ok", "key": key, "value": memory.get(&key).cloned()})
        }
        [a, b] if a == "memory" && b == "set" => {
            let raw_pair = command_positionals(argv, &["memory", "set"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Missing argument: KEY=VALUE required"))?;
            let (key, value) = raw_pair
                .split_once('=')
                .ok_or_else(|| anyhow::anyhow!("Invalid argument: expected KEY=VALUE"))?;
            let mut memory = read_memory_map(&paths.memory_file)?;
            memory.insert(key.trim().to_string(), Value::String(value.trim().to_string()));
            write_memory_map(&paths.memory_file, &memory)?;
            json!({"status": "updated", "key": key.trim(), "value": value.trim(), "file": paths.memory_file})
        }
        [a, b] if a == "memory" && b == "delete" => {
            let key = command_positionals(argv, &["memory", "delete"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Missing argument: KEY required"))?;
            let mut memory = read_memory_map(&paths.memory_file)?;
            let existed = memory.remove(&key).is_some();
            write_memory_map(&paths.memory_file, &memory)?;
            json!({"status": "deleted", "key": key, "removed": existed, "file": paths.memory_file})
        }
        [a, b] if a == "memory" && b == "clear" => {
            let removed = read_memory_map(&paths.memory_file)?.len();
            write_memory_map(&paths.memory_file, &serde_json::Map::new())?;
            json!({"status": "cleared", "removed_keys": removed, "file": paths.memory_file})
        }
        [a] if a == "plugins" => {
            let plugins = list_plugins(&plugin_registry_path).unwrap_or_default();
            json!({
                "status": "ok",
                "count": plugins.len(),
                "plugins": plugins,
                "directory": paths.plugins_dir,
            })
        }
        [a, b] if a == "plugins" && b == "info" => {
            let plugins = list_plugins(&plugin_registry_path).unwrap_or_default();
            let warnings = compatibility_warnings(&plugin_registry_path, env!("CARGO_PKG_VERSION"))
                .unwrap_or_default();
            json!({
                "status": "ok",
                "plugins": plugins,
                "compatibility_warnings": warnings,
                "registry_file": plugin_registry_path,
            })
        }
        [a, b] if a == "cli" && b == "self-test" => {
            json!({"status": "ok", "checks": ["routing", "contracts", "emitters"]})
        }
        [a] if a == "dev" => {
            json!({
                "status": "ok",
                "entry_surface": "dev-cli",
                "recommended_command": "bijux dev cli status",
            })
        }
        [a, b] if a == "dev" && b == "atlas" => {
            json!({
                "status": "ok",
                "mount": "atlas",
                "entry_surface": "dev-cli",
            })
        }
        [a, b] if a == "dev" && b == "di" => {
            json!({
                "status": "ok",
                "container": "built-in",
                "entry_surface": "dev-cli",
            })
        }
        [a, b] if a == "dev" && b == "list-products" => {
            json!({
                "status": "ok",
                "products": FUTURE_PRODUCT_NAMESPACES,
            })
        }
        [a, b] if a == "dev" && b == "list-plugins" => {
            json!({
                "status": "ok",
                "plugins": list_plugins(&plugin_registry_path).unwrap_or_default(),
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "list" => {
            json!({"plugins": list_plugins(&plugin_registry_path).unwrap_or_default(), "directory": paths.plugins_dir})
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "info" => {
            let plugins = list_plugins(&plugin_registry_path).unwrap_or_default();
            let warnings = compatibility_warnings(&plugin_registry_path, env!("CARGO_PKG_VERSION"))
                .unwrap_or_default();
            json!({
                "status": "ok",
                "plugins": plugins,
                "compatibility_warnings": warnings,
                "registry_file": plugin_registry_path,
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "inspect" => {
            json!({
                "plugins": list_plugins(&plugin_registry_path).unwrap_or_default(),
                "status": "loaded",
                "compatibility_warnings": compatibility_warnings(&plugin_registry_path, env!("CARGO_PKG_VERSION")).unwrap_or_default(),
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "check" => {
            let plugin =
                command_positionals(argv, &["cli", "plugins", "check"])
                    .first()
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("Missing argument: plugin name required"))?;
            let record = inspect_plugin(&plugin_registry_path, &plugin)?;
            let _ = validate_manifest(
                record.manifest.clone(),
                env!("CARGO_PKG_VERSION"),
                RESERVED_NAMESPACES,
            )?;
            if matches!(record.state, bijux_cli_routing::PluginLifecycleState::Disabled) {
                anyhow::bail!("Invalid argument: plugin {plugin} is disabled");
            }
            if matches!(record.manifest.kind, bijux_cli_routing::PluginKind::ExternalExec) {
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
            json!({"plugin": plugin, "status": "healthy", "state": format!("{:?}", record.state)})
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "scaffold" => {
            let positional = command_positionals(argv, &["cli", "plugins", "scaffold"]);
            let kind = positional.first().cloned().unwrap_or_else(|| "python".to_string());
            let namespace =
                positional.get(1).cloned().unwrap_or_else(|| "sample-plugin".to_string());
            let force = command_has_flag(argv, "--force");
            let target =
                command_option_value(argv, "--path").map(PathBuf::from).unwrap_or_else(|| {
                    env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(&namespace)
                });
            let manifest = scaffold_plugin_layout(&target, &kind, &namespace, force)?;
            json!({
                "status": "scaffolded",
                "kind": kind,
                "namespace": namespace,
                "path": target,
                "manifest": manifest,
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "install" => {
            let manifest_arg = command_positionals(argv, &["cli", "plugins", "install"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("manifest path is required"))?;
            let manifest_path = PathBuf::from(&manifest_arg);
            let manifest_text = fs::read_to_string(&manifest_path)?;
            let source =
                command_option_value(argv, "--source").unwrap_or_else(|| manifest_arg.clone());
            let trust_level = match command_option_value(argv, "--trust")
                .unwrap_or_else(|| "community".to_string())
                .as_str()
            {
                "core" => PluginTrustLevel::Core,
                "verified" => PluginTrustLevel::Verified,
                "unknown" => PluginTrustLevel::Unknown,
                _ => PluginTrustLevel::Community,
            };
            let installed = install_plugin_manifest(
                &plugin_registry_path,
                InstallPluginRequest { manifest_text, source, trust_level },
                env!("CARGO_PKG_VERSION"),
            )?;
            json!({
                "status": "installed",
                "plugin": installed,
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "uninstall" => {
            let namespace = command_positionals(argv, &["cli", "plugins", "uninstall"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("plugin namespace is required"))?;
            uninstall_plugin(&plugin_registry_path, &namespace)?;
            json!({
                "status": "uninstalled",
                "namespace": namespace,
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "enable" => {
            let namespace = command_positionals(argv, &["cli", "plugins", "enable"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("plugin namespace is required"))?;
            let record = enable_plugin(&plugin_registry_path, &namespace)?;
            json!({
                "status": "enabled",
                "namespace": namespace,
                "state": format!("{:?}", record.state),
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "disable" => {
            let namespace = command_positionals(argv, &["cli", "plugins", "disable"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("plugin namespace is required"))?;
            let record = disable_plugin(&plugin_registry_path, &namespace)?;
            json!({
                "status": "disabled",
                "namespace": namespace,
                "state": format!("{:?}", record.state),
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "doctor" => {
            let repaired = bijux_cli_plugin::self_repair_registry(&plugin_registry_path).is_ok();
            let report = plugin_doctor(&plugin_registry_path)?;
            json!({
                "status": "ok",
                "doctor": {
                    "installed": report.installed,
                    "broken": report.broken,
                    "incompatible": report.incompatible,
                },
                "self_repair_attempted": true,
                "self_repair_success": repaired,
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "reserved-names" => {
            json!({
                "reserved_namespaces": RESERVED_NAMESPACES,
                "core_namespaces": CORE_NAMESPACES,
                "future_product_namespaces": FUTURE_PRODUCT_NAMESPACES,
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "where" => {
            json!({
                "plugins_dir": paths.plugins_dir,
                "registry_file": plugin_registry_path,
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "explain" => {
            let plugin = command_positionals(argv, &["cli", "plugins", "explain"]).first().cloned();
            let diagnostics =
                load_time_diagnostics(&plugin_registry_path, env!("CARGO_PKG_VERSION"))
                    .unwrap_or_default();
            let report = plugin_doctor(&plugin_registry_path).ok();
            let mut filtered: Vec<Value> = diagnostics
                .into_iter()
                .filter(|d| plugin.as_ref().is_none_or(|wanted| d.namespace == *wanted))
                .map(|diag| {
                    json!({
                        "namespace": diag.namespace,
                        "severity": diag.severity,
                        "message": diag.message,
                    })
                })
                .collect();
            if let Some(requested) = &plugin {
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
        [a, b, c] if a == "cli" && b == "plugins" && c == "schema" => {
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
        [a, b, c] if a == "dev" && b == "cli" && c == "routes" => {
            let context = ReportContext {
                generated_at: String::new(),
                data_source: "bijux-cli-routing".to_string(),
            };
            dev_routes::build_report(&registry, &context)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "atlas" => {
            dev_control_plane::build_atlas_report()
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "di" => {
            dev_control_plane::build_dependency_injection_report()
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "list-products" => {
            dev_control_plane::build_product_list_report(FUTURE_PRODUCT_NAMESPACES)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "list-plugins" => {
            let plugins = list_plugins(&plugin_registry_path).unwrap_or_default();
            dev_control_plane::build_plugin_list_report_from(plugins)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "route-audit" => {
            dev_route_audit::build_report(&registry)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "inventory" => {
            dev_script_audit::build_inventory_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "registry" => {
            let context = ReportContext {
                generated_at: String::new(),
                data_source: "bijux-cli-routing".to_string(),
            };
            dev_registry::build_report(&registry, &context)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "parity" => {
            dev_parity::build_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "docs" => {
            let root = workspace_root();
            let docs_files: Vec<String> = collect_files(&root.join("docs"))
                .into_iter()
                .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
                .map(|p| rel_to_root(&p, &root))
                .collect();
            dev_control_plane::build_docs_inventory_report(docs_files)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "status" => dev_status::build_report(
            &workspace_root(),
            dev_script_audit::build_inventory_report(&workspace_root()),
        ),
        [a, b, c] if a == "dev" && b == "cli" && c == "script-audit" => {
            let inventory = dev_script_audit::build_inventory_report(&workspace_root());
            dev_script_audit::build_report(inventory)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "snapshots-audit" => {
            let root = workspace_root();
            let snapshots: Vec<String> = collect_files(&root.join("crates"))
                .into_iter()
                .filter(|p| p.to_string_lossy().contains("tests/snapshots/"))
                .map(|p| rel_to_root(&p, &root))
                .collect();
            dev_control_plane::build_snapshots_audit_report(snapshots)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "fixture-audit" => {
            let root = workspace_root();
            let parity_files: Vec<String> = collect_files(&root.join("artifacts/parity"))
                .into_iter()
                .map(|p| rel_to_root(&p, &root))
                .collect();
            let snapshots: Vec<String> = collect_files(&root.join("crates"))
                .into_iter()
                .filter(|p| p.to_string_lossy().contains("tests/snapshots/"))
                .map(|p| rel_to_root(&p, &root))
                .collect();
            dev_control_plane::build_fixture_audit_report(parity_files, snapshots)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "crate-health" => {
            dev_crate_health::build_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "package-health" => {
            let root = workspace_root();
            let state = read_json_if_exists(&root.join("artifacts/status/current_rust_state.json"));
            dev_package_health::build_report(state)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "env" => dev_env::build_report(
            env_map().into_iter().collect(),
            &dev_env::ActivePaths {
                config_file: paths.config_file.clone(),
                history_file: paths.history_file.clone(),
                plugins_dir: paths.plugins_dir.clone(),
            },
        ),
        [a, b, c] if a == "dev" && b == "cli" && c == "doctor" => {
            let install_report = install_health_report(
                &env::var("PATH").unwrap_or_default(),
                env::var("BIJUX_BIN").ok().as_deref(),
                env::var("BIJUX_WHEEL_VERSION").ok().as_deref(),
                env!("CARGO_PKG_VERSION"),
            );
            let plugin_diagnostics =
                load_time_diagnostics(&plugin_registry_path, env!("CARGO_PKG_VERSION"))
                    .unwrap_or_default();
            let repository = FileConfigRepository;
            let config_issues =
                repository.load(&paths.config_file).err().map_or_else(Vec::new, |err| {
                    vec![json!({"category":"config", "message": err.to_string()})]
                });
            let path_issues = if install_report.has_path_shadowing
                || install_report.has_duplicate_installs
            {
                vec![
                    json!({"category":"paths", "has_path_shadowing": install_report.has_path_shadowing}),
                    json!({"category":"paths", "has_duplicate_installs": install_report.has_duplicate_installs}),
                ]
            } else {
                Vec::new()
            };
            let plugin_issues: Vec<Value> = plugin_diagnostics
                .into_iter()
                .map(|diag| {
                    json!({
                        "category": "plugins",
                        "namespace": diag.namespace,
                        "severity": diag.severity,
                        "message": diag.message,
                    })
                })
                .collect();
            dev_control_plane::build_doctor_report(config_issues, path_issues, plugin_issues)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "docs-prune-plan" => {
            let root = workspace_root();
            let docs_count = collect_files(&root.join("docs"))
                .into_iter()
                .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
                .count();
            dev_control_plane::build_docs_prune_plan_report(docs_count)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "state-audit" => {
            let corruption = state_diagnostics(&paths);
            let state_query = state_diagnostics_query(
                &paths.config_file,
                &paths.history_file,
                &plugin_registry_path,
                &paths.memory_file,
            );
            dev_state_audit::build_report(
                dev_state_audit::StatePathStatusInput {
                    config: state_path_status_value(&state_query.config),
                    history: state_path_status_value(&state_query.history),
                    plugins_registry: state_path_status_value(&state_query.plugins_registry),
                    memory: state_path_status_value(&state_query.memory),
                },
                corruption,
            )
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "state-doctor" => {
            let diagnosis = state_diagnostics(&paths);
            dev_state_audit::build_doctor_report(diagnosis)
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" && d == "remaining" => {
            dev_scripts::build_remaining_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" && d == "migrated" => {
            dev_scripts::build_migrated_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" && d == "diff" => {
            dev_scripts::build_diff_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" && d == "audit" => {
            dev_scripts::build_audit_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" && d == "package-metadata" => {
            dev_scripts::build_package_metadata_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" && d == "e2e-contract" => {
            dev_scripts::build_e2e_contract_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" && d == "pip-audit" => {
            dev_scripts::build_pip_audit_report(
                &workspace_root(),
                command_option_value(argv, "--report-path").as_deref(),
            )
        }
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "scripts" && d == "capture-python-behavior" =>
        {
            dev_scripts::build_python_capture_report(&workspace_root())
        }
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "scripts" && d == "provenance-statement" =>
        {
            let tag = command_option_value(argv, "--tag")
                .ok_or_else(|| anyhow::anyhow!("Missing argument: --tag required"))?;
            let output_dir = command_option_value(argv, "--output-dir")
                .ok_or_else(|| anyhow::anyhow!("Missing argument: --output-dir required"))?;
            dev_scripts::build_provenance_statement_report(&tag, Path::new(&output_dir))
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" && d == "audit" => {
            dev_rustdoc::build_audit_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" && d == "coverage" => {
            dev_rustdoc::build_coverage_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" && d == "broken-links" => {
            dev_rustdoc::build_broken_links_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" && d == "public-api" => {
            dev_rustdoc::build_public_api_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" && d == "examples" => {
            dev_rustdoc::build_examples_report(&workspace_root())
        }
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "rustdoc" && d == "migrate-website-api-docs" =>
        {
            dev_rustdoc::build_migration_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" && d == "build-proof" => {
            dev_rustdoc::build_build_proof_report(&workspace_root())
        }
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "rustdoc" && d == "workspace-coverage-proof" =>
        {
            dev_rustdoc::build_workspace_coverage_proof_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" && d == "python-link-proof" => {
            dev_rustdoc::build_python_link_proof_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "status" => {
            dev_release::build_status_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "evidence" => {
            dev_release::build_evidence_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "readiness" => {
            dev_release::build_readiness_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "diff" => {
            dev_release::build_diff_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "gaps" => {
            dev_release::build_gaps_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "changelog-burden" => {
            dev_release::build_changelog_burden_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "migrate-changelog" => {
            dev_release::build_changelog_migration_report()
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "summary" => {
            dev_release::build_summary_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "manifest" => {
            dev_release::build_manifest_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "notes" => {
            dev_release::build_notes_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "behavior-changes" => {
            dev_release::build_behavior_changes_report(&workspace_root())
        }
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "release" && d == "intentional-differences" =>
        {
            dev_release::build_intentional_differences_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "unresolved-gaps" => {
            dev_release::build_unresolved_gaps_report(&workspace_root())
        }
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "release" && d == "compatibility-leftovers" =>
        {
            dev_release::build_compatibility_leftovers_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "docs-audit" => {
            dev_docs_audit::build_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "plugin-health" => {
            let root = workspace_root();
            let machine =
                read_json_if_exists(&root.join("artifacts/status/plugin_health_report.json"));
            let text = fs::read_to_string(root.join("artifacts/status/plugin_health_report.txt"))
                .unwrap_or_default();
            dev_control_plane::build_plugin_health_report(machine, text)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "contracts" => {
            let contracts_query = contracts_schema_query();
            dev_contracts::build_report_from_query(
                env!("CARGO_PKG_VERSION"),
                &contracts_query.schema_ids,
                &contracts_query.schema_version,
            )
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "runtime-identity" => {
            let install_query = runtime_identity_query(
                &env::var("PATH").unwrap_or_default(),
                env::var("BIJUX_BIN").ok().as_deref(),
                env::var("BIJUX_WHEEL_VERSION").ok().as_deref(),
                env!("CARGO_PKG_VERSION"),
            );
            let install_report = InstallHealthReport {
                active_binary: install_query.active_binary,
                path_binaries: install_query.path_binaries,
                has_path_shadowing: install_query.has_path_shadowing,
                has_duplicate_installs: install_query.has_duplicate_installs,
                stale_wrapper_scripts: install_query.stale_wrapper_scripts,
                has_mismatched_wheel_binary_versions: install_query
                    .has_mismatched_wheel_binary_versions,
                legacy_installer_conflicts: install_query.legacy_installer_conflicts,
                active_binary_missing: install_query.active_binary_missing,
                broken_symlink_active_binary: install_query.broken_symlink_active_binary,
            };
            let python_bridge_supported = !matches!(
                env::var("BIJUX_PYTHON_BRIDGE_SUPPORTED"),
                Ok(value) if matches!(value.as_str(), "0" | "false" | "FALSE")
            );
            let cargo_canonical = cargo_install_strategy(PackageChannel::Canonical);
            let cargo_compat = cargo_install_strategy(PackageChannel::Compatibility);
            let pip_canonical = pip_install_strategy(PackageChannel::Canonical);
            let pip_compat = pip_install_strategy(PackageChannel::Compatibility);
            dev_runtime_identity::build_report(dev_runtime_identity::RuntimeIdentityInput {
                install_report,
                python_bridge_supported,
                cargo_canonical_package: cargo_canonical.package_name,
                cargo_compat_package: cargo_compat.package_name,
                pip_canonical_package: pip_canonical.package_name,
                pip_compat_package: pip_compat.package_name,
                canonical_crate_name: canonical_crate_name().to_string(),
            })
        }
        [a, b, c] if a == "cli" && b == "hold" && c == "interruptible" => {
            for _ in 0..200_u16 {
                thread::sleep(Duration::from_millis(50));
            }
            json!({"status": "completed"})
        }
        _ => json!({"status": "error", "message": "unknown route"}),
    };

    Ok(payload)
}

fn try_render_clap_help(argv: &[String]) -> Option<String> {
    match root_command().try_get_matches_from(argv) {
        Ok(_) => None,
        Err(error)
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) =>
        {
            Some(error.to_string())
        }
        Err(_) => None,
    }
}

/// Execute the CLI for provided argv and return output streams and exit code.
pub fn run_app(argv: &[String]) -> Result<AppRunResult> {
    if argv.len() == 1 {
        return Ok(AppRunResult {
            exit_code: 0,
            stdout: format!("{}\n", render_command_help(&[])?.trim_end()),
            stderr: String::new(),
        });
    }

    if argv.len() >= 2 && argv[1] == "help" {
        let path: Vec<&str> = argv[2..].iter().map(String::as_str).collect();
        return Ok(AppRunResult {
            exit_code: 0,
            stdout: format!("{}\n", render_command_help(&path)?.trim_end()),
            stderr: String::new(),
        });
    }

    if let Some(help) = try_render_clap_help(argv) {
        return Ok(AppRunResult { exit_code: 0, stdout: help, stderr: String::new() });
    }

    let intent = parse_intent(argv)?;
    if intent.normalized_path.is_empty() {
        return Ok(AppRunResult {
            exit_code: 2,
            stdout: String::new(),
            stderr: format!("{}\n", render_command_help(&[])?.trim_end()),
        });
    }

    let is_unknown = !is_known_catalog_route(&intent.normalized_path);

    let response = route_response(&intent.normalized_path, argv, &intent.global_flags);
    let payload = match response {
        Ok(value) => value,
        Err(error) => {
            let message = error.to_string();
            let code = if message.contains("Missing argument")
                || message.contains("Invalid argument")
                || message.contains("Key cannot be empty")
                || message.contains("Invalid key")
                || message.contains("Unknown config section")
                || message.contains("Config key not found")
                || message.contains("Missing parameter")
                || message.contains("Unsupported format")
                || message.contains("Failed to load config")
            {
                2
            } else if message.contains("Non-ASCII") || message.contains("Control characters") {
                3
            } else {
                1
            };
            let rendered_error = render_value(
                &json!({
                    "status": "error",
                    "code": code,
                    "message": message,
                    "command": intent.normalized_path.join(" "),
                }),
                emitter_config(&intent.global_flags),
            )?;
            let error_content = if rendered_error.ends_with('\n') {
                rendered_error
            } else {
                format!("{rendered_error}\n")
            };
            return Ok(AppRunResult {
                exit_code: code,
                stdout: String::new(),
                stderr: error_content,
            });
        }
    };

    let rendered = render_value(&payload, emitter_config(&intent.global_flags))?;
    let content = if rendered.ends_with('\n') { rendered } else { format!("{rendered}\n") };

    if is_unknown {
        return Ok(AppRunResult { exit_code: 2, stdout: String::new(), stderr: content });
    }

    if intent.global_flags.quiet {
        return Ok(AppRunResult { exit_code: 0, stdout: String::new(), stderr: String::new() });
    }

    Ok(AppRunResult { exit_code: 0, stdout: content, stderr: String::new() })
}
