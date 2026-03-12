#![forbid(unsafe_code)]
//! Packaging and install ambiguity hardening coverage.

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use bijux_cli as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn tmp_dir(name: &str) -> PathBuf {
    let dir =
        env::temp_dir().join(format!("bijux-install-ambiguity-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir temp");
    dir
}

fn run_with_env(args: &[&str], envs: &[(&str, String)]) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("run")
}

fn run_identity(envs: &[(&str, String)]) -> Value {
    let out =
        run_with_env(&["dev", "cli", "runtime-identity", "--format", "json", "--no-pretty"], envs);
    assert_eq!(out.status.code(), Some(0));
    serde_json::from_slice(&out.stdout).expect("json")
}

fn install_channel_path(root: &PathBuf, marker: &str) -> PathBuf {
    match marker {
        "cargo" => root.join(".cargo").join("bin"),
        "pip" => root.join("site-packages").join("bin"),
        other => panic!("unsupported marker: {other}"),
    }
}

fn executable_name() -> String {
    let extension = std::env::consts::EXE_EXTENSION;
    if extension.is_empty() {
        "bijux".to_string()
    } else {
        format!("bijux.{extension}")
    }
}

fn write_executable(path: &Path, body: &str) {
    fs::write(path, body).expect("write executable");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path).expect("metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("chmod +x");
    }
}

fn write_channel_binary(dir: &PathBuf) {
    fs::create_dir_all(dir).expect("mkdir channel dir");
    write_executable(&dir.join(executable_name()), "#!/bin/sh\n");
}

fn assert_command_runs_in_channel(marker: &str, args: &[&str], envs: &[(&str, String)]) {
    let root = tmp_dir(&format!("{marker}-{:?}", args).replace([' ', '"', ',', '[', ']'], "-"));
    let dir = install_channel_path(&root, marker);
    write_channel_binary(&dir);
    let path = env::join_paths([&dir]).expect("join path").to_string_lossy().to_string();

    let mut with_path = vec![("PATH", path)];
    with_path.extend_from_slice(envs);

    let out = run_with_env(args, &with_path);
    assert_eq!(
        out.status.code(),
        Some(0),
        "command failed for {marker} {:?}: stderr={}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn pip_binary_shadowed_by_cargo_binary_is_reported() {
    let root = tmp_dir("pip-shadowed-by-cargo");
    let pip = root.join("site-packages-bin");
    let cargo = root.join(".cargo/bin");
    fs::create_dir_all(&pip).expect("mkdir pip");
    fs::create_dir_all(&cargo).expect("mkdir cargo");
    let executable = executable_name();
    write_executable(&pip.join(&executable), "#!/bin/sh\n");
    write_executable(&cargo.join(&executable), "#!/bin/sh\n");

    let path = env::join_paths([&pip, &cargo]).expect("join path").to_string_lossy().to_string();
    let payload = run_identity(&[("PATH", path)]);
    assert_eq!(payload["active_path_is_shadowed"], true);
    assert_eq!(payload["diagnostics"]["path_shadowing_detected"], true);
    assert_eq!(payload["diagnostics"]["mixed_pip_cargo_install_detected"], true);
    assert!(payload["active_binary"].as_str().expect("active").contains("site-packages-bin"));
}

#[test]
fn cargo_binary_shadowed_by_pip_binary_is_reported() {
    let root = tmp_dir("cargo-shadowed-by-pip");
    let cargo = root.join(".cargo/bin");
    let pip = root.join("site-packages-bin");
    fs::create_dir_all(&cargo).expect("mkdir cargo");
    fs::create_dir_all(&pip).expect("mkdir pip");
    let executable = executable_name();
    write_executable(&cargo.join(&executable), "#!/bin/sh\n");
    write_executable(&pip.join(&executable), "#!/bin/sh\n");

    let path = env::join_paths([&cargo, &pip]).expect("join path").to_string_lossy().to_string();
    let payload = run_identity(&[("PATH", path)]);
    assert_eq!(payload["active_path_is_shadowed"], true);
    assert_eq!(payload["diagnostics"]["path_shadowing_detected"], true);
    assert_eq!(payload["diagnostics"]["mixed_pip_cargo_install_detected"], true);
    assert!(payload["active_binary"].as_str().expect("active").contains(".cargo/bin"));
}

#[test]
fn stale_wrapper_and_deleted_cached_runtime_are_detected() {
    let root = tmp_dir("stale-wrapper");
    let wrappers = root.join("wrappers");
    fs::create_dir_all(&wrappers).expect("mkdir wrappers");
    write_executable(&wrappers.join("bijux.sh"), "#!/bin/sh\nexec /missing/bijux\n");

    let deleted_runtime = root.join("deleted-bijux");
    let path = env::join_paths([&wrappers]).expect("join path").to_string_lossy().to_string();
    let payload =
        run_identity(&[("PATH", path), ("BIJUX_BIN", deleted_runtime.display().to_string())]);

    assert_eq!(payload["diagnostics"]["stale_wrapper_detected"], true);
    assert_eq!(payload["diagnostics"]["active_binary_missing"], true);
}

#[test]
fn mismatched_wheel_and_binary_versions_are_reported() {
    let payload = run_identity(&[("BIJUX_WHEEL_VERSION", "0.0.1".to_string())]);
    assert_eq!(payload["diagnostics"]["mismatched_wheel_binary_versions"], true);
    assert_eq!(payload["diagnostics"]["active_binary_mismatch_detected"], true);
}

#[test]
fn missing_python_runtime_support_is_reported_while_rust_binary_is_active() {
    let root = tmp_dir("missing-python-runtime");
    let bin = root.join("bin");
    fs::create_dir_all(&bin).expect("mkdir bin");
    let executable = executable_name();
    write_executable(&bin.join(&executable), "#!/bin/sh\n");
    let path = env::join_paths([&bin]).expect("join").to_string_lossy().to_string();

    let payload =
        run_identity(&[("PATH", path), ("BIJUX_PYTHON_BRIDGE_SUPPORTED", "0".to_string())]);
    assert_eq!(payload["diagnostics"]["python_bridge_supported"], false);
    assert!(
        payload["active_binary"].as_str().expect("active").ends_with(&executable_name()),
        "active binary should resolve to the executable in PATH"
    );
}

#[test]
#[cfg(unix)]
fn broken_symlink_active_binary_is_detected() {
    use std::os::unix::fs::symlink;

    let root = tmp_dir("broken-symlink");
    let broken = root.join("bijux-link");
    symlink(root.join("missing-target"), &broken).expect("create symlink");

    let payload = run_identity(&[("BIJUX_BIN", broken.display().to_string())]);
    assert_eq!(payload["diagnostics"]["active_binary_missing"], true);
    assert_eq!(payload["diagnostics"]["broken_symlink_active_binary"], true);
}

#[test]
fn state_audit_reports_read_only_config_dir_shape() {
    let root = tmp_dir("read-only-config-shape");
    let blocker = root.join("not-a-directory");
    fs::write(&blocker, "block").expect("write blocker");
    let config_path = blocker.join("state.env");
    let out = run_with_env(
        &[
            "dev",
            "cli",
            "state-audit",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            config_path.to_str().expect("utf-8"),
        ],
        &[],
    );

    assert_eq!(out.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(payload["paths"]["config"]["writable"], false);
}

#[test]
fn package_health_and_runtime_identity_cover_ambiguous_install_state() {
    let root = tmp_dir("package-health-ambiguous");
    let first = root.join("first");
    let second = root.join("second-site-packages");
    fs::create_dir_all(&first).expect("mkdir first");
    fs::create_dir_all(&second).expect("mkdir second");
    let executable = executable_name();
    write_executable(&first.join(&executable), "#!/bin/sh\n");
    write_executable(&second.join(&executable), "#!/bin/sh\n");

    let path = env::join_paths([&first, &second]).expect("join").to_string_lossy().to_string();

    let runtime = run_identity(&[("PATH", path.clone())]);
    assert_eq!(runtime["active_binary_selection_is_ambiguous"], true);

    let package = run_with_env(
        &["dev", "cli", "package-health", "--format", "json", "--no-pretty"],
        &[("PATH", path)],
    );
    assert_eq!(package.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&package.stdout).expect("json");
    assert!(payload["install_state_assumptions"].is_array());
    assert!(payload["install_state_assumption_help"].is_string());
}

#[test]
fn runtime_identity_reports_pipx_install_source_when_path_contains_pipx_marker() {
    let root = tmp_dir("pipx-source");
    let pipx = root.join("pipx").join("venvs").join("bijux").join("bin");
    fs::create_dir_all(&pipx).expect("mkdir pipx");
    write_executable(&pipx.join(executable_name()), "#!/bin/sh\n");
    let path = env::join_paths([&pipx]).expect("join path").to_string_lossy().to_string();
    let payload = run_identity(&[("PATH", path)]);
    assert_eq!(payload["install_source"], "pipx");
}

#[test]
fn runtime_identity_reports_cargo_install_source_when_path_contains_cargo_marker() {
    let root = tmp_dir("cargo-source");
    let cargo = root.join(".cargo").join("bin");
    fs::create_dir_all(&cargo).expect("mkdir cargo");
    write_executable(&cargo.join(executable_name()), "#!/bin/sh\n");
    let path = env::join_paths([&cargo]).expect("join path").to_string_lossy().to_string();
    let payload = run_identity(&[("PATH", path)]);
    assert_eq!(payload["install_source"], "cargo");
}

#[test]
fn runtime_identity_reports_pip_install_source_when_path_contains_site_packages_marker() {
    let root = tmp_dir("pip-source");
    let pip = root.join("site-packages").join("bin");
    fs::create_dir_all(&pip).expect("mkdir pip");
    write_executable(&pip.join(executable_name()), "#!/bin/sh\n");
    let path = env::join_paths([&pip]).expect("join path").to_string_lossy().to_string();
    let payload = run_identity(&[("PATH", path)]);
    assert_eq!(payload["install_source"], "pip");
}

#[test]
fn runtime_identity_reports_bridge_fallback_diagnostic_when_bridge_is_unavailable() {
    let payload = run_identity(&[("BIJUX_PYTHON_BRIDGE_SUPPORTED", "0".to_string())]);
    assert_eq!(payload["diagnostics"]["python_bridge_supported"], false);
}

#[test]
fn cargo_installed_invocation_version_is_green() {
    assert_command_runs_in_channel("cargo", &["version"], &[]);
}

#[test]
fn pip_installed_invocation_version_is_green() {
    assert_command_runs_in_channel("pip", &["version"], &[]);
}

#[test]
fn cargo_installed_invocation_status_is_green() {
    assert_command_runs_in_channel("cargo", &["status", "--format", "json", "--no-pretty"], &[]);
}

#[test]
fn pip_installed_invocation_status_is_green() {
    assert_command_runs_in_channel("pip", &["status", "--format", "json", "--no-pretty"], &[]);
}

#[test]
fn cargo_installed_invocation_plugins_list_is_green() {
    assert_command_runs_in_channel(
        "cargo",
        &["plugins", "list", "--format", "json", "--no-pretty"],
        &[],
    );
}

#[test]
fn pip_installed_invocation_plugins_list_is_green() {
    assert_command_runs_in_channel(
        "pip",
        &["plugins", "list", "--format", "json", "--no-pretty"],
        &[],
    );
}

#[test]
fn cargo_installed_invocation_config_get_is_green() {
    let root = tmp_dir("cargo-config-get");
    let config_path = root.join("config.env");
    fs::write(&config_path, "BIJUXCLI_ALPHA=1\n").expect("write config");
    assert_command_runs_in_channel(
        "cargo",
        &[
            "cli",
            "config",
            "get",
            "alpha",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            config_path.to_str().expect("utf-8"),
        ],
        &[],
    );
}

#[test]
fn pip_installed_invocation_config_get_is_green() {
    let root = tmp_dir("pip-config-get");
    let config_path = root.join("config.env");
    fs::write(&config_path, "BIJUXCLI_ALPHA=1\n").expect("write config");
    assert_command_runs_in_channel(
        "pip",
        &[
            "cli",
            "config",
            "get",
            "alpha",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            config_path.to_str().expect("utf-8"),
        ],
        &[],
    );
}

#[test]
fn cli_paths_under_overridden_home_are_consistent() {
    let root = tmp_dir("paths-home-override");
    let out = run_with_env(
        &["cli", "paths", "--format", "json", "--no-pretty"],
        &[("HOME", root.display().to_string())],
    );
    assert_eq!(out.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");

    let expected_prefix = root.join(".bijux");
    assert!(payload["config"]
        .as_str()
        .expect("config")
        .contains(expected_prefix.to_str().expect("utf-8")));
    assert!(payload["history"]
        .as_str()
        .expect("history")
        .contains(expected_prefix.to_str().expect("utf-8")));
    assert!(payload["plugins"]
        .as_str()
        .expect("plugins")
        .contains(expected_prefix.to_str().expect("utf-8")));
}

#[test]
fn cli_paths_under_xdg_style_home_root_are_consistent() {
    let root = tmp_dir("paths-xdg-style-home").join(".local").join("share");
    fs::create_dir_all(&root).expect("mkdir xdg-style home");

    let out = run_with_env(
        &["cli", "paths", "--format", "json", "--no-pretty"],
        &[("HOME", root.display().to_string())],
    );
    assert_eq!(out.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");

    let expected_prefix = root.join(".bijux");
    assert!(payload["config"]
        .as_str()
        .expect("config")
        .contains(expected_prefix.to_str().expect("utf-8")));
    assert!(payload["history"]
        .as_str()
        .expect("history")
        .contains(expected_prefix.to_str().expect("utf-8")));
    assert!(payload["plugins"]
        .as_str()
        .expect("plugins")
        .contains(expected_prefix.to_str().expect("utf-8")));
}

#[test]
#[cfg(unix)]
fn state_audit_reports_unwritable_config_plugin_and_history_locations() {
    use std::os::unix::fs::PermissionsExt;

    let root = tmp_dir("state-audit-unwritable");
    let config = root.join("config.env");
    let history = root.join("history.log");
    let plugins = root.join("plugins");
    fs::create_dir_all(&plugins).expect("mkdir plugins");
    fs::write(&config, "BIJUXCLI_ALPHA=1\n").expect("write config");
    fs::write(&history, "[]\n").expect("write history");
    let registry = plugins.join("registry.json");
    fs::write(&registry, "{\"version\":\"1\",\"plugins\":{}}\n").expect("write registry");
    fs::set_permissions(&config, fs::Permissions::from_mode(0o444)).expect("readonly config");
    fs::set_permissions(&history, fs::Permissions::from_mode(0o444)).expect("readonly history");
    fs::set_permissions(&registry, fs::Permissions::from_mode(0o444)).expect("readonly registry");

    let out = run_with_env(
        &["dev", "cli", "state-audit", "--format", "json", "--no-pretty"],
        &[
            ("BIJUXCLI_CONFIG", config.display().to_string()),
            ("BIJUXCLI_HISTORY_FILE", history.display().to_string()),
            ("BIJUXCLI_PLUGINS_DIR", plugins.display().to_string()),
        ],
    );
    assert_eq!(out.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(payload["paths"]["config"]["writable"], false);
    assert_eq!(payload["paths"]["history"]["writable"], false);
    assert_eq!(payload["paths"]["plugins_registry"]["writable"], false);
}
