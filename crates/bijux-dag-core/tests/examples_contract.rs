use bijux_dag_core::parse_graph_strict;

#[test]
fn tutorial_examples_parse_as_stable_contracts() {
    let examples = [
        "../../examples/hello.dag.json",
        "../../examples/etl-constant-to-shell.dag.json",
        "../../examples/cached-branched-report.dag.json",
        "../../examples/multi-output-artifact.dag.json",
        "../../examples/replay-heavy-branching.dag.json",
        "../../examples/failure-heavy-retry.dag.json",
    ];
    for relative in examples {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let _graph = parse_graph_strict(&text)
            .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()));
    }
}
