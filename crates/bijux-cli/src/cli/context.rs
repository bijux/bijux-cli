//! Shared CLI context and state helper routines.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::Result;
use serde_json::{json, Value};

use crate::config::storage::{ConfigRepository, FileConfigRepository};
use crate::install::{
    atomic_write_text, default_compatibility_paths, discover_compatibility_paths,
    load_compatibility_config, CompatibilityConfig, PathOverrides, ENV_CONFIG_PATH,
    ENV_HISTORY_PATH, ENV_PLUGINS_PATH,
};
use crate::plugin::{
    is_reserved_namespace, plugin_doctor, prune_registry_backup, registry_path_from_plugins_dir,
    RESERVED_NAMESPACES,
};
use crate::routing::parser::ParsedGlobalFlags;

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}

pub(crate) fn env_map() -> HashMap<String, String> {
    [ENV_CONFIG_PATH, ENV_HISTORY_PATH, ENV_PLUGINS_PATH]
        .iter()
        .filter_map(|key| env::var(key).ok().map(|value| ((*key).to_string(), value)))
        .collect()
}

pub(crate) fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .unwrap_or_else(|_| Path::new(".").to_path_buf())
}

pub(crate) fn collect_files(base: &Path) -> Vec<PathBuf> {
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

pub(crate) fn rel_to_root(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

pub(crate) fn read_json_if_exists(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| json!({}))
}

pub(crate) fn command_option_value(argv: &[String], name: &str) -> Option<String> {
    let prefixed = format!("{name}=");
    if let Some(found) = argv.iter().find(|arg| arg.starts_with(&prefixed)) {
        return Some(found[prefixed.len()..].to_string());
    }
    argv.iter().position(|arg| arg == name).and_then(|idx| argv.get(idx + 1)).cloned()
}

pub(crate) fn command_has_flag(argv: &[String], flag: &str) -> bool {
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

pub(crate) fn scaffold_plugin_layout(
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
    let manifest = crate::plugin::parse_manifest_v1(&manifest_text)?;
    let _ = crate::plugin::validate_manifest(
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

pub(crate) fn read_history_entries(path: &Path, limit: usize) -> Result<Vec<Value>> {
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

pub(crate) fn write_history_entries(path: &Path, entries: &[Value]) -> Result<()> {
    write_json_document(path, &Value::Array(entries.to_vec()))
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedStatePaths {
    pub(crate) config_file: PathBuf,
    pub(crate) history_file: PathBuf,
    pub(crate) plugins_dir: PathBuf,
    pub(crate) plugin_registry_file: PathBuf,
    pub(crate) memory_file: PathBuf,
}

pub(crate) fn resolve_state_paths(flags: &ParsedGlobalFlags) -> Result<ResolvedStatePaths> {
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

pub(crate) fn read_memory_map(path: &Path) -> Result<serde_json::Map<String, Value>> {
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

pub(crate) fn write_memory_map(path: &Path, memory: &serde_json::Map<String, Value>) -> Result<()> {
    write_json_document(path, &Value::Object(memory.clone()))
}

pub(crate) fn write_json_document(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let payload = serde_json::to_string_pretty(value)?;
    atomic_write_text(path, &(payload + "\n"))?;
    Ok(())
}

pub(crate) fn state_path_status_value(status: &crate::query::StatePathStatus) -> Value {
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

pub(crate) fn state_diagnostics(paths: &ResolvedStatePaths) -> Value {
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

    if crate::plugin::self_repair_registry(&paths.plugin_registry_file).is_ok() {
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
