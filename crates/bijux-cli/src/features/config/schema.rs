#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::sync::OnceLock;

use crate::contracts::known_bijux_tools;
use crate::contracts::{
    ConfigDeprecationStatusV1, ConfigSchemaFieldV1, ConfigSchemaRegistryV1, ConfigSchemaScopeV1,
    ConfigSchemaSourceV1, ConfigSchemaValueKindV1,
};

use super::error::ConfigError;
use super::validation::validate_value;

pub(crate) const CONFIG_SCHEMA_REGISTRY_VERSION: &str = "bijux-cli-config-schema-registry-v1";

fn field(
    scope: &str,
    logical_key: &str,
    env_vars: &[&str],
    value_kind: ConfigSchemaValueKindV1,
    sensitive: bool,
    default_value: Option<&str>,
    deprecation_status: ConfigDeprecationStatusV1,
    description: &str,
) -> ConfigSchemaFieldV1 {
    ConfigSchemaFieldV1 {
        scope: scope.to_string(),
        logical_key: logical_key.to_string(),
        storage_key: logical_key_to_storage_key(logical_key),
        env_vars: env_vars.iter().map(|value| (*value).to_string()).collect(),
        value_kind,
        sensitive,
        default_value: default_value.map(ToString::to_string),
        deprecation_status,
        description: description.to_string(),
    }
}

fn cli_scope() -> ConfigSchemaScopeV1 {
    ConfigSchemaScopeV1 {
        scope: "cli".to_string(),
        source: ConfigSchemaSourceV1::BuiltIn,
        fields: vec![
            field(
                "cli",
                "cli.color",
                &["BIJUXCLI_COLOR", "BIJUX_CLI_COLOR"],
                ConfigSchemaValueKindV1::String,
                false,
                Some("auto"),
                ConfigDeprecationStatusV1::Active,
                "ANSI color policy for CLI text output.",
            ),
            field(
                "cli",
                "cli.log_level",
                &["BIJUXCLI_LOG_LEVEL", "BIJUX_CLI_LOG_LEVEL"],
                ConfigSchemaValueKindV1::String,
                false,
                Some("info"),
                ConfigDeprecationStatusV1::Active,
                "Global CLI log verbosity.",
            ),
            field(
                "cli",
                "cli.output_format",
                &["BIJUXCLI_FORMAT", "BIJUX_CLI_FORMAT"],
                ConfigSchemaValueKindV1::String,
                false,
                None,
                ConfigDeprecationStatusV1::Active,
                "Preferred machine output format.",
            ),
            field(
                "cli",
                "cli.profile",
                &["BIJUXCLI_PROFILE", "BIJUX_PROFILE"],
                ConfigSchemaValueKindV1::String,
                false,
                None,
                ConfigDeprecationStatusV1::Active,
                "Selected named config profile.",
            ),
            field(
                "cli",
                "cli.access_token",
                &["BIJUXCLI_ACCESS_TOKEN", "BIJUX_CLI_ACCESS_TOKEN"],
                ConfigSchemaValueKindV1::String,
                true,
                None,
                ConfigDeprecationStatusV1::Active,
                "Operator access token for authenticated CLI control-plane integrations.",
            ),
        ],
    }
}

fn dag_scope() -> ConfigSchemaScopeV1 {
    ConfigSchemaScopeV1 {
        scope: "dag".to_string(),
        source: ConfigSchemaSourceV1::BuiltIn,
        fields: vec![
            field(
                "dag",
                "dag.cache_dir",
                &["BIJUX_DAG_CACHE_DIR"],
                ConfigSchemaValueKindV1::Path,
                false,
                None,
                ConfigDeprecationStatusV1::Active,
                "Local DAG cache directory.",
            ),
            field(
                "dag",
                "dag.adapters_dir",
                &["BIJUX_DAG_ADAPTERS_DIR"],
                ConfigSchemaValueKindV1::Path,
                false,
                None,
                ConfigDeprecationStatusV1::Active,
                "Directory containing external DAG adapters.",
            ),
            field(
                "dag",
                "dag.jobs",
                &["BIJUX_DAG_JOBS"],
                ConfigSchemaValueKindV1::Integer,
                false,
                None,
                ConfigDeprecationStatusV1::Active,
                "Maximum DAG execution parallelism.",
            ),
            field(
                "dag",
                "dag.cache_mode",
                &["BIJUX_DAG_CACHE_MODE"],
                ConfigSchemaValueKindV1::String,
                false,
                None,
                ConfigDeprecationStatusV1::Active,
                "DAG cache policy mode.",
            ),
            field(
                "dag",
                "dag.materialize_inputs",
                &["BIJUX_DAG_MATERIALIZE_INPUTS"],
                ConfigSchemaValueKindV1::Boolean,
                false,
                Some("false"),
                ConfigDeprecationStatusV1::Active,
                "Whether DAG runtime materializes node inputs eagerly.",
            ),
            field(
                "dag",
                "dag.policy_json",
                &["BIJUX_DAG_POLICY_JSON"],
                ConfigSchemaValueKindV1::Json,
                false,
                None,
                ConfigDeprecationStatusV1::Active,
                "Structured DAG runtime policy override.",
            ),
        ],
    }
}

fn generic_app_scope(scope: &str) -> ConfigSchemaScopeV1 {
    let env_prefix = scope.to_ascii_uppercase();
    ConfigSchemaScopeV1 {
        scope: scope.to_string(),
        source: ConfigSchemaSourceV1::BuiltInShared,
        fields: vec![
            field(
                scope,
                &format!("{scope}.profile"),
                &[&format!("BIJUX_{env_prefix}_PROFILE")],
                ConfigSchemaValueKindV1::String,
                false,
                None,
                ConfigDeprecationStatusV1::Active,
                "Named runtime profile for the mounted app.",
            ),
            field(
                scope,
                &format!("{scope}.workspace_dir"),
                &[&format!("BIJUX_{env_prefix}_WORKSPACE_DIR")],
                ConfigSchemaValueKindV1::Path,
                false,
                None,
                ConfigDeprecationStatusV1::Active,
                "Preferred working directory for app execution and outputs.",
            ),
            field(
                scope,
                &format!("{scope}.log_level"),
                &[&format!("BIJUX_{env_prefix}_LOG_LEVEL")],
                ConfigSchemaValueKindV1::String,
                false,
                Some("info"),
                ConfigDeprecationStatusV1::Active,
                "Per-app log verbosity override.",
            ),
        ],
    }
}

fn load_schema_scopes() -> Vec<ConfigSchemaScopeV1> {
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

fn schema_scopes() -> &'static [ConfigSchemaScopeV1] {
    static STORAGE: OnceLock<Vec<ConfigSchemaScopeV1>> = OnceLock::new();
    STORAGE.get_or_init(load_schema_scopes).as_slice()
}

pub(crate) fn config_schema_scope(scope: &str) -> Option<ConfigSchemaScopeV1> {
    schema_scopes().iter().find(|entry| entry.scope == scope).cloned()
}

pub(crate) fn config_schema_registry() -> Vec<ConfigSchemaScopeV1> {
    schema_scopes().to_vec()
}

pub(crate) fn config_schema_registry_v1() -> ConfigSchemaRegistryV1 {
    ConfigSchemaRegistryV1 {
        schema_version: CONFIG_SCHEMA_REGISTRY_VERSION.to_string(),
        scopes: config_schema_registry(),
    }
}

pub(crate) fn config_schema_fields() -> Vec<ConfigSchemaFieldV1> {
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

pub(crate) fn schema_field_for_key(storage_key: &str) -> Option<ConfigSchemaFieldV1> {
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
            ConfigSchemaValueKindV1::Integer => {
                value.parse::<i64>().map_err(|_| {
                    ConfigError::validation(format!(
                        "Config key `{}` expects an integer value",
                        field.logical_key
                    ))
                })?;
            }
            ConfigSchemaValueKindV1::Boolean => match value {
                "true" | "false" | "1" | "0" => {}
                _ => {
                    return Err(ConfigError::validation(format!(
                        "Config key `{}` expects a boolean value",
                        field.logical_key
                    )));
                }
            },
            ConfigSchemaValueKindV1::Json => {
                serde_json::from_str::<serde_json::Value>(value).map_err(|_| {
                    ConfigError::validation(format!(
                        "Config key `{}` expects valid JSON text",
                        field.logical_key
                    ))
                })?;
            }
            ConfigSchemaValueKindV1::String | ConfigSchemaValueKindV1::Path => {}
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
