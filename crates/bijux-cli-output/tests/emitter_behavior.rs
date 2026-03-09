#![forbid(unsafe_code)]
//! Output emitter behavior tests.

use bijux_cli_contracts::{
    ColorMode, CommandPath, ErrorEnvelopeV1, ErrorPayloadV1, LogLevel, Namespace,
    OutputEnvelopeMetaV1, OutputEnvelopeV1, OutputFormat,
};
use bijux_cli_output::{
    emit_error, emit_success, format_debug_log, machine_safe_error_payload, EmitterConfig,
    OutputStream,
};
use serde_json::json;
use serde_yaml as _;
use thiserror as _;

fn sample_meta() -> OutputEnvelopeMetaV1 {
    OutputEnvelopeMetaV1 {
        version: "v1".to_string(),
        command: CommandPath {
            segments: vec![Namespace("cli".to_string()), Namespace("status".to_string())],
        },
        timestamp: "1970-01-01T00:00:00Z".to_string(),
    }
}

#[test]
fn emits_json_and_yaml_and_text_success_payloads() {
    let envelope = OutputEnvelopeV1 {
        status: "ok".to_string(),
        data: json!({"healthy": true}),
        meta: sample_meta(),
    };

    let json_out = emit_success(
        &envelope,
        EmitterConfig { format: OutputFormat::Json, pretty: false, ..EmitterConfig::default() },
    )
    .expect("json emit must succeed")
    .expect("json output must be present");
    assert_eq!(json_out.stream, OutputStream::Stdout);
    assert!(
        json_out.content.contains("\"status\":\"ok\"")
            || json_out.content.contains("\"status\": \"ok\"")
    );

    let yaml_out = emit_success(
        &envelope,
        EmitterConfig { format: OutputFormat::Yaml, pretty: true, ..EmitterConfig::default() },
    )
    .expect("yaml emit must succeed")
    .expect("yaml output must be present");
    assert_eq!(yaml_out.stream, OutputStream::Stdout);
    assert!(yaml_out.content.contains("status:"));

    let text_out = emit_success(
        &envelope,
        EmitterConfig { format: OutputFormat::Text, pretty: true, ..EmitterConfig::default() },
    )
    .expect("text emit must succeed")
    .expect("text output must be present");
    assert_eq!(text_out.stream, OutputStream::Stdout);
    assert!(text_out.content.contains("status"));
}

#[test]
fn quiet_suppresses_text_success_only() {
    let envelope = OutputEnvelopeV1 {
        status: "ok".to_string(),
        data: json!({"healthy": true}),
        meta: sample_meta(),
    };

    let text = emit_success(
        &envelope,
        EmitterConfig { format: OutputFormat::Text, quiet: true, ..EmitterConfig::default() },
    )
    .expect("text emit should not fail");
    assert!(text.is_none());

    let json_out = emit_success(
        &envelope,
        EmitterConfig { format: OutputFormat::Json, quiet: true, ..EmitterConfig::default() },
    )
    .expect("json emit should not fail");
    assert!(json_out.is_some());
}

#[test]
fn emits_error_on_stderr_with_color_policy_and_machine_payload() {
    let envelope = ErrorEnvelopeV1 {
        status: "error".to_string(),
        error: ErrorPayloadV1 {
            code: "invalid_format".to_string(),
            message: "Unsupported format".to_string(),
            category: "usage".to_string(),
            details: None,
        },
        meta: sample_meta(),
    };

    let text = emit_error(
        &envelope,
        EmitterConfig {
            format: OutputFormat::Text,
            color: ColorMode::Always,
            no_color: false,
            ..EmitterConfig::default()
        },
    )
    .expect("text error emit should succeed");
    assert_eq!(text.stream, OutputStream::Stderr);
    assert!(text.content.contains("\u{001b}[31m"));

    let plain = emit_error(
        &envelope,
        EmitterConfig {
            format: OutputFormat::Text,
            color: ColorMode::Always,
            no_color: true,
            ..EmitterConfig::default()
        },
    )
    .expect("plain error emit should succeed");
    assert!(!plain.content.contains("\u{001b}[31m"));

    let machine = machine_safe_error_payload(&envelope.error);
    assert_eq!(machine["code"], json!("invalid_format"));
    assert_eq!(machine["category"], json!("usage"));
}

#[test]
fn formats_debug_log_only_for_debug_levels() {
    let none = format_debug_log(
        "dispatch",
        EmitterConfig { log_level: LogLevel::Info, ..EmitterConfig::default() },
    );
    assert!(none.is_none());

    let yes = format_debug_log(
        "dispatch",
        EmitterConfig { log_level: LogLevel::Debug, ..EmitterConfig::default() },
    );
    assert_eq!(yes.as_deref(), Some("DEBUG dispatch"));
}
