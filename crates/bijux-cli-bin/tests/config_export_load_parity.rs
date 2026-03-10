#![forbid(unsafe_code)]
//! Config export/load parity and snapshot coverage.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli_core as _;
use bijux_cli_python as _;
use bijux_cli_repl as _;
use bijux_cli_routing as _;
use libc as _;
use serde_json as _;
use shlex as _;
use thiserror as _;
fn make_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    let path = std::env::temp_dir().join(format!("bijux-config-export-load-bin-{name}-{nanos}"));
    fs::create_dir_all(&path).expect("mkdir");
    path
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("binary should execute")
}

fn run_with_env(args: &[&str], envs: &[(&str, String)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("binary should execute")
}

fn python_cli() -> String {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().and_then(|p| p.parent()).expect("workspace root");
    root.join("bin").join("bijux").display().to_string()
}

fn run_python(args: &[&str], envs: &HashMap<String, String>) -> Output {
    let mut cmd = Command::new(python_cli());
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("python cli")
}

fn normalize_snapshot(stdout: String, path_a: &str, path_b: &str) -> String {
    stdout.replace(path_a, "<ACTIVE_CONFIG_PATH>").replace(path_b, "<EXTERNAL_PATH>")
}

#[test]
fn config_export_json_yaml_and_text_error_snapshots() {
    let temp = make_temp_dir("export-snapshots");
    let active = temp.join("active.env");
    let export_file = temp.join("export.env");
    fs::write(&active, "BIJUXCLI_ALPHA=1\nBIJUXCLI_BETA=2\n").expect("seed");

    let active_path = active.to_str().expect("utf-8");
    let export_path = export_file.to_str().expect("utf-8");

    let json = run(&[
        "cli",
        "config",
        "export",
        export_path,
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        active_path,
    ]);
    assert_eq!(json.status.code(), Some(0));
    assert_eq!(
        normalize_snapshot(
            String::from_utf8(json.stdout).expect("utf-8"),
            active_path,
            export_path
        ),
        include_str!("snapshots/config_export_json_compact.txt")
    );

    let yaml = run(&[
        "cli",
        "config",
        "export",
        export_path,
        "--format",
        "yaml",
        "--pretty",
        "--config-path",
        active_path,
    ]);
    assert_eq!(yaml.status.code(), Some(0));
    assert_eq!(
        normalize_snapshot(
            String::from_utf8(yaml.stdout).expect("utf-8"),
            active_path,
            export_path
        ),
        include_str!("snapshots/config_export_yaml_pretty.txt")
    );

    let text = run(&[
        "cli",
        "config",
        "export",
        export_path,
        "--format",
        "text",
        "--config-path",
        active_path,
    ]);
    assert_eq!(text.status.code(), Some(2));
    assert!(text.stdout.is_empty());
    assert_eq!(
        normalize_snapshot(
            String::from_utf8(text.stderr).expect("utf-8"),
            active_path,
            export_path
        ),
        include_str!("snapshots/config_export_text_error.txt")
    );
}

#[test]
fn config_export_writes_file_and_handles_missing_path_argument() {
    let temp = make_temp_dir("export-behavior");
    let active = temp.join("active.env");
    let export_file = temp.join("export.env");
    fs::write(&active, "BIJUXCLI_ALPHA=1\n").expect("seed");

    let out = run(&[
        "cli",
        "config",
        "export",
        export_file.to_str().expect("utf-8"),
        "--config-path",
        active.to_str().expect("utf-8"),
    ]);
    assert_eq!(out.status.code(), Some(0));
    let exported = fs::read_to_string(&export_file).expect("exported file");
    assert_eq!(exported, "BIJUXCLI_ALPHA=1\n");

    let missing = run(&[
        "cli",
        "config",
        "export",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        active.to_str().expect("utf-8"),
    ]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(missing.stdout.is_empty());
    assert!(!missing.stderr.is_empty());
}

#[test]
fn config_load_valid_malformed_duplicate_and_unreadable_cases() {
    let temp = make_temp_dir("load-cases");
    let active = temp.join("active.env");
    let source = temp.join("source.env");
    fs::write(&active, "BIJUXCLI_ACTIVE=1\n").expect("seed active");

    fs::write(&source, "BIJUXCLI_ALPHA=1\n").expect("seed source");
    let ok = run(&[
        "cli",
        "config",
        "load",
        source.to_str().expect("utf-8"),
        "--config-path",
        active.to_str().expect("utf-8"),
    ]);
    assert_eq!(ok.status.code(), Some(0));
    let loaded = fs::read_to_string(&active).expect("active after load");
    assert_eq!(loaded, "BIJUXCLI_ALPHA=1\n");

    fs::write(&source, "BIJUXCLI_ALPHA=1\nBIJUXCLI_ALPHA=2\n").expect("seed duplicates");
    let duplicate = run(&[
        "cli",
        "config",
        "load",
        source.to_str().expect("utf-8"),
        "--config-path",
        active.to_str().expect("utf-8"),
    ]);
    assert_eq!(duplicate.status.code(), Some(0));
    let duplicate_loaded = fs::read_to_string(&active).expect("active after duplicate load");
    assert_eq!(duplicate_loaded, "BIJUXCLI_ALPHA=2\n");

    fs::write(&source, "BROKEN\n").expect("seed malformed");
    let malformed = run(&[
        "cli",
        "config",
        "load",
        source.to_str().expect("utf-8"),
        "--config-path",
        active.to_str().expect("utf-8"),
    ]);
    assert_eq!(malformed.status.code(), Some(2));
    assert!(malformed.stdout.is_empty());
    assert!(!malformed.stderr.is_empty());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::write(&source, "BIJUXCLI_GAMMA=3\n").expect("seed unreadable");
        fs::set_permissions(&source, fs::Permissions::from_mode(0o000)).expect("chmod");
        let unreadable = run(&[
            "cli",
            "config",
            "load",
            source.to_str().expect("utf-8"),
            "--config-path",
            active.to_str().expect("utf-8"),
        ]);
        assert_eq!(unreadable.status.code(), Some(2));
        assert!(unreadable.stdout.is_empty());
        assert!(!unreadable.stderr.is_empty());
        fs::set_permissions(&source, fs::Permissions::from_mode(0o644)).expect("restore");
    }
}

#[test]
fn config_load_missing_file_and_path_traversal_style_path_handling() {
    let temp = make_temp_dir("load-paths");
    let active = temp.join("active.env");
    fs::write(&active, "BIJUXCLI_ACTIVE=1\n").expect("seed active");

    let nested = temp.join("nested");
    fs::create_dir_all(&nested).expect("mkdir");
    let source = temp.join("source.env");
    fs::write(&source, "BIJUXCLI_DELTA=4\n").expect("seed source");

    let traversal_arg = PathBuf::from("..").join("source.env");
    let traversal = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args([
            "cli",
            "config",
            "load",
            traversal_arg.to_str().expect("utf-8"),
            "--config-path",
            active.to_str().expect("utf-8"),
        ])
        .current_dir(&nested)
        .output()
        .expect("run traversal");
    assert_eq!(traversal.status.code(), Some(0));
    let loaded = fs::read_to_string(&active).expect("active after traversal load");
    assert_eq!(loaded, "BIJUXCLI_DELTA=4\n");

    let missing = run(&[
        "cli",
        "config",
        "load",
        temp.join("missing.env").to_str().expect("utf-8"),
        "--config-path",
        active.to_str().expect("utf-8"),
    ]);
    assert_eq!(missing.status.code(), Some(0));
    assert!(missing.stderr.is_empty());
}

#[test]
fn config_export_and_load_python_parity_on_exit_and_streams() {
    let temp = make_temp_dir("python-parity");
    let active = temp.join("active.env");
    let source = temp.join("source.env");
    let export_file = temp.join("export.env");
    fs::write(&active, "BIJUXCLI_ALPHA=1\n").expect("seed active");
    fs::write(&source, "BIJUXCLI_BETA=2\n").expect("seed source");

    let mut envs = HashMap::new();
    envs.insert("BIJUXCLI_CONFIG".to_string(), active.display().to_string());
    envs.insert("HOME".to_string(), temp.display().to_string());
    envs.insert("NO_COLOR".to_string(), "1".to_string());

    let py_export = run_python(
        &[
            "config",
            "export",
            export_file.to_str().expect("utf-8"),
            "--format",
            "json",
            "--no-pretty",
        ],
        &envs,
    );
    let rs_export = run_with_env(
        &[
            "cli",
            "config",
            "export",
            export_file.to_str().expect("utf-8"),
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            active.to_str().expect("utf-8"),
        ],
        &[
            ("BIJUXCLI_CONFIG", active.display().to_string()),
            ("HOME", temp.display().to_string()),
            ("NO_COLOR", "1".to_string()),
        ],
    );

    assert_eq!(py_export.status.code(), rs_export.status.code());
    assert!(py_export.stderr.is_empty());
    assert!(rs_export.stderr.is_empty());

    let py_load = run_python(
        &["config", "load", source.to_str().expect("utf-8"), "--format", "json", "--no-pretty"],
        &envs,
    );
    let rs_load = run_with_env(
        &[
            "cli",
            "config",
            "load",
            source.to_str().expect("utf-8"),
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            active.to_str().expect("utf-8"),
        ],
        &[
            ("BIJUXCLI_CONFIG", active.display().to_string()),
            ("HOME", temp.display().to_string()),
            ("NO_COLOR", "1".to_string()),
        ],
    );

    assert_eq!(py_load.status.code(), rs_load.status.code());
    assert!(py_load.stderr.is_empty());
    assert!(rs_load.stderr.is_empty());
}
