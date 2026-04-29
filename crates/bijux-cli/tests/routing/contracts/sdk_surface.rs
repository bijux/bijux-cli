#![forbid(unsafe_code)]
//! SDK surface contracts for mounted Rust apps.

use std::path::PathBuf;

use bijux_cli::contracts::{ColorMode, CommandPath, ExitCode, LogLevel, OutputFormat, PrettyMode};
use bijux_cli::sdk::{
    BijuxApp, BijuxCliHarness, CommandContext, CommandFailureBuilder, CommandResult,
    DiagnosticRecordBuilder, FeatureCapabilityDeclaration, OutputEnvelopeHelper, ProductMount,
    SdkCompatibilityWindow, SdkRenderConfig, SnapshotHelper, StreamPolicy,
};
use serde_json::json;

struct HelloApp;

impl BijuxApp for HelloApp {
    fn mount(&self) -> ProductMount {
        ProductMount::new("hello")
            .expect("namespace")
            .display_name("Hello")
            .alias("hola")
            .embedded_rust("hello::main")
            .control_embedded_rust("hello::control")
            .summary("Minimal hello app")
            .capability("json_output")
            .feature_capabilities(FeatureCapabilityDeclaration {
                uses_config: true,
                supports_completion: true,
                ..FeatureCapabilityDeclaration::default()
            })
            .compatibility(
                SdkCompatibilityWindow::new("0.3.0", Some("1.0.0".to_string()))
                    .expect("compatibility"),
            )
            .version("0.1.0")
    }

    fn route(&self, argv: &[String], ctx: &CommandContext) -> CommandResult {
        if argv.first().is_some_and(|value| value == "fail") {
            let command = ctx.command_path(&["fail"]).expect("command path");
            let error = CommandFailureBuilder::new("hello.invalid_request", "validation")
                .message("hello app rejected the request")
                .failure("bad_args")
                .context("argv", serde_json::to_value(argv).expect("argv"))
                .build()
                .expect("error payload");
            return CommandResult::failure(
                ExitCode::Usage,
                OutputEnvelopeHelper::failure(command, error, "1970-01-01T00:00:00Z")
                    .expect("error envelope"),
            );
        }

        let command = ctx.command_path(&["status"]).expect("command path");
        CommandResult::success(
            OutputEnvelopeHelper::success(
                command,
                json!({
                    "status": "ok",
                    "namespace": "hello",
                    "argv": argv,
                    "cwd": ctx.cwd.display().to_string(),
                }),
                "1970-01-01T00:00:00Z",
            )
            .expect("success envelope"),
        )
        .stdout_policy(StreamPolicy::Always)
    }
}

#[test]
fn product_mount_builder_materializes_valid_descriptor_and_manifest_json() {
    let mount = ProductMount::new("sample-app")
        .expect("namespace")
        .display_name("Sample App")
        .alias("sample")
        .binary("bijux-sample-app")
        .control_binary("bijux-sample-app")
        .summary("Sample mounted app")
        .capability("json_output")
        .capability("json_output")
        .feature_capabilities(FeatureCapabilityDeclaration {
            uses_config: true,
            supports_repl: true,
            ..FeatureCapabilityDeclaration::default()
        })
        .version("0.2.0");

    let descriptor = mount.build_descriptor().expect("descriptor");
    assert_eq!(descriptor.namespace.as_str(), "sample-app");
    assert_eq!(descriptor.display_name, "Sample App");
    assert_eq!(descriptor.aliases[0].as_str(), "sample");
    assert!(descriptor.capabilities.iter().any(|value| value == "uses_config"));
    assert!(descriptor.capabilities.iter().any(|value| value == "supports_repl"));

    let manifest = mount.manifest_json().expect("manifest json");
    let parsed: serde_json::Value = serde_json::from_str(&manifest).expect("manifest parse");
    assert_eq!(parsed["namespace"], "sample-app");
    assert_eq!(parsed["entrypoint"]["kind"], "binary");
}

#[test]
fn command_context_builder_tracks_policy_and_child_paths() {
    let context = CommandContext::builder(CommandPath::new(&["hello"]).expect("parent"))
        .cwd(PathBuf::from("/tmp/project"))
        .project_root(PathBuf::from("/tmp"))
        .config_dir(PathBuf::from("/tmp/.bijux"))
        .output_format(OutputFormat::Yaml)
        .pretty_mode(PrettyMode::Compact)
        .color_mode(ColorMode::Never)
        .verbosity(LogLevel::Debug)
        .quiet(true)
        .invocation_id("sdk-test")
        .build();

    let path = context.command_path(&["validate", "spec"]).expect("child path");
    assert_eq!(path.to_command_string(), "hello validate spec");
    assert_eq!(context.cwd, PathBuf::from("/tmp/project"));
    assert_eq!(context.project_root, Some(PathBuf::from("/tmp")));
    assert_eq!(context.config_dirs, vec![PathBuf::from("/tmp/.bijux")]);
    assert_eq!(context.output_format, OutputFormat::Yaml);
    assert_eq!(context.pretty_mode, PrettyMode::Compact);
    assert_eq!(context.color_mode, ColorMode::Never);
    assert_eq!(context.verbosity, LogLevel::Debug);
    assert!(context.quiet);
    assert_eq!(context.invocation_id, "sdk-test");
}

#[test]
fn diagnostics_builder_and_output_helpers_emit_root_compatible_shapes() {
    let diagnostic = DiagnosticRecordBuilder::new("hello.missing_input")
        .severity("error")
        .message("input file is required")
        .field("argument", json!("path"))
        .build()
        .expect("diagnostic");
    assert_eq!(diagnostic.id, "hello.missing_input");
    assert_eq!(diagnostic.severity, "error");
    assert_eq!(diagnostic.fields["argument"], "path");

    let table = OutputEnvelopeHelper::table(
        &["name", "status"],
        &[vec![json!("dag"), json!("ok")], vec![json!("canon"), json!("missing")]],
    )
    .expect("table");
    assert_eq!(table["kind"], "table");
    assert_eq!(table["columns"][0], "name");

    let error = CommandFailureBuilder::new("hello.validation", "validation")
        .message("invalid request")
        .failure("bad_args")
        .context("expected", json!(["status", "fail"]))
        .build()
        .expect("error payload");
    assert_eq!(error.code, "hello.validation");
    assert_eq!(
        error.details.as_ref().expect("details").context["expected"],
        json!(["status", "fail"])
    );
}

#[test]
fn command_result_rendering_honors_stream_policy_and_quiet_mode() {
    let command = CommandPath::new(&["hello", "status"]).expect("command");
    let success = CommandResult::success(
        OutputEnvelopeHelper::success(
            command,
            OutputEnvelopeHelper::text("ready"),
            "1970-01-01T00:00:00Z",
        )
        .expect("success envelope"),
    )
    .stdout_policy(StreamPolicy::Always);

    let rendered = success
        .render(SdkRenderConfig {
            format: OutputFormat::Text,
            pretty_mode: PrettyMode::Pretty,
            color_mode: ColorMode::Never,
            verbosity: LogLevel::Info,
            quiet: true,
            no_color: true,
        })
        .expect("rendered success");
    assert_eq!(rendered.exit_code, ExitCode::Success);
    assert!(rendered.stdout.contains("message: ready"));
    assert!(rendered.stderr.is_empty());

    let failure = CommandResult::failure(
        ExitCode::Usage,
        OutputEnvelopeHelper::failure(
            CommandPath::new(&["hello", "fail"]).expect("command"),
            CommandFailureBuilder::new("hello.validation", "validation")
                .message("bad request")
                .build()
                .expect("error"),
            "1970-01-01T00:00:00Z",
        )
        .expect("error envelope"),
    );
    let rendered = failure
        .render(SdkRenderConfig {
            format: OutputFormat::Json,
            pretty_mode: PrettyMode::Compact,
            color_mode: ColorMode::Never,
            verbosity: LogLevel::Info,
            quiet: false,
            no_color: true,
        })
        .expect("rendered failure");
    assert_eq!(rendered.exit_code, ExitCode::Usage);
    assert!(rendered.stdout.is_empty());
    assert!(rendered.stderr.contains("\"status\":\"error\""));
}

#[test]
fn sdk_harness_routes_aliases_and_emits_stable_snapshots() {
    let harness = BijuxCliHarness::new()
        .mount(HelloApp)
        .with_output_format(OutputFormat::Json)
        .with_pretty(true)
        .with_cwd("/sdk/workspace")
        .with_project_root("/workspace")
        .with_config_dir("/workspace/.bijux")
        .with_invocation_id("snapshot-invocation");

    let run = harness.run(&["hola", "status"]).expect("run");
    assert_eq!(run.matched_namespace.as_deref(), Some("hello"));
    assert_eq!(run.exit_code, ExitCode::Success);
    assert!(run.stdout.contains("\"namespace\": \"hello\""));
    assert!(run.stderr.is_empty());

    let snapshot = SnapshotHelper::render_run(&run);
    let expected = std::fs::read_to_string("tests/data/golden/sdk/hello_harness_snapshot.json")
        .expect("golden snapshot");
    assert_eq!(snapshot.trim_end(), expected.trim_end());
}

#[test]
fn sdk_harness_reports_compatibility_and_usage_failures_as_structured_errors() {
    struct FutureOnlyApp;

    impl BijuxApp for FutureOnlyApp {
        fn mount(&self) -> ProductMount {
            ProductMount::new("future")
                .expect("namespace")
                .binary("bijux-future")
                .summary("Future-only app")
                .compatibility(SdkCompatibilityWindow::new("9.9.9", None).expect("compatibility"))
        }

        fn route(&self, _argv: &[String], _ctx: &CommandContext) -> CommandResult {
            unreachable!("compatibility should reject before routing")
        }
    }

    let incompatible = BijuxCliHarness::new().mount(FutureOnlyApp).run(&["future"]).expect("run");
    assert_eq!(incompatible.exit_code, ExitCode::Usage);
    assert!(incompatible.stderr.contains("not compatible"));

    let unknown = BijuxCliHarness::new().mount(HelloApp).run(&["unknown"]).expect("run");
    assert_eq!(unknown.exit_code, ExitCode::Usage);
    assert!(unknown.stderr.contains("unknown mounted app namespace"));
}
