use bijux_dag_core::{
    BranchPathAnalysis, BranchSpec, CacheBehavior, EdgeKind, Effect, FileOutput, Node,
    NodeIoContract, Resources, RetryPolicy, SemanticNodeKind, TriggerRule,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedNode {
    pub id: String,
    pub kind: String,
    pub executor_kind: String,
    pub semantic_kind: SemanticNodeKind,
    pub deps: Vec<String>,
    pub io_contract: NodeIoContract,
    pub outputs: Vec<FileOutput>,
    pub side_effects: Vec<Effect>,
    pub retry: RetryPolicy,
    pub cache: CacheBehavior,
    pub trigger_rule: TriggerRule,
    pub timeout_ms: Option<u64>,
    pub resources: Option<Resources>,
    pub branch: Option<BranchSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedDependency {
    pub id: Option<String>,
    pub kind: EdgeKind,
    pub decision: Option<String>,
    pub from: String,
    pub from_port: String,
    pub to: String,
    pub to_port: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub planner_contract_version: String,
    pub graph_fingerprint: String,
    pub planner_fingerprint: String,
    pub execution_fingerprint: String,
    pub evidence_fingerprint: String,
    pub requested_selectors: Vec<String>,
    pub dependency_closure_enabled: bool,
    pub planned_nodes: Vec<PlannedNode>,
    pub planned_dependencies: Vec<PlannedDependency>,
    pub branch_paths: Vec<BranchPathAnalysis>,
    pub diagnostics: Vec<String>,
    // Compatibility bridge for existing runtime engine surfaces.
    pub nodes: Vec<Node>,
    pub order: Vec<String>,
    pub dep_map: HashMap<String, BTreeSet<String>>,
    pub indegree: HashMap<String, usize>,
    pub adj: HashMap<String, Vec<String>>,
    pub filter_reasons: HashMap<String, String>,
}
