#![forbid(unsafe_code)]
//! Config case replay suite from retained minimized config inputs.
//! test_type: config-case-replay

use std::fs;
use std::path::Path;
use std::process::Command;

use bijux_cli as _;
use libc as _;
use serde_json as _;
use shlex as _;
use thiserror as _;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux")).args(args).output().expect("execute")
}

#[test]
fn minimized_config_cases_replay_with_stable_exit_behavior() {
    let cases_dir = Path::new("tests/fuzz/minimized_cases/config_minimized_cases");
    let mut files: Vec<_> = fs::read_dir(cases_dir)
        .expect("minimized config cases directory must exist")
        .map(|entry| entry.expect("entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "env"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "minimized config cases must be retained");

    let scratch = std::env::temp_dir().join("bijux-config-case-replay.env");
    for case in files {
        let payload = fs::read(&case).expect("read case");
        fs::write(&scratch, payload).expect("write scratch");

        let a = run(&["cli", "config", "list", "--config-path", scratch.to_str().expect("utf-8")]);
        let b = run(&["cli", "config", "list", "--config-path", scratch.to_str().expect("utf-8")]);
        assert_eq!(a.status.code(), b.status.code(), "case={}", case.display());
        assert_eq!(a.stdout, b.stdout, "list stdout drift for case={}", case.display());
        assert_eq!(a.stderr, b.stderr, "list stderr drift for case={}", case.display());
        if a.status.success() {
            assert!(
                a.stderr.is_empty(),
                "list success should not write stderr for case={}",
                case.display()
            );
            assert!(
                !a.stdout.is_empty(),
                "list success should emit stdout for case={}",
                case.display()
            );
        } else {
            assert!(
                a.stdout.is_empty(),
                "list failure should not write stdout for case={}",
                case.display()
            );
            assert!(
                !a.stderr.is_empty(),
                "list failure should write stderr for case={}",
                case.display()
            );
        }

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
        assert_eq!(load_a.stdout, load_b.stdout, "load stdout drift for case={}", case.display());
        assert_eq!(load_a.stderr, load_b.stderr, "load stderr drift for case={}", case.display());
        if load_a.status.success() {
            assert!(
                load_a.stderr.is_empty(),
                "load success should not write stderr for case={}",
                case.display()
            );
            assert!(
                !load_a.stdout.is_empty(),
                "load success should emit stdout for case={}",
                case.display()
            );
        } else {
            assert!(
                load_a.stdout.is_empty(),
                "load failure should not write stdout for case={}",
                case.display()
            );
            assert!(
                !load_a.stderr.is_empty(),
                "load failure should write stderr for case={}",
                case.display()
            );
        }
    }
}
