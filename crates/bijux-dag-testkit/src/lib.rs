//! Shared test helpers for workspace crates.

use bijux_dag_artifacts::{Manifest, NodeTrace};
use bijux_dag_core::{
    Edge, Effect, FileOutput, Graph, Node, NodeKind, ParamValue, PortRef, RetryPolicy, SPEC_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn read_json(path: &Path) -> Value {
    let text = fs::read_to_string(path).expect("read json file");
    serde_json::from_str(&text).expect("parse json file")
}

pub fn workspace_root_from_manifest_dir(manifest_dir: &str) -> PathBuf {
    PathBuf::from(manifest_dir).join("../..")
}

pub fn load_evidence_registry(workspace_root: &Path) -> Value {
    read_json(&evidence_registry_path(workspace_root))
}

pub fn resolve_evidence_asset_by_id(registry: &Value, asset_id: &str) -> Value {
    resolve_evidence_asset_by_id_checked(registry, asset_id)
        .unwrap_or_else(|error| panic!("{error}"))
}

pub fn evidence_registry_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join("evidence/_meta/registries/evidence_registry.json")
}

pub fn load_evidence_registry_checked(workspace_root: &Path) -> Result<Value, String> {
    let path = evidence_registry_path(workspace_root);
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read evidence registry at {}: {error}", path.display()))?;
    serde_json::from_str(&text)
        .map_err(|error| format!("failed to parse evidence registry at {}: {error}", path.display()))
}

pub fn resolve_evidence_asset_by_id_checked(registry: &Value, asset_id: &str) -> Result<Value, String> {
    let assets = registry["assets"]
        .as_array()
        .expect("evidence registry assets array");
    for asset in assets {
        if asset["id"].as_str() == Some(asset_id) {
            return Ok(asset.clone());
        }
    }
    Err(format!(
        "evidence asset id not found: {asset_id}; verify evidence registry ownership and consumer mapping"
    ))
}

pub fn evidence_asset_ids(registry: &Value) -> BTreeSet<String> {
    registry["assets"]
        .as_array()
        .expect("evidence registry assets array")
        .iter()
        .filter_map(|asset| asset["id"].as_str().map(str::to_string))
        .collect()
}

pub fn graph_chain() -> Graph {
    graph_from_nodes(
        vec![const_node("a"), shell_node("b"), shell_node("c")],
        vec![("a", "out", "b", "in"), ("b", "out", "c", "in")],
    )
}

pub fn graph_diamond() -> Graph {
    graph_from_nodes(
        vec![
            const_node("a"),
            shell_node("b"),
            shell_node("c"),
            shell_node("d"),
        ],
        vec![
            ("a", "out", "b", "in"),
            ("a", "out", "c", "in"),
            ("b", "out", "d", "in"),
            ("c", "out", "d", "in"),
        ],
    )
}

pub fn graph_fanout() -> Graph {
    graph_from_nodes(
        vec![const_node("root"), shell_node("left"), shell_node("right")],
        vec![
            ("root", "out", "left", "in"),
            ("root", "out", "right", "in"),
        ],
    )
}

pub fn graph_disconnected() -> Graph {
    graph_from_nodes(vec![const_node("a"), const_node("b")], vec![])
}

pub fn graph_retry() -> Graph {
    let mut g = graph_chain();
    g.nodes[1].retry = RetryPolicy {
        max_attempts: 2,
        backoff_ms: 5,
    };
    g
}

pub fn graph_timeout() -> Graph {
    let mut g = graph_chain();
    g.nodes[1].timeout_ms = Some(1);
    g
}

pub fn graph_cache_hit() -> Graph {
    graph_chain()
}

pub fn graph_replay() -> Graph {
    graph_diamond()
}

pub fn graph_failure() -> Graph {
    let mut g = graph_chain();
    g.nodes[1].params = param_object(vec![(
        "argv",
        Value::Array(vec![
            Value::from("/bin/sh"),
            Value::from("-c"),
            Value::from("exit 2"),
        ]),
    )]);
    g
}

pub fn assert_manifest_eq_normalized(actual: &Manifest, expected: &Manifest) {
    let mut a = serde_json::to_value(actual).expect("serialize manifest");
    let mut b = serde_json::to_value(expected).expect("serialize manifest");
    normalize_manifest_timestamps(&mut a);
    normalize_manifest_timestamps(&mut b);
    assert_eq!(a, b, "manifest mismatch after normalization");
}

pub fn assert_trace_completeness(traces: &[NodeTrace], expected_nodes: &[&str]) {
    let actual: BTreeSet<String> = traces.iter().map(|t| t.node_id.clone()).collect();
    let expected: BTreeSet<String> = expected_nodes.iter().map(|v| (*v).to_string()).collect();
    assert_eq!(actual, expected, "trace node coverage mismatch");
    for trace in traces {
        assert!(!trace.status.is_empty(), "trace status is empty");
        assert!(
            trace.finished_unix_ms >= trace.started_unix_ms,
            "trace timing is invalid"
        );
    }
}

pub fn assert_node_event_sequence(statuses: &[&str]) {
    let order = ["queued", "ready", "running", "succeeded"];
    let mut cursor = 0usize;
    for status in statuses {
        while cursor < order.len() && order[cursor] != *status {
            cursor += 1;
        }
        assert!(
            cursor < order.len(),
            "illegal status sequence element: {status}"
        );
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CliCommandResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn run_cli_in_temp_repo(args: &[&str]) -> CliCommandResult {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = Command::new("cargo")
        .current_dir(temp.path())
        .env("CARGO_TARGET_DIR", temp.path().join("artifacts/target"))
        .args(args)
        .output()
        .expect("run cli");
    CliCommandResult {
        exit_code: output.status.code().unwrap_or(1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

pub fn create_corrupted_run_dir(base: &Path, kind: &str) -> PathBuf {
    let run = base.join("run-corrupt");
    fs::create_dir_all(run.join("nodes").join("n1")).expect("create run dir");
    fs::write(run.join("manifest.json"), "{}\n").expect("write manifest");
    fs::write(run.join("nodes").join("n1").join("trace.json"), "{}\n").expect("write trace");
    match kind {
        "truncated_manifest" => {
            fs::write(run.join("manifest.json"), "{\"run_id\":\"x\"").expect("truncate manifest");
        }
        "missing_trace" => {
            fs::remove_file(run.join("nodes").join("n1").join("trace.json")).expect("remove trace");
        }
        "tampered_outputs_index" => {
            fs::create_dir_all(run.join("outputs")).expect("create outputs dir");
            fs::write(
                run.join("outputs").join("index.json"),
                "{\"files\":[{\"path\":\"../x\"}]}",
            )
            .expect("tamper outputs index");
        }
        _ => {}
    }
    run
}

fn graph_from_nodes(nodes: Vec<Node>, edges: Vec<(&str, &str, &str, &str)>) -> Graph {
    Graph {
        spec: SPEC_VERSION.to_string(),
        meta: None,
        inputs: serde_json::Map::new(),
        nondeterminism_allowed: false,
        nodes,
        edges: edges
            .into_iter()
            .map(|(from_node, from_port, to_node, to_port)| Edge {
                from: PortRef {
                    node_id: from_node.to_string(),
                    port: from_port.to_string(),
                },
                to: PortRef {
                    node_id: to_node.to_string(),
                    port: to_port.to_string(),
                },
            })
            .collect(),
    }
}

fn const_node(id: &str) -> Node {
    Node {
        id: id.to_string(),
        kind: NodeKind::Const,
        inputs: vec![],
        outputs: vec![FileOutput {
            name: "out".to_string(),
            path: format!("out_{id}"),
        }],
        params: param_object(vec![("value", Value::from("ok"))]),
        container: None,
        timeout_ms: None,
        resources: None,
        tags: vec![],
        retry: RetryPolicy::default(),
        effects: vec![],
        env_allowlist: vec![],
        group: None,
    }
}

fn shell_node(id: &str) -> Node {
    Node {
        id: id.to_string(),
        kind: NodeKind::Shell,
        inputs: vec!["in".to_string()],
        outputs: vec![FileOutput {
            name: "out".to_string(),
            path: format!("out_{id}"),
        }],
        params: param_object(vec![(
            "argv",
            Value::Array(vec![
                Value::from("/bin/sh"),
                Value::from("-c"),
                Value::from(format!("echo ok > ../outputs/out_{id}")),
            ]),
        )]),
        container: None,
        timeout_ms: None,
        resources: None,
        tags: vec![],
        retry: RetryPolicy::default(),
        effects: vec![Effect::Filesystem],
        env_allowlist: vec![],
        group: None,
    }
}

fn param_object(items: Vec<(&str, Value)>) -> ParamValue {
    let mut map = BTreeMap::new();
    for (k, v) in items {
        map.insert(k.to_string(), ParamValue::Literal(v));
    }
    ParamValue::Object(map)
}

fn normalize_manifest_timestamps(value: &mut Value) {
    if let Some(obj) = value.as_object_mut() {
        for key in ["created_unix_ms", "started_unix_ms", "finished_unix_ms"] {
            if obj.contains_key(key) {
                obj.insert(key.to_string(), Value::from(0));
            }
        }
    }
}
