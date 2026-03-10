#![forbid(unsafe_code)]
//! Shared durable contracts for all Rust bijux-cli crates.

#[cfg(test)]
use proptest as _;

pub mod contracts;
pub mod schema;

pub use contracts::{
    AliasRewrite, ColorMode, CommandMetadata, CommandPath, CompatibilityRange, ConfigClearResult,
    ConfigCommandResult, ConfigConflictError, ConfigEntry, ConfigErrorKind, ConfigExportFormat,
    ConfigKey, ConfigLoadResult, ConfigMutation, ConfigParseError, ConfigPathSet,
    ConfigPersistenceError, ConfigReadSource, ConfigReloadResult, ConfigSnapshot, ConfigSource,
    ConfigValidationError, ConfigValue, ConfigWriteResult, ContractMarker, DiagnosticRecord,
    ErrorDetailsV1, ErrorEnvelopeV1, ErrorPayloadV1, ExecutionPolicy, ExitCode, GlobalFlags,
    InspectReport, InvocationEvent, InvocationTrace, LogLevel, MemoryKeyList, MemorySummary,
    Namespace, NamespaceMetadata, OutputEnvelopeMetaV1, OutputEnvelopeV1, OutputFormat,
    PluginCapability, PluginKind, PluginLifecycleState, PluginManifestV1, PrettyMode,
    ProductMountMetadata, ResolvedConfigValue, RouteSourceMetadata, OFFICIAL_PRODUCT_NAMESPACES,
};
