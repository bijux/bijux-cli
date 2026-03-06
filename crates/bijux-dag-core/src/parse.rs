//! DAG parse entrypoints.

use crate::error::GraphError;
use crate::{Graph, SPEC_VERSION};

pub fn parse_graph_strict(input: &str) -> Result<Graph, GraphError> {
    let graph: Graph = serde_json::from_str(input)?;
    if graph.spec != SPEC_VERSION {
        return Err(GraphError::InvalidSpec(graph.spec));
    }
    Ok(graph)
}
