use crate::meta::{DagId, DagVersionId};
use crate::node::NodeGroupContract;
use crate::resources::GraphDefaults;
use crate::Graph;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphExecutionPolicy {
    pub fail_fast: bool,
    pub deterministic_dispatch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphContract {
    pub dag_id: DagId,
    pub dag_version_id: DagVersionId,
    pub graph: Graph,
    pub namespace: Option<String>,
    pub owners: Vec<String>,
    pub labels: BTreeMap<String, String>,
    pub annotations: BTreeMap<String, String>,
    pub environment_tags: Vec<String>,
    pub defaults: GraphDefaults,
    pub execution_policy: GraphExecutionPolicy,
    pub node_groups: Vec<NodeGroupContract>,
}

impl GraphContract {
    pub fn normalize_with_defaults(&self) -> Graph {
        let mut graph = self.graph.clone();
        for node in &mut graph.nodes {
            if node.retry.max_attempts == 0 {
                if let Some(default_retry) = self.defaults.retry.as_ref() {
                    node.retry = default_retry.clone();
                }
            }
            if node.resources.is_none() {
                node.resources = self.defaults.resources.clone();
            }
        }
        graph
    }
}
