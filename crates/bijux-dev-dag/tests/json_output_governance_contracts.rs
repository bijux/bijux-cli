use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use tempfile as _;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn policy() -> Value {
    serde_json::from_str(
        &fs::read_to_string(root().join("configs/policy/json_output_governance.json"))
            .expect("read json output governance policy"),
    )
    .expect("parse json output governance policy")
}

fn schema_example_dir(schema_rel: &str) -> PathBuf {
    let stem = Path::new(schema_rel)
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("schema stem");
    root().join("evidence/operator/examples/stable_json").join(stem)
}

fn assert_schema_example_lockstep(schema_rel: &str) {
    let schema: Value = serde_json::from_str(
        &fs::read_to_string(root().join(schema_rel)).expect("read schema"),
    )
    .expect("parse schema");

    let required: Vec<String> = schema["required"]
        .as_array()
        .expect("required array")
        .iter()
        .map(|v| v.as_str().expect("required string").to_string())
        .collect();

    let minimal_path = schema_example_dir(schema_rel).join("minimal.json");
    let maximal_path = schema_example_dir(schema_rel).join("maximal.json");

    let minimal: Value = serde_json::from_str(&fs::read_to_string(&minimal_path).expect("read minimal"))
        .expect("parse minimal");
    let maximal: Value = serde_json::from_str(&fs::read_to_string(&maximal_path).expect("read maximal"))
        .expect("parse maximal");

    assert_eq!(minimal["schema"], schema_rel, "minimal schema link mismatch");
    assert_eq!(maximal["schema"], schema_rel, "maximal schema link mismatch");
    assert_eq!(minimal["example_type"], "minimal");
    assert_eq!(maximal["example_type"], "maximal");

    let min_data = minimal["data"].as_object().expect("minimal data object");
    let max_data = maximal["data"].as_object().expect("maximal data object");

    for key in required {
        assert!(min_data.contains_key(&key), "minimal example missing required field {key} in {schema_rel}");
        assert!(max_data.contains_key(&key), "maximal example missing required field {key} in {schema_rel}");
    }
}

#[test]
fn json_output_governance_policy_declares_schema_example_and_lockstep_rule() {
    let gov = policy();
    let rule = gov["governance_rule"].as_str().expect("governance_rule");
    assert!(rule.contains("schema authority"));
    assert!(rule.contains("minimal and maximal examples"));
    assert!(rule.contains("lockstep"));
}

#[test]
fn generated_json_output_reports_and_registries_exist() {
    for rel in [
        "docs/reports/foundation/json_command_schema_inventory_report.md",
        "docs/reports/foundation/schema_command_test_inventory_report.md",
        "docs/reports/foundation/schema_without_example_output_report.md",
        "docs/reports/foundation/commands_without_json_lockstep_report.md",
        "docs/reference/SCHEMA_REGISTRY.md",
        "docs/reference/STABLE_JSON_OUTPUT_COMMAND_REGISTRY.md",
    ] {
        assert!(root().join(rel).exists(), "missing generated json governance artifact: {rel}");
    }
}

#[test]
fn schema_example_and_lockstep_gap_reports_are_zero() {
    let example_gap = fs::read_to_string(root().join("docs/reports/foundation/schema_without_example_output_report.md"))
        .expect("read schema gap report");
    assert!(example_gap.contains("Missing schema examples: 0"));

    let lockstep_gap = fs::read_to_string(root().join("docs/reports/foundation/commands_without_json_lockstep_report.md"))
        .expect("read lockstep gap report");
    assert!(lockstep_gap.contains("Commands missing lockstep tests: 0"));
}

#[test]
fn run_show_json_schema_examples_lockstep() {
    assert_schema_example_lockstep("configs/schema/operator/run_show.schema.json");
}

#[test]
fn run_inspect_json_schema_examples_lockstep() {
    assert_schema_example_lockstep("configs/schema/operator/run_inspect.schema.json");
}

#[test]
fn run_id_explain_json_schema_examples_lockstep() {
    assert_schema_example_lockstep("configs/schema/operator/run_id_explain.schema.json");
}

#[test]
fn run_history_json_schema_examples_lockstep() {
    assert_schema_example_lockstep("configs/schema/operator/run_history.schema.json");
}

#[test]
fn run_list_json_schema_examples_lockstep() {
    assert_schema_example_lockstep("configs/schema/operator/run_list.schema.json");
}

#[test]
fn run_summary_json_schema_examples_lockstep() {
    assert_schema_example_lockstep("configs/schema/operator/run_summary.schema.json");
}

#[test]
fn run_timeline_json_schema_examples_lockstep() {
    assert_schema_example_lockstep("configs/schema/operator/run_timeline.schema.json");
}

#[test]
fn run_tree_json_schema_examples_lockstep() {
    assert_schema_example_lockstep("configs/schema/operator/run_tree.schema.json");
}

#[test]
fn runs_analytics_json_schema_examples_lockstep() {
    assert_schema_example_lockstep("configs/schema/operator/runs_analytics.schema.json");
}

#[test]
fn artifact_inspect_json_schema_examples_lockstep() {
    assert_schema_example_lockstep("configs/schema/operator/artifact_inspect.schema.json");
}

#[test]
fn artifact_trace_json_schema_examples_lockstep() {
    assert_schema_example_lockstep("configs/schema/operator/artifact_trace.schema.json");
}

#[test]
fn run_diff_json_schema_examples_lockstep() {
    assert_schema_example_lockstep("configs/schema/operator/run_diff.schema.json");
}

#[test]
fn graph_diff_json_schema_examples_lockstep() {
    assert_schema_example_lockstep("configs/schema/operator/graph_diff.schema.json");
}

#[test]
fn replay_diff_json_schema_examples_lockstep() {
    assert_schema_example_lockstep("configs/schema/operator/replay_diff.schema.json");
}

#[test]
fn replay_proof_json_schema_examples_lockstep() {
    assert_schema_example_lockstep("configs/schema/operator/replay_proof.schema.json");
}

#[test]
fn run_explain_failure_json_schema_examples_lockstep() {
    assert_schema_example_lockstep("configs/schema/operator/run_explain_failure.schema.json");
}

#[test]
fn run_doctor_json_schema_examples_lockstep() {
    assert_schema_example_lockstep("configs/schema/operator/run_doctor.schema.json");
}

#[test]
fn prove_output_json_schema_examples_lockstep() {
    assert_schema_example_lockstep("configs/schema/operator/prove_output.schema.json");
}

#[test]
fn verify_output_json_schema_examples_lockstep() {
    assert_schema_example_lockstep("configs/schema/operator/verify_output.schema.json");
}

#[test]
fn run_verify_report_json_schema_examples_lockstep() {
    assert_schema_example_lockstep("configs/schema/operator/run_verify_report.schema.json");
}

#[test]
fn capability_query_json_schema_examples_lockstep() {
    assert_schema_example_lockstep("configs/schema/operator/capability_query.schema.json");
}

#[test]
fn benchmark_report_json_schema_examples_lockstep() {
    assert_schema_example_lockstep("configs/schema/benchmarks/benchmark_report.schema.json");
}

#[test]
fn stable_command_registry_matches_policy_families() {
    let gov = policy();
    let families: BTreeSet<String> = gov["stable_command_families"]
        .as_array()
        .expect("families")
        .iter()
        .map(|f| f["family"].as_str().expect("family").to_string())
        .collect();

    let registry = fs::read_to_string(root().join("docs/reference/STABLE_JSON_OUTPUT_COMMAND_REGISTRY.md"))
        .expect("read command registry");
    for family in families {
        assert!(registry.contains(&format!("`{family}`")), "registry missing family {family}");
    }
}
