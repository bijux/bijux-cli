#![forbid(unsafe_code)]
//! Shared durable contracts for all Rust bijux-cli crates.

#[cfg(test)]
use proptest as _;

pub mod contracts;
pub mod schema;

pub use contracts::{
    ColorMode, CommandMetadata, CommandPath, CompatibilityRange, ConfigSource, ContractMarker,
    DiagnosticRecord, ErrorDetailsV1, ErrorEnvelopeV1, ErrorPayloadV1, ExecutionPolicy, ExitCode,
    GlobalFlags, InvocationEvent, InvocationTrace, LogLevel, MemoryKeyList, MemorySummary,
    Namespace, NamespaceMetadata,
    OutputEnvelopeMetaV1, OutputEnvelopeV1, OutputFormat, PluginCapability, PluginKind,
    PluginLifecycleState, PluginManifestV1, PrettyMode,
};
