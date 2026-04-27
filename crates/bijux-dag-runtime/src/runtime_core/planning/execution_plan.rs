use bijux_dag_core::{FileOutput, Node, RetryPolicy};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedNode {
    pub id: String,
    pub kind: String,
    pub deps: Vec<String>,
    pub outputs: Vec<FileOutput>,
    pub retry: RetryPolicy,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedDependency {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub planner_contract_version: String,
    pub graph_fingerprint: String,
    pub planner_fingerprint: String,
    pub execution_fingerprint: String,
    pub evidence_fingerprint: String,
    pub planned_nodes: Vec<PlannedNode>,
    pub planned_dependencies: Vec<PlannedDependency>,
    pub diagnostics: Vec<String>,
    // Compatibility bridge for existing runtime engine surfaces.
    pub nodes: Vec<Node>,
    pub order: Vec<String>,
    pub dep_map: HashMap<String, BTreeSet<String>>,
    pub indegree: HashMap<String, usize>,
    pub adj: HashMap<String, Vec<String>>,
    pub filter_reasons: HashMap<String, String>,
}
