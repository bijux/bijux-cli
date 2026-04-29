#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::sync::OnceLock;

use serde::Serialize;

use crate::contracts::known_bijux_tools;

use super::error::ConfigError;
use super::validation::validate_value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ConfigValueKind {
    String,
    Integer,
    Boolean,
    Path,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ConfigSchemaField {
    pub scope: String,
    pub logical_key: String,
    pub storage_key: String,
    pub env_vars: Vec<String>,
    pub value_kind: ConfigValueKind,
    pub sensitive: bool,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ConfigSchemaScope {
    pub scope: String,
    pub source: String,
    pub fields: Vec<ConfigSchemaField>,
}

fn field(
    scope: &str,
    logical_key: &str,
    env_vars: &[&str],
    value_kind: ConfigValueKind,
    sensitive: bool,
    description: &str,
) -> ConfigSchemaField {
    ConfigSchemaField {
        scope: scope.to_string(),
        logical_key: logical_key.to_string(),
        storage_key: logical_key_to_storage_key(logical_key),
        env_vars: env_vars.iter().map(|value| (*value).to_string()).collect(),
        value_kind,
        sensitive,
        description: description.to_string(),
    }
}

fn cli_scope() -> ConfigSchemaScope {
    ConfigSchemaScope {
        scope: "cli".to_string(),
        source: "built_in".to_string(),
        fields: vec![
            field(
                "cli",
                "cli.color",
                &["BIJUXCLI_COLOR", "BIJUX_CLI_COLOR"],
                ConfigValueKind::String,
                false,
                "ANSI color policy for CLI text output.",
            ),
            field(
                "cli",
                "cli.log_level",
                &["BIJUXCLI_LOG_LEVEL", "BIJUX_CLI_LOG_LEVEL"],
                ConfigValueKind::String,
                false,
                "Global CLI log verbosity.",
            ),
            field(
                "cli",
                "cli.output_format",
                &["BIJUXCLI_FORMAT", "BIJUX_CLI_FORMAT"],
                ConfigValueKind::String,
                false,
                "Preferred machine output format.",
            ),
            field(
                "cli",
                "cli.profile",
                &["BIJUXCLI_PROFILE", "BIJUX_PROFILE"],
                ConfigValueKind::String,
                false,
                "Selected named config profile.",
            ),
            field(
                "cli",
                "cli.access_token",
                &["BIJUXCLI_ACCESS_TOKEN", "BIJUX_CLI_ACCESS_TOKEN"],
                ConfigValueKind::String,
                true,
                "Operator access token for authenticated CLI control-plane integrations.",
            ),
        ],
    }
}

fn dag_scope() -> ConfigSchemaScope {
    ConfigSchemaScope {
        scope: "dag".to_string(),
        source: "built_in".to_string(),
        fields: vec![
            field(
                "dag",
                "dag.cache_dir",
                &["BIJUX_DAG_CACHE_DIR"],
                ConfigValueKind::Path,
                false,
                "Local DAG cache directory.",
            ),
            field(
                "dag",
                "dag.adapters_dir",
                &["BIJUX_DAG_ADAPTERS_DIR"],
                ConfigValueKind::Path,
                false,
                "Directory containing external DAG adapters.",
            ),
            field(
                "dag",
                "dag.jobs",
                &["BIJUX_DAG_JOBS"],
                ConfigValueKind::Integer,
                false,
                "Maximum DAG execution parallelism.",
            ),
            field(
                "dag",
                "dag.cache_mode",
                &["BIJUX_DAG_CACHE_MODE"],
                ConfigValueKind::String,
                false,
                "DAG cache policy mode.",
            ),
            field(
                "dag",
                "dag.materialize_inputs",
                &["BIJUX_DAG_MATERIALIZE_INPUTS"],
                ConfigValueKind::Boolean,
                false,
                "Whether DAG runtime materializes node inputs eagerly.",
            ),
            field(
                "dag",
                "dag.policy_json",
                &["BIJUX_DAG_POLICY_JSON"],
                ConfigValueKind::Json,
                false,
                "Structured DAG runtime policy override.",
            ),
        ],
    }
}

fn generic_app_scope(scope: &str) -> ConfigSchemaScope {
    let env_prefix = scope.to_ascii_uppercase();
    ConfigSchemaScope {
        scope: scope.to_string(),
        source: "built_in_shared".to_string(),
        fields: vec![
            field(
                scope,
                &format!("{scope}.profile"),
                &[&format!("BIJUX_{env_prefix}_PROFILE")],
                ConfigValueKind::String,
                false,
                "Named runtime profile for the mounted app.",
            ),
            field(
                scope,
                &format!("{scope}.workspace_dir"),
                &[&format!("BIJUX_{env_prefix}_WORKSPACE_DIR")],
                ConfigValueKind::Path,
                false,
                "Preferred working directory for app execution and outputs.",
            ),
            field(
                scope,
                &format!("{scope}.log_level"),
                &[&format!("BIJUX_{env_prefix}_LOG_LEVEL")],
                ConfigValueKind::String,
                false,
                "Per-app log verbosity override.",
            ),
        ],
    }
}

fn load_schema_scopes() -> Vec<ConfigSchemaScope> {
    let mut scopes = vec![cli_scope(), dag_scope()];
    let mut seen = BTreeSet::from(["cli".to_string(), "dag".to_string()]);
    for tool in known_bijux_tools() {
        if seen.insert(tool.namespace.to_string()) {
            scopes.push(generic_app_scope(tool.namespace));
        }
    }
    scopes.sort_by(|left, right| left.scope.cmp(&right.scope));
    scopes
}

fn schema_scopes() -> &'static [ConfigSchemaScope] {
    static STORAGE: OnceLock<Vec<ConfigSchemaScope>> = OnceLock::new();
    STORAGE.get_or_init(load_schema_scopes).as_slice()
}

pub(crate) fn config_schema_scope(scope: &str) -> Option<ConfigSchemaScope> {
    schema_scopes().iter().find(|entry| entry.scope == scope).cloned()
}

pub(crate) fn config_schema_registry() -> Vec<ConfigSchemaScope> {
    schema_scopes().to_vec()
}

pub(crate) fn config_schema_fields() -> Vec<ConfigSchemaField> {
    schema_scopes().iter().flat_map(|scope| scope.fields.iter().cloned()).collect()
}

pub(crate) fn logical_key_to_storage_key(logical_key: &str) -> String {
    logical_key.trim().replace('.', "_").to_ascii_lowercase()
}

pub(crate) fn storage_key_to_logical_key(storage_key: &str) -> String {
    for field in config_schema_fields() {
        if field.storage_key == storage_key {
            return field.logical_key;
        }
    }
    storage_key.to_string()
}

pub(crate) fn normalize_selector_to_storage_key(raw: &str) -> Result<String, ConfigError> {
    let key = raw.trim();
    if key.is_empty() {
        return Err(ConfigError::validation("Key cannot be empty"));
    }
    if !key.is_ascii() {
        return Err(ConfigError::validation(
            "Non-ASCII characters are not allowed in config keys.",
        ));
    }
    let normalized = key
        .strip_prefix("BIJUXCLI_")
        .map(|value| value.to_ascii_lowercase())
        .or_else(|| key.strip_prefix("BIJUX_").map(|value| value.to_ascii_lowercase()))
        .unwrap_or_else(|| key.to_ascii_lowercase());
    let storage_key = normalized.replace('.', "_");
    if !storage_key.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
        return Err(ConfigError::validation(
            "Invalid key: only alphanumerics, underscore, and dot allowed.",
        ));
    }
    Ok(storage_key)
}

pub(crate) fn schema_field_for_key(storage_key: &str) -> Option<ConfigSchemaField> {
    config_schema_fields().into_iter().find(|field| field.storage_key == storage_key)
}

pub(crate) fn infer_scope(storage_key: &str) -> String {
    if let Some(field) = schema_field_for_key(storage_key) {
        return field.scope;
    }
    storage_key.split('_').next().unwrap_or("global").to_string()
}

pub(crate) fn is_secret_like(storage_key: &str) -> bool {
    if schema_field_for_key(storage_key).is_some_and(|field| field.sensitive) {
        return true;
    }
    let lower = storage_key.to_ascii_lowercase();
    ["secret", "token", "password", "credential", "apikey", "api_key", "private_key"]
        .iter()
        .any(|needle| lower.contains(needle))
}

pub(crate) fn redact_value(storage_key: &str, value: &str, include_secrets: bool) -> String {
    if include_secrets || !is_secret_like(storage_key) {
        value.to_string()
    } else {
        "[redacted]".to_string()
    }
}

pub(crate) fn validate_schema_value(storage_key: &str, value: &str) -> Result<(), ConfigError> {
    validate_value(value)?;
    if let Some(field) = schema_field_for_key(storage_key) {
        match field.value_kind {
            ConfigValueKind::Integer => {
                value.parse::<i64>().map_err(|_| {
                    ConfigError::validation(format!(
                        "Config key `{}` expects an integer value",
                        field.logical_key
                    ))
                })?;
            }
            ConfigValueKind::Boolean => match value {
                "true" | "false" | "1" | "0" => {}
                _ => {
                    return Err(ConfigError::validation(format!(
                        "Config key `{}` expects a boolean value",
                        field.logical_key
                    )));
                }
            },
            ConfigValueKind::Json => {
                serde_json::from_str::<serde_json::Value>(value).map_err(|_| {
                    ConfigError::validation(format!(
                        "Config key `{}` expects valid JSON text",
                        field.logical_key
                    ))
                })?;
            }
            ConfigValueKind::String | ConfigValueKind::Path => {}
        }
    }
    Ok(())
}

pub(crate) fn env_candidates_for_storage_key(storage_key: &str) -> Vec<String> {
    if let Some(field) = schema_field_for_key(storage_key) {
        let mut values = field.env_vars;
        let generic = [
            format!("BIJUXCLI_{}", storage_key.to_ascii_uppercase()),
            format!("BIJUX_{}", storage_key.to_ascii_uppercase()),
        ];
        for item in generic {
            if !values.contains(&item) {
                values.push(item);
            }
        }
        return values;
    }
    vec![
        format!("BIJUXCLI_{}", storage_key.to_ascii_uppercase()),
        format!("BIJUX_{}", storage_key.to_ascii_uppercase()),
    ]
}
