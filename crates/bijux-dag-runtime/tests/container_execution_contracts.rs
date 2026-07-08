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

use bijux_dag_runtime::{
    container_env_isolated, container_gpu_runtime_args, map_local_path_to_container,
    validate_container_contract, validate_container_relative_path, ContainerExecutionContract,
    ContainerMount,
};
use std::collections::BTreeMap;
use std::path::Path;

#[test]
fn container_contract_requires_image_command_mount_and_valid_outputs() {
    let contract = ContainerExecutionContract {
        image: "ghcr.io/example/runner@sha256:deadbeef".to_string(),
        command: vec!["/bin/run".to_string()],
        env: BTreeMap::new(),
        mounts: vec![ContainerMount {
            local_path: "/work/run/input".to_string(),
            container_path: "/mnt/input".to_string(),
            readonly: true,
        }],
        declared_outputs: vec!["out/result.json".to_string()],
        gpu_devices: 1,
        timeout_ms: Some(10_000),
    };
    assert!(validate_container_contract(&contract).is_ok());
}

#[test]
fn container_contract_classifies_missing_image_and_missing_outputs() {
    let bad_image = ContainerExecutionContract {
        image: String::new(),
        command: vec!["/bin/run".to_string()],
        env: BTreeMap::new(),
        mounts: vec![ContainerMount {
            local_path: "/work/run/input".to_string(),
            container_path: "/mnt/input".to_string(),
            readonly: true,
        }],
        declared_outputs: vec!["out/result.json".to_string()],
        gpu_devices: 0,
        timeout_ms: None,
    };
    assert!(validate_container_contract(&bad_image)
        .expect_err("must fail")
        .contains("missing container image"));

    let bad_output = ContainerExecutionContract {
        image: "ghcr.io/example/runner:1".to_string(),
        command: vec!["/bin/run".to_string()],
        env: BTreeMap::new(),
        mounts: vec![ContainerMount {
            local_path: "/work/run/input".to_string(),
            container_path: "/mnt/input".to_string(),
            readonly: true,
        }],
        declared_outputs: vec!["../escape".to_string()],
        gpu_devices: 0,
        timeout_ms: None,
    };
    assert!(validate_container_contract(&bad_output).is_err());
}

#[test]
fn container_contract_accepts_image_starting_with_dash_as_literal_image_name() {
    let contract = ContainerExecutionContract {
        image: "-image".to_string(),
        command: vec!["/bin/run".to_string()],
        env: BTreeMap::new(),
        mounts: vec![ContainerMount {
            local_path: "/work/run/input".to_string(),
            container_path: "/mnt/input".to_string(),
            readonly: true,
        }],
        declared_outputs: vec!["out/result.json".to_string()],
        gpu_devices: 0,
        timeout_ms: None,
    };
    assert!(validate_container_contract(&contract).is_ok());
}

#[test]
fn container_contract_accepts_image_with_option_like_segments_as_image_literal() {
    let contract = ContainerExecutionContract {
        image: "registry.local/repo/--debug:latest".to_string(),
        command: vec!["/bin/run".to_string()],
        env: BTreeMap::new(),
        mounts: vec![ContainerMount {
            local_path: "/work/run/input".to_string(),
            container_path: "/mnt/input".to_string(),
            readonly: true,
        }],
        declared_outputs: vec!["out/result.json".to_string()],
        gpu_devices: 0,
        timeout_ms: None,
    };
    assert!(validate_container_contract(&contract).is_ok());
}

#[test]
fn container_gpu_runtime_args_match_supported_engines() {
    assert_eq!(
        container_gpu_runtime_args("docker", 2).expect("docker gpu args"),
        vec!["--gpus=2".to_string()]
    );
    assert_eq!(
        container_gpu_runtime_args("podman", 1).expect("podman gpu args"),
        vec!["--gpus=1".to_string()]
    );
    assert!(container_gpu_runtime_args("custom-engine", 1)
        .expect_err("unsupported engine")
        .contains("cannot request gpu devices"));
    assert!(container_gpu_runtime_args("docker", 0).expect("zero gpu").is_empty());
}

#[test]
fn container_path_mapping_and_normalization_are_stable() {
    let mapped = map_local_path_to_container(
        Path::new("/work/run"),
        Path::new("/mnt/run"),
        Path::new("/work/run/nodes/a/out.json"),
    )
    .expect("map");
    assert_eq!(mapped, "/mnt/run/nodes/a/out.json");
    assert!(validate_container_relative_path("nodes/a/out.json").is_ok());
    assert!(validate_container_relative_path("../escape").is_err());
}

#[test]
fn container_env_isolation_respects_allowlist_and_denylist() {
    let env = BTreeMap::from([
        ("PATH".to_string(), "/usr/bin".to_string()),
        ("RUST_LOG".to_string(), "debug".to_string()),
        ("AWS_SECRET_ACCESS_KEY".to_string(), "x".to_string()),
    ]);
    let allowlist =
        vec!["PATH".to_string(), "RUST_*".to_string(), "AWS_SECRET_ACCESS_KEY".to_string()];
    let denylist = vec!["AWS_SECRET_*".to_string()];
    assert!(!container_env_isolated(&env, &allowlist, &denylist));

    let strict_allowlist = vec!["PATH".to_string()];
    assert!(!container_env_isolated(&env, &strict_allowlist, &[]));
}
