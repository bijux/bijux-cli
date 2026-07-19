//! Deterministic topology entrypoints.

use crate::expansion::expand_graph;
use crate::{Graph, GraphError};
use std::collections::{BTreeSet, HashMap};

impl Graph {
    pub fn topo_order(&self) -> Result<Vec<String>, GraphError> {
        let expanded = expand_graph(self).map_err(|_| GraphError::ValidationFailed)?;
        expanded.topo_order_expanded()
    }

    fn topo_order_expanded(&self) -> Result<Vec<String>, GraphError> {
        let mut indegree = HashMap::<String, usize>::new();
        let mut adjacency = HashMap::<String, Vec<String>>::new();
        for node in &self.nodes {
            indegree.insert(node.id.clone(), 0);
            adjacency.insert(node.id.clone(), Vec::new());
        }

        for edge in &self.edges {
            if let Some(neighbors) = adjacency.get_mut(&edge.from.node_id) {
                neighbors.push(edge.to.node_id.clone());
            }
            if let Some(degree) = indegree.get_mut(&edge.to.node_id) {
                *degree += 1;
            }
        }

        let mut ready = indegree
            .iter()
            .filter_map(|(node_id, degree)| if *degree == 0 { Some(node_id.clone()) } else { None })
            .collect::<BTreeSet<_>>();
        let mut order = Vec::new();

        while let Some(node_id) = ready.iter().next().cloned() {
            ready.remove(&node_id);
            order.push(node_id.clone());
            if let Some(neighbors) = adjacency.get(&node_id) {
                for neighbor in neighbors {
                    if let Some(degree) = indegree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            ready.insert(neighbor.clone());
                        }
                    }
                }
            }
        }

        if order.len() != indegree.len() {
            return Err(GraphError::ValidationFailed);
        }

        Ok(order)
    }

    pub(crate) fn has_cycle(&self) -> bool {
        self.topo_order_expanded().is_err()
    }
}

pub fn deterministic_topology_order(graph: &Graph) -> Result<Vec<String>, GraphError> {
    graph.topo_order()
}
