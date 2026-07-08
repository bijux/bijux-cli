use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::probe_external_adapters;
use std::fs;
use std::sync::{Mutex, OnceLock};

fn process_env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().expect("env lock")
}

fn write_script(path: &std::path::Path, body: &str) {
    fs::write(path, body).expect("write script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path).expect("meta").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).expect("chmod");
    }
}

#[test]
fn malformed_external_adapter_handshakes_are_rejected_with_exact_reasons() {
    let _lock = process_env_lock();
    let dir = tempfile::tempdir().expect("tmpdir");
    let adapters = dir.path().join("adapters");
    fs::create_dir_all(&adapters).expect("mkdir");

    write_script(
        &adapters.join("invalid-json"),
        "#!/bin/sh\nif [ \"$1\" = \"info\" ]; then echo '{not-json'; exit 0; fi\nexit 1\n",
    );
    write_script(
        &adapters.join("missing-protocol"),
        "#!/bin/sh\nif [ \"$1\" = \"info\" ]; then echo '{\"adapter_id\":\"bad\",\"adapter_version\":\"0.1\",\"required_effects\":{\"filesystem\":true,\"env\":false,\"network\":false,\"clock\":false},\"supported_kinds\":[\"fake\"],\"output_schema\":\"v0.1\"}'; exit 0; fi\nexit 1\n",
    );
    write_script(
        &adapters.join("empty-schema"),
        "#!/bin/sh\nif [ \"$1\" = \"info\" ]; then echo '{\"protocol_version\":\"bijux-dag-adapter/v1\",\"adapter_id\":\"bad\",\"adapter_version\":\"0.1\",\"required_effects\":{\"filesystem\":true,\"env\":false,\"network\":false,\"clock\":false},\"supported_kinds\":[\"fake\"],\"output_schema\":\"\"}'; exit 0; fi\nexit 1\n",
    );
    write_script(
        &adapters.join("stderr-noise"),
        "#!/bin/sh\nif [ \"$1\" = \"info\" ]; then echo '{\"protocol_version\":\"bijux-dag-adapter/v1\",\"adapter_id\":\"bad\",\"adapter_version\":\"0.1\",\"required_effects\":{\"filesystem\":true,\"env\":false,\"network\":false,\"clock\":false},\"supported_kinds\":[\"fake\"],\"output_schema\":\"v0.1\"}'; echo 'noise' >&2; exit 0; fi\nexit 1\n",
    );
    let huge_schema = "x".repeat(70_000);
    let huge_payload = format!(
        "#!/bin/sh\nif [ \"$1\" = \"info\" ]; then printf '%s' '{{\"protocol_version\":\"bijux-dag-adapter/v1\",\"adapter_id\":\"huge\",\"adapter_version\":\"0.1\",\"required_effects\":{{\"filesystem\":true,\"env\":false,\"network\":false,\"clock\":false}},\"supported_kinds\":[\"fake\"],\"output_schema\":\"{huge_schema}\"}}'; exit 0; fi\nexit 1\n"
    );
    write_script(&adapters.join("huge-payload"), &huge_payload);

    std::env::set_var("BIJUX_DAG_ADAPTERS_DIR", &adapters);
    let reports = probe_external_adapters().expect("probe");
    std::env::remove_var("BIJUX_DAG_ADAPTERS_DIR");

    assert_eq!(reports.len(), 5);
    let reasons =
        reports.iter().map(|report| report.reason.clone().unwrap_or_default()).collect::<Vec<_>>();
    assert!(reasons.iter().any(|reason| reason.contains("invalid adapter manifest")));
    assert!(reasons.iter().any(|reason| reason.contains("descriptor validation failed")));
    assert!(reasons.iter().any(|reason| reason.contains("stdout only")));
    assert!(reasons.iter().any(|reason| reason.contains("payload exceeds")));
}
