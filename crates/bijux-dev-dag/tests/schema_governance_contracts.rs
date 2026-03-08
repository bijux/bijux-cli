use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use sha2::Digest;
use std::fs;
use std::path::PathBuf;
use tempfile as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn unified_schema_versioning_and_compatibility_docs_exist() {
    let root = repo_root();
    for rel in [
        "docs/spec/UNIFIED_SCHEMA_VERSIONING_POLICY.md",
        "docs/spec/STABLE_EXPERIMENTAL_SCHEMA_FIELDS.md",
        "docs/spec/SCHEMA_FIELD_DEPRECATION_POLICY.md",
        "docs/spec/SCHEMA_BACKWARD_COMPATIBILITY_GUARANTEES.md",
        "docs/spec/SCHEMA_FORWARD_COMPATIBILITY_LIMITATIONS.md",
        "docs/reference/COMPATIBILITY_MATRIX_GENERATED.md",
        "docs/reports/foundation/schema_changelog.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing schema governance doc: {rel}"
        );
    }
}

#[test]
fn compatibility_fixtures_exist_for_graph_run_artifact_and_proof() {
    let root = repo_root();
    for rel in [
        "evidence/compat/graph_schema/v0_1_supported/minimal.dag.json",
        "evidence/compat/graph_schema/unsupported_past/minimal.dag.json",
        "evidence/compat/run_schema/v0_1_supported/minimal.manifest.json",
        "evidence/compat/run_schema/unsupported_past/minimal.manifest.json",
        "evidence/compat/artifact_schema/v0_1_supported/minimal.outputs.json",
        "evidence/compat/artifact_schema/unsupported_past/minimal.outputs.json",
        "evidence/compat/proof_schema/v0_1_supported/minimal.proof.json",
        "evidence/compat/proof_schema/unsupported_past/minimal.proof.json",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing compatibility fixture: {rel}"
        );
    }
}

#[test]
fn stable_schema_hashes_are_frozen() {
    let root = repo_root();
    let baseline: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/stable_schema_hashes.json"))
            .expect("read baseline"),
    )
    .expect("parse baseline");
    let schemas = baseline["schemas"].as_object().expect("schemas object");
    for (path, expected_hash) in schemas {
        let expected = expected_hash.as_str().expect("expected hash");
        let full = root.join(path);
        let bytes = fs::read(&full).expect("read schema file");
        let actual = sha2::Sha256::digest(&bytes);
        let actual_hex = format!("{:x}", actual);
        assert_eq!(actual_hex, expected, "stable schema hash changed: {path}");
    }
}

#[test]
fn stable_json_command_surfaces_are_listed_in_cli_contract() {
    let root = repo_root();
    let baseline: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/stable_json_output_commands.json"))
            .expect("read commands baseline"),
    )
    .expect("parse commands baseline");
    let commands = baseline["commands"].as_array().expect("commands array");
    let cli_contract =
        fs::read_to_string(root.join("docs/spec/CLI_CONTRACT.md")).expect("read cli contract");
    for cmd in commands {
        let cmd = cmd.as_str().expect("command string");
        assert!(
            cli_contract.contains(cmd.split('.').last().expect("tail")),
            "stable json output command should appear in CLI contract guidance: {cmd}"
        );
    }
}

#[test]
fn experimental_fields_are_labeled_in_registry_doc() {
    let root = repo_root();
    let doc = fs::read_to_string(root.join("docs/spec/STABLE_EXPERIMENTAL_SCHEMA_FIELDS.md"))
        .expect("read stable/experimental doc");
    for field in [
        "backend_metadata",
        "signing.signature_format",
        "signing.signature",
        "semantic_portability",
    ] {
        let line = doc
            .lines()
            .find(|l| l.contains(field))
            .unwrap_or_default()
            .to_string();
        assert!(
            line.contains("(experimental)"),
            "experimental field must be labeled: {field}"
        );
    }
}

#[test]
fn release_process_mentions_schema_compatibility_review() {
    let root = repo_root();
    let doc =
        fs::read_to_string(root.join("docs/RELEASE_PROCESS.md")).expect("read release process");
    assert!(doc.contains("Schema compatibility review must be completed"));
}

#[test]
fn schema_changelog_lists_schema_files() {
    let root = repo_root();
    let doc = fs::read_to_string(root.join("docs/reports/foundation/schema_changelog.md"))
        .expect("read schema changelog");
    for schema in [
        "configs/schema/dag.schema.json",
        "configs/schema/run_manifest.schema.json",
        "configs/schema/node_trace.schema.json",
        "configs/schema/outputs_index.schema.json",
    ] {
        assert!(doc.contains(schema), "schema changelog missing: {schema}");
    }
}

#[test]
fn migration_contracts_are_deterministic_idempotent_and_report_noop() {
    let root = repo_root();
    let dag = root.join("configs/schema/fixtures/v0.1/positive/fan-out.json");

    let run_once = std::process::Command::new("cargo")
        .args([
            "run",
            "-p",
            "bijux-dag-cli",
            "--",
            "dag",
            "migrate",
            "dag",
            dag.to_str().expect("dag path"),
            "--from",
            "0.1",
            "--to",
            "0.1",
            "--dry-run",
        ])
        .current_dir(&root)
        .output()
        .expect("migrate dry-run once");
    assert!(run_once.status.success());
    let out_once = String::from_utf8_lossy(&run_once.stdout);
    assert!(out_once.contains("dry-run: no migration needed"));

    let run_twice = std::process::Command::new("cargo")
        .args([
            "run",
            "-p",
            "bijux-dag-cli",
            "--",
            "dag",
            "migrate",
            "dag",
            dag.to_str().expect("dag path"),
            "--from",
            "0.1",
            "--to",
            "0.1",
            "--dry-run",
        ])
        .current_dir(&root)
        .output()
        .expect("migrate dry-run twice");
    assert!(run_twice.status.success());
    let out_twice = String::from_utf8_lossy(&run_twice.stdout);
    assert_eq!(out_once.trim(), out_twice.trim());
}

#[test]
fn schema_governance_workflow_exists() {
    let root = repo_root();
    let workflow = root.join(".github/workflows/schema-governance.yml");
    assert!(workflow.exists());
    let body = fs::read_to_string(workflow).expect("read workflow");
    assert!(body.contains("generate schema changelog"));
    assert!(body.contains("schema governance contract tests"));
}
