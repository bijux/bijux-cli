#![forbid(unsafe_code)]
//! Output-envelope fuzz regression replay from retained minimized cases.
//! test_type: output-envelope-fuzz-regression

use std::fs;
use std::path::Path;

use bijux_cli_core as _;
use bijux_cli_output as _;
use bijux_cli_routing as _;
use serde as _;
use serde_json as _;
use serde_yaml as _;
use thiserror as _;

#[test]
fn minimized_output_cases_replay_with_stable_parse_behavior() {
    let dir = Path::new("tests/fuzz/output_minimized_cases");
    let mut files: Vec<_> = fs::read_dir(dir)
        .expect("output minimized corpus must exist")
        .map(|e| e.expect("entry").path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "output minimized corpus must not be empty");

    for path in files {
        let sample = fs::read_to_string(&path).expect("read sample");
        let a = serde_json::from_str::<bijux_cli_routing::OutputEnvelopeV1>(&sample);
        let b = serde_json::from_str::<bijux_cli_routing::OutputEnvelopeV1>(&sample);
        assert_eq!(a.is_ok(), b.is_ok(), "determinism for {}", path.display());
    }
}
