#![forbid(unsafe_code)]
//! Shared durable contracts for all Rust bijux-cli crates.

pub mod contracts;
pub mod schema;

pub use contracts::{
    ColorMode, CommandMetadata, CommandPath, CompatibilityRange, ConfigSource, ContractMarker,
    DiagnosticRecord, ErrorDetailsV1, ErrorEnvelopeV1, ErrorPayloadV1, ExecutionPolicy, ExitCode,
    GlobalFlags, InvocationEvent, InvocationTrace, LogLevel, Namespace, NamespaceMetadata,
    OutputEnvelopeMetaV1, OutputEnvelopeV1, OutputFormat, PluginCapability, PluginManifestV1,
    PrettyMode,
};
