use criterion as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_core::parse_graph_strict;
use std::fs;
use std::path::Path;

fn load(path: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(path)).expect("read fixture")
}

#[test]
fn canonical_examples_parse_and_validate() {
    for path in [
        "tests/authoring/examples/minimal.json",
        "tests/authoring/examples/medium.json",
        "tests/authoring/examples/pattern_chain.json",
        "tests/authoring/examples/pattern_diamond.json",
        "tests/authoring/examples/pattern_fanout.json",
        "tests/authoring/examples/pattern_aggregation.json",
        "tests/authoring/examples/pattern_cache_heavy.json",
        "tests/authoring/examples/pattern_replay_sensitive.json",
    ] {
        let graph = parse_graph_strict(&load(path)).expect("parse canonical example");
        let diags = graph.validate_with_warnings();
        assert!(
            diags.iter().all(|d| d.severity != bijux_dag_core::Severity::Error),
            "fixture contains validation error: {path}"
        );
    }
}

#[test]
fn bad_examples_fail_parse_or_validation() {
    for path in [
        "tests/authoring/bad/undeclared_outputs.json",
        "tests/authoring/bad/invalid_refs.json",
        "tests/authoring/bad/cycle.json",
        "tests/authoring/bad/invalid_selectors.json",
        "tests/authoring/bad/unsupported_adapter_payload.json",
    ] {
        let parsed = parse_graph_strict(&load(path));
        match parsed {
            Ok(graph) => {
                let diags = graph.validate_with_warnings();
                assert!(
                    diags.iter().any(|d| d.severity == bijux_dag_core::Severity::Error),
                    "expected validation errors for bad fixture: {path}"
                );
            }
            Err(_) => {}
        }
    }
}

#[test]
fn canonicalization_is_stable_and_non_destructive() {
    let payload = load("tests/authoring/examples/medium.json");
    let graph = parse_graph_strict(&payload).expect("parse medium");
    let canonical_once = graph.to_canonical_json().expect("canonical once");
    let parsed_again = parse_graph_strict(&canonical_once).expect("parse canonical");
    let canonical_twice = parsed_again.to_canonical_json().expect("canonical twice");
    assert_eq!(canonical_once, canonical_twice);
}
