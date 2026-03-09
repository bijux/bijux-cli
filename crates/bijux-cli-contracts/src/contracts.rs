//! Durable typed contracts for command routing, output, and plugin metadata.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Stable result marker used by integration boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ContractMarker {
    /// Contract namespace identifier.
    pub namespace: String,
}

/// Canonical command namespace segment.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
pub struct Namespace(pub String);

/// Canonical command path composed from namespace segments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CommandPath {
    /// Ordered namespace segments from root to leaf.
    pub segments: Vec<Namespace>,
}

/// Stable output format contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// JSON output.
    Json,
    /// YAML output.
    Yaml,
    /// Human-readable text output.
    Text,
}

/// Stable pretty-print policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum PrettyMode {
    /// Pretty output enabled.
    Pretty,
    /// Compact output enabled.
    Compact,
}

/// Stable color policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ColorMode {
    /// Use terminal auto detection.
    Auto,
    /// Always emit ANSI colors.
    Always,
    /// Never emit ANSI colors.
    Never,
}

/// Stable logging level contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Trace-level diagnostics.
    Trace,
    /// Debug diagnostics.
    Debug,
    /// Informational logs.
    Info,
    /// Warning logs.
    Warning,
    /// Error logs.
    Error,
    /// Critical logs.
    Critical,
}

/// Stable exit-code contract for automation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ExitCode {
    /// Successful execution.
    Success = 0,
    /// Internal failure.
    Error = 1,
    /// Usage or validation failure.
    Usage = 2,
    /// Encoding or serialization failure.
    Encoding = 3,
    /// User interrupt signal.
    Aborted = 130,
}

/// Stable config value source used for precedence diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ConfigSource {
    /// Value came from command-line flags.
    Flags,
    /// Value came from environment variables.
    Env,
    /// Value came from config file.
    Config,
    /// Value came from built-in defaults.
    Defaults,
}

/// Parsed global flags before precedence resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GlobalFlags {
    /// Optional output format override.
    pub output_format: Option<OutputFormat>,
    /// Optional pretty mode override.
    pub pretty_mode: Option<PrettyMode>,
    /// Optional color mode override.
    pub color_mode: Option<ColorMode>,
    /// Optional log-level override.
    pub log_level: Option<LogLevel>,
    /// Quiet mode.
    pub quiet: bool,
    /// Include runtime diagnostics.
    pub include_runtime: bool,
}

/// Effective execution policy after precedence resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionPolicy {
    /// Effective output format.
    pub output_format: OutputFormat,
    /// Effective pretty mode.
    pub pretty_mode: PrettyMode,
    /// Effective color mode.
    pub color_mode: ColorMode,
    /// Effective log level.
    pub log_level: LogLevel,
    /// Effective quiet mode.
    pub quiet: bool,
    /// Effective runtime metadata mode.
    pub include_runtime: bool,
}

/// Stable output envelope metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OutputEnvelopeMetaV1 {
    /// Envelope version identifier.
    pub version: String,
    /// Canonical command path.
    pub command: CommandPath,
    /// RFC3339 timestamp.
    pub timestamp: String,
}

/// Stable success payload envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OutputEnvelopeV1 {
    /// Fixed status marker.
    pub status: String,
    /// Command-specific payload.
    pub data: Value,
    /// Shared metadata.
    pub meta: OutputEnvelopeMetaV1,
}

/// Stable structured error details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ErrorDetailsV1 {
    /// Stable machine failure identifier.
    pub failure: Option<String>,
    /// Arbitrary additional context.
    pub context: BTreeMap<String, Value>,
}

/// Stable structured error payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ErrorPayloadV1 {
    /// Stable symbolic error code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Error category (`usage`, `validation`, `plugin`, `internal`).
    pub category: String,
    /// Structured optional details.
    pub details: Option<ErrorDetailsV1>,
}

/// Stable error envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ErrorEnvelopeV1 {
    /// Fixed status marker.
    pub status: String,
    /// Structured error payload.
    pub error: ErrorPayloadV1,
    /// Shared metadata.
    pub meta: OutputEnvelopeMetaV1,
}

/// Stable command metadata used by help and inspect APIs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CommandMetadata {
    /// Canonical command path.
    pub path: CommandPath,
    /// Human-readable summary.
    pub summary: String,
    /// Whether the command is hidden from help.
    pub hidden: bool,
    /// Stable aliases.
    pub aliases: Vec<CommandPath>,
}

/// Stable namespace metadata used by route-tree introspection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NamespaceMetadata {
    /// Namespace identifier.
    pub name: Namespace,
    /// Whether this namespace is reserved.
    pub reserved: bool,
    /// Owning product or component.
    pub owner: String,
}

/// Stable compatibility range contract for plugins and features.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompatibilityRange {
    /// Minimum supported version inclusive.
    pub min_inclusive: String,
    /// Optional maximum supported version exclusive.
    pub max_exclusive: Option<String>,
}

/// Stable plugin capability declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PluginCapability {
    /// Capability identifier.
    pub name: String,
    /// Optional capability version.
    pub version: Option<String>,
}

/// Stable plugin kind declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum PluginKind {
    /// Future in-process plugin ABI.
    Native,
    /// Delegated plugin loaded through host contract bridge.
    Delegated,
    /// Python delegated plugin runtime.
    Python,
    /// External executable plugin.
    ExternalExec,
}

impl Default for PluginKind {
    fn default() -> Self {
        Self::Delegated
    }
}

/// Stable plugin lifecycle state in registry and diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum PluginLifecycleState {
    /// Artifact located during discovery.
    Discovered,
    /// Manifest and contract validation passed.
    Validated,
    /// Plugin installed in registry.
    Installed,
    /// Plugin actively enabled for routing.
    Enabled,
    /// Plugin present but inactive.
    Disabled,
    /// Plugin failed validation or runtime loading.
    Broken,
    /// Plugin failed compatibility checks.
    Incompatible,
}

/// Stable plugin manifest contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PluginManifestV1 {
    /// Plugin name.
    pub name: String,
    /// Plugin version.
    pub version: String,
    /// Plugin schema version.
    pub schema_version: String,
    /// Manifest contract version.
    pub manifest_version: String,
    /// Compatibility range for host CLI.
    pub compatibility: CompatibilityRange,
    /// Declared top-level namespace.
    pub namespace: Namespace,
    /// Plugin execution kind.
    #[serde(default)]
    pub kind: PluginKind,
    /// Declared command aliases.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Plugin entrypoint (binary path or module symbol).
    pub entrypoint: String,
    /// Declared capabilities.
    pub capabilities: Vec<PluginCapability>,
}

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
    /// Event timestamp in RFC3339.
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
    /// Ordered events emitted during execution.
    pub events: Vec<InvocationEvent>,
}
