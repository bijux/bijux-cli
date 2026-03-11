#![forbid(unsafe_code)]
//! Routing graph, parser normalization, and namespace resolution.

pub mod catalog;
pub mod inventory;
pub mod parser;
pub mod query;
pub mod registry;
pub mod schema;

#[cfg(test)]
use proptest as _;

#[cfg(test)]
use serde as _;
#[cfg(test)]
use serde_json as _;

pub use crate::contracts::{
    known_bijux_tool, AliasRewrite, ColorMode, CommandMetadata, CommandPath, CompatibilityRange,
    ConfigClearResult, ConfigCommandResult, ConfigConflictError, ConfigEntry, ConfigErrorKind,
    ConfigExportFormat, ConfigKey, ConfigLoadResult, ConfigMutation, ConfigParseError,
    ConfigPathSet, ConfigPersistenceError, ConfigReadSource, ConfigReloadResult, ConfigSnapshot,
    ConfigSource, ConfigValidationError, ConfigValue, ConfigWriteResult, ContractMarker,
    DiagnosticRecord, ErrorDetailsV1, ErrorEnvelopeV1, ErrorPayloadV1, ExecutionPolicy, ExitCode,
    GlobalFlags, InspectReport, InvocationEvent, InvocationTrace, KnownBijuxTool, LogLevel,
    MemoryKeyList, MemorySummary, Namespace, NamespaceMetadata, OutputEnvelopeMetaV1,
    OutputEnvelopeV1, OutputFormat, PluginCapability, PluginKind, PluginLifecycleState,
    PluginManifestV1, PrettyMode, ProductMountMetadata, ResolvedConfigValue, RouteSourceMetadata,
    KNOWN_BIJUX_TOOLS, KNOWN_BIJUX_TOOL_NAMESPACES, OFFICIAL_PRODUCT_NAMESPACES,
};
