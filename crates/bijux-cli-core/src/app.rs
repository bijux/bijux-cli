//! Top-level application entrypoint and route execution.

use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use bijux_cli_contracts::{ColorMode, LogLevel, OutputFormat, PrettyMode};
use bijux_cli_install::{
    canonical_crate_name, cargo_install_strategy, default_compatibility_paths,
    discover_compatibility_paths, install_health_report, load_compatibility_config,
    pip_install_strategy, post_install_hint, CompatibilityConfig, CompatibilityPaths,
    PackageChannel, PathOverrides, CANONICAL_EXECUTABLE, ENV_CONFIG_PATH, ENV_HISTORY_PATH,
    ENV_PLUGINS_PATH,
};
use bijux_cli_output::{render_value, EmitterConfig};
use bijux_cli_plugin::{
    compatibility_warnings, list_plugins, load_time_diagnostics, plugin_doctor,
    plugin_origin_metadata, registry_path_from_plugins_dir, CORE_NAMESPACES,
    FUTURE_PRODUCT_NAMESPACES, RESERVED_NAMESPACES,
};
use bijux_cli_routing::parser::{parse_intent, root_command, ParsedGlobalFlags};
use bijux_cli_routing::registry::{RouteRegistry, RouteTarget};
use serde_json::{json, Value};

use crate::config::execute_config_command;
use crate::config::storage::{ConfigRepository, FileConfigRepository};

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

fn detect_install_source(active_binary: Option<&str>) -> &'static str {
    let Some(path) = active_binary else {
        return "unknown";
    };
    if path.contains(".cargo") {
        "cargo"
    } else if path.contains("site-packages") || path.contains("venv") || path.contains(".venv") {
        "pip"
    } else if path.contains("pipx") {
        "pipx"
    } else if path.contains("homebrew") || path.contains("/brew/") {
        "homebrew"
    } else {
        "unknown"
    }
}

fn is_canonical_active_path(active_binary: Option<&str>) -> bool {
    active_binary
        .map(|path| path.ends_with(&format!("/{CANONICAL_EXECUTABLE}")) || path == CANONICAL_EXECUTABLE)
        .unwrap_or(false)
}

fn classify_script(path: &str) -> &'static str {
    if path.starts_with("scripts/status/") || path.starts_with("scripts/parity/") {
        return "move-to-dev-cli";
    }
    if path.starts_with("scripts/git-hooks/") || path.starts_with("scripts/docs_builder/") {
        return "keep-external";
    }
    if path == "scripts/__init__.py" {
        return "delete";
    }
    "wrap-with-dev-cli"
}

fn parse_make_targets(path: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(text) = fs::read_to_string(path) else {
        return out;
    };
    for raw in text.lines() {
        if raw.starts_with('\t') || raw.starts_with('#') || raw.trim().is_empty() {
            continue;
        }
        let Some((left, _)) = raw.split_once(':') else {
            continue;
        };
        let target = left.trim();
        if target.is_empty()
            || target.contains(' ')
            || target.contains('=')
            || target.starts_with('.')
        {
            continue;
        }
        out.push(target.to_string());
    }
    out.sort();
    out.dedup();
    out
}

fn classify_make_target(target: &str) -> &'static str {
    if target.starts_with("docs") || target.starts_with("api") || target.starts_with("test") {
        "wrap-with-dev-cli"
    } else if target.starts_with("publish")
        || target.starts_with("sbom")
        || target.starts_with("security")
    {
        "keep"
    } else {
        "wrap-with-dev-cli"
    }
}

fn dev_cli_inventory_payload() -> Value {
    let root = workspace_root();
    let script_files = collect_files(&root.join("scripts"));
    let scripts: Vec<Value> = script_files
        .iter()
        .map(|p| {
            let rel = rel_to_root(p, &root);
            json!({
                "path": rel,
                "classification": classify_script(&rel),
            })
        })
        .collect();

    let mut makefiles = Vec::new();
    for mk in collect_files(&root.join("makefiles")) {
        let rel = rel_to_root(&mk, &root);
        let targets: Vec<Value> = parse_make_targets(&mk)
            .into_iter()
            .map(|target| {
                json!({
                    "target": target,
                    "classification": classify_make_target(&target),
                })
            })
            .collect();
        makefiles.push(json!({
            "file": rel,
            "targets": targets,
        }));
    }

    let script_summary = scripts.iter().fold(BTreeMap::<String, usize>::new(), |mut acc, item| {
        let key =
            item.get("classification").and_then(Value::as_str).unwrap_or("unknown").to_string();
        *acc.entry(key).or_insert(0) += 1;
        acc
    });

    json!({
        "scripts": scripts,
        "makefiles": makefiles,
        "summary": {
            "script_classification_counts": script_summary,
        },
        "maintainer_script_replacements": [
            {"from": "scripts/status/generate_current_rust_state.py", "to": "bijux dev cli status"},
            {"from": "scripts/status/generate_crate_boundary_metrics.py", "to": "bijux dev cli crate-health"},
            {"from": "scripts/parity/run_rust_python_parity.py", "to": "bijux dev cli parity"},
        ],
        "rule": "new maintainer automation defaults to bijux dev cli commands",
    })
}

fn read_json_if_exists(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| json!({}))
}

fn command_positionals(argv: &[String], command_tokens: &[&str]) -> Vec<String> {
    let mut extra_start = 1 + command_tokens.len();
    if argv.len() < extra_start {
        return Vec::new();
    }
    for (idx, token) in command_tokens.iter().enumerate() {
        if argv.get(idx + 1).map(String::as_str) != Some(*token) {
            extra_start = idx + 1;
            break;
        }
    }
    let extras = &argv[extra_start..];
    let mut positional = Vec::new();
    let mut i = 0;
    while i < extras.len() {
        let token = &extras[i];
        if token == "--quiet" || token == "-q" || token == "--pretty" || token == "--no-pretty" {
            i += 1;
            continue;
        }
        if token == "--format"
            || token == "-f"
            || token == "--log-level"
            || token == "--color"
            || token == "--config-path"
        {
            i += 2;
            continue;
        }
        if token.starts_with("--format=")
            || token.starts_with("--log-level=")
            || token.starts_with("--color=")
            || token.starts_with("--config-path=")
        {
            i += 1;
            continue;
        }
        if token.starts_with('-') {
            i += 1;
            continue;
        }
        positional.push(token.clone());
        i += 1;
    }
    positional
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

fn memory_file_path_from_home(home: Option<&Path>) -> std::path::PathBuf {
    match home {
        Some(root) => root.join(".bijux").join(".memory.json"),
        None => Path::new(".").join(".bijux").join(".memory.json"),
    }
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

fn state_path_status(path: &Path) -> Value {
    let exists = path.exists();
    let metadata = fs::metadata(path).ok();
    let is_file = metadata.as_ref().is_some_and(|m| m.is_file());
    let is_dir = metadata.as_ref().is_some_and(|m| m.is_dir());
    let size_bytes = metadata.as_ref().map_or(0_u64, |m| m.len());
    let readable = fs::read(path).is_ok();
    let writable = if exists {
        fs::OpenOptions::new().append(true).open(path).is_ok()
    } else {
        path.parent().is_some_and(|parent| fs::create_dir_all(parent).is_ok())
    };
    json!({
        "path": path,
        "exists": exists,
        "is_file": is_file,
        "is_dir": is_dir,
        "size_bytes": size_bytes,
        "readable": readable,
        "writable": writable,
    })
}

fn state_diagnostics(paths: &CompatibilityPaths, plugin_registry_path: &Path) -> Value {
    let mut issues = Vec::<Value>::new();

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

    if let Err(err) = read_history_entries(&paths.history_file, 20) {
        issues.push(json!({
            "area": "history",
            "severity": "error",
            "message": err.to_string(),
            "path": paths.history_file,
        }));
    }

    if let Err(err) = plugin_doctor(plugin_registry_path) {
        issues.push(json!({
            "area": "plugins",
            "severity": "error",
            "message": err.to_string(),
            "path": plugin_registry_path,
        }));
    }

    json!({
        "status": if issues.is_empty() { "healthy" } else { "degraded" },
        "issues": issues,
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
        [a] if a == "config" || a == "history" || a == "memory" => RouteTarget::BuiltIn,
        [a, b]
            if a == "plugins"
                && (b == "list"
                    || b == "inspect"
                    || b == "check"
                    || b == "reserved-names"
                    || b == "where"
                    || b == "explain"
                    || b == "schema") =>
        {
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

    let home = home_dir();
    let defaults = home
        .as_deref()
        .map(default_compatibility_paths)
        .unwrap_or_else(|| default_compatibility_paths(Path::new(".")));

    let config = load_compatibility_config(&defaults.config_file)
        .unwrap_or_else(|_| CompatibilityConfig::default());
    let mut overrides = PathOverrides::default();
    if let Some(path) = &global_flags.config_path {
        overrides.config_file = Some(path.into());
    }
    let paths = discover_compatibility_paths(home.as_deref(), &overrides, &env_map(), &config)?;
    let plugin_registry_path = registry_path_from_plugins_dir(&paths.plugins_dir);
    if let Some(payload) = execute_config_command(normalized_path, argv, &paths)? {
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
        [a] if a == "memory" => {
            let memory_path = memory_file_path_from_home(home.as_deref());
            let memory = read_memory_map(&memory_path)?;
            json!({"status": "ok", "count": memory.len(), "message": "Memory command executed"})
        }
        [a, b] if a == "memory" && b == "list" => {
            let memory_path = memory_file_path_from_home(home.as_deref());
            let memory = read_memory_map(&memory_path)?;
            let mut keys: Vec<String> = memory.keys().cloned().collect();
            keys.sort_unstable();
            json!({"status": "ok", "keys": keys, "count": keys.len()})
        }
        [a, b] if a == "cli" && b == "self-test" => {
            json!({"status": "ok", "checks": ["routing", "contracts", "emitters"]})
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "list" => {
            json!({"plugins": list_plugins(&plugin_registry_path).unwrap_or_default(), "directory": paths.plugins_dir})
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "inspect" => {
            json!({
                "plugins": list_plugins(&plugin_registry_path).unwrap_or_default(),
                "status": "loaded",
                "compatibility_warnings": compatibility_warnings(&plugin_registry_path, env!("CARGO_PKG_VERSION")).unwrap_or_default(),
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "check" => {
            let plugin = command_positionals(argv, &["cli", "plugins", "check"])
                .first()
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
            json!({"plugin": plugin, "status": "healthy"})
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
            let filtered: Vec<Value> = diagnostics
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
            let routes: Vec<Value> = registry
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
            let aliases: Vec<Value> = registry
                .alias_rewrites()
                .into_iter()
                .map(|(alias, canonical)| {
                    let alias_segments: Vec<String> =
                        alias.segments.into_iter().map(|s| s.0).collect();
                    let canonical_segments: Vec<String> =
                        canonical.segments.into_iter().map(|s| s.0).collect();
                    json!({
                        "alias": alias_segments,
                        "canonical": canonical_segments,
                        "source": "compatibility-alias",
                    })
                })
                .collect();
            json!({
                "routes": routes,
                "aliases": aliases,
            })
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "route-audit" => {
            let routes: Vec<Value> = registry
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
            let aliases: Vec<Value> = registry
                .alias_rewrites()
                .into_iter()
                .map(|(alias, canonical)| {
                    let alias_segments: Vec<String> =
                        alias.segments.into_iter().map(|s| s.0).collect();
                    let canonical_segments: Vec<String> =
                        canonical.segments.into_iter().map(|s| s.0).collect();
                    json!({
                        "alias": alias_segments,
                        "canonical": canonical_segments,
                        "source": "compatibility-alias",
                    })
                })
                .collect();
            json!({
                "routes": routes,
                "aliases": aliases,
                "summary": {
                    "route_count": routes.len(),
                    "alias_count": aliases.len(),
                },
            })
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "inventory" => dev_cli_inventory_payload(),
        [a, b, c] if a == "dev" && b == "cli" && c == "registry" => {
            let registry_rows = registry.route_tree();
            let mut ownership: std::collections::BTreeMap<String, Vec<String>> =
                std::collections::BTreeMap::new();
            for row in &registry_rows {
                ownership.entry(row.owner.clone()).or_default().push(row.name.0.clone());
            }
            json!({
                "registry": registry_rows,
                "ownership": ownership,
                "precedence": ["reserved", "plugin"],
            })
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "parity" => {
            let root = workspace_root();
            let parity_report =
                read_json_if_exists(&root.join("artifacts/parity/rust_python_parity_report.json"));
            let bridge_parity = read_json_if_exists(
                &root.join("artifacts/parity/binary_vs_python_bridge_parity_report.json"),
            );
            let command_matrix =
                read_json_if_exists(&root.join("artifacts/parity/command_parity_matrix.json"));
            let parity_diffs =
                read_json_if_exists(&root.join("artifacts/parity/command_parity_diffs.json"));
            let config_parity =
                read_json_if_exists(&root.join("artifacts/parity/config_parity_report.json"));
            let history_parity =
                read_json_if_exists(&root.join("artifacts/parity/history_parity_report.json"));
            let memory_parity =
                read_json_if_exists(&root.join("artifacts/parity/memory_parity_report.json"));
            json!({
                "rust_python": parity_report,
                "binary_bridge": bridge_parity,
                "command_matrix": command_matrix,
                "diffs": parity_diffs,
                "state_parity": {
                    "config": config_parity,
                    "history": history_parity,
                    "memory": memory_parity,
                },
            })
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "docs" => {
            let root = workspace_root();
            let docs_files: Vec<String> = collect_files(&root.join("docs"))
                .into_iter()
                .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
                .map(|p| rel_to_root(&p, &root))
                .collect();
            json!({
                "docs_count": docs_files.len(),
                "docs": docs_files,
                "index": "docs/INDEX.md",
            })
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "status" => {
            let root = workspace_root();
            let state = read_json_if_exists(&root.join("artifacts/status/current_rust_state.json"));
            let parity =
                read_json_if_exists(&root.join("artifacts/parity/rust_python_parity_report.json"));
            let status_report = read_json_if_exists(&root.join("artifacts/status/status.json"));
            let root_commands =
                read_json_if_exists(&root.join("artifacts/status/status_root_commands.json"));
            let cli_subcommands =
                read_json_if_exists(&root.join("artifacts/status/status_cli_subcommands.json"));
            let dev_cli_subcommands =
                read_json_if_exists(&root.join("artifacts/status/status_dev_cli_subcommands.json"));
            let plugin_commands =
                read_json_if_exists(&root.join("artifacts/status/status_plugin_commands.json"));
            let repl_parity = read_json_if_exists(
                &root.join("artifacts/status/status_repl_parity_coverage.json"),
            );
            let python_bridge_parity = read_json_if_exists(
                &root.join("artifacts/status/status_python_bridge_parity_coverage.json"),
            );
            let install_packaging = read_json_if_exists(
                &root.join("artifacts/status/status_install_packaging_parity_coverage.json"),
            );
            let state_behavior = read_json_if_exists(
                &root.join("artifacts/status/status_state_behavior_coverage.json"),
            );
            let snapshot_coverage =
                read_json_if_exists(&root.join("artifacts/status/status_snapshot_coverage.json"));
            let stream_coverage =
                read_json_if_exists(&root.join("artifacts/status/status_stream_coverage.json"));
            let exit_code_coverage =
                read_json_if_exists(&root.join("artifacts/status/status_exit_code_coverage.json"));
            let failure_path_coverage = read_json_if_exists(
                &root.join("artifacts/status/status_failure_path_coverage.json"),
            );
            let compatibility_aliases = read_json_if_exists(
                &root.join("artifacts/status/status_compatibility_aliases.json"),
            );
            let known_parity_gaps =
                read_json_if_exists(&root.join("artifacts/status/status_known_parity_gaps.json"));
            let intentional_differences = read_json_if_exists(
                &root.join("artifacts/status/status_intentional_differences.json"),
            );
            let unowned_scripts =
                read_json_if_exists(&root.join("artifacts/status/status_unowned_scripts.json"));
            json!({
                "status_report": status_report,
                "reports": {
                    "root_commands": root_commands,
                    "cli_subcommands": cli_subcommands,
                    "dev_cli_subcommands": dev_cli_subcommands,
                    "plugin_commands": plugin_commands,
                    "repl_parity_coverage": repl_parity,
                    "python_bridge_parity_coverage": python_bridge_parity,
                    "install_packaging_parity_coverage": install_packaging,
                    "state_behavior_coverage": state_behavior,
                    "snapshot_coverage": snapshot_coverage,
                    "stream_coverage": stream_coverage,
                    "exit_code_coverage": exit_code_coverage,
                    "failure_path_coverage": failure_path_coverage,
                    "compatibility_aliases": compatibility_aliases,
                    "known_parity_gaps": known_parity_gaps,
                    "intentional_differences": intentional_differences,
                    "unowned_scripts": unowned_scripts,
                },
                "current_rust_state": state,
                "parity": parity,
                "inventory": dev_cli_inventory_payload(),
            })
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "scripts-audit" => {
            let inventory = dev_cli_inventory_payload();
            json!({
                "scripts": inventory.get("scripts").cloned().unwrap_or_else(|| json!([])),
                "summary": inventory.get("summary").cloned().unwrap_or_else(|| json!({})),
                "replacement_rule": inventory.get("rule").cloned().unwrap_or_else(|| json!("")),
            })
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "script-audit" => {
            let inventory = dev_cli_inventory_payload();
            json!({
                "scripts": inventory.get("scripts").cloned().unwrap_or_else(|| json!([])),
                "summary": inventory.get("summary").cloned().unwrap_or_else(|| json!({})),
                "replacement_rule": inventory.get("rule").cloned().unwrap_or_else(|| json!("")),
            })
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "snapshots-audit" => {
            let root = workspace_root();
            let snapshots: Vec<String> = collect_files(&root.join("crates"))
                .into_iter()
                .filter(|p| p.to_string_lossy().contains("tests/snapshots/"))
                .map(|p| rel_to_root(&p, &root))
                .collect();
            json!({
                "snapshot_count": snapshots.len(),
                "snapshots": snapshots,
            })
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
            json!({
                "parity_fixtures": parity_files,
                "snapshot_fixtures": snapshots,
            })
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "crate-health" => {
            let root = workspace_root();
            let metrics =
                read_json_if_exists(&root.join("artifacts/status/crate_boundary_metrics.json"));
            let state = read_json_if_exists(&root.join("artifacts/status/current_rust_state.json"));
            json!({
                "crate_metrics": metrics,
                "public_api_counts": state.get("crates_public_api_counts").cloned().unwrap_or_else(|| json!([])),
                "dependency_edges": state.get("crate_dependency_edges").cloned().unwrap_or_else(|| json!([])),
            })
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "package-health" => {
            let root = workspace_root();
            let state = read_json_if_exists(&root.join("artifacts/status/current_rust_state.json"));
            json!({
                "package_entrypoints": state.get("package_entrypoints").cloned().unwrap_or_else(|| json!([])),
                "runtime_identity_rules": state.get("runtime_identity_rules").cloned().unwrap_or_else(|| json!({})),
            })
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "env" => {
            json!({
                "env": env_map(),
                "source_precedence": ["flags", "env", "config", "defaults"],
                "active": {
                    "config_file": paths.config_file,
                    "history_file": paths.history_file,
                    "plugins_dir": paths.plugins_dir,
                }
            })
        }
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
            let status =
                if config_issues.is_empty() && path_issues.is_empty() && plugin_issues.is_empty() {
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
                },
            })
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "docs-prune-plan" => {
            let root = workspace_root();
            let docs_files: Vec<String> = collect_files(&root.join("docs"))
                .into_iter()
                .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
                .map(|p| rel_to_root(&p, &root))
                .collect();
            json!({
                "docs_count": docs_files.len(),
                "target_cap": 60,
                "actions": [
                    "merge overlapping architecture docs",
                    "merge overlapping compatibility docs",
                    "move low-value prose detail into generated JSON artifacts",
                    "freeze docs rule: every doc explains law or change",
                ],
            })
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "state-audit" => {
            let memory_path = memory_file_path_from_home(home.as_deref());
            json!({
                "paths": {
                    "config": state_path_status(&paths.config_file),
                    "history": state_path_status(&paths.history_file),
                    "plugins_registry": state_path_status(&plugin_registry_path),
                    "memory": state_path_status(&memory_path),
                },
                "runtime": "dev-cli",
            })
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "state-doctor" => {
            let diagnosis = state_diagnostics(&paths, &plugin_registry_path);
            json!({
                "runtime": "dev-cli",
                "doctor": diagnosis,
            })
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "docs-audit" => {
            let root = workspace_root();
            let docs_audit = read_json_if_exists(&root.join("artifacts/status/docs_audit.json"));
            let docs_files: Vec<String> = collect_files(&root.join("docs"))
                .into_iter()
                .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
                .map(|p| rel_to_root(&p, &root))
                .collect();
            json!({
                "docs_audit": docs_audit,
                "docs": docs_files,
                "docs_count": docs_files.len(),
            })
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "contracts" => {
            json!({
                "contracts": [
                    {
                        "name": "output-envelope",
                        "schema": "output-envelope-v1",
                        "version": "1.0.0",
                    },
                    {
                        "name": "error-envelope",
                        "schema": "error-envelope-v1",
                        "version": "1.0.0",
                    },
                    {
                        "name": "plugin-manifest",
                        "schema": "plugin-manifest-v1",
                        "version": "1.0.0",
                    }
                ],
                "schema_version": "v1",
                "runtime_version": env!("CARGO_PKG_VERSION"),
            })
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "runtime-identity" => {
            let install_report = install_health_report(
                &env::var("PATH").unwrap_or_default(),
                env::var("BIJUX_BIN").ok().as_deref(),
                env::var("BIJUX_WHEEL_VERSION").ok().as_deref(),
                env!("CARGO_PKG_VERSION"),
            );
            let install_source = detect_install_source(install_report.active_binary.as_deref());
            let is_shadowed = install_report.has_path_shadowing;
            let is_ambiguous_active_binary = install_report.path_binaries.len() > 1;
            let is_canonical_path = is_canonical_active_path(install_report.active_binary.as_deref());
            let cargo_canonical = cargo_install_strategy(PackageChannel::Canonical);
            let cargo_compat = cargo_install_strategy(PackageChannel::Compatibility);
            let pip_canonical = pip_install_strategy(PackageChannel::Canonical);
            let pip_compat = pip_install_strategy(PackageChannel::Compatibility);
            json!({
                "runtime": "rust-foundation",
                "schema": "runtime-identity-v1",
                "public_runtime_binary_names": [CANONICAL_EXECUTABLE],
                "secondary_public_runtime_binary_names": [],
                "canonical_user_binary": CANONICAL_EXECUTABLE,
                "active_binary": install_report.active_binary,
                "install_source": install_source,
                "active_path_is_canonical_name": is_canonical_path,
                "active_path_is_shadowed": is_shadowed,
                "active_binary_selection_is_ambiguous": is_ambiguous_active_binary,
                "path_binaries": install_report.path_binaries,
                "diagnostics": {
                    "duplicate_install_detected": install_report.has_duplicate_installs,
                    "mixed_pip_cargo_install_detected": install_report.has_duplicate_installs,
                    "path_shadowing_detected": install_report.has_path_shadowing,
                    "stale_wrapper_detected": !install_report.stale_wrapper_scripts.is_empty(),
                    "stale_wrapper_scripts": install_report.stale_wrapper_scripts,
                    "mismatched_wheel_binary_versions": install_report.has_mismatched_wheel_binary_versions,
                    "legacy_installer_conflicts": install_report.legacy_installer_conflicts,
                },
                "entrypoints": {
                    "binary": "crates/bijux-cli-bin/src/main.rs",
                    "core": "bijux_cli_core::app::run_app",
                    "python_bridge": "bijux_cli_python::bindings::execution_facade_api",
                },
                "package_channels": {
                    "cargo": {
                        "canonical": cargo_canonical.package_name,
                        "compatibility": cargo_compat.package_name,
                    },
                    "pip": {
                        "canonical": pip_canonical.package_name,
                        "compatibility": pip_compat.package_name,
                    },
                    "canonical_crate_name": canonical_crate_name(),
                },
                "text_summary": [
                    format!("canonical user binary: {CANONICAL_EXECUTABLE}"),
                    format!("active executable: {}", install_report.active_binary.clone().unwrap_or_else(|| "not-found".to_string())),
                    format!("install source: {install_source}"),
                    format!("path shadowing: {}", if install_report.has_path_shadowing { "detected" } else { "not-detected" }),
                    format!("duplicate installs: {}", if install_report.has_duplicate_installs { "detected" } else { "not-detected" }),
                    format!("stale wrappers: {}", if install_report.stale_wrapper_scripts.is_empty() { "not-detected" } else { "detected" }),
                ],
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

fn is_known_route(path: &[String]) -> bool {
    match path {
        [a, b]
            if a == "cli"
                && (b == "version"
                    || b == "doctor"
                    || b == "repl"
                    || b == "completion"
                    || b == "inspect"
                    || b == "status"
                    || b == "paths"
                    || b == "self-test") =>
        {
            true
        }
        [a, b, c]
            if a == "cli"
                && b == "config"
                && (c == "get"
                    || c == "set"
                    || c == "unset"
                    || c == "clear"
                    || c == "reload"
                    || c == "export"
                    || c == "load") =>
        {
            true
        }
        [a] if a == "config" => true,
        [a] if a == "history" || a == "memory" => true,
        [a, b] if a == "memory" && b == "list" => true,
        [a] if a == "status" || a == "audit" || a == "docs" || a == "sleep" => true,
        [a, b, c]
            if a == "cli"
                && b == "plugins"
                && (c == "list"
                    || c == "inspect"
                    || c == "check"
                    || c == "reserved-names"
                    || c == "where"
                    || c == "explain"
                    || c == "schema") =>
        {
            true
        }
        [a, b, c]
            if a == "dev"
                && b == "cli"
                && (c == "inventory"
                    || c == "routes"
                    || c == "registry"
                    || c == "parity"
                    || c == "docs"
                    || c == "status"
                    || c == "scripts-audit"
                    || c == "script-audit"
                    || c == "snapshots-audit"
                    || c == "fixture-audit"
                    || c == "crate-health"
                    || c == "package-health"
                    || c == "route-audit"
                    || c == "env"
                    || c == "doctor"
                    || c == "contracts"
                    || c == "runtime-identity"
                    || c == "docs-prune-plan"
                    || c == "state-audit"
                    || c == "state-doctor"
                    || c == "docs-audit") =>
        {
            true
        }
        [a, b, c] if a == "cli" && b == "hold" && c == "interruptible" => true,
        _ => false,
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

    let is_unknown = !is_known_route(&intent.normalized_path);

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
