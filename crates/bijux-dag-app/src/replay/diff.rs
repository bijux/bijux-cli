use bijux_dag_artifacts::OutputsIndex;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Debug, Serialize)]
pub struct RunDiff {
    pub manifest: BTreeMap<String, Value>,
    pub graph_fingerprint: Option<Value>,
    pub nodes: BTreeMap<String, NodeDiff>,
    pub outputs: BTreeMap<String, OutputDiff>,
    pub replay_equivalence: ReplayEquivalenceReport,
}

#[derive(Debug, Serialize)]
pub struct NodeDiff {
    pub status_a: Option<Value>,
    pub status_b: Option<Value>,
    pub fp_a: Option<Value>,
    pub fp_b: Option<Value>,
    pub branch_decision_a: Option<Value>,
    pub branch_decision_b: Option<Value>,
    pub container_image_a: Option<Value>,
    pub container_image_b: Option<Value>,
    pub container_digest_a: Option<Value>,
    pub container_digest_b: Option<Value>,
    pub adapter_binary_sha256_a: Option<Value>,
    pub adapter_binary_sha256_b: Option<Value>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReplaySafetyLevel {
    Equivalent,
    SafeWithDrift,
    Risky,
    Forbidden,
    IncompleteEvidence,
    Unsupported,
}

#[derive(Debug, Serialize)]
pub struct OutputDiff {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ReplayEquivalenceReport {
    pub equivalent: bool,
    pub safety_level: ReplaySafetyLevel,
    pub reasons: Vec<String>,
    pub reason_report: ReplayReasonReport,
    pub cause_groups: BTreeMap<String, usize>,
    pub evidence_gaps: Vec<String>,
    pub branch_decision_drift_nodes: Vec<String>,
    pub container_digest_drift_nodes: Vec<String>,
    pub adapter_binary_drift_nodes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ReplayReasonReport {
    pub summary: String,
    pub compared_dimensions: Vec<String>,
    pub mismatch_dimensions: Vec<String>,
}

#[allow(clippy::too_many_arguments)]
pub fn build_run_diff(
    manifest_a: Value,
    manifest_b: Value,
    graph_fp_a: String,
    graph_fp_b: String,
    nodes_a: &HashMap<String, Value>,
    nodes_b: &HashMap<String, Value>,
    outputs_a: &HashMap<String, OutputsIndex>,
    outputs_b: &HashMap<String, OutputsIndex>,
) -> RunDiff {
    let mut manifest_diff: BTreeMap<String, Value> = BTreeMap::new();
    let mut ignore = BTreeSet::new();
    ignore.insert("run_id");
    ignore.insert("created_unix_ms");
    ignore.insert("started_unix_ms");
    ignore.insert("finished_unix_ms");
    ignore.insert("run_metadata");
    if let (Some(a), Some(b)) = (manifest_a.as_object(), manifest_b.as_object()) {
        let mut keys = BTreeSet::new();
        for k in a.keys() {
            keys.insert(k.as_str());
        }
        for k in b.keys() {
            keys.insert(k.as_str());
        }
        for k in keys {
            if ignore.contains(k) {
                continue;
            }
            let va = a.get(k);
            let vb = b.get(k);
            if va != vb {
                manifest_diff.insert(k.to_string(), json!({ "a": va, "b": vb }));
            }
        }
    }

    let graph_fingerprint = if graph_fp_a == graph_fp_b {
        None
    } else {
        Some(json!({ "a": graph_fp_a, "b": graph_fp_b }))
    };

    let mut node_diff: BTreeMap<String, NodeDiff> = BTreeMap::new();
    let mut node_outcome_diff_count = 0usize;
    let mut branch_decision_drift_nodes = Vec::new();
    let mut container_digest_drift_nodes = Vec::new();
    let mut adapter_binary_drift_nodes = Vec::new();
    let mut all_nodes: BTreeSet<String> = BTreeSet::new();
    for k in nodes_a.keys() {
        all_nodes.insert(k.clone());
    }
    for k in nodes_b.keys() {
        all_nodes.insert(k.clone());
    }
    for node_id in all_nodes {
        let a = nodes_a.get(&node_id);
        let b = nodes_b.get(&node_id);
        let status_a = a.and_then(|v| v.get("status")).cloned();
        let status_b = b.and_then(|v| v.get("status")).cloned();
        let fp_a = a.and_then(|v| v.get("fingerprint")).cloned();
        let fp_b = b.and_then(|v| v.get("fingerprint")).cloned();
        let branch_decision_a = a.and_then(|v| v.get("branch_decision")).cloned();
        let branch_decision_b = b.and_then(|v| v.get("branch_decision")).cloned();
        let container_image_a =
            a.and_then(|v| v.get("container")).and_then(|v| v.get("image")).cloned();
        let container_image_b =
            b.and_then(|v| v.get("container")).and_then(|v| v.get("image")).cloned();
        let container_digest_a =
            a.and_then(|v| v.get("container")).and_then(|v| v.get("image_digest")).cloned();
        let container_digest_b =
            b.and_then(|v| v.get("container")).and_then(|v| v.get("image_digest")).cloned();
        let adapter_binary_sha256_a = a.and_then(|v| v.get("adapter_binary_sha256")).cloned();
        let adapter_binary_sha256_b = b.and_then(|v| v.get("adapter_binary_sha256")).cloned();
        let outcome_drift = status_a != status_b || fp_a != fp_b;
        let branch_drift = branch_decision_a != branch_decision_b;
        let container_digest_drift = container_image_a == container_image_b
            && container_image_a.is_some()
            && container_digest_a != container_digest_b;
        let adapter_binary_drift = adapter_binary_sha256_a.is_some()
            && adapter_binary_sha256_b.is_some()
            && adapter_binary_sha256_a != adapter_binary_sha256_b;
        if outcome_drift {
            node_outcome_diff_count += 1;
        }
        if branch_drift {
            branch_decision_drift_nodes.push(node_id.clone());
        }
        if container_digest_drift {
            container_digest_drift_nodes.push(node_id.clone());
        }
        if adapter_binary_drift {
            adapter_binary_drift_nodes.push(node_id.clone());
        }
        if outcome_drift || branch_drift || container_digest_drift || adapter_binary_drift {
            node_diff.insert(
                node_id,
                NodeDiff {
                    status_a,
                    status_b,
                    fp_a,
                    fp_b,
                    branch_decision_a,
                    branch_decision_b,
                    container_image_a,
                    container_image_b,
                    container_digest_a,
                    container_digest_b,
                    adapter_binary_sha256_a,
                    adapter_binary_sha256_b,
                },
            );
        }
    }

    let mut out_diff: BTreeMap<String, OutputDiff> = BTreeMap::new();
    let mut all_nodes: BTreeSet<String> = BTreeSet::new();
    for k in outputs_a.keys() {
        all_nodes.insert(k.clone());
    }
    for k in outputs_b.keys() {
        all_nodes.insert(k.clone());
    }
    for node_id in all_nodes {
        let a = outputs_a.get(&node_id).map(outputs_index_to_map);
        let b = outputs_b.get(&node_id).map(outputs_index_to_map);
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut changed = Vec::new();
        let map_a = a.as_ref().and_then(|v| v.as_object());
        let map_b = b.as_ref().and_then(|v| v.as_object());
        let mut keys = BTreeSet::new();
        if let Some(m) = map_a {
            for k in m.keys() {
                keys.insert(k.clone());
            }
        }
        if let Some(m) = map_b {
            for k in m.keys() {
                keys.insert(k.clone());
            }
        }
        for k in keys {
            let va = map_a.and_then(|m| m.get(&k));
            let vb = map_b.and_then(|m| m.get(&k));
            match (va, vb) {
                (None, Some(_)) => added.push(k),
                (Some(_), None) => removed.push(k),
                (Some(a), Some(b)) => {
                    if a != b {
                        changed.push(k);
                    }
                }
                _ => {}
            }
        }
        added.sort();
        removed.sort();
        changed.sort();
        if !(added.is_empty() && removed.is_empty() && changed.is_empty()) {
            out_diff.insert(node_id, OutputDiff { added, removed, changed });
        }
    }

    let mut reasons = Vec::new();
    let mut cause_groups: BTreeMap<String, usize> = BTreeMap::new();
    let evidence_gaps = Vec::new();
    let compared_dimensions = vec![
        "manifest".to_string(),
        "graph_fingerprint".to_string(),
        "nodes".to_string(),
        "outputs".to_string(),
        "branch_decisions".to_string(),
    ];
    let mut mismatch_dimensions = Vec::new();
    if !manifest_diff.is_empty() {
        reasons.push("manifest fields differ".to_string());
        mismatch_dimensions.push("manifest".to_string());
        cause_groups.insert("manifest_drift".to_string(), 1);
    }
    if graph_fingerprint.is_some() {
        reasons.push("graph fingerprint differs".to_string());
        mismatch_dimensions.push("graph_fingerprint".to_string());
        cause_groups.insert("graph_semantics".to_string(), 1);
    }
    if node_outcome_diff_count > 0 {
        reasons.push("node status or fingerprint differs".to_string());
        mismatch_dimensions.push("nodes".to_string());
        cause_groups.insert("node_outcomes".to_string(), node_outcome_diff_count);
    }
    if !branch_decision_drift_nodes.is_empty() {
        reasons.push("branch decision differs".to_string());
        mismatch_dimensions.push("branch_decisions".to_string());
        cause_groups.insert("branch_decisions".to_string(), branch_decision_drift_nodes.len());
    }
    if !container_digest_drift_nodes.is_empty() {
        reasons.push("container image digest differs".to_string());
        mismatch_dimensions.push("container_digests".to_string());
        cause_groups.insert("container_digest".to_string(), container_digest_drift_nodes.len());
    }
    if !adapter_binary_drift_nodes.is_empty() {
        reasons.push("adapter binary hash differs".to_string());
        mismatch_dimensions.push("adapter_binary_sha256".to_string());
        cause_groups.insert("adapter_binary".to_string(), adapter_binary_drift_nodes.len());
    }
    if !out_diff.is_empty() {
        reasons.push("output content differs".to_string());
        mismatch_dimensions.push("outputs".to_string());
        cause_groups.insert("artifact_payload".to_string(), out_diff.len());
    }
    let summary = if reasons.is_empty() {
        "runs are semantically equivalent under replay contract".to_string()
    } else {
        "runs are not semantically equivalent under replay contract".to_string()
    };
    let safety_level = if !evidence_gaps.is_empty() {
        ReplaySafetyLevel::IncompleteEvidence
    } else if graph_fingerprint.is_some() {
        ReplaySafetyLevel::Forbidden
    } else if !branch_decision_drift_nodes.is_empty()
        || !container_digest_drift_nodes.is_empty()
        || !adapter_binary_drift_nodes.is_empty()
        || node_outcome_diff_count > 0
        || !out_diff.is_empty()
    {
        ReplaySafetyLevel::Risky
    } else if !manifest_diff.is_empty() {
        ReplaySafetyLevel::SafeWithDrift
    } else {
        ReplaySafetyLevel::Equivalent
    };

    RunDiff {
        manifest: manifest_diff,
        graph_fingerprint,
        nodes: node_diff,
        outputs: out_diff,
        replay_equivalence: ReplayEquivalenceReport {
            equivalent: reasons.is_empty(),
            safety_level,
            reasons,
            reason_report: ReplayReasonReport { summary, compared_dimensions, mismatch_dimensions },
            cause_groups,
            evidence_gaps,
            branch_decision_drift_nodes,
            container_digest_drift_nodes,
            adapter_binary_drift_nodes,
        },
    }
}

fn outputs_index_to_map(index: &OutputsIndex) -> Value {
    let mut files = index.files.clone();
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let mut map = serde_json::Map::new();
    for f in files {
        map.insert(f.path, json!(f.sha256));
    }
    Value::Object(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bijux_dag_artifacts::OutputFile;

    fn index(files: Vec<(&str, &str)>) -> OutputsIndex {
        OutputsIndex {
            files: files
                .into_iter()
                .map(|(p, h)| OutputFile {
                    name: p.to_string(),
                    path: p.to_string(),
                    kind: "file".to_string(),
                    media_type: "application/octet-stream".to_string(),
                    size_bytes: 0,
                    sha256: h.to_string(),
                    node_id: "n".to_string(),
                    node_fingerprint: "fp".to_string(),
                    promotable: false,
                })
                .collect(),
        }
    }

    #[test]
    fn diff_empty_when_identical() {
        let m = json!({"spec":"v","jobs":1});
        let diff = build_run_diff(
            m.clone(),
            m,
            "fp".to_string(),
            "fp".to_string(),
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert!(diff.manifest.is_empty());
        assert!(diff.graph_fingerprint.is_none());
        assert!(diff.nodes.is_empty());
        assert!(diff.outputs.is_empty());
        assert!(diff.replay_equivalence.equivalent);
        assert!(diff.replay_equivalence.reasons.is_empty());
        assert_eq!(
            diff.replay_equivalence.reason_report.summary,
            "runs are semantically equivalent under replay contract"
        );
        assert!(diff.replay_equivalence.cause_groups.is_empty());
        assert_eq!(diff.replay_equivalence.safety_level, ReplaySafetyLevel::Equivalent);
    }

    #[test]
    fn diff_output_changes_detected() {
        let mut out_a = HashMap::new();
        let mut out_b = HashMap::new();
        out_a.insert("n".to_string(), index(vec![("a.txt", "1")]));
        out_b.insert("n".to_string(), index(vec![("a.txt", "2"), ("b.txt", "3")]));
        let diff = build_run_diff(
            json!({}),
            json!({}),
            "fp".to_string(),
            "fp".to_string(),
            &HashMap::new(),
            &HashMap::new(),
            &out_a,
            &out_b,
        );
        let d = diff.outputs.get("n").unwrap();
        assert_eq!(d.added, vec!["b.txt"]);
        assert_eq!(d.changed, vec!["a.txt"]);
        assert!(!diff.replay_equivalence.equivalent);
        assert!(!diff.replay_equivalence.reasons.is_empty());
        assert_eq!(diff.replay_equivalence.cause_groups.get("artifact_payload").copied(), Some(1));
        assert_eq!(diff.replay_equivalence.safety_level, ReplaySafetyLevel::Risky);
    }

    #[test]
    fn non_semantic_manifest_fields_are_ignored() {
        let a = json!({
            "run_id": "a",
            "created_unix_ms": 1,
            "started_unix_ms": 2,
            "finished_unix_ms": 3
        });
        let b = json!({
            "run_id": "b",
            "created_unix_ms": 9,
            "started_unix_ms": 10,
            "finished_unix_ms": 11
        });
        let diff = build_run_diff(
            a,
            b,
            "fp".to_string(),
            "fp".to_string(),
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert!(diff.replay_equivalence.equivalent);
        assert!(diff.replay_equivalence.reasons.is_empty());
    }

    #[test]
    fn replay_diff_reports_environment_drift_as_manifest_drift() {
        let a = json!({"spec":"v","policy":{"deny_env":true}});
        let b = json!({"spec":"v","policy":{"deny_env":false}});
        let diff = build_run_diff(
            a,
            b,
            "fp".to_string(),
            "fp".to_string(),
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert!(!diff.replay_equivalence.equivalent);
        assert_eq!(diff.replay_equivalence.cause_groups.get("manifest_drift"), Some(&1usize));
        assert_eq!(diff.replay_equivalence.safety_level, ReplaySafetyLevel::SafeWithDrift);
    }

    #[test]
    fn replay_diff_reports_backend_capability_mismatch_as_manifest_drift() {
        let a = json!({"spec":"v","adapters":[{"adapter_id":"shell","adapter_version":"1"}]});
        let b = json!({"spec":"v","adapters":[{"adapter_id":"shell","adapter_version":"2"}]});
        let diff = build_run_diff(
            a,
            b,
            "fp".to_string(),
            "fp".to_string(),
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert!(!diff.replay_equivalence.equivalent);
        assert!(diff
            .replay_equivalence
            .reasons
            .iter()
            .any(|r| r.contains("manifest fields differ")));
    }

    #[test]
    fn replay_diff_reports_missing_artifacts_as_output_difference() {
        let mut outputs_a = HashMap::new();
        let mut outputs_b = HashMap::new();
        outputs_a.insert("n".to_string(), index(vec![("out.txt", "hash-1")]));
        outputs_b.insert("n".to_string(), index(vec![]));
        let diff = build_run_diff(
            json!({"spec":"v"}),
            json!({"spec":"v"}),
            "fp".to_string(),
            "fp".to_string(),
            &HashMap::new(),
            &HashMap::new(),
            &outputs_a,
            &outputs_b,
        );
        assert!(!diff.replay_equivalence.equivalent);
        assert_eq!(diff.replay_equivalence.cause_groups.get("artifact_payload"), Some(&1usize));
    }

    #[test]
    fn replay_diff_reports_branch_decision_drift_as_risky() {
        let mut nodes_a = HashMap::new();
        let mut nodes_b = HashMap::new();
        nodes_a.insert(
            "decide".to_string(),
            json!({"status":"success","fingerprint":"fp","branch_decision":"left"}),
        );
        nodes_b.insert(
            "decide".to_string(),
            json!({"status":"success","fingerprint":"fp","branch_decision":"right"}),
        );

        let diff = build_run_diff(
            json!({}),
            json!({}),
            "fp".to_string(),
            "fp".to_string(),
            &nodes_a,
            &nodes_b,
            &HashMap::new(),
            &HashMap::new(),
        );
        let branch = diff.nodes.get("decide").expect("branch diff");
        assert_eq!(branch.branch_decision_a, Some(json!("left")));
        assert_eq!(branch.branch_decision_b, Some(json!("right")));
        assert_eq!(diff.replay_equivalence.branch_decision_drift_nodes, vec!["decide"]);
        assert_eq!(diff.replay_equivalence.cause_groups.get("branch_decisions"), Some(&1usize));
        assert_eq!(diff.replay_equivalence.safety_level, ReplaySafetyLevel::Risky);
    }

    #[test]
    fn replay_diff_reports_container_digest_drift_as_risky() {
        let mut nodes_a = HashMap::new();
        let mut nodes_b = HashMap::new();
        nodes_a.insert(
            "container-step".to_string(),
            json!({
                "status":"success",
                "fingerprint":"fp",
                "container":{"image":"alpine:3.19","image_digest":"sha256:aaa"}
            }),
        );
        nodes_b.insert(
            "container-step".to_string(),
            json!({
                "status":"success",
                "fingerprint":"fp",
                "container":{"image":"alpine:3.19","image_digest":"sha256:bbb"}
            }),
        );

        let diff = build_run_diff(
            json!({}),
            json!({}),
            "fp".to_string(),
            "fp".to_string(),
            &nodes_a,
            &nodes_b,
            &HashMap::new(),
            &HashMap::new(),
        );
        let node = diff.nodes.get("container-step").expect("container diff");
        assert_eq!(node.container_digest_a, Some(json!("sha256:aaa")));
        assert_eq!(node.container_digest_b, Some(json!("sha256:bbb")));
        assert_eq!(diff.replay_equivalence.container_digest_drift_nodes, vec!["container-step"]);
        assert_eq!(diff.replay_equivalence.cause_groups.get("container_digest"), Some(&1usize));
        assert_eq!(diff.replay_equivalence.safety_level, ReplaySafetyLevel::Risky);
    }

    #[test]
    fn replay_diff_reports_adapter_binary_drift_as_risky() {
        let mut nodes_a = HashMap::new();
        let mut nodes_b = HashMap::new();
        nodes_a.insert(
            "external-step".to_string(),
            json!({
                "status":"success",
                "fingerprint":"fp",
                "adapter_binary_sha256":"sha256:old"
            }),
        );
        nodes_b.insert(
            "external-step".to_string(),
            json!({
                "status":"success",
                "fingerprint":"fp",
                "adapter_binary_sha256":"sha256:new"
            }),
        );

        let diff = build_run_diff(
            json!({}),
            json!({}),
            "fp".to_string(),
            "fp".to_string(),
            &nodes_a,
            &nodes_b,
            &HashMap::new(),
            &HashMap::new(),
        );
        let node = diff.nodes.get("external-step").expect("adapter diff");
        assert_eq!(node.adapter_binary_sha256_a, Some(json!("sha256:old")));
        assert_eq!(node.adapter_binary_sha256_b, Some(json!("sha256:new")));
        assert_eq!(diff.replay_equivalence.adapter_binary_drift_nodes, vec!["external-step"]);
        assert_eq!(diff.replay_equivalence.cause_groups.get("adapter_binary"), Some(&1usize));
        assert_eq!(diff.replay_equivalence.safety_level, ReplaySafetyLevel::Risky);
    }

    #[test]
    fn replay_diff_reports_resource_changes() {
        let a = json!({"spec":"v","resources":{"cpu":1,"mem_mb":256}});
        let b = json!({"spec":"v","resources":{"cpu":2,"mem_mb":256}});
        let diff = build_run_diff(
            a,
            b,
            "fp".to_string(),
            "fp".to_string(),
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert!(!diff.replay_equivalence.equivalent);
        assert!(diff.manifest.contains_key("resources"));
    }

    #[test]
    fn replay_diff_reports_grouped_mismatches_for_multiple_dimensions() {
        let manifest_a = json!({"spec":"v","jobs":1});
        let manifest_b = json!({"spec":"v","jobs":2});
        let mut nodes_a = HashMap::new();
        let mut nodes_b = HashMap::new();
        nodes_a.insert("n1".to_string(), json!({"status":"success","fingerprint":"fp1"}));
        nodes_b.insert("n1".to_string(), json!({"status":"failed","fingerprint":"fp1"}));
        let mut outputs_a = HashMap::new();
        let mut outputs_b = HashMap::new();
        outputs_a.insert("n1".to_string(), index(vec![("n1/out", "aaa")]));
        outputs_b.insert("n1".to_string(), index(vec![("n1/out", "bbb")]));

        let diff = build_run_diff(
            manifest_a,
            manifest_b,
            "g1".to_string(),
            "g2".to_string(),
            &nodes_a,
            &nodes_b,
            &outputs_a,
            &outputs_b,
        );
        assert!(!diff.replay_equivalence.equivalent);
        assert_eq!(diff.replay_equivalence.cause_groups.get("manifest_drift"), Some(&1));
        assert_eq!(diff.replay_equivalence.cause_groups.get("graph_semantics"), Some(&1));
        assert_eq!(diff.replay_equivalence.cause_groups.get("node_outcomes"), Some(&1));
        assert_eq!(diff.replay_equivalence.cause_groups.get("artifact_payload"), Some(&1));
        assert_eq!(diff.replay_equivalence.safety_level, ReplaySafetyLevel::Forbidden);
    }

    #[test]
    fn replay_diff_equivalence_report_fields_are_present_for_equivalent_runs() {
        let manifest = json!({"spec":"v","jobs":1});
        let diff = build_run_diff(
            manifest.clone(),
            manifest,
            "same".to_string(),
            "same".to_string(),
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert!(diff.replay_equivalence.equivalent);
        assert_eq!(
            diff.replay_equivalence.reason_report.summary,
            "runs are semantically equivalent under replay contract"
        );
        assert_eq!(
            diff.replay_equivalence.reason_report.compared_dimensions,
            vec!["manifest", "graph_fingerprint", "nodes", "outputs", "branch_decisions"]
        );
    }
}
