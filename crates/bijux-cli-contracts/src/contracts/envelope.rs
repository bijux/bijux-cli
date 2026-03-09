use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::command::CommandPath;

/// Stable output envelope metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OutputEnvelopeMetaV1 {
    /// Envelope version identifier.
    pub version: String,
    /// Canonical command path.
    pub command: CommandPath,
    /// RFC3339 timestamp.
    pub timestamp: String,
}

impl OutputEnvelopeMetaV1 {
    /// Build metadata with required fields.
    pub fn new(version: &str, command: CommandPath, timestamp: &str) -> Result<Self, String> {
        if version.trim().is_empty() {
            return Err("meta.version cannot be empty".to_string());
        }
        if timestamp.trim().is_empty() {
            return Err("meta.timestamp cannot be empty".to_string());
        }
        Ok(Self { version: version.to_string(), command, timestamp: timestamp.to_string() })
    }
}

/// Stable success payload envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OutputEnvelopeV1 {
    /// Fixed status marker.
    pub status: String,
    /// Command-specific payload.
    pub data: Value,
    /// Shared metadata.
    pub meta: OutputEnvelopeMetaV1,
}

impl OutputEnvelopeV1 {
    /// Build a success envelope using fixed status.
    #[must_use]
    pub fn success(data: Value, meta: OutputEnvelopeMetaV1) -> Self {
        Self { status: "ok".to_string(), data, meta }
    }
}

/// Stable structured error details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Default)]
pub struct ErrorDetailsV1 {
    /// Stable machine failure identifier.
    pub failure: Option<String>,
    /// Arbitrary additional context.
    #[serde(default)]
    pub context: BTreeMap<String, Value>,
}

/// Stable structured error payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ErrorPayloadV1 {
    /// Stable symbolic error code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Error category (`usage`, `validation`, `plugin`, `internal`).
    pub category: String,
    /// Structured optional details.
    #[serde(default)]
    pub details: Option<ErrorDetailsV1>,
}

impl ErrorPayloadV1 {
    /// Build a validated error payload.
    pub fn new(code: &str, message: &str, category: &str) -> Result<Self, String> {
        if code.trim().is_empty() {
            return Err("error.code cannot be empty".to_string());
        }
        if message.trim().is_empty() {
            return Err("error.message cannot be empty".to_string());
        }
        if category.trim().is_empty() {
            return Err("error.category cannot be empty".to_string());
        }
        Ok(Self {
            code: code.to_string(),
            message: message.to_string(),
            category: category.to_string(),
            details: None,
        })
    }
}

/// Stable error envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ErrorEnvelopeV1 {
    /// Fixed status marker.
    pub status: String,
    /// Structured error payload.
    pub error: ErrorPayloadV1,
    /// Shared metadata.
    pub meta: OutputEnvelopeMetaV1,
}

impl ErrorEnvelopeV1 {
    /// Build an error envelope using fixed status.
    #[must_use]
    pub fn failure(error: ErrorPayloadV1, meta: OutputEnvelopeMetaV1) -> Self {
        Self { status: "error".to_string(), error, meta }
    }
}
