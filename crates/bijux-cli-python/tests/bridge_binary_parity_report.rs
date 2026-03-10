#![forbid(unsafe_code)]
//! Generate and enforce parity artifact for binary vs python-bridge execution.

use bijux_cli_core as _;
use bijux_cli_routing as _;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use bijux_cli_python::execution_outcome_api;
use serde_json::{json, Value};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates dir")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn run_binary(argv: &[&str]) -> (i32, String, String) {
    let mut cmd = Command::new(resolve_bijux_binary());
    cmd.current_dir(repo_root()).args(argv);
    let out = cmd.output().expect("run binary through cargo");
    let code = out.status.code().unwrap_or(1);
    (
        code,
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn resolve_bijux_binary() -> PathBuf {
    let root = repo_root();
    let target_root = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("artifacts").join("rust").join("target"));
    let bin_path =
        target_root.join("debug").join(format!("bijux-rs{}", std::env::consts::EXE_SUFFIX));
    if bin_path.exists() {
        return bin_path;
    }

    let status = Command::new("cargo")
        .current_dir(&root)
        .args(["build", "-q", "-p", "bijux-cli-core"])
        .status()
        .expect("build bijux-rs binary");
    assert!(status.success(), "failed to build bijux-rs binary");
    bin_path
}

fn run_bridge(argv: &[&str]) -> (i32, String, String) {
    let as_vec: Vec<String> =
        std::iter::once("bijux").chain(argv.iter().copied()).map(ToString::to_string).collect();
    let raw = execution_outcome_api(&as_vec).expect("bridge execution outcome");
    let payload: Value = serde_json::from_str(&raw).expect("valid bridge payload");
    (
        payload["exit_code"].as_i64().unwrap_or(1) as i32,
        payload["stdout"].as_str().unwrap_or_default().to_string(),
        payload["stderr"].as_str().unwrap_or_default().to_string(),
    )
}

#[test]
fn binary_and_python_bridge_parity_report_is_generated() {
    let cases: Vec<Vec<&str>> = vec![
        vec!["status"],
        vec!["doctor"],
        vec!["cli", "plugins", "list"],
        vec!["dev", "cli", "runtime-identity"],
    ];

    let mut report_rows: Vec<Value> = Vec::new();

    for argv in cases {
        let command = argv.join(" ");
        let (bin_code, bin_out, bin_err) = run_binary(&argv);
        let (bridge_code, bridge_out, bridge_err) = run_bridge(&argv);

        report_rows.push(json!({
            "command": command,
            "exit_match": bin_code == bridge_code,
            "stdout_match": bin_out == bridge_out,
            "stderr_match": bin_err == bridge_err,
        }));
    }

    let payload = json!({
        "generator": "crates/bijux-cli-python/tests/bridge_binary_parity_report.rs",
        "cases": report_rows,
    });

    let out_path = repo_root()
        .join("artifacts")
        .join("parity")
        .join("binary_vs_python_bridge_parity_report.json");
    out_path.parent().expect("parent").mkdir_p();
    fs::write(&out_path, serde_json::to_string_pretty(&payload).expect("serialize") + "\n")
        .expect("write report");

    for case in payload["cases"].as_array().expect("array") {
        assert!(
            case["exit_match"].as_bool().unwrap_or(false),
            "exit mismatch for {}",
            case["command"]
        );
        assert!(
            case["stdout_match"].as_bool().unwrap_or(false),
            "stdout mismatch for {}",
            case["command"]
        );
        assert!(
            case["stderr_match"].as_bool().unwrap_or(false),
            "stderr mismatch for {}",
            case["command"]
        );
    }
}

trait MkdirP {
    fn mkdir_p(&self);
}

impl MkdirP for &std::path::Path {
    fn mkdir_p(&self) {
        std::fs::create_dir_all(self).expect("create parent dirs");
    }
}
