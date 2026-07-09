use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Canonical run directory layout contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunDirectoryLayoutContractV1 {
    pub manifest_path: String,
    pub plan_path: String,
    pub run_log_path: String,
    pub traces_root: String,
    pub nodes_root: String,
    pub outputs_index_path: String,
    pub cache_root: String,
    pub replay_root: String,
    pub summaries_root: String,
}

/// Build predictable canonical run directory layout.
pub fn build_run_directory_layout_contract(
    run_id: &str,
) -> Result<RunDirectoryLayoutContractV1, String> {
    let normalized = run_id.trim();
    if normalized.is_empty() {
        return Err("run_id must not be empty".to_string());
    }
    if normalized.contains('/') || normalized.contains('\\') || normalized.contains("..") {
        return Err("run_id must be a normalized identifier".to_string());
    }
    let root = format!("run-{normalized}");
    Ok(RunDirectoryLayoutContractV1 {
        manifest_path: format!("{root}/manifest.json"),
        plan_path: format!("{root}/plan.json"),
        run_log_path: format!("{root}/run.log.jsonl"),
        traces_root: format!("{root}/traces"),
        nodes_root: format!("{root}/nodes"),
        outputs_index_path: format!("{root}/outputs/index.json"),
        cache_root: format!("{root}/cache"),
        replay_root: format!("{root}/replay"),
        summaries_root: format!("{root}/summaries"),
    })
}

/// Content-based artifact identity for file or directory artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactContentIdentityV1 {
    pub artifact_path: String,
    pub artifact_kind: String,
    pub content_hash: String,
}

/// Build content-based identity for a file artifact.
pub fn content_identity_for_file(
    path: &str,
    bytes: &[u8],
) -> Result<ArtifactContentIdentityV1, String> {
    if path.trim().is_empty() {
        return Err("artifact path must not be empty".to_string());
    }
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash = format!("{:x}", hasher.finalize());
    Ok(ArtifactContentIdentityV1 {
        artifact_path: path.to_string(),
        artifact_kind: "file".to_string(),
        content_hash: hash,
    })
}

/// Build deterministic identity for a directory artifact from sorted file hash entries.
pub fn content_identity_for_directory(
    path: &str,
    entries: Vec<(String, String)>,
) -> Result<ArtifactContentIdentityV1, String> {
    if path.trim().is_empty() {
        return Err("artifact path must not be empty".to_string());
    }
    if entries.is_empty() {
        return Err("directory entries must not be empty".to_string());
    }
    let mut canonical = entries;
    canonical.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    let material = canonical
        .into_iter()
        .map(|(entry_path, entry_hash)| format!("{entry_path}:{entry_hash}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut hasher = Sha256::new();
    hasher.update(material.as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    Ok(ArtifactContentIdentityV1 {
        artifact_path: path.to_string(),
        artifact_kind: "directory".to_string(),
        content_hash: hash,
    })
}

/// Complete artifact inventory record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactInventoryRecordV1 {
    pub role: String,
    pub path: String,
    pub hash: String,
    pub producer_node_id: String,
    pub attempt_id: String,
    pub adapter_id: String,
    pub schema_ref: String,
    pub lineage_id: String,
}

/// Build complete artifact inventory with one producer trace per output path.
pub fn build_complete_artifact_inventory(
    records: Vec<ArtifactInventoryRecordV1>,
) -> Result<Vec<ArtifactInventoryRecordV1>, String> {
    if records.is_empty() {
        return Err("inventory records must not be empty".to_string());
    }
    let mut seen_paths = std::collections::BTreeSet::new();
    for record in &records {
        if record.path.trim().is_empty() {
            return Err("inventory path must not be empty".to_string());
        }
        if record.hash.len() != 64 || !record.hash.chars().all(|value| value.is_ascii_hexdigit()) {
            return Err(format!("invalid content hash for {}", record.path));
        }
        if !seen_paths.insert(record.path.clone()) {
            return Err(format!("duplicate output path in inventory: {}", record.path));
        }
        for (field_name, field_value) in [
            ("producer_node_id", record.producer_node_id.as_str()),
            ("attempt_id", record.attempt_id.as_str()),
            ("adapter_id", record.adapter_id.as_str()),
            ("schema_ref", record.schema_ref.as_str()),
            ("lineage_id", record.lineage_id.as_str()),
        ] {
            if field_value.trim().is_empty() {
                return Err(format!("{field_name} must not be empty"));
            }
        }
    }
    let mut sorted = records;
    sorted.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(sorted)
}

/// Cache key factors that must be visible to operators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheKeyFactorsV1 {
    pub graph_fingerprint: String,
    pub node_id: String,
    pub adapter_id: String,
    pub params_fingerprint: String,
    pub input_hashes: Vec<String>,
    pub policy_fingerprint: String,
    pub schema_fingerprint: String,
    pub environment_fingerprint: String,
}

/// Explainable cache key with canonical material.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheKeyExplainV1 {
    pub cache_key: String,
    pub canonical_material: String,
    pub factors: CacheKeyFactorsV1,
}

/// Build an explainable cache key from direct factors.
pub fn build_explainable_cache_key(
    factors: CacheKeyFactorsV1,
) -> Result<CacheKeyExplainV1, String> {
    for (field_name, field_value) in [
        ("graph_fingerprint", factors.graph_fingerprint.as_str()),
        ("node_id", factors.node_id.as_str()),
        ("adapter_id", factors.adapter_id.as_str()),
        ("params_fingerprint", factors.params_fingerprint.as_str()),
        ("policy_fingerprint", factors.policy_fingerprint.as_str()),
        ("schema_fingerprint", factors.schema_fingerprint.as_str()),
        ("environment_fingerprint", factors.environment_fingerprint.as_str()),
    ] {
        if field_value.trim().is_empty() {
            return Err(format!("{field_name} must not be empty"));
        }
    }

    let mut sorted_input_hashes = factors.input_hashes.clone();
    sorted_input_hashes.sort();
    let input_hash_material = if sorted_input_hashes.is_empty() {
        "none".to_string()
    } else {
        sorted_input_hashes.join(",")
    };
    let canonical_material = format!(
        "graph={}|node={}|adapter={}|params={}|inputs={}|policy={}|schema={}|environment={}",
        factors.graph_fingerprint,
        factors.node_id,
        factors.adapter_id,
        factors.params_fingerprint,
        input_hash_material,
        factors.policy_fingerprint,
        factors.schema_fingerprint,
        factors.environment_fingerprint
    );

    let mut hasher = Sha256::new();
    hasher.update(canonical_material.as_bytes());
    let cache_key = format!("{:x}", hasher.finalize());

    Ok(CacheKeyExplainV1 { cache_key, canonical_material, factors })
}

/// Evidence required for safe cache reuse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheReuseEvidenceV1 {
    pub cache_key: String,
    pub artifact_hash: String,
    pub schema_fingerprint: String,
    pub policy_fingerprint: String,
    pub integrity_verified: bool,
}

/// Cache reuse decision for operators and replay.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheReuseDecisionV1 {
    pub decision: String,
    pub reason: String,
}

/// Assess whether cache reuse is safe for a node attempt.
pub fn assess_cache_reuse_safety(
    expected_cache_key: &str,
    expected_artifact_hash: &str,
    expected_schema_fingerprint: &str,
    expected_policy_fingerprint: &str,
    evidence: &CacheReuseEvidenceV1,
) -> CacheReuseDecisionV1 {
    if !evidence.integrity_verified {
        return CacheReuseDecisionV1 {
            decision: "miss".to_string(),
            reason: "cache entry integrity was not verified".to_string(),
        };
    }
    if evidence.cache_key != expected_cache_key {
        return CacheReuseDecisionV1 {
            decision: "miss".to_string(),
            reason: "cache key mismatch".to_string(),
        };
    }
    if evidence.artifact_hash != expected_artifact_hash {
        return CacheReuseDecisionV1 {
            decision: "miss".to_string(),
            reason: "artifact hash mismatch".to_string(),
        };
    }
    if evidence.schema_fingerprint != expected_schema_fingerprint {
        return CacheReuseDecisionV1 {
            decision: "miss".to_string(),
            reason: "schema fingerprint mismatch".to_string(),
        };
    }
    if evidence.policy_fingerprint != expected_policy_fingerprint {
        return CacheReuseDecisionV1 {
            decision: "miss".to_string(),
            reason: "policy fingerprint mismatch".to_string(),
        };
    }
    CacheReuseDecisionV1 {
        decision: "hit".to_string(),
        reason: "safe reuse with matching key, hash, schema, policy, and integrity proof"
            .to_string(),
    }
}

/// Cache reuse compatibility context for explicit miss explanation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheReuseContextV1 {
    pub input_fingerprint: String,
    pub policy_fingerprint: String,
    pub schema_fingerprint: String,
    pub adapter_fingerprint: String,
    pub runtime_fingerprint: String,
    pub integrity_verified: bool,
}

/// Cache compatibility assessment with concrete miss reasons.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheReuseCompatibilityV1 {
    pub decision: String,
    pub reasons: Vec<String>,
}

/// Assess safe/unsafe reuse from compatibility factors.
pub fn assess_cache_reuse_compatibility(
    expected: &CacheReuseContextV1,
    candidate: &CacheReuseContextV1,
) -> CacheReuseCompatibilityV1 {
    let mut reasons = Vec::new();
    if !candidate.integrity_verified {
        reasons.push("integrity_unverified".to_string());
    }
    if expected.input_fingerprint != candidate.input_fingerprint {
        reasons.push("input_fingerprint_changed".to_string());
    }
    if expected.policy_fingerprint != candidate.policy_fingerprint {
        reasons.push("policy_fingerprint_changed".to_string());
    }
    if expected.schema_fingerprint != candidate.schema_fingerprint {
        reasons.push("schema_fingerprint_changed".to_string());
    }
    if expected.adapter_fingerprint != candidate.adapter_fingerprint {
        reasons.push("adapter_fingerprint_changed".to_string());
    }
    if expected.runtime_fingerprint != candidate.runtime_fingerprint {
        reasons.push("runtime_fingerprint_changed".to_string());
    }
    CacheReuseCompatibilityV1 {
        decision: if reasons.is_empty() { "hit".to_string() } else { "miss".to_string() },
        reasons,
    }
}

/// Replay decision for one node before execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayNodePlanDecisionV1 {
    pub node_id: String,
    pub action: String,
    pub reason: String,
}

/// Build a readable replay plan with one decision line per node.
pub fn build_replay_plan_readout(
    decisions: Vec<ReplayNodePlanDecisionV1>,
) -> Result<String, String> {
    if decisions.is_empty() {
        return Err("replay decisions must not be empty".to_string());
    }
    let mut normalized = decisions;
    normalized.sort_by(|left, right| left.node_id.cmp(&right.node_id));
    let mut lines = Vec::new();
    for decision in normalized {
        if decision.node_id.trim().is_empty() {
            return Err("node_id must not be empty".to_string());
        }
        if decision.action.trim().is_empty() {
            return Err("action must not be empty".to_string());
        }
        if decision.reason.trim().is_empty() {
            return Err("reason must not be empty".to_string());
        }
        lines.push(format!(
            "{}: action={} reason={}",
            decision.node_id, decision.action, decision.reason
        ));
    }
    Ok(lines.join("\n"))
}

/// Replay ancestry record preserving source lineage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayAncestryRecordV1 {
    pub replay_run_id: String,
    pub node_id: String,
    pub action: String,
    pub source_run_id: Option<String>,
    pub source_node_id: Option<String>,
}

/// Validate replay ancestry records for lineage preservation.
pub fn validate_replay_ancestry_records(
    records: Vec<ReplayAncestryRecordV1>,
) -> Result<Vec<ReplayAncestryRecordV1>, String> {
    if records.is_empty() {
        return Err("replay ancestry records must not be empty".to_string());
    }
    let mut seen_nodes = std::collections::BTreeSet::new();
    for record in &records {
        if record.replay_run_id.trim().is_empty() {
            return Err("replay_run_id must not be empty".to_string());
        }
        if record.node_id.trim().is_empty() {
            return Err("node_id must not be empty".to_string());
        }
        if !seen_nodes.insert(record.node_id.clone()) {
            return Err(format!("duplicate replay node ancestry: {}", record.node_id));
        }
        if record.action.trim().is_empty() {
            return Err("action must not be empty".to_string());
        }
        if record.action == "reuse" || record.action == "rerun" {
            let source_run = record.source_run_id.as_deref().unwrap_or("").trim();
            let source_node = record.source_node_id.as_deref().unwrap_or("").trim();
            if source_run.is_empty() || source_node.is_empty() {
                return Err(format!(
                    "action {} requires source_run_id and source_node_id for {}",
                    record.action, record.node_id
                ));
            }
            if source_run == record.replay_run_id && source_node == record.node_id {
                return Err(format!(
                    "replay ancestry self-cycle for node {} is not allowed",
                    record.node_id
                ));
            }
        }
    }
    let mut sorted = records;
    sorted.sort_by(|left, right| left.node_id.cmp(&right.node_id));
    Ok(sorted)
}

/// Node snapshot for run diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeRunSnapshotV1 {
    pub node_id: String,
    pub state: String,
    pub branch_decision: String,
    pub attempt_id: String,
    pub artifact_hash: String,
    pub log_hash: String,
    pub cache_decision: String,
    pub integrity_proof_hash: String,
}

/// One run diff row describing what changed for a node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunDiffEntryV1 {
    pub node_id: String,
    pub change_kind: String,
    pub changed_fields: Vec<String>,
}

/// Diff two runs by operator-meaningful node fields.
pub fn diff_run_snapshots(
    base: Vec<NodeRunSnapshotV1>,
    candidate: Vec<NodeRunSnapshotV1>,
) -> Result<Vec<RunDiffEntryV1>, String> {
    let base_map = base
        .into_iter()
        .map(|item| (item.node_id.clone(), item))
        .collect::<std::collections::BTreeMap<_, _>>();
    let candidate_map = candidate
        .into_iter()
        .map(|item| (item.node_id.clone(), item))
        .collect::<std::collections::BTreeMap<_, _>>();

    let mut all_nodes = std::collections::BTreeSet::new();
    for node_id in base_map.keys() {
        all_nodes.insert(node_id.clone());
    }
    for node_id in candidate_map.keys() {
        all_nodes.insert(node_id.clone());
    }

    let mut changes = Vec::new();
    for node_id in all_nodes {
        match (base_map.get(&node_id), candidate_map.get(&node_id)) {
            (None, Some(_)) => {
                changes.push(RunDiffEntryV1 {
                    node_id,
                    change_kind: "added".to_string(),
                    changed_fields: vec!["all".to_string()],
                });
            }
            (Some(_), None) => {
                changes.push(RunDiffEntryV1 {
                    node_id,
                    change_kind: "removed".to_string(),
                    changed_fields: vec!["all".to_string()],
                });
            }
            (Some(left), Some(right)) => {
                let mut fields = Vec::new();
                if left.state != right.state {
                    fields.push("state".to_string());
                }
                if left.branch_decision != right.branch_decision {
                    fields.push("branch_decision".to_string());
                }
                if left.attempt_id != right.attempt_id {
                    fields.push("attempt_id".to_string());
                }
                if left.artifact_hash != right.artifact_hash {
                    fields.push("artifact_hash".to_string());
                }
                if left.log_hash != right.log_hash {
                    fields.push("log_hash".to_string());
                }
                if left.cache_decision != right.cache_decision {
                    fields.push("cache_decision".to_string());
                }
                if left.integrity_proof_hash != right.integrity_proof_hash {
                    fields.push("integrity_proof_hash".to_string());
                }
                if !fields.is_empty() {
                    changes.push(RunDiffEntryV1 {
                        node_id,
                        change_kind: "changed".to_string(),
                        changed_fields: fields,
                    });
                }
            }
            (None, None) => {}
        }
    }

    Ok(changes)
}

/// Evidence bundle inputs that must verify together.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleProofInputsV1 {
    pub manifest_hash: String,
    pub plan_hash: String,
    pub trace_hash: String,
    pub inventory_hash: String,
    pub cache_proof_hash: String,
    pub replay_proof_hash: String,
}

/// Bundle verification result with tamper-sensitive details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleVerificationReportV1 {
    pub bundle_hash: String,
    pub verified: bool,
    pub failed_fields: Vec<String>,
}

/// Verify bundle evidence inputs together and detect tampering.
pub fn verify_bundle_proofs(
    expected: &BundleProofInputsV1,
    observed: &BundleProofInputsV1,
) -> BundleVerificationReportV1 {
    let mut failed_fields = Vec::new();
    for (field_name, left, right) in [
        ("manifest_hash", expected.manifest_hash.as_str(), observed.manifest_hash.as_str()),
        ("plan_hash", expected.plan_hash.as_str(), observed.plan_hash.as_str()),
        ("trace_hash", expected.trace_hash.as_str(), observed.trace_hash.as_str()),
        ("inventory_hash", expected.inventory_hash.as_str(), observed.inventory_hash.as_str()),
        (
            "cache_proof_hash",
            expected.cache_proof_hash.as_str(),
            observed.cache_proof_hash.as_str(),
        ),
        (
            "replay_proof_hash",
            expected.replay_proof_hash.as_str(),
            observed.replay_proof_hash.as_str(),
        ),
    ] {
        if left != right {
            failed_fields.push(field_name.to_string());
        }
    }
    let canonical = format!(
        "{}|{}|{}|{}|{}|{}",
        observed.manifest_hash,
        observed.plan_hash,
        observed.trace_hash,
        observed.inventory_hash,
        observed.cache_proof_hash,
        observed.replay_proof_hash
    );
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    let bundle_hash = format!("{:x}", hasher.finalize());
    BundleVerificationReportV1 { bundle_hash, verified: failed_fields.is_empty(), failed_fields }
}

#[cfg(test)]
mod tests {
    use super::{
        assess_cache_reuse_compatibility, assess_cache_reuse_safety,
        build_complete_artifact_inventory, build_explainable_cache_key, build_replay_plan_readout,
        build_run_directory_layout_contract, content_identity_for_directory,
        content_identity_for_file, diff_run_snapshots, validate_replay_ancestry_records,
        verify_bundle_proofs, ArtifactInventoryRecordV1, BundleProofInputsV1, CacheKeyFactorsV1,
        CacheReuseContextV1, CacheReuseEvidenceV1, NodeRunSnapshotV1, ReplayAncestryRecordV1,
        ReplayNodePlanDecisionV1,
    };

    #[test]
    fn run_directory_layout_contract_is_predictable() {
        let layout =
            build_run_directory_layout_contract("20260501-abc").expect("layout should build");
        assert_eq!(layout.manifest_path, "run-20260501-abc/manifest.json");
        assert_eq!(layout.outputs_index_path, "run-20260501-abc/outputs/index.json");
        assert_eq!(layout.replay_root, "run-20260501-abc/replay");
    }

    #[test]
    fn artifact_identity_is_content_based_for_files_and_directories() {
        let file_a =
            content_identity_for_file("outputs/a.json", br#"{"a":1}"#).expect("file identity");
        let file_b =
            content_identity_for_file("outputs/a.json", br#"{"a":2}"#).expect("file identity");
        assert_ne!(file_a.content_hash, file_b.content_hash);

        let dir_a = content_identity_for_directory(
            "outputs/dir",
            vec![
                ("a.txt".to_string(), file_a.content_hash.clone()),
                ("b.txt".to_string(), file_b.content_hash.clone()),
            ],
        )
        .expect("dir identity");
        let dir_b = content_identity_for_directory(
            "outputs/dir",
            vec![
                ("b.txt".to_string(), file_b.content_hash),
                ("a.txt".to_string(), file_a.content_hash),
            ],
        )
        .expect("dir identity");
        assert_eq!(dir_a.content_hash, dir_b.content_hash);
    }

    #[test]
    fn artifact_inventory_records_producer_attempt_adapter_schema_and_lineage() {
        let records = build_complete_artifact_inventory(vec![ArtifactInventoryRecordV1 {
            role: "primary".to_string(),
            path: "outputs/sample.vcf".to_string(),
            hash: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            producer_node_id: "call-variants".to_string(),
            attempt_id: "attempt-1".to_string(),
            adapter_id: "shell".to_string(),
            schema_ref: "vcf/v4.3".to_string(),
            lineage_id: "run-1:call-variants:sample.vcf".to_string(),
        }])
        .expect("inventory");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].producer_node_id, "call-variants");
        assert_eq!(records[0].adapter_id, "shell");
    }

    #[test]
    fn cache_key_explain_includes_direct_factors() {
        let factors = CacheKeyFactorsV1 {
            graph_fingerprint: "graph-abc".to_string(),
            node_id: "align-reads".to_string(),
            adapter_id: "shell".to_string(),
            params_fingerprint: "params-001".to_string(),
            input_hashes: vec!["b2".to_string(), "a1".to_string()],
            policy_fingerprint: "policy-safe".to_string(),
            schema_fingerprint: "schema-v3".to_string(),
            environment_fingerprint: "env-linux-amd64".to_string(),
        };
        let explained = build_explainable_cache_key(factors.clone()).expect("cache explain");
        assert!(explained.canonical_material.contains("graph=graph-abc"));
        assert!(explained.canonical_material.contains("node=align-reads"));
        assert!(explained.canonical_material.contains("adapter=shell"));
        assert!(explained.canonical_material.contains("inputs=a1,b2"));

        let explained_repeat = build_explainable_cache_key(factors).expect("cache explain");
        assert_eq!(explained.cache_key, explained_repeat.cache_key);
    }

    #[test]
    fn safe_cache_reuse_is_demonstrable_with_matching_evidence() {
        let evidence = CacheReuseEvidenceV1 {
            cache_key: "cache-key-123".to_string(),
            artifact_hash: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                .to_string(),
            schema_fingerprint: "schema-v1".to_string(),
            policy_fingerprint: "policy-safe".to_string(),
            integrity_verified: true,
        };
        let decision = assess_cache_reuse_safety(
            "cache-key-123",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "schema-v1",
            "policy-safe",
            &evidence,
        );
        assert_eq!(decision.decision, "hit");
        assert!(decision.reason.contains("safe reuse"));
    }

    #[test]
    fn unsafe_cache_reuse_reports_changed_factors() {
        let expected = CacheReuseContextV1 {
            input_fingerprint: "input-a".to_string(),
            policy_fingerprint: "policy-a".to_string(),
            schema_fingerprint: "schema-v1".to_string(),
            adapter_fingerprint: "shell-v1".to_string(),
            runtime_fingerprint: "runtime-1.0".to_string(),
            integrity_verified: true,
        };
        let candidate = CacheReuseContextV1 {
            input_fingerprint: "input-b".to_string(),
            policy_fingerprint: "policy-a".to_string(),
            schema_fingerprint: "schema-v2".to_string(),
            adapter_fingerprint: "shell-v2".to_string(),
            runtime_fingerprint: "runtime-1.0".to_string(),
            integrity_verified: false,
        };
        let compatibility = assess_cache_reuse_compatibility(&expected, &candidate);
        assert_eq!(compatibility.decision, "miss");
        assert!(compatibility.reasons.contains(&"integrity_unverified".to_string()));
        assert!(compatibility.reasons.contains(&"input_fingerprint_changed".to_string()));
        assert!(compatibility.reasons.contains(&"schema_fingerprint_changed".to_string()));
        assert!(compatibility.reasons.contains(&"adapter_fingerprint_changed".to_string()));
    }

    #[test]
    fn replay_plan_is_readable_before_execution() {
        let plan = build_replay_plan_readout(vec![
            ReplayNodePlanDecisionV1 {
                node_id: "align-reads".to_string(),
                action: "reuse".to_string(),
                reason: "cache key and integrity proof match".to_string(),
            },
            ReplayNodePlanDecisionV1 {
                node_id: "call-variants".to_string(),
                action: "rerun".to_string(),
                reason: "upstream input fingerprint changed".to_string(),
            },
            ReplayNodePlanDecisionV1 {
                node_id: "publish-report".to_string(),
                action: "skip".to_string(),
                reason: "selected replay scope excludes downstream publish".to_string(),
            },
            ReplayNodePlanDecisionV1 {
                node_id: "unsafe-node".to_string(),
                action: "refuse".to_string(),
                reason: "node declared non-replayable external side effect".to_string(),
            },
        ])
        .expect("replay plan");
        assert!(plan.contains("align-reads: action=reuse"));
        assert!(plan.contains("call-variants: action=rerun"));
        assert!(plan.contains("publish-report: action=skip"));
        assert!(plan.contains("unsafe-node: action=refuse"));
    }

    #[test]
    fn replay_ancestry_is_preserved_for_reused_and_rerun_nodes() {
        let validated = validate_replay_ancestry_records(vec![
            ReplayAncestryRecordV1 {
                replay_run_id: "run-200".to_string(),
                node_id: "align-reads".to_string(),
                action: "reuse".to_string(),
                source_run_id: Some("run-199".to_string()),
                source_node_id: Some("align-reads".to_string()),
            },
            ReplayAncestryRecordV1 {
                replay_run_id: "run-200".to_string(),
                node_id: "call-variants".to_string(),
                action: "rerun".to_string(),
                source_run_id: Some("run-199".to_string()),
                source_node_id: Some("call-variants".to_string()),
            },
            ReplayAncestryRecordV1 {
                replay_run_id: "run-200".to_string(),
                node_id: "publish-report".to_string(),
                action: "skip".to_string(),
                source_run_id: None,
                source_node_id: None,
            },
        ])
        .expect("ancestry");
        assert_eq!(validated.len(), 3);
        assert_eq!(validated[0].node_id, "align-reads");
        assert_eq!(validated[1].node_id, "call-variants");
    }

    #[test]
    fn run_diff_answers_what_changed() {
        let changes = diff_run_snapshots(
            vec![
                NodeRunSnapshotV1 {
                    node_id: "align-reads".to_string(),
                    state: "completed".to_string(),
                    branch_decision: "main".to_string(),
                    attempt_id: "attempt-1".to_string(),
                    artifact_hash: "a1".to_string(),
                    log_hash: "l1".to_string(),
                    cache_decision: "miss".to_string(),
                    integrity_proof_hash: "p1".to_string(),
                },
                NodeRunSnapshotV1 {
                    node_id: "call-variants".to_string(),
                    state: "completed".to_string(),
                    branch_decision: "main".to_string(),
                    attempt_id: "attempt-1".to_string(),
                    artifact_hash: "b1".to_string(),
                    log_hash: "l2".to_string(),
                    cache_decision: "miss".to_string(),
                    integrity_proof_hash: "p2".to_string(),
                },
            ],
            vec![
                NodeRunSnapshotV1 {
                    node_id: "align-reads".to_string(),
                    state: "completed".to_string(),
                    branch_decision: "main".to_string(),
                    attempt_id: "attempt-1".to_string(),
                    artifact_hash: "a1".to_string(),
                    log_hash: "l1".to_string(),
                    cache_decision: "hit".to_string(),
                    integrity_proof_hash: "p1".to_string(),
                },
                NodeRunSnapshotV1 {
                    node_id: "call-variants".to_string(),
                    state: "completed".to_string(),
                    branch_decision: "secondary".to_string(),
                    attempt_id: "attempt-2".to_string(),
                    artifact_hash: "b2".to_string(),
                    log_hash: "l3".to_string(),
                    cache_decision: "miss".to_string(),
                    integrity_proof_hash: "p3".to_string(),
                },
            ],
        )
        .expect("run diff");
        assert_eq!(changes.len(), 2);
        let align =
            changes.iter().find(|entry| entry.node_id == "align-reads").expect("align change");
        assert_eq!(align.change_kind, "changed");
        assert!(align.changed_fields.contains(&"cache_decision".to_string()));
    }

    #[test]
    fn bundle_verification_detects_tampering() {
        let expected = BundleProofInputsV1 {
            manifest_hash: "m1".to_string(),
            plan_hash: "p1".to_string(),
            trace_hash: "t1".to_string(),
            inventory_hash: "i1".to_string(),
            cache_proof_hash: "c1".to_string(),
            replay_proof_hash: "r1".to_string(),
        };
        let observed = BundleProofInputsV1 {
            manifest_hash: "m1".to_string(),
            plan_hash: "p1".to_string(),
            trace_hash: "tampered-trace".to_string(),
            inventory_hash: "i1".to_string(),
            cache_proof_hash: "c1".to_string(),
            replay_proof_hash: "r1".to_string(),
        };
        let report = verify_bundle_proofs(&expected, &observed);
        assert!(!report.verified);
        assert!(report.failed_fields.contains(&"trace_hash".to_string()));
    }
}
