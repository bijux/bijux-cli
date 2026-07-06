use bijux_dag_core::{
    BranchSpec, ContainerSpec, DagBuilder, Effect, Graph, NodeBuilder, NodeKind, OutputKind,
    SemanticNodeKind, TriggerRule,
};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

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
    pub fn shell_node<S: AsRef<str>>(
        self,
        id: &str,
        inputs: &[S],
        argv: &[S],
        output_path: &str,
    ) -> Self {
        let mut builder = NodeBuilder::new(id, NodeKind::Shell)
            .output("out", output_path)
            .effect(Effect::Filesystem);
        for input in inputs {
            builder = builder.input(input.as_ref());
        }
        builder = builder.param_literal(json!({
            "argv": argv.iter().map(|value| Value::String(value.as_ref().to_string())).collect::<Vec<_>>()
        }));
        self.node(builder.build())
    }

    #[must_use]
    pub fn branch_node<S: AsRef<str>>(
        self,
        id: &str,
        inputs: &[S],
        argv: &[S],
        decisions: &[S],
        default_decision: &str,
    ) -> Self {
        let mut builder = NodeBuilder::new(id, NodeKind::Shell)
            .semantic_kind(SemanticNodeKind::Branch)
            .output("decision", &format!("{id}/decision.txt"))
            .effect(Effect::Filesystem)
            .trigger_rule(TriggerRule::AllSuccess)
            .branch(BranchSpec {
                decisions: decisions
                    .iter()
                    .map(|value| value.as_ref().to_string())
                    .collect::<Vec<_>>(),
                default_decision: Some(default_decision.to_string()),
                decision_output: "decision".to_string(),
            });
        for input in inputs {
            builder = builder.input(input.as_ref());
        }
        builder = builder.param_literal(json!({
            "argv": argv.iter().map(|value| Value::String(value.as_ref().to_string())).collect::<Vec<_>>()
        }));
        self.node(builder.build())
    }

    #[must_use]
    pub fn container_node<S: AsRef<str>>(
        self,
        id: &str,
        image: &str,
        argv: &[S],
        output_path: &str,
    ) -> Self {
        let mut node = NodeBuilder::new(id, NodeKind::Container)
            .output("out", output_path)
            .effect(Effect::Filesystem)
            .build();
        node.container = Some(ContainerSpec {
            image: image.to_string(),
            argv: argv.iter().map(|value| value.as_ref().to_string()).collect::<Vec<_>>(),
            env_allowlist: Vec::new(),
            workdir: Some("/workspace".to_string()),
            engine: "docker".to_string(),
        });
        self.node(node)
    }

    #[must_use]
    pub fn external_node(self, id: &str, adapter_kind: &str, output_path: &str) -> Self {
        self.node(
            NodeBuilder::new(id, NodeKind::External(adapter_kind.to_string()))
                .output("out", output_path)
                .effect(Effect::Filesystem)
                .param_literal(json!({"mode":"success"}))
                .build(),
        )
    }

    #[must_use]
    pub fn node(mut self, node: bijux_dag_core::Node) -> Self {
        self.builder = self.builder.node(node);
        self
    }

    #[must_use]
    pub fn edge(mut self, from_node: &str, from_port: &str, to_node: &str, to_port: &str) -> Self {
        self.builder = self.builder.edge(from_node, from_port, to_node, to_port);
        self
    }

    pub fn build(self) -> Graph {
        self.builder.build()
    }

    pub fn to_pretty_json(self) -> String {
        serde_json::to_string_pretty(&self.build()).expect("serialize fixture graph")
    }
}

pub fn graph_map_reduce_fixture() -> Graph {
    DagFixture::new()
        .const_node("seed", json!({"items":["a","b","c"]}))
        .shell_node(
            "map_left",
            &["in"],
            &["/bin/sh", "-c", "printf left > ../outputs/map_left.txt"],
            "map_left.txt",
        )
        .shell_node(
            "map_mid",
            &["in"],
            &["/bin/sh", "-c", "printf mid > ../outputs/map_mid.txt"],
            "map_mid.txt",
        )
        .shell_node(
            "map_right",
            &["in"],
            &["/bin/sh", "-c", "printf right > ../outputs/map_right.txt"],
            "map_right.txt",
        )
        .shell_node(
            "reduce",
            &["left", "mid", "right"],
            &[
                "/bin/sh",
                "-c",
                "printf '%s-%s-%s' \"$(cat ../inputs/map_left/left)\" \"$(cat ../inputs/map_mid/mid)\" \"$(cat ../inputs/map_right/right)\" > ../outputs/reduce.txt",
            ],
            "reduce.txt",
        )
        .edge("seed", "out", "map_left", "in")
        .edge("seed", "out", "map_mid", "in")
        .edge("seed", "out", "map_right", "in")
        .edge("map_left", "out", "reduce", "left")
        .edge("map_mid", "out", "reduce", "mid")
        .edge("map_right", "out", "reduce", "right")
        .build()
}

fn reduce_collection_command(output_name: &str) -> String {
    format!(
        "python3 -c \"import json, pathlib; manifest=json.load(open('../inputs/reduce.collection.json')); values=[]; \
base=pathlib.Path('../inputs'); collect=lambda rel: sorted((base / rel).rglob('value.txt')) if (base / rel).is_dir() else [base / rel]; \
paths=[]; [paths.extend(collect(item['local_path'])) for item in manifest['items'] if item.get('local_path')]; \
values=[path.read_text() for path in paths]; \
(pathlib.Path('../outputs') / '{output_name}').write_text(','.join(values))\""
    )
}

pub fn graph_semantic_map_reduce_fixture() -> Graph {
    DagFixture::new()
        .node(
            NodeBuilder::new("seed", NodeKind::Const)
                .output("out", "seed/out.json")
                .param_literal(json!({"value": ["alpha", "beta", "gamma"]}))
                .build(),
        )
        .node(
            {
                let mut node = NodeBuilder::new("map", NodeKind::Shell)
                .semantic_kind(SemanticNodeKind::Map)
                .input("in")
                .output("out", "mapped")
                .effect(Effect::Filesystem)
                .param_literal(json!({
                    "argv": [
                        "/bin/sh",
                        "-c",
                        "value=$(tr -d '\"' < ../inputs/seed/in); mkdir -p ../outputs/mapped; printf '%s' \"$value\" > ../outputs/mapped/value.txt"
                    ]
                }))
                .build();
                node.outputs[0].kind = OutputKind::Directory;
                node
            },
        )
        .node(
            NodeBuilder::new("reduce", NodeKind::Shell)
                .semantic_kind(SemanticNodeKind::Reduce)
                .input("mapped")
                .output("out", "reduce.txt")
                .effect(Effect::Filesystem)
                .param_literal(json!({
                    "argv": [
                        "/bin/sh",
                        "-c",
                        reduce_collection_command("reduce.txt")
                    ]
                }))
                .build(),
        )
        .edge("seed", "out", "map", "in")
        .edge("map", "out", "reduce", "mapped")
        .build()
}

pub fn graph_branch_join_fixture() -> Graph {
    DagFixture::new()
        .const_node("seed", json!("left"))
        .branch_node(
            "decide",
            &["in"],
            &["/bin/sh", "-c", "printf left > ../outputs/decide/decision.txt"],
            &["left", "right"],
            "left",
        )
        .const_node("left", json!("left-branch"))
        .const_node("right", json!("right-branch"))
        .shell_node(
            "join",
            &["lhs"],
            &["/bin/sh", "-c", "cat ../inputs/left/lhs > ../outputs/join.txt"],
            "join.txt",
        )
        .edge("seed", "out", "decide", "in")
        .edge("decide", "decision", "left", "in")
        .edge("decide", "decision", "right", "in")
        .edge("left", "out", "join", "lhs")
        .build()
}

fn normalize_snapshot_value(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, child) in map.iter_mut() {
                if matches!(
                    key.as_str(),
                    "created_unix_ms"
                        | "started_unix_ms"
                        | "finished_unix_ms"
                        | "unix_ms"
                        | "ts"
                        | "pid"
                ) {
                    *child = Value::from(0);
                } else if key == "tool_version" {
                    *child = Value::String("0.0.0+snapshot".to_string());
                } else {
                    normalize_snapshot_value(child);
                }
            }
        }
        Value::Array(values) => {
            for child in values {
                normalize_snapshot_value(child);
            }
        }
        _ => {}
    }
}

fn collect_files(root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    collect_files_recursive(root, root, &mut files);
    files.sort();
    files
}

fn collect_files_recursive(root: &Path, path: &Path, files: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(root, &path, files);
        } else if path.is_file() {
            let rel = path
                .strip_prefix(root)
                .expect("relative path")
                .to_string_lossy()
                .replace('\\', "/");
            files.push(rel);
        }
    }
}

fn read_json_if_exists(path: &Path) -> Option<Value> {
    let text = fs::read_to_string(path).ok()?;
    let mut value = serde_json::from_str::<Value>(&text).ok()?;
    normalize_snapshot_value(&mut value);
    Some(value)
}

fn read_text_or_binary(path: &Path) -> Option<Value> {
    let bytes = fs::read(path).ok()?;
    match String::from_utf8(bytes.clone()) {
        Ok(text) => Some(Value::String(text)),
        Err(_) => Some(json!({
            "binary_bytes": bytes.len(),
            "sha256": bijux_dag_artifacts::hash::sha256_hex(&bytes),
        })),
    }
}

pub fn collect_run_dir_snapshot(run_dir: &Path) -> Value {
    let mut node_traces = Map::new();
    let mut node_indexes = Map::new();
    let mut output_payloads = Map::new();
    let nodes_root = run_dir.join("nodes");
    if let Ok(entries) = fs::read_dir(&nodes_root) {
        for entry in entries.flatten() {
            let node_dir = entry.path();
            let Some(node_id) = node_dir.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            if let Some(trace) = read_json_if_exists(&node_dir.join("trace.json")) {
                node_traces.insert(node_id.to_string(), trace);
            }
            let mut indexes = Map::new();
            if let Some(inputs) = read_json_if_exists(&node_dir.join("inputs").join("index.json")) {
                indexes.insert("inputs".to_string(), inputs);
            }
            if let Some(outputs) = read_json_if_exists(&node_dir.join("outputs").join("index.json"))
            {
                indexes.insert("outputs".to_string(), outputs);
            }
            if !indexes.is_empty() {
                node_indexes.insert(node_id.to_string(), Value::Object(indexes));
            }
            if let Ok(output_entries) = fs::read_dir(node_dir.join("outputs")) {
                let mut node_outputs = Map::new();
                for output_entry in output_entries.flatten() {
                    let output_path = output_entry.path();
                    if output_path.file_name().and_then(|value| value.to_str())
                        == Some("index.json")
                    {
                        continue;
                    }
                    if let Some(payload) = read_text_or_binary(&output_path) {
                        let rel = output_path
                            .strip_prefix(&node_dir)
                            .expect("node relative output")
                            .to_string_lossy()
                            .replace('\\', "/");
                        node_outputs.insert(rel, payload);
                    }
                }
                if !node_outputs.is_empty() {
                    output_payloads.insert(node_id.to_string(), Value::Object(node_outputs));
                }
            }
        }
    }

    let mut root_outputs = Map::new();
    if let Ok(entries) = fs::read_dir(run_dir.join("outputs")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.file_name().and_then(|value| value.to_str()) == Some("index.json") {
                continue;
            }
            if let Some(payload) = read_text_or_binary(&path) {
                let rel = path
                    .strip_prefix(run_dir)
                    .expect("run relative output")
                    .to_string_lossy()
                    .replace('\\', "/");
                root_outputs.insert(rel, payload);
            }
        }
    }

    let run_log_events = fs::read_to_string(run_dir.join("run.log.jsonl"))
        .ok()
        .map(|raw| {
            raw.lines()
                .filter_map(|line| serde_json::from_str::<Value>(line).ok())
                .map(|mut value| {
                    normalize_snapshot_value(&mut value);
                    value
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    json!({
        "files": collect_files(run_dir),
        "manifest": read_json_if_exists(&run_dir.join("manifest.json")),
        "graph_snapshot": read_json_if_exists(&run_dir.join("graph.snapshot.json")),
        "outputs_index": read_json_if_exists(&run_dir.join("outputs").join("index.json")),
        "node_traces": node_traces,
        "node_indexes": node_indexes,
        "node_outputs": output_payloads,
        "root_outputs": root_outputs,
        "run_log_events": run_log_events,
    })
}

pub fn write_graph_fixture(path: &Path, graph: &Graph) {
    let payload = serde_json::to_vec_pretty(graph).expect("serialize graph fixture");
    fs::write(path, payload).expect("write graph fixture");
}

pub fn fixture_path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

pub fn update_or_assert_snapshot(path: &Path, actual: &Value) {
    let rendered =
        format!("{}\n", serde_json::to_string_pretty(actual).expect("render snapshot payload"));
    if std::env::var("BIJUX_UPDATE_GOLDENS").as_deref() == Ok("1") {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create snapshot parent");
        }
        fs::write(path, rendered).expect("write snapshot");
        return;
    }
    let expected = fs::read_to_string(path).expect("read snapshot");
    assert_eq!(rendered, expected, "snapshot drift at {}", path.display());
}

pub fn fixture_snapshot_path(manifest_dir: &str, relative_path: &str) -> PathBuf {
    PathBuf::from(manifest_dir).join(relative_path)
}
