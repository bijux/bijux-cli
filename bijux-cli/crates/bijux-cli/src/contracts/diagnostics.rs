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
    /// Event timestamp as unix milliseconds encoded as string.
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

/// Stable route metadata used by inspect and dev diagnostics surfaces.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RouteSourceMetadata {
    /// Route command segments.
    pub segments: Vec<String>,
    /// Route owner (`bijux-cli` or `plugin`).
    pub owner: String,
    /// Resolution source (`built-in`, `compatibility-alias`, `plugin`).
    pub source: String,
}

/// Stable alias rewrite metadata for routing diagnostics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AliasRewrite {
    /// Alias path segments.
    pub alias: Vec<String>,
    /// Canonical normalized path segments.
    pub canonical: Vec<String>,
    /// Rewrite source marker.
    pub source: String,
}

/// Stable inspect payload contract for machine-readable diagnostics output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct InspectReport {
    /// Command execution status.
    pub status: String,
    /// Route source metadata rows.
    pub route_sources: Vec<RouteSourceMetadata>,
    /// Alias rewrite metadata rows.
    pub alias_rewrites: Vec<AliasRewrite>,
}
