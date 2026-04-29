use bijux_dag_core::parse_graph_strict;
use std::fs;
use std::path::PathBuf;

fn authoring_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../evidence/dag/authoring")
}

fn collect_fixtures(dir: &str) -> Vec<PathBuf> {
    let root = authoring_root().join(dir);
    let mut files = fs::read_dir(root)
        .expect("fixture dir")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    files.sort();
    files
}

#[test]
fn authoring_examples_roundtrip_through_canonical_json() {
    for path in collect_fixtures("examples") {
        let raw = fs::read_to_string(&path).expect("read example fixture");
        let graph = parse_graph_strict(&raw).expect("parse example");
        let canonical = graph.to_canonical_json().expect("canonical example");
        let reparsed = parse_graph_strict(&canonical).expect("reparse canonical");
        assert_eq!(
            graph.graph_id().expect("graph id"),
            reparsed.graph_id().expect("reparsed graph id"),
            "schema roundtrip drifted for {}",
            path.display()
        );
    }
}

#[test]
fn authoring_patterns_roundtrip_through_struct_and_back() {
    for path in collect_fixtures("patterns") {
        let raw = fs::read_to_string(&path).expect("read pattern fixture");
        let graph = parse_graph_strict(&raw).expect("parse pattern");
        let encoded = serde_json::to_string_pretty(&graph).expect("encode pattern");
        let decoded = parse_graph_strict(&encoded).expect("decode encoded pattern");
        assert_eq!(
            graph.graph_id().expect("graph id"),
            decoded.graph_id().expect("decoded graph id"),
            "typed roundtrip drifted for {}",
            path.display()
        );
    }
}
