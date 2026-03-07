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
