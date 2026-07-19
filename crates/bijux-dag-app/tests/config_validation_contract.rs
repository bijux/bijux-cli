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

use bijux_dag_app::{MaterializeInputsSurface, RuntimeSurfaceConfig};

#[test]
fn unknown_fields_are_rejected() {
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
      "unknown": true
    }
    "#;

    let parsed = serde_json::from_str::<RuntimeSurfaceConfig>(payload);
    assert!(parsed.is_err());
}

#[test]
fn malformed_value_types_are_rejected() {
    let payload = r#"
    {
      "jobs": "two",
      "cache_mode": "read",
      "materialize_inputs": "direct",
      "policy": {
        "deny_network": false,
        "deny_env": false,
        "deny_clock": false,
        "clean_env": true,
        "allowed_env": []
      }
    }
    "#;

    let parsed = serde_json::from_str::<RuntimeSurfaceConfig>(payload);
    assert!(parsed.is_err());
}

#[test]
fn known_materialization_values_parse() {
    let mode: MaterializeInputsSurface = serde_json::from_str("\"all\"").expect("parse");
    assert_eq!(mode, MaterializeInputsSurface::All);
}
