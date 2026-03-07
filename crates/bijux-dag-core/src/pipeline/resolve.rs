//! DAG resolve entrypoints.

use crate::{Graph, GraphError, ResolvedGraph};

pub fn resolve_graph(graph: &Graph) -> Result<ResolvedGraph, GraphError> {
    graph.resolve_graph()
}
