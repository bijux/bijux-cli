#![forbid(unsafe_code)]
//! Registry and command internals coverage for plugin lifecycle operations.
//! test_type: plugin-failure-path

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli_plugin::{
    compatibility_check, disable_plugin, enable_plugin, install_plugin, load_registry,
    plugin_doctor, registry_path_from_plugins_dir, uninstall_plugin, InstallPluginRequest,
    PluginTrustLevel,
};
use bijux_cli_routing as _;
use semver as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use thiserror as _;

fn temp_plugins_dir(label: &str) -> PathBuf {
    let ts =
        SystemTime::now().duration_since(UNIX_EPOCH).expect("clock should be monotonic").as_nanos();
    let dir = std::env::temp_dir().join(format!("bijux-plugin-{label}-{ts}"));
    fs::create_dir_all(&dir).expect("directory should be created");
    dir
}

fn manifest_text(namespace: &str) -> String {
    format!(
        r#"{{
  "name": "{namespace}",
  "version": "1.0.0",
  "schema_version": "1",
  "manifest_version": "1",
  "compatibility": {{ "min_inclusive": "0.1.0", "max_exclusive": "2.0.0" }},
  "namespace": "{namespace}",
  "kind": "delegated",
  "aliases": ["{namespace}-alias"],
  "entrypoint": "{namespace}.plugin:run",
  "capabilities": [{{"name": "exec", "version": "1"}}]
}}"#
    )
}

#[test]
fn compatibility_and_doctor_report_failure_paths_deterministically() {
    let plugins_dir = temp_plugins_dir("doctor");
    let registry_path = registry_path_from_plugins_dir(&plugins_dir);

    let installed = install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("inspector"),
            source: "local:/tmp/inspector".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect("plugin should install");

    let is_compatible =
        compatibility_check(&installed.manifest, "0.1.0").expect("check should pass");
    assert!(is_compatible);

    let report = plugin_doctor(&registry_path).expect("doctor should run");
    assert_eq!(report.installed, 1);
    assert!(report.broken.is_empty());
    assert!(report.incompatible.is_empty());

    let registry = load_registry(&registry_path).expect("registry should load");
    assert_eq!(registry.version, "1");

    let missing_enable =
        enable_plugin(&registry_path, "ghost").expect_err("missing plugin should fail");
    assert!(format!("{missing_enable}").contains("plugin not found"));

    let missing_disable =
        disable_plugin(&registry_path, "ghost").expect_err("missing plugin should fail");
    assert!(format!("{missing_disable}").contains("plugin not found"));

    let missing_uninstall =
        uninstall_plugin(&registry_path, "ghost").expect_err("missing plugin should fail");
    assert!(format!("{missing_uninstall}").contains("plugin not found"));

    let bad_version = compatibility_check(&installed.manifest, "9.9.9").expect("check should run");
    assert!(!bad_version);

    // Corrupt registry and ensure doctor reports deterministic corruption failure.
    fs::write(&registry_path, "{broken-json").expect("write corruption");
    let corrupted = plugin_doctor(&registry_path).expect_err("doctor should fail on bad json");
    assert!(format!("{corrupted}").contains("corrupted"));
}
