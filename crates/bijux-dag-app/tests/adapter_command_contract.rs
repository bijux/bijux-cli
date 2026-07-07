use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tar as _;
use tempfile as _;
use thiserror as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root")
}

fn dag_command(root: &Path) -> Command {
    let cargo_bin = std::env::var("CARGO")
        .ok()
        .or_else(|| option_env!("CARGO").map(ToOwned::to_owned))
        .unwrap_or_else(|| "cargo".to_string());
    let mut command = Command::new(cargo_bin);
    command.current_dir(root).env("CARGO_TARGET_DIR", root.join("artifacts/target")).args([
        "run",
        "--quiet",
        "-p",
        "bijux-dag-cli",
        "--",
    ]);
    command
}

fn run_json(root: &Path, args: &[&str]) -> (i32, serde_json::Value, String) {
    let output = dag_command(root).args(args).output().expect("run dag command");
    let code = output.status.code().unwrap_or(1);
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let payload = serde_json::from_slice(&output.stdout).expect("parse json envelope");
    (code, payload, stderr)
}

#[test]
#[ignore = "experimental"]
fn adapters_describe_json_contains_descriptor_fields() {
    let root = repo_root();
    let (code, payload, stderr) = run_json(&root, &["--json", "adapters", "describe"]);
    assert_eq!(code, 0, "command failed: {stderr}");
    let descriptors = payload["data"]["descriptors"].as_array().expect("descriptors");
    assert!(descriptors.iter().any(|descriptor| {
        descriptor["id"] == "shell"
            && descriptor.get("protocol_version").is_some()
            && descriptor.get("cache_compatibility").is_some()
            && descriptor.get("supports_timeout").is_some()
            && descriptor.get("supports_cancel").is_some()
    }));
    assert!(descriptors.iter().any(|descriptor| {
        descriptor["id"] == "http"
            && descriptor["supported_kinds"]
                .as_array()
                .is_some_and(|kinds| kinds.iter().any(|kind| kind == "http"))
            && descriptor.get("supports_timeout").is_some()
    }));
    assert!(descriptors.iter().any(|descriptor| {
        descriptor["id"] == "python"
            && descriptor["supported_kinds"]
                .as_array()
                .is_some_and(|kinds| kinds.iter().any(|kind| kind == "python"))
            && descriptor.get("supports_timeout").is_some()
    }));
    assert!(descriptors.iter().any(|descriptor| {
        descriptor["id"] == "file_transform"
            && descriptor["supported_kinds"]
                .as_array()
                .is_some_and(|kinds| kinds.iter().any(|kind| kind == "file_transform"))
            && descriptor.get("supports_timeout").is_some()
    }));
}

#[test]
#[ignore = "experimental"]
fn adapters_admit_json_reports_unsupported_nodes() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmpdir");
    let dag = tmp.path().join("graph.json");
    fs::write(
        &dag,
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[{"id":"x","kind":"missing.kind","outputs":[{"name":"out","path":"out"}],"params":{}}],
          "edges":[]
        }"#,
    )
    .expect("write graph");
    let (code, payload, _) =
        run_json(&root, &["--json", "adapters", "admit", dag.to_string_lossy().as_ref()]);
    assert_eq!(code, 3);
    assert_eq!(payload["data"]["supported"], false);
    assert_eq!(payload["data"]["entries"][0]["node_id"], "x");
}

#[test]
#[ignore = "experimental"]
fn adapters_doctor_reports_external_handshake_rejections_with_reasons() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmpdir");
    let adapters = tmp.path().join("adapters");
    fs::create_dir_all(&adapters).expect("mkdir");
    let script = adapters.join("bad-adapter");
    fs::write(
        &script,
        "#!/bin/sh\nif [ \"$1\" = \"info\" ]; then echo '{\"adapter_id\":\"bad\"}'; exit 0; fi\nexit 1\n",
    )
    .expect("write");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script).expect("meta").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script, perms).expect("chmod");
    }
    let output = dag_command(&root)
        .env("BIJUX_DAG_ADAPTERS_DIR", &adapters)
        .args(["--json", "adapters", "doctor"])
        .output()
        .expect("run dag command");
    assert_eq!(output.status.code().unwrap_or(1), 3);
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    let reports = payload["data"]["external_handshakes"].as_array().expect("reports");
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0]["status"], "rejected");
    assert!(reports[0]["reason"].as_str().unwrap_or_default().contains("invalid adapter manifest"));
}

#[test]
#[ignore = "experimental"]
fn adapters_conformance_json_reports_scenario_matrix() {
    let root = repo_root();
    let (code, payload, stderr) = run_json(&root, &["--json", "adapters", "conformance"]);
    assert_eq!(code, 0, "command failed: {stderr}");
    let suites = payload["data"]["suites"].as_array().expect("suites");
    let file_transform = suites
        .iter()
        .find(|suite| suite["adapter_id"] == "file_transform")
        .expect("file_transform suite");
    let http = suites.iter().find(|suite| suite["adapter_id"] == "http").expect("http suite");
    let shell = suites.iter().find(|suite| suite["adapter_id"] == "shell").expect("shell suite");
    let python = suites.iter().find(|suite| suite["adapter_id"] == "python").expect("python suite");
    let file_transform_scenarios =
        file_transform["scenarios"].as_array().expect("file_transform scenarios");
    let scenarios = shell["scenarios"].as_array().expect("shell scenarios");
    assert!(file_transform_scenarios
        .iter()
        .any(|scenario| scenario["scenario"] == "success" && scenario["status"] == "pass"));
    assert!(file_transform_scenarios
        .iter()
        .any(|scenario| scenario["scenario"] == "failure" && scenario["status"] == "pass"));
    assert!(file_transform_scenarios
        .iter()
        .any(|scenario| scenario["scenario"] == "cache_output" && scenario["status"] == "pass"));
    assert!(file_transform_scenarios.iter().any(|scenario| scenario["scenario"]
        == "missing_executable"
        && scenario["status"] == "skip"));
    assert!(scenarios.iter().any(|scenario| scenario["scenario"] == "timeout"));
    assert!(scenarios.iter().any(|scenario| scenario["scenario"] == "cache_output"));
    let http_scenarios = http["scenarios"].as_array().expect("http scenarios");
    assert!(http_scenarios
        .iter()
        .any(|scenario| scenario["scenario"] == "failure" && scenario["status"] == "pass"));
    assert!(http_scenarios
        .iter()
        .any(|scenario| scenario["scenario"] == "timeout" && scenario["status"] == "pass"));
    assert!(http_scenarios.iter().any(
        |scenario| scenario["scenario"] == "missing_executable" && scenario["status"] == "skip"
    ));
    let python_scenarios = python["scenarios"].as_array().expect("python scenarios");
    assert!(python_scenarios.iter().any(|scenario| scenario["scenario"] == "timeout"));
    assert!(python_scenarios.iter().any(
        |scenario| scenario["scenario"] == "workdir_isolation" && scenario["status"] == "pass"
    ));
    assert!(python_scenarios.iter().any(
        |scenario| scenario["scenario"] == "missing_executable" && scenario["status"] == "pass"
    ));
}

#[test]
#[ignore = "experimental"]
fn adapters_cache_compat_json_rejects_schema_drift() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmpdir");
    let meta = tmp.path().join("meta.json");
    fs::write(
        &meta,
        r#"{
          "adapter_id":"shell",
          "adapter_version":"0.1",
          "produces_outputs_schema_version":"schema/v1"
        }"#,
    )
    .expect("write");
    let (code, payload, stderr) = run_json(
        &root,
        &[
            "--json",
            "adapters",
            "cache-compat",
            meta.to_string_lossy().as_ref(),
            "--expected-schema",
            "schema/v2",
        ],
    );
    assert_eq!(code, 3, "unexpected command status: {stderr}");
    assert_eq!(payload["data"]["compatible"], false);
    assert!(payload["data"]["reason"].as_str().unwrap_or_default().contains("fingerprint-exact"));
}

#[test]
#[ignore = "experimental"]
fn adapters_reference_prints_generated_markdown_contract() {
    let root = repo_root();
    let output =
        dag_command(&root).args(["adapters", "reference"]).output().expect("run dag command");
    assert_eq!(output.status.code().unwrap_or(1), 0);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("# Adapter Contract"));
    assert!(stdout.contains("## Registered adapters"));
    assert!(stdout.contains("## Fake batch executor"));
}
