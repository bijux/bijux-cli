use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use std::fs;
use std::path::PathBuf;
use tempfile as _;

use serde_json::{json, Value};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn evidence_ledger_path(root: &std::path::Path) -> PathBuf {
    let canonical = root.join("evidence/dag/ownership/evidence_ledger.json");
    if canonical.exists() {
        return canonical;
    }
    root.join("evidence/ownership/evidence_ledger.json")
}

fn validate_asset_entry(entry: &Value) -> Result<(), String> {
    let object = entry.as_object().ok_or_else(|| "evidence entry must be object".to_string())?;

    for required in ["id", "kind", "owner", "status", "canonical_path", "consumers"] {
        if !object.contains_key(required) {
            return Err(format!("missing required field `{required}`"));
        }
    }

    let id = entry
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| "id must be non-empty string".to_string())?;
    let kind = entry
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("kind missing for {id}"))?;
    let owner = entry
        .get("owner")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("owner missing for {id}"))?;
    let canonical_path = entry
        .get("canonical_path")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("canonical_path missing for {id}"))?;
    let consumers = entry
        .get("consumers")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("consumers missing for {id}"))?;

    if owner.trim().is_empty() {
        return Err(format!("owner is empty for {id}"));
    }
    if canonical_path.trim().is_empty() {
        return Err(format!("canonical_path is empty for {id}"));
    }
    if consumers.is_empty() {
        return Err(format!("consumers is empty for {id}"));
    }

    let allowed_kinds =
        ["authoring", "battle", "cache", "compat", "fault", "operator", "perf", "compare"];
    if !allowed_kinds.contains(&kind) {
        return Err(format!("unknown evidence kind `{kind}` for {id}"));
    }

    let release_blocking = entry.get("release_blocking").and_then(Value::as_bool).unwrap_or(false);
    let trust_properties =
        entry.get("trust_properties").and_then(Value::as_array).cloned().unwrap_or_default();
    if release_blocking && trust_properties.is_empty() {
        return Err(format!("release-blocking evidence must declare trust_properties for {id}"));
    }

    if entry.get("status").and_then(Value::as_str) == Some("duplicate") {
        let duplicate_of = entry.get("duplicate_of").unwrap_or(&Value::Null);
        if duplicate_of.is_null()
            || duplicate_of.as_str().map_or(true, |value| value.trim().is_empty())
        {
            return Err(format!("duplicate asset must declare duplicate_of for {id}"));
        }
    }

    let derived_from = entry.get("derived_from").unwrap_or(&Value::Null);
    if !derived_from.is_null()
        && derived_from.as_str().map_or(true, |value| value.trim().is_empty())
    {
        return Err(format!("derived asset linkage is invalid for {id}"));
    }

    Ok(())
}

#[test]
fn evidence_schema_files_exist() {
    let root = repo_root();
    for rel in [
        "configs/dag/schema/evidence_asset.schema.json",
        "configs/dag/schema/evidence_family.schema.json",
        "configs/dag/schema/evidence_cache_metadata.schema.json",
        "configs/dag/schema/evidence_battle_metadata.schema.json",
        "configs/dag/schema/evidence_perf_metadata.schema.json",
        "configs/dag/schema/evidence_compare_metadata.schema.json",
        "configs/dag/schema/evidence_compat_metadata.schema.json",
        "configs/dag/schema/evidence_fault_metadata.schema.json",
        "configs/dag/schema/evidence_authoring_metadata.schema.json",
    ] {
        assert!(root.join(rel).exists(), "missing evidence schema file: {rel}");
    }
}

#[test]
fn evidence_ledger_entries_use_strict_schema_keys() {
    let root = repo_root();
    let ledger: Value = serde_json::from_str(
        &fs::read_to_string(evidence_ledger_path(&root)).expect("read ledger"),
    )
    .expect("parse ledger");
    let entries = ledger["entries"].as_array().expect("entries array");
    for entry in entries {
        validate_asset_entry(entry).expect("entry should satisfy strict evidence schema");
    }
}

#[test]
fn invalid_evidence_metadata_fails_fast() {
    let invalid = json!({"id": "", "kind": "unknown"});
    assert!(
        validate_asset_entry(&invalid).is_err(),
        "invalid evidence metadata must fail validation"
    );
}

#[test]
fn missing_owner_fails() {
    let candidate = json!({
        "id": "asset-1",
        "kind": "authoring",
        "status": "keep",
        "canonical_path": "evidence/authoring/examples/hello.dag.json",
        "consumers": ["example-tests"]
    });
    assert!(validate_asset_entry(&candidate).is_err());
}

#[test]
fn missing_canonical_path_fails() {
    let candidate = json!({
        "id": "asset-1",
        "kind": "authoring",
        "owner": "team-authoring",
        "status": "keep",
        "consumers": ["example-tests"]
    });
    assert!(validate_asset_entry(&candidate).is_err());
}

#[test]
fn unknown_evidence_kind_fails() {
    let candidate = json!({
        "id": "asset-1",
        "kind": "mystery-kind",
        "owner": "team-authoring",
        "status": "keep",
        "canonical_path": "evidence/authoring/examples/hello.dag.json",
        "consumers": ["example-tests"]
    });
    assert!(validate_asset_entry(&candidate).is_err());
}

#[test]
fn release_blocking_requires_trust_properties() {
    let candidate = json!({
        "id": "asset-1",
        "kind": "battle",
        "owner": "team-battle",
        "status": "keep",
        "canonical_path": "evidence/battle/workflows/e2e_minimal.json",
        "consumers": ["battle-suite"],
        "release_blocking": true,
        "trust_properties": []
    });
    assert!(validate_asset_entry(&candidate).is_err());
}

#[test]
fn duplicate_assets_must_declare_duplicate_of() {
    let candidate = json!({
        "id": "asset-1",
        "kind": "compat",
        "owner": "team-compat",
        "status": "duplicate",
        "canonical_path": "evidence/compat/scenarios/historical_fixture_validation.json",
        "consumers": ["compat-suite"],
        "release_blocking": false,
        "trust_properties": ["compatibility"],
        "duplicate_of": null
    });
    assert!(validate_asset_entry(&candidate).is_err());
}

#[test]
fn derived_assets_must_declare_source_linkage() {
    let candidate = json!({
        "id": "asset-1",
        "kind": "perf",
        "owner": "team-performance",
        "status": "keep",
        "canonical_path": "evidence/perf/scenarios/tiny_canonical.json",
        "consumers": ["performance-suite"],
        "release_blocking": false,
        "trust_properties": ["resource-accounting"],
        "derived_from": "   "
    });
    assert!(validate_asset_entry(&candidate).is_err());
}

#[test]
fn evidence_schema_verify_command_is_wired() {
    let root = repo_root();
    let source =
        fs::read_to_string(root.join("crates/bijux-dev/src/commands/mod.rs")).expect("read");
    assert!(
        source.contains("verify.evidence-schema"),
        "verify evidence-schema command must be wired"
    );
}
