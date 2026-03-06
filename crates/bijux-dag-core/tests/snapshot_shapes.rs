use bijux_dag_core::parse_graph_strict;
use std::fs;
use std::path::PathBuf;

fn snapshot_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
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
    ];
    for fixture in fixtures {
        let text = fs::read_to_string(snapshot_path(fixture)).unwrap();
        parse_graph_strict(&text).unwrap();
    }
}
