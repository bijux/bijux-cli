use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::command::CommandPath;
use super::execution::ExecutionPolicy;

/// Stable diagnostic record contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DiagnosticRecord {
    /// Stable diagnostic identifier.
    pub id: String,
    /// Severity (`info`, `warning`, `error`).
    pub severity: String,
    /// Human-readable message.
    pub message: String,
    /// Machine-readable context.
    pub fields: BTreeMap<String, Value>,
}

/// Stable invocation event used by trace logs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct InvocationEvent {
    /// Event timestamp in RFC3339.
    pub timestamp: String,
    /// Event name.
    pub name: String,
    /// Event payload.
    pub payload: BTreeMap<String, Value>,
}

/// Stable invocation trace contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct InvocationTrace {
    /// Unique invocation identifier.
    pub invocation_id: String,
    /// Original command path.
    pub command: CommandPath,
    /// Effective execution policy.
    pub policy: ExecutionPolicy,
    /// Ordered execution events.
    pub events: Vec<InvocationEvent>,
}

/// Stable memory summary payload contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MemorySummary {
    /// Command execution status.
    pub status: String,
    /// Number of keys stored in memory state.
    pub count: usize,
    /// Human-readable summary message.
    pub message: String,
}

/// Stable memory key listing payload contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MemoryKeyList {
    /// Command execution status.
    pub status: String,
    /// Sorted memory keys.
    pub keys: Vec<String>,
    /// Number of keys returned.
    pub count: usize,
}
