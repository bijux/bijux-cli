#![forbid(unsafe_code)]
//! Integration tests for compatibility configuration and filesystem behavior.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli_contracts as _;
use bijux_cli_python::{
    acquire_state_lock, default_compatibility_paths, discover_compatibility_paths,
    ensure_history_file, ensure_plugins_dir, load_compatibility_config, parse_compatibility_config,
    write_compatibility_config, CompatibilityConfig, CompatibilityError, PathOverrides,
    ENV_CONFIG_PATH, ENV_HISTORY_PATH,
};
use thiserror as _;
use serde_json as _;

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("bijux-cli-{label}-{nanos}"));
    fs::create_dir_all(&dir).expect("temp directory should be created");
    dir
}

#[test]
fn missing_config_file_returns_default_config() {
    let root = temp_dir("missing-config");
    let config_path = root.join("missing.env");

    let loaded = load_compatibility_config(&config_path).expect("loading should succeed");
    assert_eq!(loaded, CompatibilityConfig::default());
}

#[test]
fn malformed_config_file_returns_parse_error() {
    let malformed = parse_compatibility_config("BIJUXCLI_CONFIG\n").expect_err("must fail");
    assert!(matches!(malformed, CompatibilityError::MalformedConfigLine { .. }));
}

#[test]
fn env_values_override_config_values() {
    let home = temp_dir("env-overrides");
    let config = CompatibilityConfig {
        config_file: Some(PathBuf::from("from-config.env")),
        history_file: Some(PathBuf::from("history-from-config.log")),
        plugins_dir: None,
    };

    let mut env_map = HashMap::new();
    env_map.insert(ENV_HISTORY_PATH.to_string(), "from-env.log".to_string());

    let resolved = discover_compatibility_paths(Some(&home), &PathOverrides::default(), &env_map, &config)
        .expect("resolution should succeed");

    assert_eq!(resolved.config_file, home.join("from-config.env"));
    assert_eq!(resolved.history_file, home.join("from-env.log"));
}

#[test]
fn cli_values_override_env_values() {
    let home = temp_dir("cli-overrides");
    let mut env_map = HashMap::new();
    env_map.insert(ENV_CONFIG_PATH.to_string(), "from-env.env".to_string());

    let overrides = PathOverrides {
        config_file: Some(PathBuf::from("/explicit/config.env")),
        history_file: None,
        plugins_dir: None,
    };

    let resolved = discover_compatibility_paths(
        Some(&home),
        &overrides,
        &env_map,
        &CompatibilityConfig::default(),
    )
    .expect("resolution should succeed");

    assert_eq!(resolved.config_file, PathBuf::from("/explicit/config.env"));
}

#[test]
fn safe_write_and_lock_and_state_paths_are_supported() {
    let home = temp_dir("state");
    let defaults = default_compatibility_paths(&home);

    let config = CompatibilityConfig {
        config_file: Some(defaults.config_file.clone()),
        history_file: Some(defaults.history_file.clone()),
        plugins_dir: Some(defaults.plugins_dir.clone()),
    };

    write_compatibility_config(&defaults.config_file, &config).expect("write should succeed");
    ensure_history_file(&defaults.history_file).expect("history should be created");
    ensure_plugins_dir(&defaults.plugins_dir).expect("plugins dir should be created");

    let lock_path = home.join(".bijux/state.lock");
    let _guard = acquire_state_lock(&lock_path).expect("first lock should succeed");
    let second = acquire_state_lock(&lock_path).expect_err("second lock should fail");
    assert!(matches!(second, CompatibilityError::LockHeld(_)));
}
