//! DAG parse entrypoints.

use crate::error::GraphError;
use crate::{Graph, SPEC_VERSION};

pub fn parse_graph_strict(input: &str) -> Result<Graph, GraphError> {
    let mut graph: Graph = serde_json::from_str(input)?;
    if graph.spec == "0.1" || graph.spec == "v0.1" || graph.spec == "v1" {
        graph.spec = SPEC_VERSION.to_string();
    }
    if graph.spec != SPEC_VERSION {
        return Err(GraphError::InvalidSpec(graph.spec));
    }
    if graph_has_ambiguous_output_path(&graph) {
        return Err(GraphError::ValidationFailed);
    }
    Ok(graph)
}

fn graph_has_ambiguous_output_path(graph: &Graph) -> bool {
    for node in &graph.nodes {
        for output in &node.outputs {
            if output.path.contains("..") {
                return true;
            }
            if output.path.starts_with('/') || output.path.starts_with('\\') {
                return true;
            }
        }
    }
    for definition in graph.subgraphs.values() {
        if graph_has_ambiguous_output_path(&definition.graph) {
            return true;
        }
    }
    false
}
