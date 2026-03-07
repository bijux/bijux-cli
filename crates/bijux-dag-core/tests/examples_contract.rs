use criterion as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_core::parse_graph_strict;

#[test]
fn tutorial_examples_parse_as_stable_contracts() {
    let examples = [
        "../../evidence/authoring/examples/hello.dag.json",
        "../../evidence/authoring/examples/etl-constant-to-shell.dag.json",
        "../../evidence/authoring/examples/cached-branched-report.dag.json",
        "../../evidence/authoring/examples/multi-output-artifact.dag.json",
        "../../evidence/authoring/examples/replay-heavy-branching.dag.json",
        "../../evidence/authoring/examples/failure-heavy-retry.dag.json",
    ];
    for relative in examples {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let _graph = parse_graph_strict(&text)
            .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()));
    }
}
