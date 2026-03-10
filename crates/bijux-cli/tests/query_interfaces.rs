#![forbid(unsafe_code)]
//! Query interface shape and stability checks for core-owned dev bridge data.

use std::fs;

use bijux_cli::query::{parity_status_query, state_diagnostics_query};

#[test]
fn state_diagnostics_query_shape_is_stable() {
    let root = std::env::temp_dir().join(format!("bijux-query-shape-{}", std::process::id()));
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
    assert!(!query.history.exists);
    assert_eq!(query.memory.size_bytes, 0);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn state_diagnostics_query_is_deterministic_for_same_inputs() {
    let root = std::env::temp_dir().join(format!("bijux-query-determinism-{}", std::process::id()));
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
fn parity_status_query_shape_is_stable() {
    let root = std::env::temp_dir().join(format!("bijux-parity-status-{}", std::process::id()));
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
