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
pub fn build_run_directory_layout_contract(run_id: &str) -> Result<RunDirectoryLayoutContractV1, String> {
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
pub fn content_identity_for_file(path: &str, bytes: &[u8]) -> Result<ArtifactContentIdentityV1, String> {
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
pub fn build_explainable_cache_key(factors: CacheKeyFactorsV1) -> Result<CacheKeyExplainV1, String> {
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

    Ok(CacheKeyExplainV1 {
        cache_key,
        canonical_material,
        factors,
    })
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
        reason: "safe reuse with matching key, hash, schema, policy, and integrity proof".to_string(),
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
        decision: if reasons.is_empty() {
            "hit".to_string()
        } else {
            "miss".to_string()
        },
        reasons,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        assess_cache_reuse_compatibility, assess_cache_reuse_safety, build_complete_artifact_inventory, build_explainable_cache_key,
        build_run_directory_layout_contract, content_identity_for_directory, content_identity_for_file,
        ArtifactInventoryRecordV1, CacheKeyFactorsV1, CacheReuseContextV1, CacheReuseEvidenceV1,
    };

    #[test]
    fn g061_run_directory_layout_contract_is_predictable() {
        let layout = build_run_directory_layout_contract("20260501-abc")
            .expect("layout should build");
        assert_eq!(layout.manifest_path, "run-20260501-abc/manifest.json");
        assert_eq!(layout.outputs_index_path, "run-20260501-abc/outputs/index.json");
        assert_eq!(layout.replay_root, "run-20260501-abc/replay");
    }

    #[test]
    fn g062_artifact_identity_is_content_based_for_files_and_directories() {
        let file_a = content_identity_for_file("outputs/a.json", br#"{"a":1}"#)
            .expect("file identity");
        let file_b = content_identity_for_file("outputs/a.json", br#"{"a":2}"#)
            .expect("file identity");
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
    fn g063_artifact_inventory_records_producer_attempt_adapter_schema_and_lineage() {
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
    fn g064_cache_key_explain_includes_direct_factors() {
        let factors = CacheKeyFactorsV1 {
            graph_fingerprint: "graph-abc".to_string(),
            node_id: "align-reads".to_string(),
            adapter_id: "shell".to_string(),
            params_fingerprint: "params-001".to_string(),
            input_hashes: vec![
                "b2".to_string(),
                "a1".to_string(),
            ],
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
    fn g065_safe_cache_reuse_is_demonstrable_with_matching_evidence() {
        let evidence = CacheReuseEvidenceV1 {
            cache_key: "cache-key-123".to_string(),
            artifact_hash: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
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
    fn g066_unsafe_cache_reuse_reports_changed_factors() {
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
}
