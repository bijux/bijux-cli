#![forbid(unsafe_code)]
//! Contracts for runtime-identity and package-health hardening.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

fn tmp_dir(name: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!(
        "bijux-runtime-package-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir temp");
    dir
}

fn run(args: &[&str], envs: &[(&str, String)]) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(args);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    cmd.output().expect("run")
}

fn run_runtime_identity_json(envs: &[(&str, String)]) -> Value {
    let out = run(
        &[
            "dev",
            "cli",
            "runtime-identity",
            "--format",
            "json",
            "--no-pretty",
        ],
        envs,
    );
    assert!(
        out.status.success(),
        "runtime-identity failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("runtime-identity json")
}

fn run_package_health_json(envs: &[(&str, String)]) -> Value {
    let out = run(
        &[
            "dev",
            "cli",
            "package-health",
            "--format",
            "json",
            "--no-pretty",
        ],
        envs,
    );
    assert!(
        out.status.success(),
        "package-health failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("package-health json")
}

#[test]
fn runtime_identity_and_package_health_json_and_text_contracts() {
    let runtime_json = run_runtime_identity_json(&[]);
    let package_json = run_package_health_json(&[]);
    assert!(runtime_json.is_object());
    assert!(package_json.is_object());

    let runtime_text = run(&["dev", "cli", "runtime-identity", "--format", "text"], &[]);
    let package_text = run(&["dev", "cli", "package-health", "--format", "text"], &[]);
    assert!(runtime_text.status.success());
    assert!(package_text.status.success());
    assert!(!String::from_utf8_lossy(&runtime_text.stdout)
        .trim()
        .is_empty());
    assert!(!String::from_utf8_lossy(&package_text.stdout)
        .trim()
        .is_empty());
}

#[test]
fn runtime_identity_detects_pure_cargo_and_pure_pip_paths() {
    let root = tmp_dir("pure-install");
    let cargo = root.join(".cargo").join("bin");
    let pip = root.join("site-packages").join("bin");
    fs::create_dir_all(&cargo).expect("mkdir cargo");
    fs::create_dir_all(&pip).expect("mkdir pip");
    fs::write(cargo.join("bijux"), "#!/bin/sh\n").expect("write cargo binary");
    fs::write(pip.join("bijux"), "#!/bin/sh\n").expect("write pip binary");

    let cargo_path = env::join_paths([&cargo])
        .expect("join")
        .to_string_lossy()
        .to_string();
    let pip_path = env::join_paths([&pip])
        .expect("join")
        .to_string_lossy()
        .to_string();

    let cargo_payload = run_runtime_identity_json(&[("PATH", cargo_path)]);
    let pip_payload = run_runtime_identity_json(&[("PATH", pip_path)]);
    assert_eq!(cargo_payload["install_source"], "cargo");
    assert_eq!(pip_payload["install_source"], "pip");
}

#[test]
fn runtime_identity_detects_mixed_install_ambiguity_and_path_shadowing() {
    let root = tmp_dir("mixed-shadow");
    let first = root.join("first-site-packages");
    let second = root.join(".cargo").join("bin");
    fs::create_dir_all(&first).expect("mkdir first");
    fs::create_dir_all(&second).expect("mkdir second");
    fs::write(first.join("bijux"), "#!/bin/sh\n").expect("write first");
    fs::write(second.join("bijux"), "#!/bin/sh\n").expect("write second");
    let path = env::join_paths([&first, &second])
        .expect("join")
        .to_string_lossy()
        .to_string();

    let payload = run_runtime_identity_json(&[("PATH", path)]);
    assert_eq!(payload["active_binary_selection_is_ambiguous"], true);
    assert_eq!(payload["diagnostics"]["path_shadowing_detected"], true);
    assert_eq!(
        payload["diagnostics"]["mixed_pip_cargo_install_detected"],
        true
    );
}

#[test]
fn runtime_identity_detects_stale_wrapper_and_binary_version_mismatch() {
    let root = tmp_dir("wrapper-mismatch");
    let wrappers = root.join("wrappers");
    fs::create_dir_all(&wrappers).expect("mkdir wrappers");
    fs::write(
        wrappers.join("bijux.sh"),
        "#!/bin/sh\nexec /missing/bijux\n",
    )
    .expect("write wrapper");
    let path = env::join_paths([&wrappers])
        .expect("join path")
        .to_string_lossy()
        .to_string();

    let payload = run_runtime_identity_json(&[
        ("PATH", path),
        (
            "BIJUX_BIN",
            root.join("missing-bijux").to_string_lossy().to_string(),
        ),
        ("BIJUX_WHEEL_VERSION", "0.0.1".to_string()),
    ]);
    assert_eq!(payload["diagnostics"]["stale_wrapper_detected"], true);
    assert_eq!(
        payload["diagnostics"]["mismatched_wheel_binary_versions"],
        true
    );
}

#[test]
#[cfg(unix)]
fn runtime_identity_detects_broken_symlink_active_binary() {
    use std::os::unix::fs::symlink;

    let root = tmp_dir("broken-symlink");
    let broken = root.join("bijux-link");
    symlink(root.join("missing-target"), &broken).expect("symlink");
    let payload = run_runtime_identity_json(&[("BIJUX_BIN", broken.to_string_lossy().to_string())]);
    assert_eq!(payload["diagnostics"]["broken_symlink_active_binary"], true);
}

#[test]
fn package_health_reports_python_runtime_relevance_and_assumptions() {
    let payload = run_package_health_json(&[("BIJUX_PYTHON_BRIDGE_SUPPORTED", "0".to_string())]);
    assert!(
        payload["runtime_identity_rules"].is_object(),
        "runtime identity rules shape should be present"
    );
    assert!(
        payload["install_state_assumptions"]
            .as_array()
            .is_some_and(|rows| !rows.is_empty()),
        "install assumptions should be present"
    );
}

#[test]
fn runtime_identity_and_package_health_are_deterministic_under_ambiguity() {
    let root = tmp_dir("deterministic-ambiguity");
    let first = root.join("first");
    let second = root.join("second-site-packages");
    fs::create_dir_all(&first).expect("mkdir first");
    fs::create_dir_all(&second).expect("mkdir second");
    fs::write(first.join("bijux"), "#!/bin/sh\n").expect("write first");
    fs::write(second.join("bijux"), "#!/bin/sh\n").expect("write second");
    let path = env::join_paths([&first, &second])
        .expect("join")
        .to_string_lossy()
        .to_string();
    let envs = [("PATH", path)];

    let runtime_first = run_runtime_identity_json(&envs);
    let runtime_second = run_runtime_identity_json(&envs);
    let package_first = run_package_health_json(&envs);
    let package_second = run_package_health_json(&envs);

    assert_eq!(
        runtime_first, runtime_second,
        "runtime identity output drift"
    );
    assert_eq!(package_first, package_second, "package health output drift");
}
