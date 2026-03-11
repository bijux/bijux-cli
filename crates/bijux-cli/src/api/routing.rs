#![forbid(unsafe_code)]
//! Route identity and registry facade.

/// Canonical route catalog helpers.
pub mod catalog {
    pub use crate::routing::catalog::{
        cli_config_subcommands, cli_plugins_subcommands, cli_root_aliases, dev_cli_subcommands,
        is_known_route, normalize_command_path, repl_reference_commands,
    };
}

/// Parser and normalization interfaces.
pub mod parser {
    pub use crate::routing::parser::{
        parse_intent, root_command, ParseError, ParsedGlobalFlags, ParsedIntent,
    };
}

/// Registry resolution interfaces.
pub mod registry {
    pub use crate::routing::registry::{RouteError, RouteRegistry, RouteTarget};
}

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
