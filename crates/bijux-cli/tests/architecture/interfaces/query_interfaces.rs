#![forbid(unsafe_code)]
//! Query interface shape and stability checks for core-owned dev bridge data.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli::features::diagnostics::{parity_status_query, state_diagnostics_query};

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "bijux-query-{label}-{}-{nanos}",
        std::process::id()
    ))
}

#[test]
fn state_diagnostics_query_shape_is_stable() {
    let root = unique_temp_dir("shape");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir");

    let config = root.join("config.env");
    let history = root.join("history.json");
    let plugins_registry = root.join("plugins-registry.json");
    let memory = root.join("memory.json");
    fs::write(&config, "A=1\n").expect("write config");

    let query = state_diagnostics_query(&config, &history, &plugins_registry, &memory);
    assert!(query.config.exists);
    assert!(query.config.is_file);
    assert!(!query.config.is_dir);
    assert!(!query.history.exists);
    assert!(!query.history.is_file);
    assert!(!query.history.is_dir);
    assert_eq!(query.memory.size_bytes, 0);
    assert!(!query.memory.exists);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn state_diagnostics_query_is_deterministic_for_same_inputs() {
    let root = unique_temp_dir("determinism");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir");

    let config = root.join("config.env");
    let history = root.join("history.json");
    let plugins_registry = root.join("plugins-registry.json");
    let memory = root.join("memory.json");
    fs::write(&config, "A=1\n").expect("write config");

    let first = state_diagnostics_query(&config, &history, &plugins_registry, &memory);
    let second = state_diagnostics_query(&config, &history, &plugins_registry, &memory);
    assert_eq!(first, second);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn state_diagnostics_query_is_read_only_for_missing_parent_directories() {
    let root = unique_temp_dir("readonly");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir");

    let config = root.join("config.env");
    fs::write(&config, "A=1\n").expect("write config");

    let missing_history_parent = root.join("nested/history");
    let missing_plugins_parent = root.join("nested/plugins");
    let missing_memory_parent = root.join("nested/memory");

    let history = missing_history_parent.join("history.json");
    let plugins_registry = missing_plugins_parent.join("plugins-registry.json");
    let memory = missing_memory_parent.join("memory.json");

    assert!(!missing_history_parent.exists());
    assert!(!missing_plugins_parent.exists());
    assert!(!missing_memory_parent.exists());

    let query = state_diagnostics_query(&config, &history, &plugins_registry, &memory);
    assert!(
        query.history.writable,
        "writable hint should remain true when nearest existing ancestor is writable"
    );
    assert!(
        query.plugins_registry.writable,
        "writable hint should remain true when nearest existing ancestor is writable"
    );
    assert!(
        query.memory.writable,
        "writable hint should remain true when nearest existing ancestor is writable"
    );

    assert!(
        !missing_history_parent.exists()
            && !missing_plugins_parent.exists()
            && !missing_memory_parent.exists(),
        "state diagnostics queries must not create directories as a side effect"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn parity_status_query_shape_is_stable() {
    let root = unique_temp_dir("parity-shape");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("artifacts/parity")).expect("mkdir parity");
    fs::create_dir_all(root.join("artifacts/status")).expect("mkdir status");
    fs::write(
        root.join("artifacts/parity/command_parity_matrix.json"),
        "{\"ok\":true}\n",
    )
    .expect("write parity");
    fs::write(root.join("artifacts/status/status.json"), "{\"ok\":true}\n").expect("write status");
    fs::write(
        root.join("artifacts/status/command_migration_matrix.json"),
        "{\"ok\":true}\n",
    )
    .expect("write migration");

    let query = parity_status_query(&root);
    assert!(query.command_parity_matrix_exists);
    assert!(query.status_report_exists);
    assert!(query.command_migration_matrix_exists);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn parity_status_query_is_read_only_for_missing_artifact_tree() {
    let root = unique_temp_dir("parity-readonly");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir root");
    let parity_dir = root.join("artifacts/parity");
    let status_dir = root.join("artifacts/status");
    assert!(!parity_dir.exists());
    assert!(!status_dir.exists());

    let query = parity_status_query(&root);
    assert!(!query.command_parity_matrix_exists);
    assert!(!query.status_report_exists);
    assert!(!query.command_migration_matrix_exists);
    assert!(
        !parity_dir.exists() && !status_dir.exists(),
        "parity query must not create artifacts"
    );

    let _ = fs::remove_dir_all(&root);
}
