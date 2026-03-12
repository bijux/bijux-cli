#![forbid(unsafe_code)]

//! Config domain contract invariants.

use bijux_cli::contracts::config::{
    ConfigClearResult, ConfigCommandResult, ConfigEntry, ConfigErrorKind, ConfigExportFormat,
    ConfigKey, ConfigLoadResult, ConfigMutation, ConfigPathSet, ConfigReloadResult, ConfigSnapshot,
    ConfigSource as ConfigReadSource, ConfigValidationError, ConfigValue, ConfigWriteResult,
    ResolvedConfigValue,
};
use proptest as _;
use serde as _;
use serde_json as _;

use clap as _;
use schemars as _;
use semver as _;
use thiserror as _;

#[test]
fn config_key_normalization_and_validation() {
    let key = ConfigKey::new(" BIJUXCLI_SAMPLE_KEY ").expect("valid key");
    assert_eq!(key.as_str(), "sample_key");

    assert!(ConfigKey::new(" ").is_err());
    assert!(ConfigKey::new("sample.key").is_err());
    assert!(ConfigKey::new("invalid-key").is_err());
    assert!(ConfigKey::new("ümlaut").is_err());
}

#[test]
fn config_value_validation_rejects_control_chars() {
    assert!(ConfigValue::new("plain_value").is_ok());
    assert!(ConfigValue::new("line\nbreak").is_err());
    assert!(ConfigValue::new("tab\tvalue").is_err());
}

#[test]
fn config_domain_types_serialize_roundtrip() {
    let key = ConfigKey::new("alpha").expect("key");
    let value = ConfigValue::new("1").expect("value");
    let entry = ConfigEntry {
        key: key.clone(),
        value: value.clone(),
    };
    let snapshot = ConfigSnapshot {
        entries: std::collections::BTreeMap::from([(key.clone(), value.clone())]),
    };
    let paths = ConfigPathSet {
        config_file: "/tmp/.bijux/.env".to_string(),
        history_file: "/tmp/.bijux/.history".to_string(),
        plugins_dir: "/tmp/.bijux/.plugins".to_string(),
    };
    let load = ConfigLoadResult {
        snapshot: snapshot.clone(),
        paths: paths.clone(),
    };
    let write = ConfigWriteResult {
        updated: true,
        entry_count: 1,
        target_path: paths.config_file.clone(),
    };
    let resolved = ResolvedConfigValue {
        key: key.clone(),
        value: value.clone(),
        source: ConfigReadSource::File,
        source_path: Some(paths.config_file.clone()),
    };
    let clear = ConfigClearResult {
        status: "ok".to_string(),
        removed_keys: 1,
    };
    let reload = ConfigReloadResult {
        status: "ok".to_string(),
        reloaded_path: paths.config_file.clone(),
    };
    let command = ConfigCommandResult {
        status: "ok".to_string(),
        command: "config get".to_string(),
    };

    for encoded in [
        serde_json::to_string(&entry).expect("serialize"),
        serde_json::to_string(&snapshot).expect("serialize"),
        serde_json::to_string(&paths).expect("serialize"),
        serde_json::to_string(&load).expect("serialize"),
        serde_json::to_string(&write).expect("serialize"),
        serde_json::to_string(&resolved).expect("serialize"),
        serde_json::to_string(&clear).expect("serialize"),
        serde_json::to_string(&reload).expect("serialize"),
        serde_json::to_string(&command).expect("serialize"),
    ] {
        assert!(!encoded.is_empty());
    }

    let set = ConfigMutation::Set {
        key: key.clone(),
        value: value.clone(),
    };
    let unset = ConfigMutation::Unset { key };
    assert!(matches!(set, ConfigMutation::Set { .. }));
    assert!(matches!(unset, ConfigMutation::Unset { .. }));

    let _ = ConfigExportFormat::Json;
    let _ = ConfigErrorKind::Validation;

    let validation = ConfigValidationError {
        key: None,
        message: "invalid key".to_string(),
    };
    assert_eq!(validation.message, "invalid key");
}
