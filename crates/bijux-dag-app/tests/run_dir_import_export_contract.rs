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
use tar as _;
use tempfile as _;
use thiserror as _;

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn output_path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn run_dag(args: &[&str], cwd: &Path) -> (i32, String, String) {
    let output = Command::new("cargo")
        .current_dir(cwd)
        .env("CARGO_TARGET_DIR", cwd.join("artifacts/target"))
        .args(["run", "-p", "bijux-dag-cli", "--", "dag"])
        .args(args)
        .output()
        .expect("run dag command");
    (
        output.status.code().unwrap_or(1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn run_json(args: &[&str], cwd: &Path) -> Value {
    let (code, stdout, stderr) = run_dag(args, cwd);
    assert_eq!(code, 0, "command failed: {stderr}");
    serde_json::from_str(&stdout).expect("parse json envelope")
}

fn extract_run_dir(payload: &Value) -> PathBuf {
    PathBuf::from(payload["data"]["run_dir"].as_str().expect("run_dir string"))
}

#[test]
fn strict_verify_rejects_missing_required_artifacts() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let run_dir = temp.path().join("broken-run");
    fs::create_dir_all(run_dir.join("outputs")).expect("create outputs");
    fs::write(
        run_dir.join("manifest.json"),
        r#"{"manifest_version":"run-manifest/v0.1"}"#,
    )
    .expect("write manifest");
    fs::write(run_dir.join("outputs").join("index.json"), "{}").expect("write outputs index");

    let (code, _stdout, _stderr) = run_dag(
        &[
            "verify",
            "--json",
            &output_path_string(&run_dir),
            "--strict",
        ],
        &root,
    );
    assert_ne!(
        code, 0,
        "strict verify must fail on missing required artifacts"
    );
}

#[test]
fn standard_verify_tolerates_missing_optional_provenance_file() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");

    let run = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
        ],
        &root,
    );
    let run_dir = extract_run_dir(&run);
    let optional_provenance = run_dir.join("provenance.json");
    if optional_provenance.exists() {
        fs::remove_file(&optional_provenance).expect("remove optional provenance");
    }

    let (code, _stdout, _stderr) =
        run_dag(&["verify", "--json", &output_path_string(&run_dir)], &root);
    assert_eq!(
        code, 0,
        "standard verify should tolerate missing optional artifacts"
    );
}

#[test]
fn export_modes_emit_documented_payload_shapes() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let run = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
        ],
        &root,
    );
    let run_dir = extract_run_dir(&run);

    let manifest_only_bundle = temp.path().join("bundle-manifest-only.json");
    let _ = run_json(
        &[
            "export",
            "--json",
            &output_path_string(&run_dir),
            "--out",
            &output_path_string(&manifest_only_bundle),
            "--manifest-only",
        ],
        &root,
    );
    let manifest_only: Value = serde_json::from_str(
        &fs::read_to_string(&manifest_only_bundle).expect("read manifest-only bundle"),
    )
    .expect("parse manifest-only bundle");
    assert_eq!(manifest_only["export_mode"], "manifest-only");
    assert!(manifest_only["files"].is_null());

    let with_files_bundle = temp.path().join("bundle-with-files.json");
    let _ = run_json(
        &[
            "export",
            "--json",
            &output_path_string(&run_dir),
            "--out",
            &output_path_string(&with_files_bundle),
            "--with-files",
        ],
        &root,
    );
    let with_files: Value = serde_json::from_str(
        &fs::read_to_string(&with_files_bundle).expect("read with-files bundle"),
    )
    .expect("parse with-files bundle");
    assert_eq!(with_files["export_mode"], "with-files");
    assert!(with_files["files"].is_object());

    let imported = run_json(
        &["import", "--json", &output_path_string(&with_files_bundle)],
        &root,
    );
    assert_eq!(imported["data"]["provenance_source"], "native-run");
}

#[test]
fn export_without_artifacts_and_import_verify_only_roundtrip_contract() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");

    let run = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
        ],
        &root,
    );
    let run_dir = extract_run_dir(&run);
    let bundle = temp.path().join("bundle-without-artifacts.json");

    let _ = run_json(
        &[
            "export",
            "--json",
            "--from-run",
            &output_path_string(&run_dir),
            "--out",
            &output_path_string(&bundle),
            "--without-artifacts",
        ],
        &root,
    );

    let exported: Value =
        serde_json::from_str(&fs::read_to_string(&bundle).expect("read exported bundle"))
            .expect("parse exported bundle");
    assert_eq!(exported["export_mode"], "without-artifacts");
    assert_eq!(exported["outputs"], serde_json::json!({}));
    assert!(exported["files"].is_null());

    let imported = run_json(
        &[
            "import",
            "--json",
            "--verify-only",
            &output_path_string(&bundle),
        ],
        &root,
    );
    assert_eq!(imported["ok"], true);
    assert_eq!(imported["data"]["verify_only"], true);
}

#[test]
fn graph_snapshot_only_bundle_roundtrip_is_stable() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let run = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
        ],
        &root,
    );
    let run_dir = extract_run_dir(&run);
    let bundle = temp.path().join("bundle-graph-only-roundtrip.json");

    let _ = run_json(
        &[
            "export",
            "--json",
            &output_path_string(&run_dir),
            "--out",
            &output_path_string(&bundle),
            "--manifest-only",
        ],
        &root,
    );
    let imported = run_json(&["import", "--json", &output_path_string(&bundle)], &root);
    assert_eq!(imported["ok"], true);
    assert_eq!(imported["data"]["export_mode"], "manifest-only");
}

#[test]
fn import_rejects_unsupported_bundle_version_fixture() {
    let root = repo_root();
    let unsupported = root.join("evidence/compat/export_bundle/unsupported_past/bundle.json");
    let (code, stdout, _stderr) = run_dag(
        &["import", "--json", &output_path_string(&unsupported)],
        &root,
    );
    assert_ne!(code, 0, "unsupported bundle version must fail");
    let payload: Value = serde_json::from_str(&stdout).expect("parse import failure payload");
    let message = payload["errors"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|entry| entry.get("message"))
        .and_then(Value::as_str)
        .or_else(|| payload["error"]["message"].as_str())
        .expect("error message");
    assert!(!message.trim().is_empty());
}

#[test]
fn import_rejects_truncated_bundle_with_clear_failure() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let bundle = temp.path().join("truncated-bundle.json");
    fs::write(&bundle, "{").expect("write truncated bundle");

    let (code, _stdout, _stderr) =
        run_dag(&["import", "--json", &output_path_string(&bundle)], &root);
    assert_ne!(code, 0, "truncated bundle must fail import");
}

#[test]
fn kubernetes_origin_bundle_export_contract_preserves_import_summary_provenance() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let run = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
        ],
        &root,
    );
    let run_dir = extract_run_dir(&run);
    let bundle_path = temp.path().join("bundle-k8s-origin.json");

    let _ = run_json(
        &[
            "export",
            "--json",
            &output_path_string(&run_dir),
            "--out",
            &output_path_string(&bundle_path),
            "--with-files",
        ],
        &root,
    );

    let mut bundle: Value =
        serde_json::from_str(&fs::read_to_string(&bundle_path).expect("read bundle"))
            .expect("parse bundle");
    bundle["provenance"]["source"] = serde_json::json!("kubernetes-run");
    bundle["provenance"]["imported"] = serde_json::json!(true);
    fs::write(
        &bundle_path,
        serde_json::to_vec_pretty(&bundle).expect("encode bundle"),
    )
    .expect("rewrite bundle");

    let imported = run_json(
        &["import", "--json", &output_path_string(&bundle_path)],
        &root,
    );
    assert_eq!(imported["data"]["provenance_source"], "kubernetes-run");
}

#[test]
fn kubernetes_replay_from_import_conformance_simulation_is_stable() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let run = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
        ],
        &root,
    );
    let run_dir = extract_run_dir(&run);

    let bundle_path = temp.path().join("bundle-k8s-replay.json");
    let _ = run_json(
        &[
            "export",
            "--json",
            &output_path_string(&run_dir),
            "--out",
            &output_path_string(&bundle_path),
            "--with-files",
        ],
        &root,
    );
    let mut bundle: Value =
        serde_json::from_str(&fs::read_to_string(&bundle_path).expect("read bundle"))
            .expect("parse bundle");
    bundle["provenance"]["source"] = serde_json::json!("kubernetes-run");
    bundle["provenance"]["imported"] = serde_json::json!(true);
    fs::write(
        &bundle_path,
        serde_json::to_vec_pretty(&bundle).expect("encode bundle"),
    )
    .expect("rewrite bundle");

    let import_payload = run_json(
        &["import", "--json", &output_path_string(&bundle_path)],
        &root,
    );
    assert_eq!(
        import_payload["data"]["provenance_source"],
        "kubernetes-run"
    );

    let replay_out = temp.path().join("replay-runs");
    fs::create_dir_all(&replay_out).expect("create replay out");
    let replay_payload = run_json(
        &[
            "replay",
            "--json",
            &output_path_string(&run_dir),
            "--out",
            &output_path_string(&replay_out),
        ],
        &root,
    );
    let replay_run_dir = extract_run_dir(&replay_payload);

    let diff_payload = run_json(
        &[
            "diff",
            "--json",
            &output_path_string(&run_dir),
            &output_path_string(&replay_run_dir),
        ],
        &root,
    );
    assert_eq!(diff_payload["ok"], true);
}

#[test]
fn hpc_origin_bundle_replay_from_import_conformance_is_stable() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let run = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
        ],
        &root,
    );
    let run_dir = extract_run_dir(&run);

    let bundle_path = temp.path().join("bundle-hpc-replay.json");
    let _ = run_json(
        &[
            "export",
            "--json",
            &output_path_string(&run_dir),
            "--out",
            &output_path_string(&bundle_path),
            "--with-files",
        ],
        &root,
    );
    let mut bundle: Value =
        serde_json::from_str(&fs::read_to_string(&bundle_path).expect("read bundle"))
            .expect("parse bundle");
    bundle["provenance"]["source"] = serde_json::json!("hpc-run");
    bundle["provenance"]["imported"] = serde_json::json!(true);
    fs::write(
        &bundle_path,
        serde_json::to_vec_pretty(&bundle).expect("encode bundle"),
    )
    .expect("rewrite bundle");

    let import_payload = run_json(
        &["import", "--json", &output_path_string(&bundle_path)],
        &root,
    );
    assert_eq!(import_payload["data"]["provenance_source"], "hpc-run");

    let replay_out = temp.path().join("replay-runs");
    fs::create_dir_all(&replay_out).expect("create replay out");
    let replay_payload = run_json(
        &[
            "replay",
            "--json",
            &output_path_string(&run_dir),
            "--out",
            &output_path_string(&replay_out),
        ],
        &root,
    );
    let replay_run_dir = extract_run_dir(&replay_payload);
    let diff_payload = run_json(
        &[
            "diff",
            "--json",
            &output_path_string(&run_dir),
            &output_path_string(&replay_run_dir),
        ],
        &root,
    );
    assert_eq!(diff_payload["ok"], true);
}

#[test]
fn hpc_origin_bundle_supports_offline_import_inspection_without_repo_checkout_data() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let bundle = temp.path().join("hpc-offline.json");
    fs::write(
        &bundle,
        serde_json::to_vec_pretty(&serde_json::json!({
            "bundle_version": "export-bundle/v0.1",
            "export_mode": "manifest-only",
            "provenance": {"source": "hpc-run", "imported": true},
            "manifest": {"manifest_version": "run-manifest/v0.1"},
            "graph_snapshot": {"spec":"bijux-dag/v0.1","nodes":[],"edges":[]},
            "node_traces": {},
            "outputs": {}
        }))
        .expect("encode offline bundle"),
    )
    .expect("write offline bundle");

    let imported = run_json(&["import", "--json", &output_path_string(&bundle)], &root);
    assert_eq!(imported["ok"], true);
    assert_eq!(imported["data"]["provenance_source"], "hpc-run");
}

#[test]
fn remote_origin_bundle_replay_from_import_conformance_is_stable() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let run = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
        ],
        &root,
    );
    let run_dir = extract_run_dir(&run);

    let bundle_path = temp.path().join("bundle-remote-replay.json");
    let _ = run_json(
        &[
            "export",
            "--json",
            &output_path_string(&run_dir),
            "--out",
            &output_path_string(&bundle_path),
            "--with-files",
        ],
        &root,
    );
    let mut bundle: Value =
        serde_json::from_str(&fs::read_to_string(&bundle_path).expect("read bundle"))
            .expect("parse bundle");
    bundle["provenance"]["source"] = serde_json::json!("remote-run");
    bundle["provenance"]["imported"] = serde_json::json!(true);
    fs::write(
        &bundle_path,
        serde_json::to_vec_pretty(&bundle).expect("encode bundle"),
    )
    .expect("rewrite bundle");

    let import_payload = run_json(
        &["import", "--json", &output_path_string(&bundle_path)],
        &root,
    );
    assert_eq!(import_payload["data"]["provenance_source"], "remote-run");

    let replay_out = temp.path().join("replay-runs");
    fs::create_dir_all(&replay_out).expect("create replay out");
    let replay_payload = run_json(
        &[
            "replay",
            "--json",
            &output_path_string(&run_dir),
            "--out",
            &output_path_string(&replay_out),
        ],
        &root,
    );
    let replay_run_dir = extract_run_dir(&replay_payload);
    let diff_payload = run_json(
        &[
            "diff",
            "--json",
            &output_path_string(&run_dir),
            &output_path_string(&replay_run_dir),
        ],
        &root,
    );
    assert_eq!(diff_payload["ok"], true);
}
