#![allow(dead_code)]

use serde::Deserialize;
use serde_json::Value;
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

pub fn workspace_root_from_manifest_dir(manifest_dir: &str) -> PathBuf {
    PathBuf::from(manifest_dir).join("../..")
}

pub fn load_workspace_fixture_text(manifest_dir: &str, relative_path: &str) -> String {
    let workspace_root = workspace_root_from_manifest_dir(manifest_dir);
    let path = resolve_workspace_fixture_path(&workspace_root, relative_path);
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("failed to read workspace fixture {}: {error}", path.display())
    })
}

pub fn load_workspace_fixture_json(manifest_dir: &str, relative_path: &str) -> Value {
    let payload = load_workspace_fixture_text(manifest_dir, relative_path);
    serde_json::from_str(&payload)
        .unwrap_or_else(|error| panic!("failed to parse fixture json {relative_path}: {error}"))
}

pub fn load_workspace_fixture_typed<T: for<'de> Deserialize<'de>>(
    manifest_dir: &str,
    relative_path: &str,
) -> T {
    let payload = load_workspace_fixture_text(manifest_dir, relative_path);
    serde_json::from_str(&payload)
        .unwrap_or_else(|error| panic!("failed to parse fixture type {relative_path}: {error}"))
}

pub fn load_bundle_fixture_json(manifest_dir: &str, relative_path: &str) -> Value {
    load_workspace_fixture_json(manifest_dir, relative_path)
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
