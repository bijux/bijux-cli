#![forbid(unsafe_code)]
//! Command-surface contracts plus routing graph and namespace resolution.

#[path = "../../bijux-cli/src/routing/catalog.rs"]
pub mod catalog;
#[path = "../../bijux-cli/src/routing/contracts/mod.rs"]
pub mod contracts;
#[path = "../../bijux-cli/src/routing/inventory.rs"]
pub mod inventory;
#[path = "../../bijux-cli/src/routing/parser.rs"]
pub mod parser;
#[path = "../../bijux-cli/src/routing/query.rs"]
pub mod query;
#[path = "../../bijux-cli/src/routing/registry.rs"]
pub mod registry;
#[path = "../../bijux-cli/src/routing/schema.rs"]
pub mod schema;

#[cfg(test)]
use proptest as _;

#[cfg(test)]
use serde as _;
#[cfg(test)]
use serde_json as _;

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
