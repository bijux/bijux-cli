#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use serde_json::{json, Map, Value};

use crate::features::config::error::ConfigError;
use crate::features::config::schema::{
    config_schema_registry, config_schema_registry_v1, config_schema_scope,
    env_candidates_for_storage_key, infer_scope, logical_key_to_storage_key,
    normalize_selector_to_storage_key, redact_value, schema_field_for_key,
    storage_key_to_logical_key, validate_schema_value, CONFIG_SCHEMA_REGISTRY_VERSION,
};
use crate::features::config::storage::{ConfigRepository, FileConfigRepository};
use crate::features::install::{acquire_state_lock, CompatibilityError, StateLockGuard};

#[derive(Debug, Clone, Default)]
pub(crate) struct LayeredConfigOptions {
    pub profile: Option<String>,
    pub include_secrets: bool,
    pub portable: bool,
    pub overrides: Vec<String>,
}

#[derive(Debug, Clone)]
struct LayerSource {
    name: &'static str,
    path: Option<PathBuf>,
    profile: Option<String>,
    format: &'static str,
    entries: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default)]
struct ProjectConfigDiscovery {
    root: Option<PathBuf>,
    config_path: Option<PathBuf>,
    config_format: Option<&'static str>,
    profile_path: Option<PathBuf>,
    profile_format: Option<&'static str>,
}

pub(crate) fn schema_report(scope: Option<&str>) -> Result<Value, ConfigError> {
    if let Some(scope_name) = scope {
        let scope_payload = config_schema_scope(scope_name).ok_or_else(|| {
            ConfigError::not_found(format!("Unknown config schema scope: {scope_name}"))
        })?;
        return Ok(json!({
            "status": "ok",
            "schema_version": CONFIG_SCHEMA_REGISTRY_VERSION,
            "scope": scope_payload.scope,
            "source": scope_payload.source,
            "fields": scope_payload.fields,
        }));
    }

    let registry = config_schema_registry_v1();
    Ok(json!({
        "status": "ok",
        "schema_version": registry.schema_version,
        "scopes": registry.scopes,
    }))
}

pub(crate) fn schema_docs_report(scope: Option<&str>) -> Result<Value, ConfigError> {
    let markdown = schema_docs_markdown(scope)?;
    Ok(json!({
        "status": "ok",
        "schema_version": CONFIG_SCHEMA_REGISTRY_VERSION,
        "scope": scope,
        "markdown": markdown,
    }))
}

pub(crate) fn validate_report(
    global_config_path: &Path,
    cwd: &Path,
    profile: Option<&str>,
    overrides: &[String],
) -> Result<Value, ConfigError> {
    let resolved =
        resolve_layered_config_with_overrides(global_config_path, cwd, profile, overrides)?;
    let mut warnings = resolved.warnings;
    warnings.extend(warnings_for_unknown_entries(&resolved.effective_entries));

    let errors = validate_entries(&resolved.effective_entries)
        .into_iter()
        .map(|message| json!(message))
        .collect::<Vec<_>>();

    Ok(json!({
        "status": if errors.is_empty() { "ok" } else { "error" },
        "valid": errors.is_empty(),
        "profile": profile,
        "precedence": precedence_names(),
        "project_discovery": project_discovery_payload(&resolved.project_discovery),
        "layers": resolved
            .layers
            .iter()
            .map(|layer| layer_report(layer, false))
            .collect::<Vec<_>>(),
        "effective": effective_entries_payload(&resolved.effective_entries, false),
        "errors": errors,
        "warnings": warnings.into_iter().map(|message| json!(message)).collect::<Vec<_>>(),
    }))
}

pub(crate) fn explain_report(
    global_config_path: &Path,
    cwd: &Path,
    raw_key: &str,
    profile: Option<&str>,
    overrides: &[String],
    include_secrets: bool,
) -> Result<Value, ConfigError> {
    let resolved =
        resolve_layered_config_with_overrides(global_config_path, cwd, profile, overrides)?;
    let storage_key = normalize_selector(raw_key)?;
    let logical_key = storage_key_to_logical_key(&storage_key);
    let effective_value = resolved
        .effective_entries
        .get(&storage_key)
        .cloned()
        .ok_or_else(|| ConfigError::not_found(format!("Config key not found: {raw_key}")))?;

    let field = schema_field_for_key(&storage_key);
    let env_candidates = env_candidates_for_storage_key(&storage_key);
    let layer_entries = resolved
        .layers
        .iter()
        .filter_map(|layer| {
            layer.entries.get(&storage_key).map(|value| {
                json!({
                    "layer": layer.name,
                    "path": layer.path,
                    "profile": layer.profile,
                    "format": layer.format,
                    "value": redact_value(&storage_key, value, include_secrets),
                    "redacted": !include_secrets && redact_value(&storage_key, value, false) != *value,
                })
            })
        })
        .collect::<Vec<_>>();

    let env_values = env_candidates
        .iter()
        .filter_map(|env_key| std::env::var(env_key).ok().map(|value| (env_key, value)))
        .map(|(env_key, value)| {
            json!({
                "env": env_key,
                "value": redact_value(&storage_key, &value, include_secrets),
                "redacted": !include_secrets && redact_value(&storage_key, &value, false) != value,
            })
        })
        .collect::<Vec<_>>();

    Ok(json!({
        "status": "ok",
        "key": raw_key,
        "precedence": precedence_names(),
        "logical_key": logical_key,
        "storage_key": storage_key,
        "scope": field.as_ref().map(|entry| entry.scope.clone()).unwrap_or_else(|| infer_scope(&storage_key)),
        "schema": field,
        "effective": {
            "value": redact_value(&storage_key, &effective_value, include_secrets),
            "redacted": !include_secrets && redact_value(&storage_key, &effective_value, false) != effective_value,
        },
        "layers": layer_entries,
        "environment": {
            "candidates": env_candidates,
            "active": env_values,
        },
        "project_discovery": project_discovery_payload(&resolved.project_discovery),
    }))
}

pub(crate) fn diff_report(
    global_config_path: &Path,
    cwd: &Path,
    raw_key: Option<&str>,
    from_profile: Option<&str>,
    to_profile: Option<&str>,
    overrides: &[String],
    include_secrets: bool,
) -> Result<Value, ConfigError> {
    let from =
        resolve_layered_config_with_overrides(global_config_path, cwd, from_profile, overrides)?;
    let to = resolve_layered_config_with_overrides(global_config_path, cwd, to_profile, overrides)?;

    let keys = if let Some(key) = raw_key {
        BTreeSet::from([normalize_selector(key)?])
    } else {
        from.effective_entries
            .keys()
            .chain(to.effective_entries.keys())
            .cloned()
            .collect::<BTreeSet<_>>()
    };

    let mut changes = Vec::new();
    let mut unchanged_count = 0usize;

    for storage_key in keys {
        let before = from.effective_entries.get(&storage_key).cloned();
        let after = to.effective_entries.get(&storage_key).cloned();
        if before == after {
            unchanged_count += 1;
            continue;
        }

        let before_redacted = before
            .as_ref()
            .map(|value| !include_secrets && redact_value(&storage_key, value, false) != *value)
            .unwrap_or(false);
        let after_redacted = after
            .as_ref()
            .map(|value| !include_secrets && redact_value(&storage_key, value, false) != *value)
            .unwrap_or(false);
        let field = schema_field_for_key(&storage_key);

        changes.push(json!({
            "logical_key": storage_key_to_logical_key(&storage_key),
            "storage_key": storage_key,
            "scope": field.as_ref().map(|entry| entry.scope.clone()).unwrap_or_else(|| infer_scope(&storage_key)),
            "schema": field,
            "from": before.as_ref().map(|value| redact_value(&storage_key, value, include_secrets)),
            "from_redacted": before_redacted,
            "to": after.as_ref().map(|value| redact_value(&storage_key, value, include_secrets)),
            "to_redacted": after_redacted,
        }));
    }

    Ok(json!({
        "status": "ok",
        "key": raw_key,
        "precedence": precedence_names(),
        "from_profile": from_profile,
        "to_profile": to_profile,
        "changed_count": changes.len(),
        "unchanged_count": unchanged_count,
        "changes": changes,
        "from_context": {
            "profile": from_profile,
            "project_discovery": project_discovery_payload(&from.project_discovery),
        },
        "to_context": {
            "profile": to_profile,
            "project_discovery": project_discovery_payload(&to.project_discovery),
        }
    }))
}

pub(crate) fn repair_report(global_config_path: &Path) -> Result<Value, ConfigError> {
    let original = if global_config_path.exists() {
        fs::read_to_string(global_config_path)
            .map_err(|err| ConfigError::persistence(err.to_string()))?
    } else {
        String::new()
    };
    let repaired = repair_env_text(&original)?;
    let repaired_text = render_env_map(&repaired.entries);
    let changed = repaired_text != original;

    let backup_path = changed.then(|| global_config_path.with_extension("bak"));
    if changed {
        let _guard = config_lock(global_config_path)?;
        if let Some(path) = &backup_path {
            fs::write(path, &original).map_err(|err| ConfigError::persistence(err.to_string()))?;
        }
        FileConfigRepository
            .save(global_config_path, &repaired.entries)
            .map_err(|err| ConfigError::persistence(err.to_string()))?;
    }

    let warning_rows = repaired.warnings.iter().map(|message| json!(message)).collect::<Vec<_>>();
    let issue_rows = repaired
        .issues
        .iter()
        .map(|issue| {
            json!({
                "line": issue.line,
                "content": issue.content,
                "issue": issue.issue,
                "remediation": issue.remediation,
            })
        })
        .collect::<Vec<_>>();
    let remediation = repaired
        .issues
        .iter()
        .map(|issue| issue.remediation.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let dropped_line_count = repaired.issues.len();

    Ok(json!({
        "status": "ok",
        "changed": changed,
        "file": global_config_path,
        "backup": backup_path,
        "warnings": warning_rows,
        "issues": issue_rows,
        "remediation": remediation,
        "dropped_line_count": dropped_line_count,
        "entry_count": repaired.entries.len(),
    }))
}

pub(crate) fn export_report(
    global_config_path: &Path,
    cwd: &Path,
    target_path: &Path,
    options: &LayeredConfigOptions,
) -> Result<Value, ConfigError> {
    if options.portable {
        let resolved = resolve_layered_config_with_overrides(
            global_config_path,
            cwd,
            options.profile.as_deref(),
            &options.overrides,
        )?;
        let payload = json!({
            "format": "bijux-cli-config-bundle-v1",
            "profile": options.profile,
            "project_discovery": project_discovery_payload(&resolved.project_discovery),
            "entries": effective_entries_payload(&resolved.effective_entries, options.include_secrets),
        });
        write_json_pretty(target_path, &payload)?;
        return Ok(json!({
            "status": "exported",
            "file": target_path,
            "file_format": "portable_json",
            "profile": options.profile,
        }));
    }

    let source_path = profile_env_path(global_config_path, options.profile.as_deref())
        .unwrap_or_else(|| global_config_path.to_path_buf());
    let values = FileConfigRepository.load(&source_path)?;
    FileConfigRepository.save(target_path, &values)?;
    Ok(json!({
        "status": "exported",
        "file": target_path,
        "source": source_path,
        "file_format": "env",
        "profile": options.profile,
    }))
}

pub(crate) fn load_report(
    global_config_path: &Path,
    source_path: &Path,
    options: &LayeredConfigOptions,
) -> Result<Value, ConfigError> {
    if !source_path.exists() {
        return Err(ConfigError::not_found(format!(
            "Config source file not found: {}",
            source_path.display()
        )));
    }
    if !source_path.is_file() {
        return Err(ConfigError::validation(format!(
            "Config source path must be a file: {}",
            source_path.display()
        )));
    }

    let target_path = profile_env_path(global_config_path, options.profile.as_deref())
        .unwrap_or_else(|| global_config_path.to_path_buf());
    let _guard = config_lock(&target_path)?;
    let values = if options.portable {
        let payload: Value = serde_json::from_str(
            &fs::read_to_string(source_path)
                .map_err(|err| ConfigError::persistence(err.to_string()))?,
        )
        .map_err(|err| ConfigError::parse(format!("Invalid portable config bundle: {err}")))?;
        let entries = payload.get("entries").and_then(Value::as_object).ok_or_else(|| {
            ConfigError::parse("Portable config bundle is missing `entries` object")
        })?;
        parse_portable_entries(entries)?
    } else {
        FileConfigRepository.load(source_path)?
    };
    FileConfigRepository.save(&target_path, &values)?;
    Ok(json!({
        "status": "loaded",
        "file": source_path,
        "target": target_path,
        "file_format": if options.portable { "portable_json" } else { "env" },
        "profile": options.profile,
    }))
}

#[derive(Debug, Clone)]
struct ResolvedConfig {
    layers: Vec<LayerSource>,
    effective_entries: BTreeMap<String, String>,
    warnings: Vec<String>,
    project_discovery: ProjectConfigDiscovery,
}

#[cfg_attr(not(test), allow(dead_code))]
fn resolve_layered_config(
    global_config_path: &Path,
    cwd: &Path,
    profile: Option<&str>,
) -> Result<ResolvedConfig, ConfigError> {
    resolve_layered_config_with_overrides(global_config_path, cwd, profile, &[])
}

fn resolve_layered_config_with_overrides(
    global_config_path: &Path,
    cwd: &Path,
    profile: Option<&str>,
    overrides: &[String],
) -> Result<ResolvedConfig, ConfigError> {
    let mut layers = Vec::new();
    let mut warnings = Vec::new();

    layers.push(LayerSource {
        name: "defaults",
        path: None,
        profile: None,
        format: "built_in",
        entries: default_entries(),
    });

    layers.push(LayerSource {
        name: "global_file",
        path: Some(global_config_path.to_path_buf()),
        profile: None,
        format: "env",
        entries: FileConfigRepository.load(global_config_path)?,
    });

    if let Some(profile_name) = profile {
        if let Some((path, format)) =
            discover_profile_layer(global_config_path.parent(), profile_name)?
        {
            layers.push(LayerSource {
                name: "global_profile",
                path: Some(path.clone()),
                profile: Some(profile_name.to_string()),
                format,
                entries: load_layer_file(&path, format)?,
            });
        }
    }

    let project_discovery = discover_project_config(cwd, profile)?;
    if let Some(path) = &project_discovery.config_path {
        let format = project_discovery.config_format.expect("config format");
        layers.push(LayerSource {
            name: "project_file",
            path: Some(path.clone()),
            profile: None,
            format,
            entries: load_layer_file(path, format)?,
        });
    }
    if let Some(path) = &project_discovery.profile_path {
        let format = project_discovery.profile_format.expect("profile format");
        layers.push(LayerSource {
            name: "project_profile",
            path: Some(path.clone()),
            profile: profile.map(ToOwned::to_owned),
            format,
            entries: load_layer_file(path, format)?,
        });
    }

    let env_entries = gather_env_overrides();
    if !env_entries.is_empty() {
        layers.push(LayerSource {
            name: "environment",
            path: None,
            profile: None,
            format: "env",
            entries: env_entries,
        });
    }

    let cli_overrides = parse_cli_overrides(overrides)?;
    if !cli_overrides.is_empty() {
        layers.push(LayerSource {
            name: "cli_overrides",
            path: None,
            profile: None,
            format: "argv",
            entries: cli_overrides,
        });
    }

    if project_discovery.config_path.is_none() {
        warnings
            .push("No project config file discovered under .bijux/config.{toml,json}".to_string());
    }
    if profile.is_some() && project_discovery.profile_path.is_none() {
        warnings.push("No project profile overlay discovered for the selected profile".to_string());
    }

    let mut effective_entries = BTreeMap::new();
    for layer in &layers {
        for (key, value) in &layer.entries {
            effective_entries.insert(key.clone(), value.clone());
        }
    }

    Ok(ResolvedConfig { layers, effective_entries, warnings, project_discovery })
}

fn default_entries() -> BTreeMap<String, String> {
    BTreeMap::new()
}

fn parse_cli_overrides(overrides: &[String]) -> Result<BTreeMap<String, String>, ConfigError> {
    let mut out = BTreeMap::new();
    for raw in overrides {
        let Some((raw_key, raw_value)) = raw.split_once('=') else {
            return Err(ConfigError::validation(format!(
                "Invalid override `{raw}`; expected KEY=VALUE"
            )));
        };
        let storage_key = normalize_selector(raw_key)?;
        validate_schema_value(&storage_key, raw_value)?;
        out.insert(storage_key, raw_value.to_string());
    }
    Ok(out)
}

fn precedence_names() -> Vec<&'static str> {
    vec![
        "defaults",
        "global_file",
        "global_profile",
        "project_file",
        "project_profile",
        "environment",
        "cli_overrides",
    ]
}

fn load_layer_file(
    path: &Path,
    format: &'static str,
) -> Result<BTreeMap<String, String>, ConfigError> {
    match format {
        "env" => FileConfigRepository.load(path),
        "json" => load_json_layer(path),
        "toml" => load_toml_layer(path),
        other => Err(ConfigError::validation(format!("Unsupported config layer format: {other}"))),
    }
}

fn load_json_layer(path: &Path) -> Result<BTreeMap<String, String>, ConfigError> {
    let payload: Value = serde_json::from_str(
        &fs::read_to_string(path).map_err(|err| ConfigError::persistence(err.to_string()))?,
    )
    .map_err(|err| ConfigError::parse(format!("Invalid JSON config: {err}")))?;
    let mut out = BTreeMap::new();
    flatten_json_value(None, &payload, &mut out)?;
    Ok(out)
}

fn flatten_json_value(
    prefix: Option<&str>,
    value: &Value,
    out: &mut BTreeMap<String, String>,
) -> Result<(), ConfigError> {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let next = match prefix {
                    Some(existing) => format!("{existing}.{key}"),
                    None => key.to_string(),
                };
                flatten_json_value(Some(&next), child, out)?;
            }
            Ok(())
        }
        Value::Null => Err(ConfigError::validation("Null values are not allowed in config layers")),
        Value::String(text) => {
            let selector = prefix.ok_or_else(|| {
                ConfigError::validation(
                    "Top-level scalar config values require an explicit object key",
                )
            })?;
            out.insert(logical_key_to_storage_key(selector), text.clone());
            Ok(())
        }
        Value::Bool(flag) => {
            let selector = prefix.ok_or_else(|| {
                ConfigError::validation(
                    "Top-level scalar config values require an explicit object key",
                )
            })?;
            out.insert(logical_key_to_storage_key(selector), flag.to_string());
            Ok(())
        }
        Value::Number(number) => {
            let selector = prefix.ok_or_else(|| {
                ConfigError::validation(
                    "Top-level scalar config values require an explicit object key",
                )
            })?;
            out.insert(logical_key_to_storage_key(selector), number.to_string());
            Ok(())
        }
        Value::Array(_) => {
            let selector = prefix.ok_or_else(|| {
                ConfigError::validation(
                    "Top-level array config values require an explicit object key",
                )
            })?;
            out.insert(logical_key_to_storage_key(selector), value.to_string());
            Ok(())
        }
    }
}

fn load_toml_layer(path: &Path) -> Result<BTreeMap<String, String>, ConfigError> {
    let payload: toml::Value = toml::from_str(
        &fs::read_to_string(path).map_err(|err| ConfigError::persistence(err.to_string()))?,
    )
    .map_err(|err| ConfigError::parse(format!("Invalid TOML config: {err}")))?;
    let json_value = serde_json::to_value(payload)
        .map_err(|err| ConfigError::parse(format!("Invalid TOML conversion: {err}")))?;
    let mut out = BTreeMap::new();
    flatten_json_value(None, &json_value, &mut out)?;
    Ok(out)
}

fn gather_env_overrides() -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for scope in config_schema_registry() {
        for field in scope.fields {
            for env_key in env_candidates_for_storage_key(&field.storage_key) {
                if let Ok(value) = std::env::var(&env_key) {
                    out.insert(field.storage_key.clone(), value);
                    break;
                }
            }
        }
    }
    out
}

fn discover_profile_layer(
    parent: Option<&Path>,
    profile: &str,
) -> Result<Option<(PathBuf, &'static str)>, ConfigError> {
    let Some(parent_dir) = parent else {
        return Ok(None);
    };
    let profile_dir = parent_dir.join("profiles");
    let env_path = profile_dir.join(format!("{profile}.env"));
    if env_path.exists() {
        return Ok(Some((env_path, "env")));
    }
    let toml_path = profile_dir.join(format!("{profile}.toml"));
    if toml_path.exists() {
        return Ok(Some((toml_path, "toml")));
    }
    let json_path = profile_dir.join(format!("{profile}.json"));
    if json_path.exists() {
        return Ok(Some((json_path, "json")));
    }
    Ok(None)
}

fn discover_project_config(
    cwd: &Path,
    profile: Option<&str>,
) -> Result<ProjectConfigDiscovery, ConfigError> {
    for candidate in cwd.ancestors() {
        let bijux_dir = candidate.join(".bijux");
        if !bijux_dir.is_dir() {
            continue;
        }
        let mut discovery = ProjectConfigDiscovery {
            root: Some(candidate.to_path_buf()),
            ..ProjectConfigDiscovery::default()
        };
        let toml_path = bijux_dir.join("config.toml");
        let json_path = bijux_dir.join("config.json");
        if toml_path.exists() {
            discovery.config_path = Some(toml_path);
            discovery.config_format = Some("toml");
        } else if json_path.exists() {
            discovery.config_path = Some(json_path);
            discovery.config_format = Some("json");
        }
        if let Some(profile_name) = profile {
            let profile_dir = bijux_dir.join("profiles");
            let profile_toml = profile_dir.join(format!("{profile_name}.toml"));
            let profile_json = profile_dir.join(format!("{profile_name}.json"));
            if profile_toml.exists() {
                discovery.profile_path = Some(profile_toml);
                discovery.profile_format = Some("toml");
            } else if profile_json.exists() {
                discovery.profile_path = Some(profile_json);
                discovery.profile_format = Some("json");
            }
        }
        return Ok(discovery);
    }
    Ok(ProjectConfigDiscovery::default())
}

fn project_discovery_payload(discovery: &ProjectConfigDiscovery) -> Value {
    json!({
        "root": discovery.root,
        "config_path": discovery.config_path,
        "config_format": discovery.config_format,
        "profile_path": discovery.profile_path,
        "profile_format": discovery.profile_format,
    })
}

fn layer_report(layer: &LayerSource, include_secrets: bool) -> Value {
    json!({
        "name": layer.name,
        "path": layer.path,
        "profile": layer.profile,
        "format": layer.format,
        "entries": effective_entries_payload(&layer.entries, include_secrets),
    })
}

fn effective_entries_payload(
    values: &BTreeMap<String, String>,
    include_secrets: bool,
) -> BTreeMap<String, Value> {
    values
        .iter()
        .map(|(storage_key, value)| {
            (
                storage_key_to_logical_key(storage_key),
                json!({
                    "storage_key": storage_key,
                    "value": redact_value(storage_key, value, include_secrets),
                    "redacted": !include_secrets && redact_value(storage_key, value, false) != *value,
                }),
            )
        })
        .collect()
}

fn warnings_for_unknown_entries(values: &BTreeMap<String, String>) -> Vec<String> {
    values
        .keys()
        .filter(|storage_key| schema_field_for_key(storage_key).is_none())
        .map(|storage_key| {
            format!("Unknown config key `{storage_key}` is present in the effective configuration")
        })
        .collect()
}

fn validate_entries(values: &BTreeMap<String, String>) -> Vec<String> {
    let mut errors = Vec::new();
    for (storage_key, value) in values {
        if let Err(err) = validate_schema_value(storage_key, value) {
            errors.push(err.to_string());
        }
    }
    errors
}

fn schema_docs_markdown(scope: Option<&str>) -> Result<String, ConfigError> {
    let scopes = if let Some(scope_name) = scope {
        vec![config_schema_scope(scope_name).ok_or_else(|| {
            ConfigError::not_found(format!("Unknown config schema scope: {scope_name}"))
        })?]
    } else {
        config_schema_registry()
    };

    let mut markdown = String::from(
        "---\n\
title: Generated Config Reference\n\
audience: mixed\n\
type: generated-reference\n\
status: canonical\n\
owner: bijux-cli-docs\n\
generated_from: bijux-cli-config-schema-registry-v1\n\
---\n\n\
# Generated Config Reference\n\n\
This page is generated from the built-in `bijux-cli` config schema registry.\n\
Use `bijux config docs --format json` when you need the same content from the runtime.\n",
    );

    for scope_entry in scopes {
        markdown.push_str("\n## `");
        markdown.push_str(&scope_entry.scope);
        markdown.push_str("`\n\n");
        markdown.push_str(
            "| Logical key | Storage key | Type | Environment | Sensitive | Default | Deprecation | Description |\n",
        );
        markdown.push_str("| --- | --- | --- | --- | --- | --- | --- | --- |\n");
        for field in &scope_entry.fields {
            let env_vars = if field.env_vars.is_empty() {
                "-".to_string()
            } else {
                field
                    .env_vars
                    .iter()
                    .map(|value| format!("`{value}`"))
                    .collect::<Vec<_>>()
                    .join("<br>")
            };
            markdown.push_str(&format!(
                "| `{}` | `{}` | `{}` | {} | `{}` | {} | `{}` | {} |\n",
                field.logical_key,
                field.storage_key,
                serde_json::to_string(&field.value_kind)
                    .unwrap_or_else(|_| "\"string\"".to_string())
                    .trim_matches('"'),
                env_vars,
                if field.sensitive { "yes" } else { "no" },
                field
                    .default_value
                    .as_deref()
                    .map(|value| format!("`{value}`"))
                    .unwrap_or_else(|| "-".to_string()),
                serde_json::to_string(&field.deprecation_status)
                    .unwrap_or_else(|_| "\"active\"".to_string())
                    .trim_matches('"'),
                field.description.replace('|', "\\|"),
            ));
        }
    }

    Ok(markdown)
}

fn normalize_selector(raw_key: &str) -> Result<String, ConfigError> {
    normalize_selector_to_storage_key(raw_key)
}

fn profile_env_path(global_config_path: &Path, profile: Option<&str>) -> Option<PathBuf> {
    profile.map(|name| {
        global_config_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("profiles")
            .join(format!("{name}.env"))
    })
}

fn config_lock(path: &Path) -> Result<StateLockGuard, ConfigError> {
    let file_name = path.file_name().and_then(|entry| entry.to_str()).unwrap_or("config");
    let lock_path = path.with_file_name(format!("{file_name}.lock"));
    for attempt in 0..100 {
        match acquire_state_lock(&lock_path) {
            Ok(guard) => return Ok(guard),
            Err(CompatibilityError::LockHeld(_)) if attempt < 99 => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(err) => return Err(ConfigError::persistence(err.to_string())),
        }
    }
    Err(ConfigError::persistence(format!("state lock remained held at {}", lock_path.display())))
}

fn render_env_map(values: &BTreeMap<String, String>) -> String {
    let mut rendered = String::new();
    for (key, value) in values {
        let _ = std::fmt::Write::write_fmt(
            &mut rendered,
            format_args!("BIJUXCLI_{}={value}\n", key.to_ascii_uppercase()),
        );
    }
    rendered
}

struct RepairReport {
    entries: BTreeMap<String, String>,
    warnings: Vec<String>,
    issues: Vec<RepairIssue>,
}

#[derive(Debug, Clone)]
struct RepairIssue {
    line: usize,
    content: String,
    issue: String,
    remediation: String,
}

fn repair_env_text(text: &str) -> Result<RepairReport, ConfigError> {
    let mut entries = BTreeMap::new();
    let mut warnings = Vec::new();
    let mut issues = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, raw_line) in text.lines().enumerate() {
        let line_no = index + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((raw_key, raw_value)) = raw_line.split_once('=') else {
            warnings.push(format!("Dropped malformed line {line_no}: {raw_line}"));
            issues.push(RepairIssue {
                line: line_no,
                content: raw_line.to_string(),
                issue: "malformed-line".to_string(),
                remediation: "Use KEY=VALUE format for each non-comment config line.".to_string(),
            });
            continue;
        };
        let normalized = match normalize_selector(raw_key) {
            Ok(value) => value,
            Err(err) => {
                warnings.push(format!(
                    "Dropped invalid key at line {line_no}: {} ({err})",
                    raw_key.trim()
                ));
                issues.push(RepairIssue {
                    line: line_no,
                    content: raw_line.to_string(),
                    issue: "invalid-key".to_string(),
                    remediation:
                        "Use ASCII keys with letters, numbers, underscore, or dot notation."
                            .to_string(),
                });
                continue;
            }
        };
        if let Err(err) = validate_schema_value(&normalized, raw_value.trim()) {
            warnings
                .push(format!("Dropped invalid value for `{normalized}` at line {line_no}: {err}"));
            issues.push(RepairIssue {
                line: line_no,
                content: raw_line.to_string(),
                issue: "invalid-value".to_string(),
                remediation: format!(
                    "Provide a value compatible with the schema for `{normalized}`."
                ),
            });
            continue;
        }
        if seen.contains(&normalized) {
            warnings.push(format!(
                "Replaced duplicate key `{normalized}` at line {line_no} with last value"
            ));
        }
        seen.insert(normalized.clone());
        entries.insert(normalized, raw_value.trim().to_string());
    }
    Ok(RepairReport { entries, warnings, issues })
}

fn parse_portable_entries(
    entries: &Map<String, Value>,
) -> Result<BTreeMap<String, String>, ConfigError> {
    let mut out = BTreeMap::new();
    for (logical_key, payload) in entries {
        let storage_key = logical_key_to_storage_key(logical_key);
        let value = payload.get("value").and_then(Value::as_str).ok_or_else(|| {
            ConfigError::parse(format!("Portable entry `{logical_key}` is missing string `value`"))
        })?;
        validate_schema_value(&storage_key, value)?;
        out.insert(storage_key, value.to_string());
    }
    Ok(out)
}

fn write_json_pretty(path: &Path, payload: &Value) -> Result<(), ConfigError> {
    let _guard = config_lock(path)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| ConfigError::persistence(err.to_string()))?;
    }
    let rendered = serde_json::to_string_pretty(payload)
        .map_err(|err| ConfigError::persistence(err.to_string()))?;
    crate::infrastructure::fs_store::atomic_write_text(path, &(rendered + "\n"))
        .map_err(|err| ConfigError::persistence(err.to_string()))
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{LazyLock, Mutex, MutexGuard};
    use std::time::{SystemTime, UNIX_EPOCH};

    use serde_json::json;

    use crate::contracts::ConfigSchemaRegistryV1;
    use crate::features::config::schema::CONFIG_SCHEMA_REGISTRY_VERSION;

    use super::{
        diff_report, discover_project_config, explain_report, parse_portable_entries,
        repair_env_text, resolve_layered_config, schema_docs_report, schema_report,
        validate_report, LayeredConfigOptions,
    };

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    struct EnvVarGuard {
        key: &'static str,
        original: Option<OsString>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let original = std::env::var_os(key);
            std::env::set_var(key, value);
            Self { key, original }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(value) = &self.original {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    fn env_lock() -> MutexGuard<'static, ()> {
        ENV_LOCK.lock().expect("env lock")
    }

    fn temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
        let root = std::env::temp_dir().join(format!("bijux-layered-config-{name}-{nanos}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("mkdir");
        root
    }

    #[test]
    fn schema_report_exposes_cli_scope() {
        let payload = schema_report(Some("cli")).expect("schema");
        assert_eq!(payload["scope"], "cli");
        assert!(payload["fields"].as_array().is_some_and(|items| !items.is_empty()));
    }

    #[test]
    fn schema_report_roundtrip_uses_versioned_registry_contract() {
        let payload = schema_report(None).expect("schema");
        assert_eq!(payload["schema_version"], CONFIG_SCHEMA_REGISTRY_VERSION);
        let registry_payload = json!({
            "schema_version": payload["schema_version"],
            "scopes": payload["scopes"],
        });
        let registry: ConfigSchemaRegistryV1 =
            serde_json::from_value(registry_payload).expect("registry contract");
        assert!(!registry.scopes.is_empty(), "registry should expose at least one scope");
    }

    #[test]
    fn schema_docs_report_emits_markdown_reference() {
        let payload = schema_docs_report(Some("cli")).expect("docs");
        let markdown = payload["markdown"].as_str().expect("markdown");
        assert!(markdown.contains("# Generated Config Reference"));
        assert!(markdown.contains("## `cli`"));
        assert!(markdown.contains("`cli.access_token`"));
        assert!(markdown.contains("| Logical key | Storage key | Type | Environment | Sensitive | Default | Deprecation | Description |"));
    }

    #[test]
    fn schema_docs_report_matches_checked_in_generated_reference() {
        let payload = schema_docs_report(None).expect("docs");
        let markdown = payload["markdown"].as_str().expect("markdown");
        let reference_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/bijux-cli/interfaces/generated-config-reference.md");
        let expected = fs::read_to_string(reference_path).expect("generated reference doc");
        assert_eq!(format!("{markdown}\n"), expected);
    }

    #[test]
    fn project_discovery_prefers_toml_and_profile_overlay() {
        let root = temp_dir("project-discovery");
        let project = root.join("repo");
        fs::create_dir_all(project.join(".bijux/profiles")).expect("mkdir");
        fs::write(project.join(".bijux/config.toml"), "[dag]\njobs = 4\n").expect("toml");
        fs::write(project.join(".bijux/config.json"), "{\"dag\":{\"jobs\":3}}").expect("json");
        fs::write(project.join(".bijux/profiles/dev.toml"), "[cli]\nlog_level = 'debug'\n")
            .expect("profile");

        let discovery = discover_project_config(&project, Some("dev")).expect("discovery");
        assert_eq!(discovery.config_format, Some("toml"));
        assert_eq!(discovery.profile_format, Some("toml"));
    }

    #[test]
    fn layered_resolution_merges_global_project_and_env_layers() {
        let _env_lock = env_lock();
        let root = temp_dir("resolve");
        let global = root.join("global.env");
        let project = root.join("project");
        fs::create_dir_all(project.join(".bijux/profiles")).expect("mkdir");
        fs::write(&global, "BIJUXCLI_CLI_LOG_LEVEL=info\n").expect("global");
        fs::write(project.join(".bijux/config.toml"), "[dag]\njobs = 3\n").expect("project");
        fs::write(project.join(".bijux/profiles/dev.toml"), "[dag]\ncache_mode = 'strict'\n")
            .expect("profile");
        let _dag_jobs = EnvVarGuard::set("BIJUX_DAG_JOBS", "6");

        let resolved = resolve_layered_config(&global, &project, Some("dev")).expect("resolved");
        assert_eq!(
            resolved.effective_entries.get("cli_log_level").map(String::as_str),
            Some("info")
        );
        assert_eq!(resolved.effective_entries.get("dag_jobs").map(String::as_str), Some("6"));
        assert_eq!(
            resolved.effective_entries.get("dag_cache_mode").map(String::as_str),
            Some("strict")
        );
    }

    #[test]
    fn explain_report_redacts_secret_like_values() {
        let root = temp_dir("explain");
        let global = root.join("global.env");
        fs::write(&global, "BIJUXCLI_CLI_ACCESS_TOKEN=secret-token\n").expect("global");

        let payload = explain_report(&global, &root, "cli.access_token", None, &Vec::new(), false)
            .expect("explain");
        assert_eq!(payload["effective"]["value"], "[redacted]");
    }

    #[test]
    fn diff_report_compares_profiles_with_redaction() {
        let root = temp_dir("diff");
        let global = root.join("global.env");
        let project = root.join("project");
        fs::create_dir_all(project.join(".bijux/profiles")).expect("mkdir");
        fs::write(&global, "BIJUXCLI_CLI_LOG_LEVEL=info\nBIJUXCLI_CLI_ACCESS_TOKEN=alpha\n")
            .expect("global");
        fs::write(project.join(".bijux/profiles/dev.toml"), "[cli]\nlog_level = 'debug'\n")
            .expect("profile");

        let payload = diff_report(&global, &project, None, None, Some("dev"), &Vec::new(), false)
            .expect("diff");
        assert_eq!(payload["status"], "ok");
        assert!(payload["changed_count"].as_u64().is_some_and(|count| count >= 1));
        let changes = payload["changes"].as_array().expect("changes");
        assert!(changes.iter().any(|entry| entry["logical_key"] == "cli.log_level"));
    }

    #[test]
    fn validate_report_declares_deterministic_precedence_and_cli_overrides() {
        let _env_lock = env_lock();
        let root = temp_dir("validate-precedence");
        let global = root.join("global.env");
        let project = root.join("project");
        fs::create_dir_all(project.join(".bijux")).expect("mkdir");
        fs::write(&global, "BIJUXCLI_CLI_LOG_LEVEL=info\n").expect("global");
        fs::write(project.join(".bijux/config.toml"), "[cli]\nlog_level = 'warn'\n")
            .expect("project");
        let _log_level = EnvVarGuard::set("BIJUXCLI_CLI_LOG_LEVEL", "error");

        let payload =
            validate_report(&global, &project, None, &["cli.log_level=debug".to_string()])
                .expect("validate");
        assert_eq!(
            payload["precedence"],
            json!([
                "defaults",
                "global_file",
                "global_profile",
                "project_file",
                "project_profile",
                "environment",
                "cli_overrides"
            ])
        );
        assert_eq!(payload["effective"]["cli.log_level"]["value"], "debug");
    }

    #[test]
    fn repair_env_text_drops_bad_lines_and_keeps_last_duplicate() {
        let repaired =
            repair_env_text("BIJUXCLI_ALPHA=1\nBROKEN\nBIJUXCLI_ALPHA=2\n").expect("repair");
        assert_eq!(repaired.entries.get("alpha").map(String::as_str), Some("2"));
        assert_eq!(repaired.warnings.len(), 2);
        assert_eq!(repaired.issues.len(), 1);
        assert_eq!(repaired.issues[0].issue, "malformed-line");
    }

    #[test]
    fn repair_env_text_drops_invalid_keys_without_aborting_repair() {
        let repaired = repair_env_text("BIJUXCLI_ALPHA=1\nBIJUXCLI_BÄD=2\n").expect("repair");
        assert_eq!(repaired.entries.get("alpha").map(String::as_str), Some("1"));
        assert_eq!(repaired.issues.len(), 1);
        assert_eq!(repaired.issues[0].issue, "invalid-key");
        assert!(repaired.issues[0].remediation.contains("ASCII keys"));
    }

    #[test]
    fn portable_entries_require_string_values() {
        let err =
            parse_portable_entries(json!({"dag.jobs": {"value": 2}}).as_object().expect("object"))
                .expect_err("must fail");
        assert!(err.to_string().contains("missing string `value`"));
    }

    #[test]
    fn layered_options_default_to_nonportable_and_redacted() {
        let options = LayeredConfigOptions::default();
        assert!(!options.portable);
        assert!(!options.include_secrets);
        assert!(options.profile.is_none());
    }
}
