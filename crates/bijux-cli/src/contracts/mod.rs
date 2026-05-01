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
/// Root operator flow contracts for backlog iteration 01.
pub mod operator_iteration01;
/// Official app, plugin, and SDK contracts for backlog iteration 02.
pub mod operator_iteration02;
/// Plugin manifest and compatibility contracts.
pub mod plugin;
/// Official product-mount reservation contracts.
pub mod product_mount;
/// Read-only schema inventory query interfaces.
pub mod query;
/// JSON Schema generation helpers.
pub mod schema;

pub use command::{CommandMetadata, CommandPath, Namespace, NamespaceMetadata};
pub use config::{
    ConfigDeprecationStatusV1, ConfigSchemaFieldV1, ConfigSchemaRegistryV1, ConfigSchemaScopeV1,
    ConfigSchemaSourceV1, ConfigSchemaValueKindV1,
};
pub use envelope::{
    CommandEnvelopeV1, CommandFailureClassV1, CommandFailureV1, CommandWarningV1, ErrorDetailsV1,
    ErrorEnvelopeV1, ErrorPayloadV1, OutputEnvelopeMetaV1, OutputEnvelopeV1,
};
pub use execution::{
    ColorMode, ConfigSource, ExecutionPolicy, ExitCode, GlobalFlags, LogLevel, OutputFormat,
    PrettyMode,
};
pub use marker::ContractMarker;
pub use operator_iteration01::{
    build_actionable_error_envelope, build_command_explain_record,
    build_compact_operator_help_entrypoint, build_completion_snapshot_from_registry,
    build_install_diagnosis_bundle, build_official_app_discovery_report,
    build_python_bridge_command_parity_report, build_script_stable_command_envelope,
    classify_command_side_effect, evaluate_output_mode_parity, ActionableErrorEnvelopeV1,
    ActionableFailureClassV1, CommandExplainV1, CommandSideEffectClassV1,
    CommandSideEffectPreviewV1, CompactHelpEntryPointV1, CompletionRouteEntryV1,
    CompletionSnapshotV1, InstallDiagnosticComponentV1, InstallDiagnosisBundleV1,
    OfficialAppDiscoveryReportV1, OfficialAppRouteDescriptorV1, OutputModeParityEntryV1,
    OutputModeParityReportV1, PythonBridgeParityEntryV1, PythonBridgeParityReportV1,
    ScriptStableCommandEnvelopeV1,
};
pub use plugin::{
    CompatibilityRange, PluginCapability, PluginKind, PluginLifecycleState, PluginManifestV2,
    PluginTrustClass,
};
pub use product_mount::{
    canonical_bijux_tool_namespace, known_bijux_tool, known_bijux_tool_by_query,
    known_bijux_tool_namespaces, known_bijux_tools, official_product_namespaces,
    official_status_allows_runtime_dispatch, validate_product_mount_descriptor, KnownBijuxTool,
    ProductCompatibilityWindow, ProductEntrypoint, ProductEntrypointKind, ProductHelpMetadata,
    ProductMountDescriptor, ProductMountDescriptorBuilder, ProductMountMetadata,
    ProductRegistryDocument, ProductRegistryEntry,
};
pub use query::{
    contracts_schema_query, version_compatibility_lanes_query, ContractsSchemaQuery,
    VersionCompatibilityLaneQuery, VersionCompatibilitySurface,
};
pub use schema::{
    command_envelope_v1_schema, config_schema_registry_v1_schema, error_envelope_v1_schema,
    official_product_registry_schema, output_envelope_v1_schema, plugin_manifest_v2_schema,
    product_mount_descriptor_schema,
};
