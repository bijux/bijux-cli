use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_app::RuntimeSurfaceConfig;

#[test]
fn deprecated_fields_are_rejected_until_explicitly_supported() {
    let payload = r#"
    {
      "jobs": 2,
      "cache_mode": "read",
      "materialize_inputs": "direct",
      "policy": {
        "deny_network": false,
        "deny_env": false,
        "deny_clock": false,
        "clean_env": true,
        "allowed_env": []
      },
      "deprecated_cache_strategy": "legacy"
    }
    "#;

    let parsed = serde_json::from_str::<RuntimeSurfaceConfig>(payload);
    assert!(parsed.is_err());
}
