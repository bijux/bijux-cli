use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Normalized config key.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
pub struct ConfigKey(pub String);

impl ConfigKey {
    /// Build a validated key.
    pub fn new(raw: &str) -> Result<Self, String> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err("config key cannot be empty".to_string());
        }
        if !trimmed.is_ascii() {
            return Err("config key must be ASCII".to_string());
        }
        if trimmed.contains('.') {
            return Err("config key cannot contain section separator '.'".to_string());
        }

        let normalized = trimmed
            .strip_prefix("BIJUXCLI_")
            .or_else(|| trimmed.strip_prefix("BIJUX_"))
            .unwrap_or(trimmed)
            .to_ascii_lowercase();
        if !normalized.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
            return Err("config key must contain only alphanumerics and '_'".to_string());
        }
        Ok(Self(normalized))
    }

    /// Borrow normalized key.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Validated config value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigValue(pub String);

impl ConfigValue {
    /// Build a validated value.
    pub fn new(raw: &str) -> Result<Self, String> {
        if !raw.is_ascii() {
            return Err("config value must be ASCII".to_string());
        }
        if raw.chars().any(|ch| matches!(ch, '\r' | '\n' | '\t' | '\u{000B}' | '\u{000C}')) {
            return Err("config value cannot contain control characters".to_string());
        }
        Ok(Self(raw.to_string()))
    }

    /// Borrow value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One key/value config entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigEntry {
    /// Entry key.
    pub key: ConfigKey,
    /// Entry value.
    pub value: ConfigValue,
}

/// Snapshot of stored config state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigSnapshot {
    /// All active entries by normalized key.
    pub entries: BTreeMap<ConfigKey, ConfigValue>,
}

/// Config mutation operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ConfigMutation {
    /// Insert or update a key.
    Set {
        /// Key to set.
        key: ConfigKey,
        /// Value to persist.
        value: ConfigValue,
    },
    /// Remove a key if present.
    Unset {
        /// Key to remove.
        key: ConfigKey,
    },
    /// Remove all keys.
    Clear,
}

/// Source of resolved config read value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ConfigSource {
    /// Command-line argument source.
    Flags,
    /// Environment variable source.
    Env,
    /// File-backed config source.
    File,
    /// Built-in defaults source.
    Defaults,
}

/// Resolved value including source metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ResolvedConfigValue {
    /// Resolved key.
    pub key: ConfigKey,
    /// Resolved value.
    pub value: ConfigValue,
    /// Winning source.
    pub source: ConfigSource,
    /// Source file when applicable.
    pub source_path: Option<String>,
}

/// Canonical file paths used by config operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigPathSet {
    /// Active config file path.
    pub config_file: String,
    /// Active history file path.
    pub history_file: String,
    /// Active plugin directory path.
    pub plugins_dir: String,
}

/// Result of loading config state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigLoadResult {
    /// Loaded snapshot.
    pub snapshot: ConfigSnapshot,
    /// Paths used for this load.
    pub paths: ConfigPathSet,
}

/// Result of persisting config state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigWriteResult {
    /// Whether storage was updated.
    pub updated: bool,
    /// Number of entries after write.
    pub entry_count: usize,
    /// Target path written.
    pub target_path: String,
}

/// Stable value kind contract for schema-registry fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConfigSchemaValueKindV1 {
    /// UTF-8 text value.
    String,
    /// Signed integer value.
    Integer,
    /// Boolean value.
    Boolean,
    /// Filesystem path value.
    Path,
    /// JSON text value.
    Json,
}

/// Stable field source classification for config schema entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConfigSchemaSourceV1 {
    /// Field is owned directly by this crate.
    BuiltIn,
    /// Field is generated from shared app policy.
    BuiltInShared,
}

/// Stable deprecation status for config schema entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConfigDeprecationStatusV1 {
    /// Field is active and fully supported.
    Active,
    /// Field remains available but is scheduled for removal.
    Deprecated,
}

/// Stable schema-field contract for one logical config key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigSchemaFieldV1 {
    /// Scope identifier (`cli`, `dag`, mounted app namespace, ...).
    pub scope: String,
    /// Operator-facing logical dotted key.
    pub logical_key: String,
    /// Storage key persisted in env-style files.
    pub storage_key: String,
    /// Environment variable aliases checked in precedence order.
    pub env_vars: Vec<String>,
    /// Field value kind.
    pub value_kind: ConfigSchemaValueKindV1,
    /// Whether the value is secret-bearing and should be redacted by default.
    pub sensitive: bool,
    /// Optional default value used by core runtime policy.
    pub default_value: Option<String>,
    /// Deprecation status marker.
    pub deprecation_status: ConfigDeprecationStatusV1,
    /// Human description for docs and explain surfaces.
    pub description: String,
}

/// Stable registry scope contract grouping config keys by scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigSchemaScopeV1 {
    /// Scope identifier.
    pub scope: String,
    /// Scope source class.
    pub source: ConfigSchemaSourceV1,
    /// Fields within this scope.
    pub fields: Vec<ConfigSchemaFieldV1>,
}

/// Stable versioned config-schema registry contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigSchemaRegistryV1 {
    /// Registry schema id.
    pub schema_version: String,
    /// Scope inventory.
    pub scopes: Vec<ConfigSchemaScopeV1>,
}

/// Export format for config output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ConfigExportFormat {
    /// Dotenv output.
    Env,
    /// JSON output.
    Json,
    /// YAML output.
    Yaml,
}

/// Generic config command result envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigCommandResult {
    /// Command status marker.
    pub status: String,
    /// Command target path.
    pub command: String,
}

/// Config error category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigErrorKind {
    /// Key/value validation failed.
    Validation,
    /// Config text parsing failed.
    Parse,
    /// File persistence failed.
    Persistence,
    /// Concurrent or semantic conflict detected.
    Conflict,
    /// Requested key/value not found.
    NotFound,
}

/// Key/value validation failure details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigValidationError {
    /// Failing key when available.
    pub key: Option<ConfigKey>,
    /// Validation reason.
    pub message: String,
}

/// Config parse failure details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigParseError {
    /// 1-based line number.
    pub line: usize,
    /// Raw line content.
    pub content: String,
    /// Parse reason.
    pub message: String,
}

/// Config persistence failure details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigPersistenceError {
    /// Target path.
    pub path: String,
    /// Operation name.
    pub operation: String,
    /// Failure reason.
    pub message: String,
}

/// Config conflict details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigConflictError {
    /// Conflicting key.
    pub key: Option<ConfigKey>,
    /// Conflict reason.
    pub message: String,
}

/// Result payload for reload operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigReloadResult {
    /// Reload status.
    pub status: String,
    /// Path reloaded.
    pub reloaded_path: String,
}

/// Result payload for clear operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigClearResult {
    /// Clear status.
    pub status: String,
    /// Number of removed keys.
    pub removed_keys: usize,
}
