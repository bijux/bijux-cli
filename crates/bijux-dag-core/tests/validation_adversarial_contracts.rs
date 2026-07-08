use bijux_dag_core::{parse_graph_strict, Severity};
use std::fs;
use std::path::PathBuf;

mod support;

use support::load_workspace_fixture_text;

fn negative_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../evidence/dag/authoring/negative")
}

#[test]
fn negative_authoring_fixtures_never_pass_strict_validation() {
    for entry in fs::read_dir(negative_fixture_root()).expect("negative fixtures") {
        let entry = entry.expect("fixture entry");
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let payload = fs::read_to_string(&path).expect("read negative fixture");
        match parse_graph_strict(&payload) {
            Ok(graph) => {
                let diagnostics = graph.validate_with_warnings();
                assert!(
                    diagnostics.iter().any(|diag| diag.severity == Severity::Error),
                    "negative fixture parsed without error diagnostics: {}",
                    path.display()
                );
            }
            Err(_) => {}
        }
    }
}

#[test]
fn adversarial_inline_cases_reject_path_escapes_and_weird_selector_shapes() {
    let path_escape = r#"{
      "spec":"bijux-dag/v0.1",
      "nodes":[{"id":"n","kind":"const","outputs":[{"name":"out","path":"../escape"}],"params":{"value":1}}],
      "edges":[]
    }"#;
    assert!(parse_graph_strict(path_escape).is_err());

    let invalid_branch = r#"{
      "spec":"bijux-dag/v0.1",
      "nodes":[
        {
          "id":"decide",
          "kind":"shell",
          "semantic_kind":"branch",
          "inputs":["in"],
          "outputs":[{"name":"decision","path":"decision.txt"}],
          "effects":["filesystem"],
          "params":{"argv":["/bin/sh","-c","printf left > ../outputs/decision.txt"]},
          "branch":{"decisions":[],"decision_output":"decision"}
        }
      ],
      "edges":[]
    }"#;
    let graph = parse_graph_strict(invalid_branch).expect("parse invalid branch");
    let diagnostics = graph.validate_with_warnings();
    assert!(diagnostics.iter().any(|diag| diag.severity == Severity::Error));
}

#[test]
fn negative_fixtures_cover_expected_authoring_failure_classes() {
    for relative in [
        "evidence/dag/authoring/negative/cycle.json",
        "evidence/dag/authoring/negative/invalid_container_workdir.json",
        "evidence/dag/authoring/negative/invalid_refs.json",
        "evidence/dag/authoring/negative/invalid_selectors.json",
        "evidence/dag/authoring/negative/missing_required_input_binding.json",
        "evidence/dag/authoring/negative/undeclared_outputs.json",
        "evidence/dag/authoring/negative/unsupported_adapter_payload.json",
    ] {
        let payload = load_workspace_fixture_text(env!("CARGO_MANIFEST_DIR"), relative);
        assert!(
            parse_graph_strict(&payload).is_err()
                || parse_graph_strict(&payload)
                    .expect("parsed negative graph")
                    .validate_with_warnings()
                    .iter()
                    .any(|diag| diag.severity == Severity::Error),
            "negative fixture escaped validation: {relative}"
        );
    }
}
