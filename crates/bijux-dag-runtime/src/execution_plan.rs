use bijux_dag_core::Node;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub nodes: Vec<Node>,
    pub order: Vec<String>,
    pub dep_map: HashMap<String, BTreeSet<String>>,
    pub indegree: HashMap<String, usize>,
    pub adj: HashMap<String, Vec<String>>,
    pub filter_reasons: HashMap<String, String>,
}
