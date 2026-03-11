#![forbid(unsafe_code)]

//! Ensures contract models serialize and deserialize without loss.

use std::collections::BTreeMap;

use bijux_cli::contracts::{
    AliasRewrite, ColorMode, CommandMetadata, CommandPath, CompatibilityRange, ConfigSource,
    DiagnosticRecord, ErrorDetailsV1, ErrorEnvelopeV1, ErrorPayloadV1, ExecutionPolicy, ExitCode,
    GlobalFlags, InspectReport, InvocationEvent, InvocationTrace, LogLevel, MemoryKeyList,
    MemorySummary, Namespace, NamespaceMetadata, OutputEnvelopeMetaV1, OutputEnvelopeV1,
    OutputFormat, PluginCapability, PluginKind, PluginLifecycleState, PluginManifestV1, PrettyMode,
    RouteSourceMetadata,
};
use clap as _;
use proptest as _;
use schemars as _;
use semver as _;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;
use thiserror as _;

fn roundtrip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let encoded = serde_json::to_vec(value).expect("serialize must succeed");
    let decoded: T = serde_json::from_slice(&encoded).expect("deserialize must succeed");
    assert_eq!(&decoded, value);
}

#[test]
fn roundtrip_for_all_contract_types() {
    let ns = Namespace("cli".to_string());
    let path = CommandPath {
        segments: vec![ns.clone(), Namespace("status".to_string())],
    };

    let flags = GlobalFlags {
        output_format: Some(OutputFormat::Json),
        pretty_mode: Some(PrettyMode::Compact),
        color_mode: Some(ColorMode::Never),
        log_level: Some(LogLevel::Debug),
        quiet: false,
        include_runtime: true,
    };

    let policy = ExecutionPolicy {
        output_format: OutputFormat::Json,
        pretty_mode: PrettyMode::Compact,
        color_mode: ColorMode::Never,
        log_level: LogLevel::Debug,
        quiet: false,
        include_runtime: true,
    };

    let meta = OutputEnvelopeMetaV1 {
        version: "v1".to_string(),
        command: path.clone(),
        timestamp: "2026-03-09T00:00:00Z".to_string(),
    };

    let output = OutputEnvelopeV1 {
        status: "ok".to_string(),
        data: json!({"healthy": true}),
        meta: meta.clone(),
    };

    let error = ErrorEnvelopeV1 {
        status: "error".to_string(),
        error: ErrorPayloadV1 {
            code: "invalid_format".to_string(),
            message: "Unsupported format".to_string(),
            category: "usage".to_string(),
            details: Some(ErrorDetailsV1 {
                failure: Some("invalid_format".to_string()),
                context: BTreeMap::from([("flag".to_string(), json!("--format"))]),
            }),
        },
        meta: meta.clone(),
    };

    let cmd_meta = CommandMetadata {
        path: path.clone(),
        summary: "Show status".to_string(),
        hidden: false,
        aliases: vec![CommandPath {
            segments: vec![Namespace("status".to_string())],
        }],
    };

    let ns_meta = NamespaceMetadata {
        name: ns.clone(),
        reserved: true,
        owner: "bijux-cli".to_string(),
    };

    let plugin_manifest = PluginManifestV1 {
        name: "sample".to_string(),
        version: "1.2.3".to_string(),
        schema_version: "1".to_string(),
        manifest_version: "1".to_string(),
        compatibility: CompatibilityRange {
            min_inclusive: "1.0.0".to_string(),
            max_exclusive: Some("2.0.0".to_string()),
        },
        namespace: Namespace("sample".to_string()),
        kind: PluginKind::Delegated,
        aliases: vec!["sample-status".to_string()],
        entrypoint: "sample_plugin:main".to_string(),
        capabilities: vec![PluginCapability {
            name: "inspect".to_string(),
            version: Some("1".to_string()),
        }],
    };

    let diagnostic = DiagnosticRecord {
        id: "diag-1".to_string(),
        severity: "warning".to_string(),
        message: "example".to_string(),
        fields: BTreeMap::from([("component".to_string(), json!("routing"))]),
    };

    let invocation = InvocationTrace {
        invocation_id: "inv-123".to_string(),
        command: path,
        policy,
        events: vec![InvocationEvent {
            timestamp: "2026-03-09T00:00:01Z".to_string(),
            name: "dispatch".to_string(),
            payload: BTreeMap::from([("route".to_string(), json!("cli status"))]),
        }],
    };

    let memory_summary = MemorySummary {
        status: "ok".to_string(),
        count: 2,
        message: "Memory command executed".to_string(),
    };
    let memory_list = MemoryKeyList {
        status: "ok".to_string(),
        keys: vec!["alpha".to_string(), "beta".to_string()],
        count: 2,
    };
    let inspect_report = InspectReport {
        status: "ok".to_string(),
        route_sources: vec![RouteSourceMetadata {
            segments: vec!["cli".to_string(), "status".to_string()],
            owner: "bijux-cli".to_string(),
            source: "built-in".to_string(),
        }],
        alias_rewrites: vec![AliasRewrite {
            alias: vec!["plugins".to_string(), "inspect".to_string()],
            canonical: vec![
                "cli".to_string(),
                "plugins".to_string(),
                "inspect".to_string(),
            ],
            source: "compatibility-alias".to_string(),
        }],
    };

    roundtrip(&ns);
    roundtrip(&flags);
    roundtrip(&ConfigSource::Flags);
    roundtrip(&ExitCode::Usage);
    roundtrip(&PluginLifecycleState::Enabled);
    roundtrip(&cmd_meta);
    roundtrip(&ns_meta);
    roundtrip(&plugin_manifest);
    roundtrip(&output);
    roundtrip(&error);
    roundtrip(&diagnostic);
    roundtrip(&invocation);
    roundtrip(&memory_summary);
    roundtrip(&memory_list);
    roundtrip(&inspect_report);
}

#[test]
fn contract_deserialization_rejects_invalid_payload_shapes() {
    let bad_output = json!({
        "status": "ok",
        "data": {"healthy": true}
    });
    let output_err =
        serde_json::from_value::<OutputEnvelopeV1>(bad_output).expect_err("missing meta must fail");
    assert!(output_err.to_string().contains("meta"));

    let bad_error = json!({
        "status": "error",
        "error": {
            "code": 17,
            "message": "bad",
            "category": "usage"
        },
        "meta": {
            "version": "v1",
            "command": {"segments": ["cli", "status"]},
            "timestamp": "2026-03-09T00:00:00Z"
        }
    });
    let err_envelope = serde_json::from_value::<ErrorEnvelopeV1>(bad_error)
        .expect_err("wrong code type must fail");
    let err_text = err_envelope.to_string();
    assert!(err_text.contains("invalid type") || err_text.contains("code"));

    let bad_manifest = json!({
        "name": "sample",
        "version": "1.2.3",
        "schema_version": "1",
        "manifest_version": "1",
        "compatibility": {
            "min_inclusive": "1.0.0",
            "max_exclusive": "2.0.0"
        },
        "namespace": "sample",
        "kind": "not-a-real-kind",
        "aliases": [],
        "entrypoint": "sample_plugin:main",
        "capabilities": []
    });
    let manifest_err = serde_json::from_value::<PluginManifestV1>(bad_manifest)
        .expect_err("unknown plugin kind must fail");
    assert!(manifest_err.to_string().contains("kind"));
}
