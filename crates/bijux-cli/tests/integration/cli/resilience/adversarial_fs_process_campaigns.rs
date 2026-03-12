#![forbid(unsafe_code)]
//! Adversarial filesystem/process hardening coverage.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use bijux_cli as _;
use libc as _;
use serde_json as _;
use shlex as _;
use thiserror as _;

fn temp_dir(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "bijux-fs-process-adversarial-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("mkdir temp");
    path
}

fn run_with_env(args: &[&str], envs: &[(&str, String)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("run command")
}

fn assert_known_status(out: &Output, context: &str) {
    let code = out.status.code();
    assert!(
        matches!(code, Some(0) | Some(1) | Some(2)),
        "{context} produced unexpected status {code:?}"
    );
    match code {
        Some(0) => {
            assert!(
                out.stderr.is_empty(),
                "{context} succeeded but wrote stderr"
            );
            assert!(
                !out.stdout.is_empty(),
                "{context} succeeded but produced empty stdout"
            );
            let _: serde_json::Value = serde_json::from_slice(&out.stdout)
                .expect("successful machine path should emit json");
        }
        Some(1) | Some(2) => {
            assert!(
                out.stdout.is_empty(),
                "{context} failure must not write stdout"
            );
            assert!(
                !out.stderr.is_empty(),
                "{context} failure must write stderr"
            );
        }
        _ => unreachable!("handled above"),
    }
}

#[test]
fn missing_parent_and_type_flip_path_cases_are_handled_without_corruption() {
    let root = temp_dir("missing-parent-type-flip");

    let missing_parent_config = root.join("missing").join("parent").join("config.env");
    let out_cfg = run_with_env(
        &[
            "cli",
            "config",
            "set",
            "alpha=1",
            "--config-path",
            missing_parent_config.to_str().expect("utf-8"),
        ],
        &[],
    );
    assert_known_status(&out_cfg, "missing parent config write");

    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    let registry = plugins_dir.join("registry.json");
    fs::write(&registry, "{\"plugins\":[]}").expect("seed registry");
    fs::remove_file(&registry).expect("remove registry file");
    fs::create_dir_all(&registry).expect("replace file with directory");

    let out_registry = run_with_env(
        &["cli", "plugins", "list", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_PLUGINS_DIR", plugins_dir.display().to_string())],
    );
    assert_known_status(&out_registry, "registry replaced by directory");

    let history_path = root.join("history.log");
    fs::create_dir_all(&history_path).expect("replace history file path with directory");
    let out_history = run_with_env(
        &["history", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", history_path.display().to_string())],
    );
    assert_known_status(&out_history, "history file replaced by directory");
}

#[test]
#[cfg(unix)]
fn broken_symlink_and_permission_denied_paths_surface_stable_failures() {
    use std::os::unix::fs::symlink;
    use std::os::unix::fs::PermissionsExt;

    let root = temp_dir("symlink-permissions");

    let config_link = root.join("config-link.env");
    symlink(root.join("missing-config-target.env"), &config_link).expect("create config symlink");
    let cfg_out = run_with_env(
        &[
            "cli",
            "config",
            "list",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            config_link.to_str().expect("utf-8"),
        ],
        &[],
    );
    assert_known_status(&cfg_out, "broken config symlink");

    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    let registry = plugins_dir.join("registry.json");
    symlink(root.join("missing-registry-target.json"), &registry).expect("create registry symlink");
    let reg_out = run_with_env(
        &["cli", "plugins", "list", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_PLUGINS_DIR", plugins_dir.display().to_string())],
    );
    assert_known_status(&reg_out, "broken registry symlink");

    let history = root.join("history.log");
    fs::write(&history, "status\n").expect("seed history");
    fs::set_permissions(&history, fs::Permissions::from_mode(0o000)).expect("chmod history 000");
    let hist_out = run_with_env(
        &["history", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", history.display().to_string())],
    );
    fs::set_permissions(&history, fs::Permissions::from_mode(0o644))
        .expect("restore history perms");
    assert_known_status(&hist_out, "unreadable history");

    let cfg_dir = root.join("cfgdir");
    fs::create_dir_all(&cfg_dir).expect("mkdir cfgdir");
    let config = cfg_dir.join("active.env");
    fs::write(&config, "BIJUXCLI_ALPHA=1\n").expect("seed config");
    fs::set_permissions(&cfg_dir, fs::Permissions::from_mode(0o555)).expect("chmod cfgdir");
    let write_fail = run_with_env(
        &[
            "cli",
            "config",
            "set",
            "alpha=2",
            "--config-path",
            config.to_str().expect("utf-8"),
        ],
        &[],
    );
    fs::set_permissions(&cfg_dir, fs::Permissions::from_mode(0o755)).expect("restore cfgdir");
    assert_known_status(&write_fail, "unwritable config");
}

#[test]
fn rename_race_and_temp_leftovers_keep_commands_non_panicking() {
    let root = temp_dir("rename-temp-leftovers");
    let config = root.join("active.env");
    fs::write(&config, "BIJUXCLI_ALPHA=1\n").expect("seed config");

    let moved = root.join("active-renamed.env");
    fs::rename(&config, &moved).expect("rename config during race simulation");
    fs::write(config.with_extension("tmp"), "stale-temp\n").expect("write stale temp");
    fs::write(config.with_extension("partial"), "partial-write\n").expect("write stale partial");

    let cfg_out = run_with_env(
        &[
            "cli",
            "config",
            "list",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            config.to_str().expect("utf-8"),
        ],
        &[],
    );
    assert_known_status(&cfg_out, "config rename race");

    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    let registry = plugins_dir.join("registry.json");
    fs::write(&registry, "{\"plugins\":[]}").expect("seed registry");
    let registry_moved = plugins_dir.join("registry-renamed.json");
    fs::rename(&registry, &registry_moved).expect("rename registry");
    fs::write(registry.with_extension("tmp"), "stale\n").expect("write stale tmp");

    let plugins_out = run_with_env(
        &["cli", "plugins", "list", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_PLUGINS_DIR", plugins_dir.display().to_string())],
    );
    assert_known_status(&plugins_out, "plugin registry rename race");
}

#[test]
fn child_process_failure_paths_surface_normalized_failures_when_plugins_are_broken() {
    let root = temp_dir("child-process-failure");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    fs::write(
        plugins_dir.join("registry.json"),
        "{\"plugins\":[{\"name\":\"broken\",\"path\":\"/definitely/missing/plugin\",\"enabled\":true}]}",
    )
    .expect("write broken registry");

    let check_out = run_with_env(
        &["cli", "plugins", "check", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_PLUGINS_DIR", plugins_dir.display().to_string())],
    );
    assert_known_status(&check_out, "plugin check broken path");

    let inspect_out = run_with_env(
        &[
            "cli",
            "plugins",
            "inspect",
            "broken",
            "--format",
            "json",
            "--no-pretty",
        ],
        &[("BIJUXCLI_PLUGINS_DIR", plugins_dir.display().to_string())],
    );
    assert_known_status(&inspect_out, "plugin inspect broken path");
}

#[test]
#[cfg(unix)]
fn interrupted_process_behavior_is_normalized_for_interactive_entrypoint() {
    use std::os::unix::process::ExitStatusExt;
    use std::time::Duration;

    let mut child = Command::new(env!("CARGO_BIN_EXE_bijux"))
        .args(["sleep", "5"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn sleep");

    // Give startup a short window, then interrupt via SIGINT to emulate Ctrl-C.
    std::thread::sleep(Duration::from_millis(40));
    let status = Command::new("kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .expect("kill command should execute");
    assert!(status.success(), "SIGINT command should succeed");

    let status = child.wait().expect("wait interrupt");
    if let Some(code) = status.code() {
        assert!(
            matches!(code, 0 | 130),
            "unexpected SIGINT normalized exit code: {code}"
        );
    } else {
        // Signaled exits are acceptable normalization on Unix, but must match SIGINT.
        assert_eq!(status.signal(), Some(libc::SIGINT));
    }
}
