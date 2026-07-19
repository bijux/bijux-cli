#![forbid(unsafe_code)]
//! Python bridge conversion and exception mapping coverage for runtime contracts.

use bijux_cli as _;
use bijux_cli_python::{
    classify_failure, command_tree_introspection_api, execution_facade_api, execution_outcome_api,
    plugin_registry_inspection_api, python_exception_tag, BridgeErrorKind,
};
use serde_json::Value;
use std::path::PathBuf;

fn parse_json(text: &str) -> Value {
    serde_json::from_str(text).expect("valid json")
}

#[test]
fn python_exception_mapping_covers_usage_validation_plugin_and_internal_failures() {
    assert_eq!(python_exception_tag(classify_failure(2, "usage: bad args")), "UsageError");
    assert_eq!(python_exception_tag(classify_failure(1, "validation failed")), "ValidationError");
    assert_eq!(
        python_exception_tag(classify_failure(1, "plugin registry failed to load")),
        "InternalError"
    );
    assert_eq!(python_exception_tag(classify_failure(1, "runtime panic path")), "InternalError");
    assert_eq!(classify_failure(2, "usage"), BridgeErrorKind::Usage);
}

#[test]
fn error_and_success_envelope_fields_survive_python_conversion_intact() {
    let success = parse_json(
        &execution_outcome_api(&["bijux".to_string(), "status".to_string()])
            .expect("status outcome"),
    );
    for key in ["exit_code", "stdout", "stderr", "error_kind"] {
        assert!(success.get(key).is_some(), "missing success field: {key}");
    }
    assert_eq!(success["error_kind"], Value::Null);

    let usage_failure = parse_json(
        &execution_outcome_api(&["bijux".to_string(), "ghost".to_string(), "status".to_string()])
            .expect("usage outcome"),
    );
    for key in ["exit_code", "stdout", "stderr", "error_kind"] {
        assert!(usage_failure.get(key).is_some(), "missing error field: {key}");
    }
    assert_eq!(usage_failure["error_kind"], "UsageError");
}

#[test]
fn diagnostics_and_inspection_payloads_survive_conversion_with_stable_shape() {
    let plugin_diag = parse_json(
        &execution_facade_api(&[
            "bijux".to_string(),
            "cli".to_string(),
            "plugins".to_string(),
            "doctor".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "--no-pretty".to_string(),
        ])
        .expect("plugin diagnostics"),
    );
    assert!(plugin_diag.get("status").is_some());

    let config_diag = parse_json(
        &execution_facade_api(&[
            "bijux".to_string(),
            "cli".to_string(),
            "config".to_string(),
            "reload".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "--no-pretty".to_string(),
        ])
        .expect("config diagnostics"),
    );
    assert!(config_diag.get("status").is_some());

    let route_inspect = parse_json(&command_tree_introspection_api());
    assert_eq!(route_inspect["root"], "bijux");
    assert!(route_inspect["namespaces"].is_array());
    assert_eq!(route_inspect["source"], "runtime-inspect");
}

#[test]
fn bridge_conversions_preserve_field_names_optional_semantics_and_order_sensitive_lists() {
    let payload = parse_json(
        &execution_outcome_api(&["bijux".to_string(), "status".to_string()])
            .expect("status outcome"),
    );
    let keys = payload.as_object().expect("object").keys().map(|k| k.as_str()).collect::<Vec<_>>();
    assert_eq!(keys, vec!["error_kind", "exit_code", "stderr", "stdout"]);

    let tree = parse_json(&command_tree_introspection_api());
    let namespaces = tree["namespaces"].as_array().expect("namespaces array");
    let mut sorted =
        namespaces.iter().filter_map(Value::as_str).map(ToString::to_string).collect::<Vec<_>>();
    let original = sorted.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(original, sorted, "namespace ordering drifted");
}

#[test]
fn conversion_failures_and_unsupported_runtime_conditions_are_normalized_clearly() {
    let missing_registry = plugin_registry_inspection_api(&PathBuf::from(
        "/tmp/bijux-nonexistent-registry-does-not-exist.json",
    ))
    .expect("missing registry should be normalized");
    let parsed = parse_json(&missing_registry);
    assert_eq!(parsed["version"], "1");
    assert!(parsed["plugins"].is_object());

    #[cfg(not(feature = "python-extension"))]
    {
        // Without the extension feature, the Rust bridge still exposes stable APIs.
        let marker = parse_json(
            &execution_facade_api(&["bijux".to_string(), "version".to_string()]).expect("version"),
        );
        assert!(marker.get("version").is_some() || marker.get("status").is_some());
    }
}

#[test]
fn malformed_plugin_registry_is_rejected_at_bridge_boundary() {
    let root = std::env::temp_dir()
        .join(format!("bijux-bridge-malformed-registry-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");
    let registry = root.join("registry.json");
    std::fs::write(&registry, "{broken-json").expect("write malformed registry");

    let error = plugin_registry_inspection_api(&registry).expect_err("must reject malformed json");
    assert!(error.to_string().contains("invalid plugin registry json"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn bridge_import_failure_paths_are_distinct_from_command_failures() {
    let command_failure = parse_json(
        &execution_outcome_api(&["bijux".to_string(), "ghost".to_string()]).expect("usage failure"),
    );
    assert_eq!(command_failure["error_kind"], "UsageError");

    // Import/link errors should remain an outer runtime concern and must not be
    // encoded as command-level usage failures in bridge envelopes.
    let bridge_self_check = parse_json(
        &execution_outcome_api(&["bijux".to_string(), "version".to_string()]).expect("version"),
    );
    assert_ne!(bridge_self_check["stderr"], Value::String("ImportError".to_string()));
}
