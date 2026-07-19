#![allow(dead_code)]

use bijux_dag_core::{
    BranchSpec, DagBuilder, Effect, Graph, NodeBuilder, NodeKind, SemanticNodeKind, TriggerRule,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub fn branch_semantics_graph_json() -> &'static str {
    r#"{
      "spec":"bijux-dag/v0.1",
      "meta":{"name":"branch-contract","owners":[],"tags":[]},
      "nodes":[
        {"id":"seed","kind":"const","inputs":[],"outputs":[{"name":"out","path":"seed/out"}],"params":{"value":1}},
        {
          "id":"decide",
          "kind":"shell",
          "semantic_kind":"branch",
          "inputs":["in"],
          "outputs":[{"name":"decision","path":"decide/decision.txt"}],
          "effects":["filesystem"],
          "params":{"argv":["echo","left"]},
          "branch":{"decisions":["left","right"],"default_decision":"left","decision_output":"decision"}
        },
        {"id":"left","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"left/out"}],"params":{"value":"left"},"trigger_rule":"any_success"},
        {"id":"right","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"right/out"}],"params":{"value":"right"},"trigger_rule":"any_success"},
        {"id":"join","kind":"shell","inputs":["lhs"],"outputs":[{"name":"out","path":"join/out"}],"params":{"argv":["echo","join"]},"effects":["filesystem"]}
      ],
      "edges":[
        {"id":"seed-to-decide","from":{"node_id":"seed","port":"out"},"to":{"node_id":"decide","port":"in"}},
        {"id":"branch-left","kind":"conditional","decision":"left","from":{"node_id":"decide","port":"decision"},"to":{"node_id":"left","port":"in"}},
        {"id":"branch-right","kind":"conditional","decision":"right","from":{"node_id":"decide","port":"decision"},"to":{"node_id":"right","port":"in"}},
        {"id":"left-to-join","kind":"control","from":{"node_id":"left","port":"out"},"to":{"node_id":"join","port":"lhs"}}
      ]
    }"#
}

pub fn load_workspace_fixture_text(manifest_dir: &str, relative_path: &str) -> String {
    let workspace_root = workspace_root_from_manifest_dir(manifest_dir);
    let path = resolve_workspace_fixture_path(&workspace_root, relative_path);
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("failed to read workspace fixture {}: {error}", path.display())
    })
}

pub struct DagFixture {
    builder: DagBuilder,
}

impl Default for DagFixture {
    fn default() -> Self {
        Self::new()
    }
}

impl DagFixture {
    #[must_use]
    pub fn new() -> Self {
        Self { builder: DagBuilder::new() }
    }

    #[must_use]
    pub fn const_node(self, id: &str, value: Value) -> Self {
        self.node(
            NodeBuilder::new(id, NodeKind::Const)
                .output("out", &format!("{id}/out.json"))
                .param_literal(value)
                .build(),
        )
    }

    #[must_use]
    pub fn edge(mut self, from_node: &str, from_port: &str, to_node: &str, to_port: &str) -> Self {
        self.builder = self.builder.edge(from_node, from_port, to_node, to_port);
        self
    }

    #[must_use]
    pub fn node(mut self, node: bijux_dag_core::Node) -> Self {
        self.builder = self.builder.node(node);
        self
    }

    pub fn build(self) -> Graph {
        self.builder.build()
    }
}

pub fn graph_branch_join_fixture() -> Graph {
    DagFixture::new()
        .const_node("seed", json!("left"))
        .node(
            NodeBuilder::new("decide", NodeKind::Shell)
                .semantic_kind(SemanticNodeKind::Branch)
                .input("in")
                .output("decision", "decide/decision.txt")
                .effect(Effect::Filesystem)
                .trigger_rule(TriggerRule::AllSuccess)
                .branch(BranchSpec {
                    decisions: vec!["left".to_string(), "right".to_string()],
                    default_decision: Some("left".to_string()),
                    decision_output: "decision".to_string(),
                })
                .param_literal(
                    json!({"argv":["/bin/sh","-c","printf left > ../outputs/decide/decision.txt"]}),
                )
                .build(),
        )
        .const_node("left", json!("left-branch"))
        .const_node("right", json!("right-branch"))
        .node(
            NodeBuilder::new("join", NodeKind::Shell)
                .input("lhs")
                .output("out", "join.txt")
                .effect(Effect::Filesystem)
                .param_literal(
                    json!({"argv":["/bin/sh","-c","cat ../inputs/left/lhs > ../outputs/join.txt"]}),
                )
                .build(),
        )
        .edge("seed", "out", "decide", "in")
        .edge("decide", "decision", "left", "in")
        .edge("decide", "decision", "right", "in")
        .edge("left", "out", "join", "lhs")
        .build()
}

fn workspace_root_from_manifest_dir(manifest_dir: &str) -> PathBuf {
    PathBuf::from(manifest_dir).join("../..")
}

fn resolve_workspace_fixture_path(workspace_root: &Path, relative_path: &str) -> PathBuf {
    let canonical = workspace_root.join(relative_path);
    if canonical.exists() {
        return canonical;
    }

    if let Some(remapped) = remap_legacy_evidence_path(relative_path) {
        let remapped_path = workspace_root.join(remapped);
        if remapped_path.exists() {
            return remapped_path;
        }
    }

    canonical
}

fn remap_legacy_evidence_path(relative_path: &str) -> Option<String> {
    let normalized = relative_path.strip_prefix("./").unwrap_or(relative_path);
    let remainder = normalized.strip_prefix("evidence/")?;
    if remainder.starts_with("dag/") {
        return None;
    }
    Some(format!("evidence/dag/{remainder}"))
}
