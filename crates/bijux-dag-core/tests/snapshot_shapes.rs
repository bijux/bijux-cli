use criterion as _;
use hex as _;
use serde as _;
use serde_json as _;
use serde_yaml as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;
use unicode_normalization as _;

use bijux_dag_core::parse_graph_strict;
use std::fs;
use std::path::PathBuf;

fn planner_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("planner")
        .join(name)
}

#[test]
fn canonical_shape_fixtures_parse_strictly() {
    let fixtures = [
        "linear.dag.json",
        "fan_out.dag.json",
        "fan_in.dag.json",
        "diamond.dag.json",
        "isolated_groups.dag.json",
        "retry_heavy.dag.json",
        "resource_heavy.dag.json",
        "replay_oriented.dag.json",
    ];
    for fixture in fixtures {
        let text = fs::read_to_string(planner_fixture_path(fixture)).unwrap();
        parse_graph_strict(&text).unwrap();
    }
}
