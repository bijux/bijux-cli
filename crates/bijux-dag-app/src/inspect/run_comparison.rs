use crate::run_views::{inspect_summary, resolve_run_dir};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ExactFieldComparison {
    a: Value,
    b: Value,
    equal: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ExactMapComparison {
    a: Value,
    b: Value,
    equal: Option<bool>,
    changed_keys: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ExactListComparison {
    a: Value,
    b: Value,
    equal: Option<bool>,
    changed_items: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct MeaningfulDivergence {
    dimension: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject: Option<String>,
    a: Value,
    b: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RunComparisonMaterial {
    graph_fingerprint: Value,
    execution_fingerprint: Value,
    input_values: Option<BTreeMap<String, Value>>,
    selected_nodes: Option<Vec<String>>,
    node_statuses: Option<BTreeMap<String, Value>>,
    output_hashes: Option<BTreeMap<String, Value>>,
}

pub fn runs_compare(root: &Path, run_a: &str, run_b: &str) -> Result<Value, std::io::Error> {
    let run_dir_a = resolve_run_dir(root, run_a);
    let run_dir_b = resolve_run_dir(root, run_b);
    let summary_a = inspect_summary(&run_dir_a)?;
    let summary_b = inspect_summary(&run_dir_b)?;
    let material_a = load_run_comparison_material(&run_dir_a)?;
    let material_b = load_run_comparison_material(&run_dir_b)?;

    let graph_fingerprint = exact_field_comparison(
        material_a.graph_fingerprint.clone(),
        material_b.graph_fingerprint.clone(),
    );
    let execution_fingerprint = exact_field_comparison(
        material_a.execution_fingerprint.clone(),
        material_b.execution_fingerprint.clone(),
    );
    let input_values = exact_map_comparison(
        material_a.input_values.clone(),
        material_b.input_values.clone(),
        "changed_inputs",
    );
    let selected_nodes = exact_list_comparison(
        material_a.selected_nodes.clone(),
        material_b.selected_nodes.clone(),
        "changed_nodes",
    );
    let node_statuses = exact_map_comparison(
        material_a.node_statuses.clone(),
        material_b.node_statuses.clone(),
        "changed_nodes",
    );
    let output_hashes = exact_map_comparison(
        material_a.output_hashes.clone(),
        material_b.output_hashes.clone(),
        "changed_outputs",
    );

    Ok(json!({
        "run_a": run_a,
        "run_b": run_b,
        "status": {"a": summary_a.get("status").cloned().unwrap_or(Value::Null), "b": summary_b.get("status").cloned().unwrap_or(Value::Null)},
        "retries": {"a": summary_a.get("retry_count").cloned().unwrap_or(Value::Null), "b": summary_b.get("retry_count").cloned().unwrap_or(Value::Null)},
        "cache_hits": {"a": summary_a.get("cache_hits").cloned().unwrap_or(Value::Null), "b": summary_b.get("cache_hits").cloned().unwrap_or(Value::Null)},
        "artifact_count": {"a": summary_a.get("artifact_count").cloned().unwrap_or(Value::Null), "b": summary_b.get("artifact_count").cloned().unwrap_or(Value::Null)},
        "timing_ms": {"a": summary_a.get("timing_ms").cloned().unwrap_or(Value::Null), "b": summary_b.get("timing_ms").cloned().unwrap_or(Value::Null)},
        "graph_fingerprint": graph_fingerprint,
        "execution_fingerprint": execution_fingerprint,
        "input_values": input_values,
        "selected_nodes": selected_nodes,
        "node_statuses": node_statuses,
        "output_hashes": output_hashes,
        "first_meaningful_divergence": first_meaningful_divergence(&material_a, &material_b),
    }))
}

fn load_run_comparison_material(run_dir: &Path) -> Result<RunComparisonMaterial, std::io::Error> {
    let manifest = read_json_conservative(&run_dir.join("manifest.json"))?;
    let run_snapshot = read_optional_json_conservative(&run_dir.join("run.snapshot.json"))?;

    let graph_fingerprint = manifest
        .get("graph_fingerprint")
        .cloned()
        .filter(|value| !value.is_null())
        .or_else(|| {
            run_snapshot
                .as_ref()
                .and_then(|snapshot| snapshot.get("graph_fingerprint").cloned())
                .filter(|value| !value.is_null())
        })
        .or_else(|| {
            snapshot_payload_path(run_dir)
                .and_then(|path| read_json_conservative(&path).ok())
                .and_then(|snapshot| snapshot.get("graph_fingerprint").cloned())
                .filter(|value| !value.is_null())
        })
        .unwrap_or(Value::Null);
    let execution_fingerprint =
        manifest.get("execution_fingerprint").cloned().unwrap_or(Value::Null);
    let input_values = manifest
        .get("run_metadata")
        .and_then(|metadata| metadata.get("graph_inputs"))
        .and_then(json_object_to_btreemap);
    let selected_nodes = run_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.get("selected_nodes"))
        .and_then(Value::as_array)
        .map(|items| {
            let mut selected =
                items.iter().filter_map(Value::as_str).map(ToOwned::to_owned).collect::<Vec<_>>();
            selected.sort();
            selected.dedup();
            selected
        });
    let node_statuses = load_node_statuses(run_dir)?;
    let output_hashes = load_output_hashes(run_dir)?;

    Ok(RunComparisonMaterial {
        graph_fingerprint,
        execution_fingerprint,
        input_values: if manifest.is_null() {
            None
        } else {
            Some(input_values.unwrap_or_default())
        },
        selected_nodes,
        node_statuses,
        output_hashes,
    })
}

fn exact_field_comparison(a: Value, b: Value) -> ExactFieldComparison {
    let equal = if a.is_null() || b.is_null() { None } else { Some(a == b) };
    ExactFieldComparison { a, b, equal }
}

fn exact_map_comparison(
    a: Option<BTreeMap<String, Value>>,
    b: Option<BTreeMap<String, Value>>,
    changed_keys_name: &str,
) -> Value {
    let equal = match (&a, &b) {
        (Some(left), Some(right)) => Some(left == right),
        _ => None,
    };
    let changed_keys = match (&a, &b) {
        (Some(left), Some(right)) => {
            let keys = left
                .keys()
                .chain(right.keys())
                .cloned()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .filter(|key| left.get(key) != right.get(key))
                .collect::<Vec<_>>();
            Some(keys)
        }
        _ => None,
    };
    let mut payload = serde_json::to_value(ExactMapComparison {
        a: a.map(map_to_value).unwrap_or(Value::Null),
        b: b.map(map_to_value).unwrap_or(Value::Null),
        equal,
        changed_keys: None,
    })
    .unwrap_or(Value::Null);
    if let Value::Object(ref mut object) = payload {
        object.insert(
            changed_keys_name.to_string(),
            changed_keys.map_or(Value::Null, |keys| json!(keys)),
        );
    }
    payload
}

fn exact_list_comparison(
    a: Option<Vec<String>>,
    b: Option<Vec<String>>,
    changed_items_name: &str,
) -> Value {
    let equal = match (&a, &b) {
        (Some(left), Some(right)) => Some(left == right),
        _ => None,
    };
    let changed_items = match (&a, &b) {
        (Some(left), Some(right)) => {
            let items = left
                .iter()
                .chain(right.iter())
                .cloned()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .filter(|item| {
                    left.binary_search(item).is_err() || right.binary_search(item).is_err()
                })
                .collect::<Vec<_>>();
            Some(items)
        }
        _ => None,
    };
    let mut payload = serde_json::to_value(ExactListComparison {
        a: a.map(|items| json!(items)).unwrap_or(Value::Null),
        b: b.map(|items| json!(items)).unwrap_or(Value::Null),
        equal,
        changed_items: None,
    })
    .unwrap_or(Value::Null);
    if let Value::Object(ref mut object) = payload {
        object.insert(
            changed_items_name.to_string(),
            changed_items.map_or(Value::Null, |items| json!(items)),
        );
    }
    payload
}

fn first_meaningful_divergence(a: &RunComparisonMaterial, b: &RunComparisonMaterial) -> Value {
    if !a.graph_fingerprint.is_null()
        && !b.graph_fingerprint.is_null()
        && a.graph_fingerprint != b.graph_fingerprint
    {
        return json!(MeaningfulDivergence {
            dimension: "graph_fingerprint".to_string(),
            subject: None,
            a: a.graph_fingerprint.clone(),
            b: b.graph_fingerprint.clone(),
        });
    }
    if !a.execution_fingerprint.is_null()
        && !b.execution_fingerprint.is_null()
        && a.execution_fingerprint != b.execution_fingerprint
    {
        return json!(MeaningfulDivergence {
            dimension: "execution_fingerprint".to_string(),
            subject: None,
            a: a.execution_fingerprint.clone(),
            b: b.execution_fingerprint.clone(),
        });
    }
    if let Some(divergence) = first_map_divergence("input_values", &a.input_values, &b.input_values)
    {
        return json!(divergence);
    }
    if let Some(divergence) =
        first_list_divergence("selected_nodes", &a.selected_nodes, &b.selected_nodes)
    {
        return json!(divergence);
    }
    if let Some(divergence) =
        first_map_divergence("node_statuses", &a.node_statuses, &b.node_statuses)
    {
        return json!(divergence);
    }
    if let Some(divergence) =
        first_map_divergence("output_hashes", &a.output_hashes, &b.output_hashes)
    {
        return json!(divergence);
    }
    Value::Null
}

fn first_map_divergence(
    dimension: &str,
    a: &Option<BTreeMap<String, Value>>,
    b: &Option<BTreeMap<String, Value>>,
) -> Option<MeaningfulDivergence> {
    let (Some(left), Some(right)) = (a, b) else {
        return None;
    };
    left.keys().chain(right.keys()).cloned().collect::<BTreeSet<_>>().into_iter().find_map(|key| {
        let value_a = left.get(&key).cloned().unwrap_or(Value::Null);
        let value_b = right.get(&key).cloned().unwrap_or(Value::Null);
        (value_a != value_b).then_some(MeaningfulDivergence {
            dimension: dimension.to_string(),
            subject: Some(key),
            a: value_a,
            b: value_b,
        })
    })
}

fn first_list_divergence(
    dimension: &str,
    a: &Option<Vec<String>>,
    b: &Option<Vec<String>>,
) -> Option<MeaningfulDivergence> {
    let (Some(left), Some(right)) = (a, b) else {
        return None;
    };
    left.iter().chain(right.iter()).cloned().collect::<BTreeSet<_>>().into_iter().find_map(|item| {
        let left_has = left.binary_search(&item).is_ok();
        let right_has = right.binary_search(&item).is_ok();
        (left_has != right_has).then_some(MeaningfulDivergence {
            dimension: dimension.to_string(),
            subject: Some(item),
            a: json!(left_has),
            b: json!(right_has),
        })
    })
}

fn load_node_statuses(run_dir: &Path) -> Result<Option<BTreeMap<String, Value>>, std::io::Error> {
    let mut statuses = BTreeMap::new();
    let nodes_dir = run_dir.join("nodes");
    if !nodes_dir.exists() {
        return Ok(Some(statuses));
    }
    for entry in fs::read_dir(nodes_dir)? {
        let entry = entry?;
        let node_id = entry.file_name().to_string_lossy().to_string();
        let trace_path = entry.path().join("trace.json");
        if !trace_path.exists() {
            continue;
        }
        let trace = read_json_conservative(&trace_path)?;
        let Some(status) = trace.get("status").cloned().filter(|value| !value.is_null()) else {
            return Ok(None);
        };
        statuses.insert(node_id, status);
    }
    Ok(Some(statuses))
}

fn load_output_hashes(run_dir: &Path) -> Result<Option<BTreeMap<String, Value>>, std::io::Error> {
    let nodes_dir = run_dir.join("nodes");
    if !nodes_dir.exists() {
        return Ok(Some(BTreeMap::new()));
    }
    let mut hashes = BTreeMap::new();
    for entry in fs::read_dir(nodes_dir)? {
        let entry = entry?;
        let node_id = entry.file_name().to_string_lossy().to_string();
        let index_path = entry.path().join("outputs").join("index.json");
        if !index_path.exists() {
            continue;
        }
        let index = read_json_conservative(&index_path)?;
        let Some(files) = index.get("files").and_then(Value::as_array) else {
            return Ok(None);
        };
        for file in files {
            let Some(path) = file.get("path").and_then(Value::as_str) else {
                return Ok(None);
            };
            let Some(sha256) = file.get("sha256").cloned() else {
                return Ok(None);
            };
            hashes.insert(format!("{node_id}:{path}"), sha256);
        }
    }
    Ok(Some(hashes))
}

fn json_object_to_btreemap(value: &Value) -> Option<BTreeMap<String, Value>> {
    value.as_object().map(|object| {
        object.iter().map(|(key, value)| (key.clone(), value.clone())).collect::<BTreeMap<_, _>>()
    })
}

fn map_to_value(map: BTreeMap<String, Value>) -> Value {
    serde_json::to_value(map).unwrap_or(Value::Null)
}

fn read_optional_json_conservative(path: &Path) -> Result<Option<Value>, std::io::Error> {
    if !path.exists() {
        return Ok(None);
    }
    read_json_conservative(path).map(Some)
}

fn read_json_conservative(path: &Path) -> Result<Value, std::io::Error> {
    let payload = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&payload).unwrap_or(Value::Null))
}

fn snapshot_payload_path(run_dir: &Path) -> Option<PathBuf> {
    let current = run_dir.join("graph.snapshot.json");
    if current.exists() {
        return Some(current);
    }
    let legacy = run_dir.join("snapshot.json");
    if legacy.exists() {
        return Some(legacy);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::runs_compare;
    use serde_json::{json, Value};
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::Path;

    fn write_run(
        root: &Path,
        run_id: &str,
        manifest: serde_json::Value,
        run_snapshot: serde_json::Value,
        node_statuses: &[(&str, &str)],
        node_outputs: &[(&str, &str, &str)],
    ) {
        let run_dir = root.join(run_id);
        fs::create_dir_all(&run_dir).expect("create run");
        fs::create_dir_all(run_dir.join("outputs")).expect("create outputs");
        fs::write(
            run_dir.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest).expect("manifest"),
        )
        .expect("write manifest");
        fs::write(
            run_dir.join("run.snapshot.json"),
            serde_json::to_vec_pretty(&run_snapshot).expect("snapshot"),
        )
        .expect("write snapshot");
        fs::write(
            run_dir.join("graph.snapshot.json"),
            serde_json::to_vec_pretty(&json!({
                "graph": {"nodes": [], "edges": []},
                "graph_fingerprint": manifest["graph_fingerprint"].clone(),
            }))
            .expect("graph snapshot"),
        )
        .expect("write graph snapshot");
        fs::write(
            run_dir.join("outputs").join("index.json"),
            serde_json::to_vec_pretty(&json!({"files": []})).expect("outputs"),
        )
        .expect("write outputs");

        for (node_id, status) in node_statuses {
            let node_dir = run_dir.join("nodes").join(node_id);
            fs::create_dir_all(node_dir.join("outputs")).expect("create node outputs");
            fs::write(
                node_dir.join("trace.json"),
                serde_json::to_vec_pretty(&json!({
                    "node_id": node_id,
                    "status": status,
                    "attempt": 1,
                    "started_unix_ms": 1,
                    "finished_unix_ms": 2,
                }))
                .expect("trace"),
            )
            .expect("write trace");
        }

        let mut outputs_by_node = BTreeMap::<String, Vec<serde_json::Value>>::new();
        for (node_id, path, sha256) in node_outputs {
            outputs_by_node.entry((*node_id).to_string()).or_default().push(json!({
                "name": path,
                "path": path,
                "kind": "file",
                "media_type": "text/plain",
                "size_bytes": 1,
                "sha256": sha256,
                "node_id": node_id,
                "node_fingerprint": format!("fp-{node_id}"),
            }));
        }
        for (node_id, files) in outputs_by_node {
            let node_dir = run_dir.join("nodes").join(node_id).join("outputs");
            fs::create_dir_all(&node_dir).expect("create output dir");
            fs::write(
                node_dir.join("index.json"),
                serde_json::to_vec_pretty(&json!({ "files": files })).expect("index"),
            )
            .expect("write output index");
        }
    }

    #[test]
    fn runs_compare_reports_first_meaningful_divergence_in_priority_order() {
        let tmp = tempfile::tempdir().expect("tempdir");
        write_run(
            tmp.path(),
            "run-a",
            json!({
                "run_id": "run-a",
                "status": "success",
                "graph_fingerprint": "graph-a",
                "execution_fingerprint": "exec-a",
                "run_metadata": {"graph_inputs": {"seed": 1}},
            }),
            json!({"selected_nodes": ["build"]}),
            &[("build", "success")],
            &[("build", "report.txt", "sha-a")],
        );
        write_run(
            tmp.path(),
            "run-b",
            json!({
                "run_id": "run-b",
                "status": "success",
                "graph_fingerprint": "graph-b",
                "execution_fingerprint": "exec-b",
                "run_metadata": {"graph_inputs": {"seed": 2}},
            }),
            json!({"selected_nodes": ["publish"]}),
            &[("publish", "failed")],
            &[("publish", "report.txt", "sha-b")],
        );

        let report = runs_compare(tmp.path(), "run-a", "run-b").expect("compare");
        assert_eq!(report["first_meaningful_divergence"]["dimension"], "graph_fingerprint");
        assert_eq!(report["graph_fingerprint"]["equal"], false);
        assert_eq!(report["execution_fingerprint"]["equal"], false);
        assert_eq!(report["input_values"]["changed_inputs"], json!(["seed"]));
        assert_eq!(report["selected_nodes"]["changed_nodes"], json!(["build", "publish"]));
        assert_eq!(report["node_statuses"]["changed_nodes"], json!(["build", "publish"]));
        assert_eq!(
            report["output_hashes"]["changed_outputs"],
            json!(["build:report.txt", "publish:report.txt"])
        );
    }

    #[test]
    fn runs_compare_degrades_unknown_fields_to_null_without_guessing() {
        let tmp = tempfile::tempdir().expect("tempdir");
        write_run(
            tmp.path(),
            "run-a",
            json!({
                "run_id": "run-a",
                "status": "success",
                "graph_fingerprint": "graph-a",
                "execution_fingerprint": "exec-a",
            }),
            json!({"selected_nodes": ["build"]}),
            &[("build", "success")],
            &[("build", "report.txt", "sha-a")],
        );
        let corrupt_run = tmp.path().join("run-b");
        fs::create_dir_all(corrupt_run.join("nodes").join("build").join("outputs"))
            .expect("create corrupt run");
        fs::create_dir_all(corrupt_run.join("outputs")).expect("create corrupt outputs");
        fs::write(corrupt_run.join("manifest.json"), "{bad").expect("manifest");
        fs::write(corrupt_run.join("run.snapshot.json"), "{bad").expect("snapshot");
        fs::write(corrupt_run.join("graph.snapshot.json"), "{bad").expect("graph snapshot");
        fs::write(corrupt_run.join("outputs").join("index.json"), "{\"files\":[]}")
            .expect("outputs");
        fs::write(corrupt_run.join("nodes").join("build").join("trace.json"), "{bad")
            .expect("trace");
        fs::write(
            corrupt_run.join("nodes").join("build").join("outputs").join("index.json"),
            "{bad",
        )
        .expect("output index");

        let report = runs_compare(tmp.path(), "run-a", "run-b").expect("compare");
        assert_eq!(report["graph_fingerprint"]["b"], Value::Null);
        assert_eq!(report["graph_fingerprint"]["equal"], Value::Null);
        assert_eq!(report["selected_nodes"]["b"], Value::Null);
        assert_eq!(report["selected_nodes"]["equal"], Value::Null);
        assert_eq!(report["output_hashes"]["b"], Value::Null);
        assert_eq!(report["output_hashes"]["equal"], Value::Null);
        assert_eq!(report["first_meaningful_divergence"], Value::Null);
    }
}
