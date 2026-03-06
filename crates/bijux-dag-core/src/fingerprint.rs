//! Graph fingerprint entrypoints.

use crate::{Graph, GraphError};

pub fn graph_fingerprint(graph: &Graph) -> Result<String, GraphError> {
    graph.graph_fingerprint()
}

pub fn canonical_json(graph: &Graph) -> Result<String, GraphError> {
    graph.to_canonical_json()
}
