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
/// Official product-mount reservation contracts.
pub mod product_mount;
/// Read-only schema inventory query interfaces.
pub mod query;
/// JSON Schema generation helpers.
pub mod schema;

pub use command::{CommandMetadata, CommandPath, Namespace, NamespaceMetadata};
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
pub use product_mount::{
    known_bijux_tool, KnownBijuxTool, ProductMountMetadata, KNOWN_BIJUX_TOOLS,
    KNOWN_BIJUX_TOOL_NAMESPACES, OFFICIAL_PRODUCT_NAMESPACES,
};
pub use query::{contracts_schema_query, ContractsSchemaQuery};
pub use schema::{error_envelope_v1_schema, output_envelope_v1_schema, plugin_manifest_v1_schema};
