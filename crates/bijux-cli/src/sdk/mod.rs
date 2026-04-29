#![forbid(unsafe_code)]
//! Rust SDK surfaces for mounted Bijux apps.

mod harness;

use std::collections::BTreeMap;
use std::path::PathBuf;

use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::contracts::{
    ColorMode, CommandPath, ErrorDetailsV1, ErrorEnvelopeV1, ErrorPayloadV1, ExitCode, LogLevel,
    Namespace, OutputEnvelopeMetaV1, OutputEnvelopeV1, OutputFormat, PrettyMode,
    ProductEntrypoint, ProductEntrypointKind, ProductMountDescriptor,
};
use crate::contracts::diagnostics::DiagnosticRecord;
use crate::shared::output::{emit_error, emit_success, EmitterConfig};
use crate::shared::version::runtime_semver;

pub use harness::{BijuxCliHarness, HarnessRun, SnapshotHelper};

/// Standard feature-capability declarations for mounted apps.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FeatureCapabilityDeclaration {
    pub uses_config: bool,
    pub uses_history: bool,
    pub uses_memory: bool,
    pub uses_plugins: bool,
    pub supports_completion: bool,
    pub supports_repl: bool,
}

impl FeatureCapabilityDeclaration {
    /// Convert boolean declarations into stable capability strings.
    #[must_use]
    pub fn capability_labels(&self) -> Vec<String> {
        let mut labels = Vec::new();
        if self.uses_config {
            labels.push("uses_config".to_string());
        }
        if self.uses_history {
            labels.push("uses_history".to_string());
        }
        if self.uses_memory {
            labels.push("uses_memory".to_string());
        }
        if self.uses_plugins {
            labels.push("uses_plugins".to_string());
        }
        if self.supports_completion {
            labels.push("supports_completion".to_string());
        }
        if self.supports_repl {
            labels.push("supports_repl".to_string());
        }
        labels
    }
}

/// Runtime compatibility declaration for mounted apps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SdkCompatibilityWindow {
    pub min_cli_version: String,
    pub max_cli_version_exclusive: Option<String>,
}

impl SdkCompatibilityWindow {
    /// Build a compatibility window validated as semver.
    pub fn new(
        min_cli_version: impl Into<String>,
        max_cli_version_exclusive: Option<String>,
    ) -> Result<Self, String> {
        let min_cli_version = min_cli_version.into();
        Version::parse(&min_cli_version)
            .map_err(|error| format!("min_cli_version is not valid semver: {error}"))?;
        if let Some(max) = &max_cli_version_exclusive {
            let min = Version::parse(&min_cli_version).expect("validated min semver");
            let max_version = Version::parse(max)
                .map_err(|error| format!("max_cli_version_exclusive is not valid semver: {error}"))?;
            if max_version <= min {
                return Err(
                    "max_cli_version_exclusive must be greater than min_cli_version".to_string(),
                );
            }
        }
        Ok(Self { min_cli_version, max_cli_version_exclusive })
    }
}

/// Compatibility-check report for mounted apps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SdkCompatibilityReport {
    pub compatible: bool,
    pub host_cli_version: String,
    pub min_cli_version: String,
    pub max_cli_version_exclusive: Option<String>,
    pub reasons: Vec<String>,
}

/// Mounted-app metadata materialized from the high-level SDK builder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BijuxAppMetadata {
    pub namespace: String,
    pub display_name: String,
    pub aliases: Vec<String>,
    pub summary: String,
    pub version: Option<String>,
    pub entrypoint_kind: ProductEntrypointKind,
    pub entrypoint: String,
    pub control_entrypoint_kind: ProductEntrypointKind,
    pub control_entrypoint: String,
    pub capabilities: Vec<String>,
    pub feature_capabilities: FeatureCapabilityDeclaration,
    pub compatibility: Option<SdkCompatibilityWindow>,
}

/// High-level SDK builder for mounted apps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductMount {
    namespace: Namespace,
    display_name: Option<String>,
    aliases: Vec<String>,
    runtime_entrypoint: Option<ProductEntrypoint>,
    control_entrypoint: Option<ProductEntrypoint>,
    summary: Option<String>,
    version: Option<String>,
    capabilities: Vec<String>,
    feature_capabilities: FeatureCapabilityDeclaration,
    compatibility: Option<SdkCompatibilityWindow>,
}

impl ProductMount {
    /// Start a new mounted-app contract builder.
    pub fn new(raw_namespace: &str) -> Result<Self, String> {
        Ok(Self {
            namespace: Namespace::new(raw_namespace)?,
            display_name: None,
            aliases: Vec::new(),
            runtime_entrypoint: None,
            control_entrypoint: None,
            summary: None,
            version: None,
            capabilities: Vec::new(),
            feature_capabilities: FeatureCapabilityDeclaration::default(),
            compatibility: None,
        })
    }

    #[must_use]
    pub fn display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    #[must_use]
    pub fn alias(mut self, value: impl Into<String>) -> Self {
        self.aliases.push(value.into());
        self
    }

    #[must_use]
    pub fn summary(mut self, value: impl Into<String>) -> Self {
        self.summary = Some(value.into());
        self
    }

    #[must_use]
    pub fn version(mut self, value: impl Into<String>) -> Self {
        self.version = Some(value.into());
        self
    }

    #[must_use]
    pub fn capability(mut self, value: impl Into<String>) -> Self {
        self.capabilities.push(value.into());
        self
    }

    #[must_use]
    pub fn feature_capabilities(mut self, value: FeatureCapabilityDeclaration) -> Self {
        self.feature_capabilities = value;
        self
    }

    #[must_use]
    pub fn compatibility(mut self, value: SdkCompatibilityWindow) -> Self {
        self.compatibility = Some(value);
        self
    }

    #[must_use]
    pub fn binary(self, command: impl Into<String>) -> Self {
        self.runtime_entrypoint(ProductEntrypointKind::Binary, command)
    }

    #[must_use]
    pub fn python_module(self, module: impl Into<String>) -> Self {
        self.runtime_entrypoint(ProductEntrypointKind::PythonModule, module)
    }

    #[must_use]
    pub fn python_console_script(self, command: impl Into<String>) -> Self {
        self.runtime_entrypoint(ProductEntrypointKind::PythonConsoleScript, command)
    }

    #[must_use]
    pub fn plugin_process(self, command: impl Into<String>) -> Self {
        self.runtime_entrypoint(ProductEntrypointKind::PluginProcess, command)
    }

    #[must_use]
    pub fn embedded_rust(self, symbol: impl Into<String>) -> Self {
        self.runtime_entrypoint(ProductEntrypointKind::EmbeddedRust, symbol)
    }

    #[must_use]
    pub fn control_binary(self, command: impl Into<String>) -> Self {
        self.control_entrypoint(ProductEntrypointKind::Binary, command)
    }

    #[must_use]
    pub fn control_python_module(self, module: impl Into<String>) -> Self {
        self.control_entrypoint(ProductEntrypointKind::PythonModule, module)
    }

    #[must_use]
    pub fn control_python_console_script(self, command: impl Into<String>) -> Self {
        self.control_entrypoint(ProductEntrypointKind::PythonConsoleScript, command)
    }

    #[must_use]
    pub fn control_plugin_process(self, command: impl Into<String>) -> Self {
        self.control_entrypoint(ProductEntrypointKind::PluginProcess, command)
    }

    #[must_use]
    pub fn control_embedded_rust(self, symbol: impl Into<String>) -> Self {
        self.control_entrypoint(ProductEntrypointKind::EmbeddedRust, symbol)
    }

    #[must_use]
    fn runtime_entrypoint(mut self, kind: ProductEntrypointKind, command: impl Into<String>) -> Self {
        self.runtime_entrypoint = Some(ProductEntrypoint { kind, command: command.into() });
        self
    }

    #[must_use]
    fn control_entrypoint(
        mut self,
        kind: ProductEntrypointKind,
        command: impl Into<String>,
    ) -> Self {
        self.control_entrypoint = Some(ProductEntrypoint { kind, command: command.into() });
        self
    }

    #[must_use]
    pub fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    #[must_use]
    pub fn matches_query(&self, query: &str) -> bool {
        let normalized = Namespace::normalize(query);
        self.namespace.as_str() == normalized
            || self.aliases.iter().any(|alias| Namespace::normalize(alias) == normalized)
    }

    /// Build the validated product-mount descriptor consumed by the root runtime.
    pub fn build_descriptor(&self) -> Result<ProductMountDescriptor, String> {
        let runtime_entrypoint = self
            .runtime_entrypoint
            .clone()
            .ok_or_else(|| "product mount runtime entrypoint is required".to_string())?;
        let control_entrypoint =
            self.control_entrypoint.clone().unwrap_or_else(|| runtime_entrypoint.clone());
        let display_name = self
            .display_name
            .clone()
            .unwrap_or_else(|| default_display_name(self.namespace.as_str()));
        let summary = self
            .summary
            .clone()
            .ok_or_else(|| "product mount summary is required".to_string())?;
        let aliases = self
            .aliases
            .iter()
            .map(|alias| Namespace::new(alias))
            .collect::<Result<Vec<_>, _>>()?;

        let mut builder = ProductMountDescriptor::builder(self.namespace.clone())
            .display_name(display_name)
            .entrypoint(runtime_entrypoint.kind.clone(), runtime_entrypoint.command.clone())
            .control_entrypoint(
                control_entrypoint.kind.clone(),
                control_entrypoint.command.clone(),
            )
            .help_summary(summary);

        for alias in aliases {
            builder = builder.alias(alias);
        }
        for capability in merged_capabilities(&self.capabilities, &self.feature_capabilities) {
            builder = builder.capability(capability);
        }
        if let Some(version) = &self.version {
            builder = builder.version(version.clone());
        }
        builder.build()
    }

    /// Render the validated product-mount descriptor as canonical JSON.
    pub fn manifest_json(&self) -> Result<String, String> {
        let descriptor = self.build_descriptor()?;
        serde_json::to_string_pretty(&descriptor)
            .map_err(|error| format!("failed to render product mount manifest JSON: {error}"))
    }

    /// Materialize metadata suitable for app-author tooling and docs.
    pub fn metadata(&self) -> Result<BijuxAppMetadata, String> {
        let descriptor = self.build_descriptor()?;
        Ok(BijuxAppMetadata {
            namespace: descriptor.namespace.as_str().to_string(),
            display_name: descriptor.display_name,
            aliases: descriptor.aliases.iter().map(|alias| alias.as_str().to_string()).collect(),
            summary: descriptor.help.summary,
            version: descriptor.version,
            entrypoint_kind: descriptor.entrypoint.kind.clone(),
            entrypoint: descriptor.entrypoint.command.clone(),
            control_entrypoint_kind: descriptor.control_entrypoint.kind.clone(),
            control_entrypoint: descriptor.control_entrypoint.command.clone(),
            capabilities: descriptor.capabilities,
            feature_capabilities: self.feature_capabilities.clone(),
            compatibility: self.compatibility.clone(),
        })
    }

    /// Check whether this app is compatible with the current host runtime.
    pub fn compatibility_report(&self) -> Result<Option<SdkCompatibilityReport>, String> {
        let Some(window) = &self.compatibility else {
            return Ok(None);
        };

        let host_cli_version = runtime_semver().to_string();
        let host =
            Version::parse(&host_cli_version).map_err(|error| format!("host semver is invalid: {error}"))?;
        let min = Version::parse(&window.min_cli_version)
            .map_err(|error| format!("min_cli_version is invalid: {error}"))?;
        let max = window
            .max_cli_version_exclusive
            .as_ref()
            .map(|value| Version::parse(value))
            .transpose()
            .map_err(|error| format!("max_cli_version_exclusive is invalid: {error}"))?;

        let mut reasons = Vec::new();
        if host < min {
            reasons.push(format!(
                "host version `{host_cli_version}` is below required minimum `{}`",
                window.min_cli_version
            ));
        }
        if let Some(max_version) = &max {
            if host >= *max_version {
                reasons.push(format!(
                    "host version `{host_cli_version}` is not below exclusive maximum `{}`",
                    max_version
                ));
            }
        }

        Ok(Some(SdkCompatibilityReport {
            compatible: reasons.is_empty(),
            host_cli_version,
            min_cli_version: window.min_cli_version.clone(),
            max_cli_version_exclusive: window.max_cli_version_exclusive.clone(),
            reasons,
        }))
    }
}

/// Execution context passed to mounted app handlers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CommandContext {
    pub cwd: PathBuf,
    pub project_root: Option<PathBuf>,
    pub config_dirs: Vec<PathBuf>,
    pub output_format: OutputFormat,
    pub pretty_mode: PrettyMode,
    pub color_mode: ColorMode,
    pub verbosity: LogLevel,
    pub quiet: bool,
    pub invocation_id: String,
    pub parent_command: CommandPath,
}

impl CommandContext {
    /// Start a builder from the required parent command path.
    #[must_use]
    pub fn builder(parent_command: CommandPath) -> CommandContextBuilder {
        CommandContextBuilder::new(parent_command)
    }

    /// Build a child command path below the mounted app command root.
    pub fn command_path(&self, tail_segments: &[&str]) -> Result<CommandPath, String> {
        let mut segments = self
            .parent_command
            .segments
            .iter()
            .map(|segment| segment.as_str().to_string())
            .collect::<Vec<_>>();
        segments.extend(tail_segments.iter().map(|segment| segment.to_string()));
        let refs = segments.iter().map(String::as_str).collect::<Vec<_>>();
        CommandPath::new(&refs)
    }
}

/// Builder for mounted-app execution context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandContextBuilder {
    cwd: PathBuf,
    project_root: Option<PathBuf>,
    config_dirs: Vec<PathBuf>,
    output_format: OutputFormat,
    pretty_mode: PrettyMode,
    color_mode: ColorMode,
    verbosity: LogLevel,
    quiet: bool,
    invocation_id: String,
    parent_command: CommandPath,
}

impl CommandContextBuilder {
    fn new(parent_command: CommandPath) -> Self {
        Self {
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            project_root: None,
            config_dirs: Vec::new(),
            output_format: OutputFormat::Json,
            pretty_mode: PrettyMode::Pretty,
            color_mode: ColorMode::Auto,
            verbosity: LogLevel::Info,
            quiet: false,
            invocation_id: "bijux-sdk-invocation".to_string(),
            parent_command,
        }
    }

    #[must_use]
    pub fn cwd(mut self, value: impl Into<PathBuf>) -> Self {
        self.cwd = value.into();
        self
    }

    #[must_use]
    pub fn project_root(mut self, value: impl Into<PathBuf>) -> Self {
        self.project_root = Some(value.into());
        self
    }

    #[must_use]
    pub fn config_dir(mut self, value: impl Into<PathBuf>) -> Self {
        self.config_dirs.push(value.into());
        self
    }

    #[must_use]
    pub fn output_format(mut self, value: OutputFormat) -> Self {
        self.output_format = value;
        self
    }

    #[must_use]
    pub fn pretty_mode(mut self, value: PrettyMode) -> Self {
        self.pretty_mode = value;
        self
    }

    #[must_use]
    pub fn color_mode(mut self, value: ColorMode) -> Self {
        self.color_mode = value;
        self
    }

    #[must_use]
    pub fn verbosity(mut self, value: LogLevel) -> Self {
        self.verbosity = value;
        self
    }

    #[must_use]
    pub fn quiet(mut self, value: bool) -> Self {
        self.quiet = value;
        self
    }

    #[must_use]
    pub fn invocation_id(mut self, value: impl Into<String>) -> Self {
        self.invocation_id = value.into();
        self
    }

    #[must_use]
    pub fn build(self) -> CommandContext {
        CommandContext {
            cwd: self.cwd,
            project_root: self.project_root,
            config_dirs: self.config_dirs,
            output_format: self.output_format,
            pretty_mode: self.pretty_mode,
            color_mode: self.color_mode,
            verbosity: self.verbosity,
            quiet: self.quiet,
            invocation_id: self.invocation_id,
            parent_command: self.parent_command,
        }
    }
}

/// Public render configuration for mounted-app command results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SdkRenderConfig {
    pub format: OutputFormat,
    pub pretty_mode: PrettyMode,
    pub color_mode: ColorMode,
    pub verbosity: LogLevel,
    pub quiet: bool,
    pub no_color: bool,
}

impl Default for SdkRenderConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::Json,
            pretty_mode: PrettyMode::Pretty,
            color_mode: ColorMode::Never,
            verbosity: LogLevel::Info,
            quiet: false,
            no_color: true,
        }
    }
}

impl From<SdkRenderConfig> for EmitterConfig {
    fn from(value: SdkRenderConfig) -> Self {
        Self {
            format: value.format,
            pretty: matches!(value.pretty_mode, PrettyMode::Pretty),
            color: value.color_mode,
            log_level: value.verbosity,
            quiet: value.quiet,
            no_color: value.no_color,
        }
    }
}

/// Stream-routing policy for app-command results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StreamPolicy {
    Auto,
    Always,
    Never,
}

/// Mounted-app result envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "kind", content = "envelope")]
pub enum CommandEnvelope {
    Success(OutputEnvelopeV1),
    Error(ErrorEnvelopeV1),
}

/// Standard mounted-app command result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CommandResult {
    pub exit_code: ExitCode,
    pub envelope: CommandEnvelope,
    pub stdout_policy: StreamPolicy,
    pub stderr_policy: StreamPolicy,
}

impl CommandResult {
    /// Construct a successful command result.
    #[must_use]
    pub fn success(envelope: OutputEnvelopeV1) -> Self {
        Self {
            exit_code: ExitCode::Success,
            envelope: CommandEnvelope::Success(envelope),
            stdout_policy: StreamPolicy::Auto,
            stderr_policy: StreamPolicy::Never,
        }
    }

    /// Construct a failed command result.
    #[must_use]
    pub fn failure(exit_code: ExitCode, envelope: ErrorEnvelopeV1) -> Self {
        Self {
            exit_code,
            envelope: CommandEnvelope::Error(envelope),
            stdout_policy: StreamPolicy::Never,
            stderr_policy: StreamPolicy::Auto,
        }
    }

    #[must_use]
    pub fn stdout_policy(mut self, value: StreamPolicy) -> Self {
        self.stdout_policy = value;
        self
    }

    #[must_use]
    pub fn stderr_policy(mut self, value: StreamPolicy) -> Self {
        self.stderr_policy = value;
        self
    }

    /// Render the command result according to the requested output configuration.
    pub fn render(&self, cfg: SdkRenderConfig) -> Result<RenderedCommandResult, String> {
        let emitter = EmitterConfig::from(cfg);
        match &self.envelope {
            CommandEnvelope::Success(envelope) => {
                let mut stdout = String::new();
                if !matches!(self.stdout_policy, StreamPolicy::Never) {
                    let effective = if matches!(self.stdout_policy, StreamPolicy::Always) {
                        EmitterConfig { quiet: false, ..emitter }
                    } else {
                        emitter
                    };
                    if let Some(rendered) =
                        emit_success(envelope, effective).map_err(|error| error.to_string())?
                    {
                        stdout = rendered.content;
                    }
                }
                Ok(RenderedCommandResult {
                    exit_code: self.exit_code,
                    stdout,
                    stderr: String::new(),
                })
            }
            CommandEnvelope::Error(envelope) => {
                let stderr = if matches!(self.stderr_policy, StreamPolicy::Never) {
                    String::new()
                } else {
                    emit_error(envelope, emitter)
                        .map_err(|error| error.to_string())?
                        .content
                };
                Ok(RenderedCommandResult {
                    exit_code: self.exit_code,
                    stdout: String::new(),
                    stderr,
                })
            }
        }
    }
}

/// Rendered command result returned by the SDK harness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RenderedCommandResult {
    pub exit_code: ExitCode,
    pub stdout: String,
    pub stderr: String,
}

/// Builder for root-compatible diagnostic records.
#[derive(Debug, Clone, PartialEq)]
pub struct DiagnosticRecordBuilder {
    id: String,
    severity: String,
    message: Option<String>,
    fields: BTreeMap<String, Value>,
}

impl DiagnosticRecordBuilder {
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            severity: "error".to_string(),
            message: None,
            fields: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn severity(mut self, value: impl Into<String>) -> Self {
        self.severity = value.into();
        self
    }

    #[must_use]
    pub fn message(mut self, value: impl Into<String>) -> Self {
        self.message = Some(value.into());
        self
    }

    #[must_use]
    pub fn field(mut self, key: impl Into<String>, value: Value) -> Self {
        self.fields.insert(key.into(), value);
        self
    }

    pub fn build(self) -> Result<DiagnosticRecord, String> {
        if self.id.trim().is_empty() {
            return Err("diagnostic id cannot be empty".to_string());
        }
        if self.severity.trim().is_empty() {
            return Err("diagnostic severity cannot be empty".to_string());
        }
        let message = self.message.ok_or_else(|| "diagnostic message is required".to_string())?;
        if message.trim().is_empty() {
            return Err("diagnostic message cannot be empty".to_string());
        }
        Ok(DiagnosticRecord { id: self.id, severity: self.severity, message, fields: self.fields })
    }
}

/// Builder for root-compatible structured failures.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandFailureBuilder {
    code: String,
    category: String,
    message: Option<String>,
    failure: Option<String>,
    context: BTreeMap<String, Value>,
}

impl CommandFailureBuilder {
    #[must_use]
    pub fn new(code: impl Into<String>, category: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            category: category.into(),
            message: None,
            failure: None,
            context: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn message(mut self, value: impl Into<String>) -> Self {
        self.message = Some(value.into());
        self
    }

    #[must_use]
    pub fn failure(mut self, value: impl Into<String>) -> Self {
        self.failure = Some(value.into());
        self
    }

    #[must_use]
    pub fn context(mut self, key: impl Into<String>, value: Value) -> Self {
        self.context.insert(key.into(), value);
        self
    }

    pub fn build(self) -> Result<ErrorPayloadV1, String> {
        let message = self.message.ok_or_else(|| "error message is required".to_string())?;
        let mut payload = ErrorPayloadV1::new(&self.code, &message, &self.category)?;
        if self.failure.is_some() || !self.context.is_empty() {
            payload.details = Some(ErrorDetailsV1 { failure: self.failure, context: self.context });
        }
        Ok(payload)
    }
}

/// Stable helpers for mounted-app envelopes and payload shapes.
pub struct OutputEnvelopeHelper;

impl OutputEnvelopeHelper {
    pub fn success(
        command: CommandPath,
        data: Value,
        timestamp: &str,
    ) -> Result<OutputEnvelopeV1, String> {
        let meta = OutputEnvelopeMetaV1::new("v1", command, timestamp)?;
        Ok(OutputEnvelopeV1::success(data, meta))
    }

    pub fn failure(
        command: CommandPath,
        error: ErrorPayloadV1,
        timestamp: &str,
    ) -> Result<ErrorEnvelopeV1, String> {
        let meta = OutputEnvelopeMetaV1::new("v1", command, timestamp)?;
        Ok(ErrorEnvelopeV1::failure(error, meta))
    }

    #[must_use]
    pub fn json(value: Value) -> Value {
        value
    }

    #[must_use]
    pub fn text(message: impl Into<String>) -> Value {
        json!({ "message": message.into() })
    }

    pub fn table(columns: &[&str], rows: &[Vec<Value>]) -> Result<Value, String> {
        if columns.is_empty() {
            return Err("table columns cannot be empty".to_string());
        }
        for row in rows {
            if row.len() != columns.len() {
                return Err("table rows must match the column count".to_string());
            }
        }
        Ok(json!({
            "kind": "table",
            "columns": columns,
            "rows": rows,
        }))
    }

    #[must_use]
    pub fn quiet() -> Value {
        json!({})
    }
}

/// Trait implemented by mounted Rust apps.
pub trait BijuxApp {
    fn mount(&self) -> ProductMount;
    fn route(&self, argv: &[String], ctx: &CommandContext) -> CommandResult;

    fn namespace(&self) -> String {
        self.mount().namespace().as_str().to_string()
    }

    fn metadata(&self) -> Result<BijuxAppMetadata, String> {
        self.mount().metadata()
    }

    fn manifest_descriptor(&self) -> Result<ProductMountDescriptor, String> {
        self.mount().build_descriptor()
    }
}

fn default_display_name(namespace: &str) -> String {
    namespace
        .split('-')
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn merged_capabilities(
    capabilities: &[String],
    feature_capabilities: &FeatureCapabilityDeclaration,
) -> Vec<String> {
    let mut merged = capabilities.to_vec();
    merged.extend(feature_capabilities.capability_labels());
    merged.sort();
    merged.dedup();
    merged
}
