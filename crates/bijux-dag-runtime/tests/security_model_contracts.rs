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

use bijux_dag_core::{
    ContainerSpec, Effect, FileOutput, Node, NodeKind, ParamValue, RetryPolicy, SemanticNodeKind,
    TriggerRule,
};
use bijux_dag_runtime::{
    authorize_input_path, authorize_output_path, declared_environment, effective_env_allowlist,
    is_allowed_env_key, is_denied_env_key, shape_environment,
};
use std::collections::BTreeMap;
use std::fs;

#[test]
fn clean_env_and_allowlist_contract_is_deterministic() {
    let ambient = BTreeMap::from([
        ("PATH".to_string(), "/usr/bin".to_string()),
        ("HOME".to_string(), "/home/test".to_string()),
        ("AWS_SECRET_ACCESS_KEY".to_string(), "secret".to_string()),
    ]);
    let explicit = BTreeMap::from([
        ("PATH".to_string(), "/custom/bin".to_string()),
        ("RUST_LOG".to_string(), "info".to_string()),
    ]);
    let allowlist = vec!["PATH".to_string(), "RUST_*".to_string()];
    let denylist = vec!["AWS_*".to_string()];

    let clean = shape_environment(&ambient, true, &allowlist, &denylist, &explicit);
    assert_eq!(clean.get("PATH"), Some(&"/custom/bin".to_string()));
    assert_eq!(clean.get("RUST_LOG"), Some(&"info".to_string()));
    assert!(!clean.contains_key("HOME"));
    assert!(!clean.contains_key("AWS_SECRET_ACCESS_KEY"));

    let permissive = shape_environment(&ambient, false, &allowlist, &denylist, &explicit);
    assert_eq!(permissive.get("PATH"), Some(&"/custom/bin".to_string()));
    assert_eq!(permissive.get("RUST_LOG"), Some(&"info".to_string()));
    assert!(!permissive.contains_key("HOME"));
    assert!(!permissive.contains_key("AWS_SECRET_ACCESS_KEY"));
}

#[test]
fn env_pattern_matching_contract_works_for_exact_and_prefix() {
    let allowlist = vec!["PATH".to_string(), "RUST_*".to_string()];
    let denylist = vec!["AWS_*".to_string(), "SECRET_TOKEN".to_string()];
    assert!(is_allowed_env_key("PATH", &allowlist));
    assert!(is_allowed_env_key("RUST_LOG", &allowlist));
    assert!(!is_allowed_env_key("HOME", &allowlist));
    assert!(is_denied_env_key("AWS_REGION", &denylist));
    assert!(is_denied_env_key("SECRET_TOKEN", &denylist));
    assert!(!is_denied_env_key("PATH", &denylist));
}

#[test]
fn declared_environment_wrapper_blocks_undeclared_ambient_keys() {
    let ambient = BTreeMap::from([
        ("HOME".to_string(), "/home/test".to_string()),
        ("BIJUX_ALLOWED".to_string(), "ok".to_string()),
    ]);

    let shaped = declared_environment(&ambient, false, &[], &[]);
    assert!(shaped.is_empty());

    let declared = declared_environment(&ambient, false, &["BIJUX_ALLOWED".to_string()], &[]);
    assert_eq!(declared.get("BIJUX_ALLOWED"), Some(&"ok".to_string()));
    assert!(!declared.contains_key("HOME"));
}

#[test]
fn effective_env_allowlist_merges_node_and_container_bindings() {
    let node = Node {
        id: "container-node".to_string(),
        kind: NodeKind::Container,
        semantic_kind: SemanticNodeKind::Task,
        inputs: vec![],
        outputs: vec![FileOutput::new("out".to_string(), "out.txt".to_string())],
        params: ParamValue::default(),
        container: Some(ContainerSpec {
            image: "alpine:3.20".to_string(),
            argv: vec!["/bin/true".to_string()],
            env_allowlist: vec!["CONTAINER_ONLY".to_string(), "SHARED".to_string()],
            workdir: None,
            engine: "docker".to_string(),
        }),
        timeout_ms: None,
        resources: None,
        tags: vec![],
        retry: RetryPolicy::default(),
        cache: Default::default(),
        effects: vec![Effect::Filesystem, Effect::Env],
        env_allowlist: vec!["NODE_ONLY".to_string(), "SHARED".to_string()],
        group: None,
        trigger_rule: TriggerRule::AllSuccess,
        branch: None,
        dynamic: None,
    };

    assert_eq!(
        effective_env_allowlist(&node),
        vec!["CONTAINER_ONLY".to_string(), "NODE_ONLY".to_string(), "SHARED".to_string()]
    );
}

#[test]
fn input_and_output_authorization_reject_path_traversal_and_symlink_escape() {
    let temp = tempfile::tempdir().expect("tempdir");
    let input_root = temp.path().join("input");
    let output_root = temp.path().join("output");
    fs::create_dir_all(&input_root).expect("input root");
    fs::create_dir_all(&output_root).expect("output root");

    let valid_input = input_root.join("file.txt");
    fs::write(&valid_input, "ok").expect("write input");
    let valid_output = output_root.join("node").join("result.txt");
    fs::create_dir_all(valid_output.parent().expect("output parent")).expect("mkdir output parent");
    fs::write(&valid_output, "ok").expect("write output");

    assert!(authorize_input_path(&input_root, &valid_input).is_ok());
    assert!(authorize_output_path(&output_root, &valid_output).is_ok());

    let outside = temp.path().join("outside.txt");
    fs::write(&outside, "x").expect("write outside");
    assert!(authorize_input_path(&input_root, &outside).is_err());
    assert!(authorize_output_path(&output_root, &outside).is_err());

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let link = output_root.join("escape-link");
        symlink(&outside, &link).expect("symlink");
        assert!(authorize_output_path(&output_root, &link).is_err());
    }
}
