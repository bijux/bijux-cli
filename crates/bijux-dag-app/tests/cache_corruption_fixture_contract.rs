use bijux_dag_artifacts::{hash::sha256_hex, OutputFile, OutputsIndex};
use bijux_dag_runtime::{cache_key_explanation, CacheKeyInput, CACHE_METADATA_VERSION};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn dag_bin(cwd: &Path) -> Command {
    let cargo_bin = std::env::var("CARGO")
        .ok()
        .or_else(|| option_env!("CARGO").map(ToOwned::to_owned))
        .unwrap_or_else(|| "cargo".to_string());
    let mut command = Command::new(cargo_bin);
    command
        .current_dir(cwd)
        .env("CARGO_TARGET_DIR", cwd.join("artifacts/target"))
        .env("CARGO_TERM_COLOR", "never")
        .env("CARGO_TERM_PROGRESS_WHEN", "never")
        .args(["run", "--quiet", "-p", "bijux-dag-cli", "--"]);
    command
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root")
}

fn run_command(args: &[&str], cwd: &Path) -> (i32, String, String) {
    let output = dag_bin(cwd).args(args).output().expect("run dag command");
    let code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (code, stdout, stderr)
}

fn parse_json(stdout: &str, code: i32, stderr: &str) -> Value {
    serde_json::from_str(stdout)
        .unwrap_or_else(|error| panic!("parse json output: {error}; code={code}; stderr={stderr}"))
}

fn default_meta(label: &str) -> (String, Value) {
    let key_input = CacheKeyInput {
        execution_fingerprint: format!("exec-{label}"),
        node_definition_fingerprint: format!("node-{label}"),
        declared_environment_fingerprint: format!("env-{label}"),
        input_lineage_fingerprint: format!("inputs-{label}"),
        adapter_id: "shell".to_string(),
        adapter_version: "0.1".to_string(),
        adapter_binary_sha256: None,
        output_schema_version: "v0.1".to_string(),
        policy_fingerprint: "policy-fixed".to_string(),
        execution_contract_fingerprint: "exec-contract-fixed".to_string(),
        backend_class: "local".to_string(),
    };
    let key = cache_key_explanation(&key_input).key;
    (
        key.clone(),
        json!({
            "cache_metadata_version": CACHE_METADATA_VERSION,
            "cache_key": key,
            "node_fingerprint": key_input.execution_fingerprint,
            "node_definition_fingerprint": key_input.node_definition_fingerprint,
            "declared_environment_fingerprint": key_input.declared_environment_fingerprint,
            "input_lineage_fingerprint": key_input.input_lineage_fingerprint,
            "adapter_id": key_input.adapter_id,
            "adapter_version": key_input.adapter_version,
            "adapter_binary_sha256": key_input.adapter_binary_sha256,
            "produces_outputs_schema_version": key_input.output_schema_version,
            "policy_fingerprint": key_input.policy_fingerprint,
            "execution_contract_fingerprint": key_input.execution_contract_fingerprint,
            "backend_class": key_input.backend_class,
            "source_run_id":"run-fixed",
            "cache_source":"local"
        }),
    )
}

fn write_cache_entry(base: &Path, key: &str, meta: &Value, payload: &[u8]) {
    let entry = base.join(key);
    fs::create_dir_all(entry.join("outputs")).expect("create outputs dir");
    fs::write(
        entry.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "manifest_version":"cache-entry/v0.1",
            "cache_key": key,
            "node_id":"node-a",
            "outputs":[{
                "name":"payload",
                "path":"payload.bin",
                "kind":"file",
                "media_type":"application/octet-stream",
                "required": true
            }]
        }))
        .expect("serialize manifest"),
    )
    .expect("write manifest");
    fs::write(entry.join("outputs").join("payload.bin"), payload).expect("write payload");
    let index = OutputsIndex {
        files: vec![OutputFile {
            name: "payload".to_string(),
            path: "payload.bin".to_string(),
            kind: "file".to_string(),
            media_type: "application/octet-stream".to_string(),
            size_bytes: payload.len() as u64,
            sha256: sha256_hex(payload),
            node_id: "node-a".to_string(),
            node_fingerprint: meta
                .get("node_fingerprint")
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string(),
            promotable: false,
        }],
    };
    fs::write(
        entry.join("outputs").join("index.json"),
        serde_json::to_vec_pretty(&index).expect("serialize outputs index"),
    )
    .expect("write outputs index");
    fs::write(entry.join("meta.json"), serde_json::to_vec_pretty(meta).expect("serialize meta"))
        .expect("write meta");
}

fn apply_corruption(cache_dir: &Path, fixture_name: &str) -> String {
    let (key, meta) = default_meta("cache-entry");
    write_cache_entry(cache_dir, &key, &meta, b"ok\n");
    let entry = cache_dir.join(&key);
    match fixture_name {
        "hash_mismatch" => {
            fs::write(entry.join("outputs").join("payload.bin"), b"tampered\n")
                .expect("tamper payload");
        }
        "missing_manifest" => {
            fs::remove_file(entry.join("manifest.json")).expect("remove manifest");
        }
        "missing_meta" => {
            fs::remove_file(entry.join("meta.json")).expect("remove meta");
        }
        "missing_outputs_proof" => {
            let proofless = json!({
                "cache_metadata_version": CACHE_METADATA_VERSION,
                "node_fingerprint": "exec-proofless",
                "adapter_id":"shell",
                "adapter_version":"0.1",
                "source_run_id":"run-fixed",
                "cache_source":"local"
            });
            fs::write(entry.join("meta.json"), serde_json::to_vec_pretty(&proofless).unwrap())
                .expect("write proofless meta");
        }
        "truncated_meta" => {
            fs::write(
                entry.join("meta.json"),
                format!("{{\"cache_metadata_version\":\"{CACHE_METADATA_VERSION}\""),
            )
            .expect("truncate meta");
        }
        "unsupported_metadata_version" => {
            let (_, mut meta) = default_meta("cache-entry");
            meta["cache_metadata_version"] = Value::String("cache-meta/v9.9".to_string());
            fs::write(entry.join("meta.json"), serde_json::to_vec_pretty(&meta).unwrap())
                .expect("write unsupported meta");
        }
        other => panic!("unsupported corruption fixture: {other}"),
    }
    key
}

#[test]
fn cache_corruption_fixtures_are_classified_by_verify_and_explain() {
    let root = repo_root();
    let fixture_root = root.join("evidence/dag/cache/corrupt");
    for fixture in [
        "hash_mismatch",
        "missing_manifest",
        "missing_meta",
        "missing_outputs_proof",
        "truncated_meta",
        "unsupported_metadata_version",
    ] {
        let _fixture_payload = fs::read_to_string(fixture_root.join(format!("{fixture}.json")))
            .expect("read evidence fixture");
        let temp = tempfile::tempdir().expect("tempdir");
        let cache_dir = temp.path().join("cache");
        fs::create_dir_all(&cache_dir).expect("create cache dir");
        let key = apply_corruption(&cache_dir, fixture);

        let (verify_code, verify_stdout, verify_stderr) = run_command(
            &["--json", "cache", "verify", "--cache-dir", cache_dir.to_str().unwrap()],
            &root,
        );
        if fixture == "truncated_meta" {
            assert_ne!(verify_code, 0);
            assert!(
                verify_stdout.trim().is_empty(),
                "expected fatal malformed-meta verify path to emit no json payload, got: {verify_stdout}"
            );
            assert!(
                verify_stderr.trim().is_empty(),
                "unexpected stderr for truncated meta verify fixture: {verify_stderr}"
            );
        } else {
            let verify_payload = parse_json(&verify_stdout, verify_code, &verify_stderr);
            assert!(verify_code == 0 || verify_code == 3);
            let corrupt_total = verify_payload["data"]["corrupt_total"].as_u64().unwrap_or(0);
            let expected_corrupt = matches!(
                fixture,
                "hash_mismatch"
                    | "missing_manifest"
                    | "missing_meta"
                    | "missing_outputs_proof"
                    | "unsupported_metadata_version"
            );
            assert_eq!(
                corrupt_total > 0,
                expected_corrupt,
                "unexpected verify classification for fixture {fixture}"
            );
        }

        let (explain_code, explain_stdout, explain_stderr) = run_command(
            &[
                "--json",
                "cache",
                "explain",
                "--cache-dir",
                cache_dir.to_str().unwrap(),
                "--key",
                key.as_str(),
            ],
            &root,
        );
        if fixture == "truncated_meta" {
            assert_ne!(explain_code, 0);
            assert!(
                explain_stdout.trim().is_empty(),
                "expected fatal malformed-meta path to emit no json payload, got: {explain_stdout}"
            );
            assert!(
                explain_stderr.trim().is_empty(),
                "unexpected stderr for truncated meta fixture: {explain_stderr}"
            );
            continue;
        }
        let explain_payload = parse_json(&explain_stdout, explain_code, &explain_stderr);
        assert!(explain_code == 0 || explain_code == 3);
        let taxonomy = explain_payload["data"]["taxonomy"].as_array().expect("taxonomy");
        assert!(!taxonomy.is_empty(), "expected explain taxonomy for corruption fixture {fixture}");
        let expected_labels: &[&str] = match fixture {
            "hash_mismatch" => &["hash_mismatch", "artifact_corrupt"],
            "missing_manifest" => &["artifact_missing"],
            "missing_meta" => &["artifact_missing"],
            "missing_outputs_proof" => &["policy_mismatch"],
            "unsupported_metadata_version" => &["schema_mismatch"],
            other => panic!("unexpected corruption fixture: {other}"),
        };
        assert!(
            taxonomy.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|label| expected_labels.iter().any(|expected| label == *expected))
            }),
            "taxonomy {:?} missing expected labels {:?} for fixture {}",
            taxonomy,
            expected_labels,
            fixture
        );
    }
}
