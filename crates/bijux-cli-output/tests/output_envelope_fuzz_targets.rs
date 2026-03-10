#![forbid(unsafe_code)]
//! Output and envelope fuzz targets for serializer/rendering hardening.
//! test_type: output-envelope-fuzz

use std::collections::BTreeMap;

use bijux_cli_contracts::{
    CommandPath, ErrorDetailsV1, ErrorEnvelopeV1, ErrorPayloadV1, Namespace,
    OutputEnvelopeMetaV1, OutputEnvelopeV1, OutputFormat,
};
use bijux_cli_core as _;
use bijux_cli_output::{emit_error, emit_success, render_value, EmitterConfig, OutputStream};
use serde as _;
use serde_json::{json, Value};
use serde_yaml as _;
use thiserror as _;

fn meta() -> OutputEnvelopeMetaV1 {
    OutputEnvelopeMetaV1 {
        version: "v1".to_string(),
        command: CommandPath {
            segments: vec![Namespace("cli".to_string()), Namespace("status".to_string())],
        },
        timestamp: "1970-01-01T00:00:00Z".to_string(),
    }
}

fn payload(seed: u64, size: usize) -> Value {
    let mut state = seed;
    let rows: Vec<Value> = (0..size)
        .map(|i| {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            json!({
                "id": i,
                "n": state % 1_000_000,
                "label": format!("row-{i}-{}", state % 10_000),
            })
        })
        .collect();
    json!({"rows": rows, "summary": {"count": size}})
}

#[test]
fn fuzz_success_envelope_serialization_is_stable() {
    let envelope = OutputEnvelopeV1::success(payload(17, 32), meta());
    let a = serde_json::to_string(&envelope).expect("serialize success envelope");
    let b = serde_json::to_string(&envelope).expect("serialize success envelope");
    assert_eq!(a, b);
}

#[test]
fn fuzz_error_envelope_serialization_is_stable() {
    let envelope = ErrorEnvelopeV1::failure(
        ErrorPayloadV1 {
            code: "validation_failed".to_string(),
            message: "invalid key".to_string(),
            category: "validation".to_string(),
            details: Some(ErrorDetailsV1 {
                failure: Some("bad_input".to_string()),
                context: BTreeMap::from([
                    ("field".to_string(), json!("config.alpha")),
                    ("nested".to_string(), json!({"depth":[1,2,3]})),
                ]),
            }),
        },
        meta(),
    );
    let a = serde_json::to_string(&envelope).expect("serialize error envelope");
    let b = serde_json::to_string(&envelope).expect("serialize error envelope");
    assert_eq!(a, b);
}

#[test]
fn fuzz_json_yaml_text_emitters_render_without_corruption() {
    let value = payload(99, 40);

    let json_out = render_value(
        &value,
        EmitterConfig { format: OutputFormat::Json, pretty: false, ..EmitterConfig::default() },
    )
    .expect("json render");
    assert!(json_out.starts_with('{'));

    let yaml_out = render_value(
        &value,
        EmitterConfig { format: OutputFormat::Yaml, pretty: true, ..EmitterConfig::default() },
    )
    .expect("yaml render");
    assert!(yaml_out.contains("rows:"));

    let text_out = render_value(
        &value,
        EmitterConfig { format: OutputFormat::Text, pretty: false, ..EmitterConfig::default() },
    )
    .expect("text render");
    assert!(text_out.starts_with('{'));
}

#[test]
fn fuzz_nested_diagnostics_multiline_unicode_empty_and_large_payload_rendering() {
    let err = ErrorEnvelopeV1::failure(
        ErrorPayloadV1 {
            code: "internal_error".to_string(),
            message: "line1\nline2 – unicode ✓".to_string(),
            category: "internal".to_string(),
            details: Some(ErrorDetailsV1 {
                failure: Some("panic-normalized".to_string()),
                context: BTreeMap::from([
                    ("trace".to_string(), json!("αβγ")),
                    ("payload".to_string(), payload(7, 5)),
                ]),
            }),
        },
        meta(),
    );

    let json_err = emit_error(
        &err,
        EmitterConfig { format: OutputFormat::Json, pretty: false, ..EmitterConfig::default() },
    )
    .expect("emit json error");
    assert_eq!(json_err.stream, OutputStream::Stderr);
    assert!(json_err.content.contains("line1\\nline2"));

    let empty = json!({});
    let empty_text = render_value(
        &empty,
        EmitterConfig { format: OutputFormat::Text, pretty: false, ..EmitterConfig::default() },
    )
    .expect("empty text render");
    assert_eq!(empty_text, "{}");

    let large = payload(1234, 800);
    let large_json = render_value(
        &large,
        EmitterConfig { format: OutputFormat::Json, pretty: false, ..EmitterConfig::default() },
    )
    .expect("large json render");
    assert!(large_json.len() > 1000);
}

#[test]
fn fuzz_malformed_envelope_deserialization_is_rejected() {
    let malformed = [
        "{",
        "[]",
        "{\"status\":\"ok\"}",
        "{\"status\":\"error\",\"meta\":{}}",
    ];
    for sample in malformed {
        let parsed = serde_json::from_str::<OutputEnvelopeV1>(sample);
        assert!(parsed.is_err());
    }
}

#[test]
fn fuzz_route_inspection_json_rendering_is_deterministic() {
    let route_payload = json!({
        "routes": [
            {"segments":["cli","status"],"owner":"bijux-cli","source":"built-in"},
            {"segments":["dev","cli","routes"],"owner":"bijux-cli","source":"built-in"}
        ],
        "aliases": [
            {"alias":["status"],"canonical":["cli","status"],"source":"compatibility-alias"}
        ]
    });

    let a = render_value(
        &route_payload,
        EmitterConfig { format: OutputFormat::Json, pretty: false, ..EmitterConfig::default() },
    )
    .expect("route render a");
    let b = render_value(
        &route_payload,
        EmitterConfig { format: OutputFormat::Json, pretty: false, ..EmitterConfig::default() },
    )
    .expect("route render b");
    assert_eq!(a, b);
}

#[test]
fn fuzz_output_field_order_invariant_for_machine_rendering() {
    let envelope = OutputEnvelopeV1::success(json!({"z": 1, "a": 2, "m": 3}), meta());
    let out = emit_success(
        &envelope,
        EmitterConfig { format: OutputFormat::Json, pretty: false, ..EmitterConfig::default() },
    )
    .expect("emit")
    .expect("present");
    let out2 = emit_success(
        &envelope,
        EmitterConfig { format: OutputFormat::Json, pretty: false, ..EmitterConfig::default() },
    )
    .expect("emit2")
    .expect("present2");
    assert_eq!(out.content, out2.content);
}
