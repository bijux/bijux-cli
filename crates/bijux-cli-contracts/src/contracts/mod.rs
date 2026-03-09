//! Durable typed contracts grouped by functional surface.

/// Command-path and namespace contracts.
pub mod command;
/// Config domain contracts.
pub mod config;
/// Diagnostic and trace contracts.
pub mod diagnostics;
/// Output and error envelope contracts.
pub mod envelope;
/// Execution-policy and flag contracts.
pub mod execution;
/// Shared marker contracts.
pub mod marker;
/// Plugin manifest and compatibility contracts.
pub mod plugin;

pub use command::{CommandMetadata, CommandPath, Namespace, NamespaceMetadata};
pub use config::{
    ConfigClearResult, ConfigCommandResult, ConfigConflictError, ConfigEntry, ConfigErrorKind,
    ConfigExportFormat, ConfigKey, ConfigLoadResult, ConfigMutation, ConfigParseError,
    ConfigPathSet, ConfigPersistenceError, ConfigReloadResult, ConfigSnapshot,
    ConfigSource as ConfigReadSource, ConfigValidationError, ConfigValue, ConfigWriteResult,
    ResolvedConfigValue,
};
pub use diagnostics::{
    AliasRewrite, DiagnosticRecord, InspectReport, InvocationEvent, InvocationTrace, MemoryKeyList,
    MemorySummary, RouteSourceMetadata,
};
pub use envelope::{
    ErrorDetailsV1, ErrorEnvelopeV1, ErrorPayloadV1, OutputEnvelopeMetaV1, OutputEnvelopeV1,
};
pub use execution::{
    ColorMode, ConfigSource, ExecutionPolicy, ExitCode, GlobalFlags, LogLevel, OutputFormat,
    PrettyMode,
};
pub use marker::ContractMarker;
pub use plugin::{
    CompatibilityRange, PluginCapability, PluginKind, PluginLifecycleState, PluginManifestV1,
};
