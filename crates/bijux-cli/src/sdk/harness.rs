#![forbid(unsafe_code)]
//! Deterministic harness helpers for mounted Rust apps.

use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::contracts::{CommandPath, ExitCode, OutputFormat, PrettyMode};

use super::{
    BijuxApp, CommandContext, CommandEnvelope, CommandFailureBuilder, CommandResult,
    OutputEnvelopeHelper, ProductMount, SdkRenderConfig,
};

/// Result of running a mounted app through the SDK harness.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HarnessRun {
    pub query: String,
    pub matched_namespace: Option<String>,
    pub exit_code: ExitCode,
    pub stdout: String,
    pub stderr: String,
    pub envelope: CommandEnvelope,
}

/// Stable snapshot rendering helpers for harness output.
pub struct SnapshotHelper;

impl SnapshotHelper {
    #[must_use]
    pub fn normalize_text(value: &str) -> String {
        value.replace("\r\n", "\n")
    }

    #[must_use]
    pub fn render_run(run: &HarnessRun) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "query": run.query,
            "matched_namespace": run.matched_namespace,
            "exit_code": run.exit_code,
            "stdout": Self::normalize_text(&run.stdout),
            "stderr": Self::normalize_text(&run.stderr),
            "envelope": run.envelope,
        }))
        .expect("harness snapshot should serialize")
    }
}

/// In-process harness for mounted Rust apps.
pub struct BijuxCliHarness {
    apps: Vec<Box<dyn BijuxApp>>,
    render: SdkRenderConfig,
    cwd: PathBuf,
    project_root: Option<PathBuf>,
    config_dirs: Vec<PathBuf>,
    invocation_id: String,
    timestamp: String,
}

impl BijuxCliHarness {
    #[must_use]
    pub fn new() -> Self {
        Self {
            apps: Vec::new(),
            render: SdkRenderConfig::default(),
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            project_root: None,
            config_dirs: Vec::new(),
            invocation_id: "bijux-sdk-harness".to_string(),
            timestamp: "1970-01-01T00:00:00Z".to_string(),
        }
    }

    #[must_use]
    pub fn mount<T>(mut self, app: T) -> Self
    where
        T: BijuxApp + 'static,
    {
        self.apps.push(Box::new(app));
        self
    }

    #[must_use]
    pub fn with_output_format(mut self, format: OutputFormat) -> Self {
        self.render.format = format;
        self
    }

    #[must_use]
    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = cwd.into();
        self
    }

    #[must_use]
    pub fn with_pretty(mut self, pretty: bool) -> Self {
        self.render.pretty_mode = if pretty { PrettyMode::Pretty } else { PrettyMode::Compact };
        self
    }

    #[must_use]
    pub fn with_quiet(mut self, quiet: bool) -> Self {
        self.render.quiet = quiet;
        self
    }

    #[must_use]
    pub fn with_project_root(mut self, project_root: impl Into<PathBuf>) -> Self {
        self.project_root = Some(project_root.into());
        self
    }

    #[must_use]
    pub fn with_config_dir(mut self, config_dir: impl Into<PathBuf>) -> Self {
        self.config_dirs.push(config_dir.into());
        self
    }

    #[must_use]
    pub fn with_invocation_id(mut self, invocation_id: impl Into<String>) -> Self {
        self.invocation_id = invocation_id.into();
        self
    }

    #[must_use]
    pub fn with_timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = timestamp.into();
        self
    }

    pub fn run(&self, argv: &[&str]) -> Result<HarnessRun, String> {
        let Some(query) = argv.first() else {
            return self.render_harness_error(
                "",
                None,
                CommandFailureBuilder::new("sdk.usage.missing_namespace", "usage")
                    .message("mounted app harness requires a namespace query")
                    .build()?,
                ExitCode::Usage,
            );
        };

        let Some((app, mount)) = self.resolve_app(query) else {
            return self.render_harness_error(
                query,
                None,
                CommandFailureBuilder::new("sdk.route.unknown_namespace", "usage")
                    .message(format!("unknown mounted app namespace `{query}`"))
                    .context("query", Value::String((*query).to_string()))
                    .build()?,
                ExitCode::Usage,
            );
        };

        if let Some(report) = mount.compatibility_report()? {
            if !report.compatible {
                return self.render_harness_error(
                    query,
                    Some(mount.namespace().as_str().to_string()),
                    CommandFailureBuilder::new("sdk.compatibility.unsupported_host", "validation")
                        .message("mounted app is not compatible with this bijux host")
                        .context(
                            "compatibility",
                            serde_json::to_value(report)
                                .map_err(|error| format!("failed to serialize compatibility report: {error}"))?,
                        )
                        .build()?,
                    ExitCode::Usage,
                );
            }
        }

        let parent_command = CommandPath::new(&[mount.namespace().as_str()])?;
        let mut builder = CommandContext::builder(parent_command)
            .cwd(self.cwd.clone())
            .output_format(self.render.format)
            .pretty_mode(self.render.pretty_mode)
            .color_mode(self.render.color_mode)
            .verbosity(self.render.verbosity)
            .quiet(self.render.quiet)
            .invocation_id(self.invocation_id.clone());
        if let Some(project_root) = &self.project_root {
            builder = builder.project_root(project_root.clone());
        }
        for config_dir in &self.config_dirs {
            builder = builder.config_dir(config_dir.clone());
        }
        let ctx = builder.build();
        let route_args = argv.iter().skip(1).map(|value| (*value).to_string()).collect::<Vec<_>>();
        let command_result = app.route(&route_args, &ctx);
        let rendered = command_result.render(self.render)?;
        Ok(HarnessRun {
            query: (*query).to_string(),
            matched_namespace: Some(mount.namespace().as_str().to_string()),
            exit_code: rendered.exit_code,
            stdout: rendered.stdout,
            stderr: rendered.stderr,
            envelope: command_result.envelope,
        })
    }

    fn resolve_app(&self, query: &str) -> Option<(&dyn BijuxApp, ProductMount)> {
        self.apps.iter().find_map(|app| {
            let mount = app.mount();
            mount.matches_query(query).then_some((app.as_ref(), mount))
        })
    }

    fn render_harness_error(
        &self,
        query: &str,
        matched_namespace: Option<String>,
        error: crate::contracts::ErrorPayloadV1,
        exit_code: ExitCode,
    ) -> Result<HarnessRun, String> {
        let command = if let Some(namespace) = &matched_namespace {
            CommandPath::new(&[namespace.as_str()])?
        } else if query.is_empty() {
            CommandPath::new(&["apps"])?
        } else {
            CommandPath::new(&[query])?
        };
        let envelope = OutputEnvelopeHelper::failure(command, error, &self.timestamp)?;
        let result = CommandResult::failure(exit_code, envelope.clone());
        let rendered = result.render(self.render)?;
        Ok(HarnessRun {
            query: query.to_string(),
            matched_namespace,
            exit_code: rendered.exit_code,
            stdout: rendered.stdout,
            stderr: rendered.stderr,
            envelope: CommandEnvelope::Error(envelope),
        })
    }
}
