use criterion as _;
use hex as _;
use serde as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_core::{
    graph_lowering_boundary_note, map_planner_error_to_graph_error, planner_alignment_required_doc,
    planner_alignment_required_schema, planner_alignment_required_test,
    planner_diagnostics_from_error, PlannerError,
};

#[test]
fn planner_error_codes_contract_is_stable() {
    let matrix = [
        PlannerError::ValidationFailed,
        PlannerError::UnsupportedNodeKind("shell".to_string()),
        PlannerError::Topology("cycle".to_string()),
        PlannerError::Fingerprint("hash".to_string()),
    ];

    for err in matrix {
        let diags = planner_diagnostics_from_error(&err);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].id, "P4000");
        assert_eq!(diags[0].severity, bijux_dag_core::PlannerSeverity::Error);
        assert!(!diags[0].message.is_empty());
        let mapped = map_planner_error_to_graph_error(err);
        assert!(mapped.to_string().contains("planner"));
    }
}

#[test]
fn planner_explain_schema_and_alignment_contract_paths_are_pinned() {
    assert_eq!(
        planner_alignment_required_schema(),
        "configs/schema/execution_plan.schema.json"
    );
    assert_eq!(
        planner_alignment_required_doc(),
        "docs/spec/PLANNER_CONTRACT.md"
    );
    assert_eq!(
        planner_alignment_required_test(),
        "crates/bijux-dag-core/tests/planner_contract.rs"
    );
    assert!(graph_lowering_boundary_note().contains("before execution planning"));
}

#[test]
fn planner_explain_schema_file_exists() {
    let schema_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../configs/schema/planner_explain.schema.json");
    let payload = std::fs::read_to_string(schema_path).expect("planner explain schema");
    let schema: serde_json::Value = serde_json::from_str(&payload).expect("schema parse");

    assert_eq!(schema["$schema"], "https://json-schema.org/draft/2020-12/schema");
    assert_eq!(schema["type"], "object");
    assert!(schema["required"]
        .as_array()
        .expect("required")
        .iter()
        .any(|v| v == "diagnostics"));
}
