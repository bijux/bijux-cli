use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
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

mod support;

fn repo_root() -> PathBuf {
    support::repo_root_from_manifest_dir(env!("CARGO_MANIFEST_DIR"))
}

fn output_path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn run_dag(args: &[&str], cwd: &Path) -> (i32, String, String) {
    support::run_dag_command(args, cwd)
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
    fs::write(run_dir.join("manifest.json"), r#"{"manifest_version":"run-manifest/v0.1"}"#)
        .expect("write manifest");
    fs::write(run_dir.join("outputs").join("index.json"), "{}").expect("write outputs index");

    let (code, _stdout, _stderr) =
        run_dag(&["verify", "--json", &output_path_string(&run_dir), "--strict"], &root);
    assert_ne!(code, 0, "strict verify must fail on missing required artifacts");
}

#[test]
fn standard_verify_tolerates_missing_optional_provenance_file() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");

    let run = run_json(
        &["run", "--json", &output_path_string(&graph), "--out", &output_path_string(&out_dir)],
        &root,
    );
    let run_dir = extract_run_dir(&run);
    let optional_provenance = run_dir.join("provenance.json");
    if optional_provenance.exists() {
        fs::remove_file(&optional_provenance).expect("remove optional provenance");
    }

    let (code, _stdout, _stderr) =
        run_dag(&["verify", "--json", &output_path_string(&run_dir)], &root);
    assert_eq!(code, 0, "standard verify should tolerate missing optional artifacts");
}

#[test]
fn strict_verify_reports_evidence_and_event_completeness() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");

    let run = run_json(
        &["run", "--json", &output_path_string(&graph), "--out", &output_path_string(&out_dir)],
        &root,
    );
    let run_dir = extract_run_dir(&run);
    let report = run_json(&["verify", "--json", &output_path_string(&run_dir), "--strict"], &root);
    assert_eq!(report["ok"], true);
    assert_eq!(report["data"]["artifacts_checked"]["schema_index"], true);
    assert_eq!(report["data"]["artifacts_checked"]["manifest_finalized"], true);
    assert_eq!(report["data"]["artifacts_checked"]["run_complete_marker"], true);
    assert_eq!(
        report["data"]["evidence_completeness"]["missing_root_files"],
        serde_json::json!([])
    );
    assert_eq!(report["data"]["event_log_completeness"]["complete"], true);
}

#[test]
fn verify_detects_post_finalize_output_mutation_and_redacted_export_stays_structural() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");

    let run = run_json(
        &["run", "--json", &output_path_string(&graph), "--out", &output_path_string(&out_dir)],
        &root,
    );
    let run_dir = extract_run_dir(&run);
    let outputs_index: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("outputs").join("index.json"))
            .expect("read outputs index"),
    )
    .expect("parse outputs index");
    let output_file = outputs_index["files"][0]["path"].as_str().expect("output path");
    fs::write(run_dir.join(output_file), b"mutated after finalize").expect("mutate output");
    let (verify_code, _stdout, _stderr) =
        run_dag(&["verify", "--json", &output_path_string(&run_dir), "--deep"], &root);
    assert_ne!(verify_code, 0, "deep verify must fail after output mutation");

    let clean_run = run_json(
        &["run", "--json", &output_path_string(&graph), "--out", &output_path_string(&out_dir)],
        &root,
    );
    let clean_run_dir = extract_run_dir(&clean_run);
    let bundle = temp.path().join("bundle-redacted.json");
    let _ = run_json(
        &[
            "export",
            "--json",
            &output_path_string(&clean_run_dir),
            "--out",
            &output_path_string(&bundle),
            "--with-files",
            "--redact",
        ],
        &root,
    );
    let exported: Value =
        serde_json::from_str(&fs::read_to_string(&bundle).expect("read redacted bundle"))
            .expect("parse redacted bundle");
    assert_eq!(exported["redaction"]["irreversible"], true);
    assert_eq!(exported["provenance"]["source_run_dir"], "[redacted]");
    let redacted_file = exported["files"]
        .as_object()
        .and_then(|files| files.values().next())
        .and_then(Value::as_object)
        .and_then(|node_files| node_files.values().next())
        .and_then(Value::as_str)
        .expect("redacted file payload");
    let decoded = BASE64.decode(redacted_file).expect("decode redacted payload");
    assert_eq!(decoded, b"[redacted]");

    let imported =
        run_json(&["import", "--json", "--verify-only", &output_path_string(&bundle)], &root);
    assert_eq!(imported["ok"], true);
}

#[test]
fn export_modes_emit_documented_payload_shapes() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let run = run_json(
        &["run", "--json", &output_path_string(&graph), "--out", &output_path_string(&out_dir)],
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

    let imported = run_json(&["import", "--json", &output_path_string(&with_files_bundle)], &root);
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
        &["run", "--json", &output_path_string(&graph), "--out", &output_path_string(&out_dir)],
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

    let imported =
        run_json(&["import", "--json", "--verify-only", &output_path_string(&bundle)], &root);
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
        &["run", "--json", &output_path_string(&graph), "--out", &output_path_string(&out_dir)],
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
    let (code, stdout, _stderr) =
        run_dag(&["import", "--json", &output_path_string(&unsupported)], &root);
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
        &["run", "--json", &output_path_string(&graph), "--out", &output_path_string(&out_dir)],
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
    fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).expect("encode bundle"))
        .expect("rewrite bundle");

    let imported = run_json(&["import", "--json", &output_path_string(&bundle_path)], &root);
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
        &["run", "--json", &output_path_string(&graph), "--out", &output_path_string(&out_dir)],
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
    fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).expect("encode bundle"))
        .expect("rewrite bundle");

    let import_payload = run_json(&["import", "--json", &output_path_string(&bundle_path)], &root);
    assert_eq!(import_payload["data"]["provenance_source"], "kubernetes-run");

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
        &["diff", "--json", &output_path_string(&run_dir), &output_path_string(&replay_run_dir)],
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
        &["run", "--json", &output_path_string(&graph), "--out", &output_path_string(&out_dir)],
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
    fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).expect("encode bundle"))
        .expect("rewrite bundle");

    let import_payload = run_json(&["import", "--json", &output_path_string(&bundle_path)], &root);
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
        &["diff", "--json", &output_path_string(&run_dir), &output_path_string(&replay_run_dir)],
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
        &["run", "--json", &output_path_string(&graph), "--out", &output_path_string(&out_dir)],
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
    fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).expect("encode bundle"))
        .expect("rewrite bundle");

    let import_payload = run_json(&["import", "--json", &output_path_string(&bundle_path)], &root);
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
        &["diff", "--json", &output_path_string(&run_dir), &output_path_string(&replay_run_dir)],
        &root,
    );
    assert_eq!(diff_payload["ok"], true);
}

#[test]
fn artifact_heavy_bundle_roundtrip_verify_only_is_stable() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let run = run_json(
        &["run", "--json", &output_path_string(&graph), "--out", &output_path_string(&out_dir)],
        &root,
    );
    let run_dir = extract_run_dir(&run);
    let bundle_path = temp.path().join("bundle-artifact-heavy.json");

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
    let payload = bundle["files"]
        .as_object()
        .and_then(|m| m.values().next())
        .and_then(Value::as_object)
        .and_then(|m| m.values().next())
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let files = bundle["files"].as_object_mut().expect("files map");
    let first_node = files.keys().next().cloned().expect("node files key");
    let node_files =
        files.get_mut(&first_node).and_then(Value::as_object_mut).expect("node file map");
    for i in 0..200 {
        node_files.insert(format!("synthetic/artifact-{i}.bin"), Value::String(payload.clone()));
    }
    fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).expect("encode bundle"))
        .expect("rewrite bundle");

    let imported =
        run_json(&["import", "--json", "--verify-only", &output_path_string(&bundle_path)], &root);
    assert_eq!(imported["ok"], true);
}

#[test]
fn imported_run_replay_and_diff_against_original_are_stable() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let run = run_json(
        &["run", "--json", &output_path_string(&graph), "--out", &output_path_string(&out_dir)],
        &root,
    );
    let run_dir = extract_run_dir(&run);
    let bundle_path = temp.path().join("bundle-replay-diff.json");
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
    let imported = run_json(&["import", "--json", &output_path_string(&bundle_path)], &root);
    assert_eq!(imported["ok"], true);

    let replay_out = temp.path().join("replay");
    fs::create_dir_all(&replay_out).expect("create replay out");
    let replay = run_json(
        &[
            "replay",
            "--json",
            &output_path_string(&run_dir),
            "--out",
            &output_path_string(&replay_out),
        ],
        &root,
    );
    let replay_dir = extract_run_dir(&replay);

    let diff = run_json(
        &["diff", "--json", &output_path_string(&run_dir), &output_path_string(&replay_dir)],
        &root,
    );
    assert_eq!(diff["ok"], true);
}

#[test]
fn import_tolerates_missing_optional_payloads_and_rejects_missing_required_payloads() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let optional_bundle = temp.path().join("optional-missing.json");
    fs::write(
        &optional_bundle,
        serde_json::to_vec_pretty(&serde_json::json!({
            "bundle_version": "export-bundle/v0.1",
            "export_mode": "manifest-only",
            "manifest": {"manifest_version": "run-manifest/v0.1"},
            "graph_snapshot": {"spec":"bijux-dag/v0.1","nodes":[],"edges":[]},
            "node_traces": {},
            "outputs": {}
        }))
        .expect("encode optional bundle"),
    )
    .expect("write optional bundle");
    let optional_imported =
        run_json(&["import", "--json", &output_path_string(&optional_bundle)], &root);
    assert_eq!(optional_imported["ok"], true);

    let required_bundle = temp.path().join("required-missing.json");
    fs::write(
        &required_bundle,
        serde_json::to_vec_pretty(&serde_json::json!({
            "bundle_version": "export-bundle/v0.1",
            "export_mode": "manifest-only",
            "graph_snapshot": {"spec":"bijux-dag/v0.1","nodes":[],"edges":[]},
            "node_traces": {},
            "outputs": {}
        }))
        .expect("encode required bundle"),
    )
    .expect("write required bundle");
    let (code, _stdout, _stderr) =
        run_dag(&["import", "--json", &output_path_string(&required_bundle)], &root);
    assert_ne!(code, 0, "import must fail when required payload is missing");
}

#[test]
fn import_rejects_corrupted_file_payload_before_acceptance() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let corrupted = temp.path().join("corrupted-file-payload.json");
    fs::write(
        &corrupted,
        serde_json::to_vec_pretty(&serde_json::json!({
            "bundle_version": "export-bundle/v0.1",
            "export_mode": "with-files",
            "manifest": {"manifest_version": "run-manifest/v0.1"},
            "graph_snapshot": {"spec":"bijux-dag/v0.1","nodes":[],"edges":[]},
            "node_traces": {},
            "outputs": {},
            "files": {"n1": {"out/file.txt": "%%%not-base64%%%"}}
        }))
        .expect("encode corrupted bundle"),
    )
    .expect("write corrupted bundle");

    let (code, _stdout, _stderr) =
        run_dag(&["import", "--json", &output_path_string(&corrupted)], &root);
    assert_ne!(code, 0, "corrupted file payload must be rejected");
}

#[test]
fn import_supports_offline_inspection_path_portability_and_line_endings() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let bundle = temp.path().join("portability.json");
    let raw = "{\n  \"bundle_version\": \"export-bundle/v0.1\",\n  \"export_mode\": \"manifest-only\",\n  \"provenance\": {\"source\": \"native-run\", \"source_run_dir\": \"C:\\\\work\\\\run-1\", \"source_run_id\": \"run-1\", \"parent_run_id\": \"run-0\", \"lineage\": [\"a\", \"b\"]},\n  \"manifest\": {\"manifest_version\": \"run-manifest/v0.1\"},\n  \"graph_snapshot\": {\"spec\":\"bijux-dag/v0.1\",\"nodes\":[],\"edges\":[]},\n  \"node_traces\": {},\n  \"outputs\": {}\n}\n";
    fs::write(&bundle, raw.replace('\n', "\r\n")).expect("write crlf bundle");

    let imported =
        run_json(&["import", "--json", "--verify-only", &output_path_string(&bundle)], &root);
    assert_eq!(imported["ok"], true);
    assert_eq!(imported["data"]["preservation"]["lineage"], true);
    assert_eq!(imported["data"]["preservation"]["run_ancestry"], true);
    assert_eq!(imported["data"]["preservation"]["graph_identity"], true);
    assert_eq!(imported["data"]["preservation"]["artifact_identity"], true);
}

#[test]
fn import_accepts_supported_older_bundle_fixture_and_export_handles_older_manifest() {
    let root = repo_root();
    let supported = root.join("evidence/compat/export_bundle/v0_1_supported/bundle.json");
    let imported =
        run_json(&["import", "--json", "--verify-only", &output_path_string(&supported)], &root);
    assert_eq!(imported["ok"], true);

    let temp = tempfile::tempdir().expect("tempdir");
    let run_dir = temp.path().join("legacy-run");
    fs::create_dir_all(run_dir.join("outputs")).expect("create outputs dir");
    fs::create_dir_all(run_dir.join("nodes").join("n1")).expect("create node dir");
    fs::write(
        run_dir.join("manifest.json"),
        fs::read_to_string(root.join("evidence/compat/run_dir/v0_1_supported/manifest.json"))
            .expect("read legacy manifest"),
    )
    .expect("write legacy manifest");
    fs::write(
        run_dir.join("graph.snapshot.json"),
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"n1","kind":"const","inputs":[],"outputs":[{"name":"value","path":"value.txt"}],"params":{"value":"x"}}],"edges":[]}"#,
    )
    .expect("write graph snapshot");
    fs::write(
        run_dir.join("nodes").join("n1").join("trace.json"),
        r#"{"node_id":"n1","status":"success"}"#,
    )
    .expect("write node trace");
    fs::write(
        run_dir.join("outputs").join("index.json"),
        r#"{"n1":{"node_id":"n1","files":[{"path":"value.txt","sha256":"abc","bytes":1}],"dirs":[]}}"#,
    )
    .expect("write outputs index");

    let bundle_out = temp.path().join("legacy-export.json");
    let exported = run_json(
        &[
            "export",
            "--json",
            &output_path_string(&run_dir),
            "--out",
            &output_path_string(&bundle_out),
            "--manifest-only",
        ],
        &root,
    );
    assert_eq!(exported["ok"], true);
}

#[test]
fn export_provenance_only_and_redacted_bundle_preserve_source_run_records() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let run = run_json(
        &["run", "--json", &output_path_string(&graph), "--out", &output_path_string(&out_dir)],
        &root,
    );
    let run_dir = extract_run_dir(&run);
    let before_manifest = fs::read(run_dir.join("manifest.json")).expect("read manifest before");

    let bundle = temp.path().join("bundle-provenance-redacted.json");
    let _ = run_json(
        &[
            "export",
            "--json",
            &output_path_string(&run_dir),
            "--out",
            &output_path_string(&bundle),
            "--provenance-only",
            "--redact",
        ],
        &root,
    );

    let exported: Value =
        serde_json::from_str(&fs::read_to_string(&bundle).expect("read bundle")).expect("parse");
    assert_eq!(exported["export_mode"], "provenance-only");
    assert_eq!(exported["node_traces"], serde_json::json!({}));
    assert_eq!(exported["outputs"], serde_json::json!({}));
    assert_eq!(exported["provenance"]["source_run_dir"], "[redacted]");

    let imported =
        run_json(&["import", "--json", "--verify-only", &output_path_string(&bundle)], &root);
    assert_eq!(imported["ok"], true);
    assert_eq!(imported["data"]["fidelity"]["level"], "graded");

    let after_manifest = fs::read(run_dir.join("manifest.json")).expect("read manifest after");
    assert_eq!(
        before_manifest, after_manifest,
        "redacted export must not mutate source run records"
    );
}
