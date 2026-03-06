//! Deterministic topology entrypoints.

use crate::{Graph, GraphError};

pub fn deterministic_topology_order(graph: &Graph) -> Result<Vec<String>, GraphError> {
    graph.topo_order()
}
