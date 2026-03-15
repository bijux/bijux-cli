#![forbid(unsafe_code)]
//! Routed official product namespace delegation contracts.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use bijux_cli::contracts::known_bijux_tools;

fn run_with_env(args: &[&str], envs: &[(&str, &Path)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bijux"));
    command.args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("binary should execute")
}

fn temp_dir(name: &str) -> PathBuf {
    let root = env::temp_dir()
        .join(format!("bijux-official-product-routes-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir temp");
    root
}

#[cfg(unix)]
fn write_stub_binary(bin_dir: &Path, binary_name: &str) {
    use std::os::unix::fs::PermissionsExt;

    let script = format!(
        "#!/bin/sh\nprintf 'stub:{binary_name}\\n'\nprintf 'args:%s\\n' \"$*\"\n"
    );
    let path = bin_dir.join(binary_name);
    fs::write(&path, script).expect("write stub");
    let mut permissions = fs::metadata(&path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).expect("chmod");
}

#[cfg(windows)]
fn write_stub_binary(bin_dir: &Path, binary_name: &str) {
    let script = format!("@echo off\r\necho stub:{binary_name}\r\necho args:%*\r\n");
    fs::write(bin_dir.join(format!("{binary_name}.bat")), script).expect("write stub");
}

fn write_all_stubs(bin_dir: &Path) {
    for tool in known_bijux_tools() {
        write_stub_binary(bin_dir, tool.runtime_binary_name);
        write_stub_binary(bin_dir, tool.control_binary_name);
    }
}

#[test]
fn official_runtime_routes_delegate_to_runtime_binaries_for_every_reserved_namespace() {
    let root = temp_dir("runtime");
    let bin_dir = root.join("bin");
    fs::create_dir_all(&bin_dir).expect("mkdir bin");
    write_all_stubs(&bin_dir);

    for tool in known_bijux_tools() {
        let out = run_with_env(&[tool.namespace, "status"], &[("PATH", &bin_dir)]);
        assert_eq!(out.status.code(), Some(0), "runtime route should succeed for {}", tool.namespace);
        assert!(out.stderr.is_empty(), "runtime route should keep stderr empty for {}", tool.namespace);
        let stdout = String::from_utf8(out.stdout).expect("utf-8");
        assert!(stdout.contains(&format!("stub:{}", tool.runtime_binary_name)));
        assert!(stdout.contains("args:status"));
    }
}

#[test]
fn official_control_routes_delegate_to_control_binaries_for_every_reserved_namespace() {
    let root = temp_dir("control");
    let bin_dir = root.join("bin");
    fs::create_dir_all(&bin_dir).expect("mkdir bin");
    write_all_stubs(&bin_dir);

    for tool in known_bijux_tools() {
        let out = run_with_env(&["dev", tool.namespace, "status"], &[("PATH", &bin_dir)]);
        assert_eq!(out.status.code(), Some(0), "control route should succeed for {}", tool.namespace);
        assert!(out.stderr.is_empty(), "control route should keep stderr empty for {}", tool.namespace);
        let stdout = String::from_utf8(out.stdout).expect("utf-8");
        assert!(stdout.contains(&format!("stub:{}", tool.control_binary_name)));
        assert!(stdout.contains("args:status"));
    }
}

#[test]
fn help_routes_delegate_for_runtime_and_control_product_surfaces() {
    let root = temp_dir("help");
    let bin_dir = root.join("bin");
    fs::create_dir_all(&bin_dir).expect("mkdir bin");
    write_all_stubs(&bin_dir);

    let runtime = run_with_env(&["help", "atlas"], &[("PATH", &bin_dir)]);
    assert_eq!(runtime.status.code(), Some(0));
    let runtime_stdout = String::from_utf8(runtime.stdout).expect("utf-8");
    assert!(runtime_stdout.contains("stub:bijux-atlas"));
    assert!(runtime_stdout.contains("args:--help"));

    let control = run_with_env(&["help", "dev", "atlas"], &[("PATH", &bin_dir)]);
    assert_eq!(control.status.code(), Some(0));
    let control_stdout = String::from_utf8(control.stdout).expect("utf-8");
    assert!(control_stdout.contains("stub:bijux-dev-atlas"));
    assert!(control_stdout.contains("args:--help"));
}
