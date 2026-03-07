//! DAG canonicalization entrypoints.

use crate::{Graph, GraphError};

pub fn canonicalize_graph(graph: &Graph) -> Graph {
    graph.canonicalize()
}

pub fn canonical_json(graph: &Graph) -> Result<String, GraphError> {
    graph.to_canonical_json()
}
