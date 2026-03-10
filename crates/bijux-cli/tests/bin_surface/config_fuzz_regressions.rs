#![forbid(unsafe_code)]
//! Config fuzz regression suite from minimized crash-style cases.
//! test_type: config-fuzz-regression

use std::fs;
use std::path::Path;
use std::process::Command;

use bijux_cli as _;
use bijux_cli_python as _;
use libc as _;
use serde_json as _;
use shlex as _;
use thiserror as _;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("execute")
}

#[test]
fn minimized_config_cases_replay_with_stable_exit_behavior() {
    let cases_dir = Path::new("tests/fuzz/config_minimized_cases");
    let mut files: Vec<_> = fs::read_dir(cases_dir)
        .expect("minimized config cases directory must exist")
        .map(|entry| entry.expect("entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "env"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "minimized config cases must be retained");

    let scratch = std::env::temp_dir().join("bijux-config-fuzz-replay.env");
    for case in files {
        let payload = fs::read(&case).expect("read case");
        fs::write(&scratch, payload).expect("write scratch");

        let a = run(&["cli", "config", "list", "--config-path", scratch.to_str().expect("utf-8")]);
        let b = run(&["cli", "config", "list", "--config-path", scratch.to_str().expect("utf-8")]);
        assert_eq!(a.status.code(), b.status.code(), "case={}", case.display());

        let load_a = run(&[
            "cli",
            "config",
            "load",
            scratch.to_str().expect("utf-8"),
            "--config-path",
            scratch.to_str().expect("utf-8"),
        ]);
        let load_b = run(&[
            "cli",
            "config",
            "load",
            scratch.to_str().expect("utf-8"),
            "--config-path",
            scratch.to_str().expect("utf-8"),
        ]);
        assert_eq!(load_a.status.code(), load_b.status.code(), "case={}", case.display());
    }
}
