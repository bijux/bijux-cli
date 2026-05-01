use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Artifact schema descriptor contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactSchemaDescriptorV1 {
    pub artifact_id: String,
    pub media_type: String,
    pub role: String,
    pub schema_version: String,
    pub verifier: String,
    pub producer_contract: String,
}

/// Validate artifact schema descriptor completeness before consumption.
pub fn validate_artifact_schema_descriptor(
    descriptor: &ArtifactSchemaDescriptorV1,
) -> Result<(), String> {
    for (name, value) in [
        ("artifact_id", descriptor.artifact_id.as_str()),
        ("media_type", descriptor.media_type.as_str()),
        ("role", descriptor.role.as_str()),
        ("schema_version", descriptor.schema_version.as_str()),
        ("verifier", descriptor.verifier.as_str()),
        ("producer_contract", descriptor.producer_contract.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!("artifact schema descriptor field '{}' must not be empty", name));
        }
    }
    if !descriptor.media_type.contains('/') {
        return Err("artifact schema descriptor media_type must be a valid type/subtype".to_string());
    }
    Ok(())
}

/// Artifact lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactLifecycleStateV1 {
    Draft,
    Verified,
    Retained,
    Archived,
    Exported,
    Deleted,
}

/// Validate explicit lifecycle transition.
pub fn validate_artifact_lifecycle_transition(
    from: ArtifactLifecycleStateV1,
    to: ArtifactLifecycleStateV1,
) -> Result<(), String> {
    use ArtifactLifecycleStateV1::{Archived, Deleted, Draft, Exported, Retained, Verified};
    let legal = matches!(
        (from, to),
        (Draft, Verified)
            | (Verified, Retained)
            | (Retained, Archived)
            | (Archived, Exported)
            | (Draft, Deleted)
            | (Verified, Deleted)
            | (Retained, Deleted)
            | (Archived, Deleted)
            | (Exported, Deleted)
    );
    if legal {
        Ok(())
    } else {
        Err(format!(
            "illegal artifact lifecycle transition {:?} -> {:?}",
            from, to
        ))
    }
}

/// Artifact retention classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionClassV1 {
    Ephemeral,
    Operational,
    Audit,
    Release,
    Scientific,
}

/// Retention decision for one artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionDecisionV1 {
    pub class: RetentionClassV1,
    pub replay_critical: bool,
    pub allow_delete: bool,
    pub reason: String,
}

/// Enforce retention class behavior with replay-critical protection.
pub fn enforce_retention_class(
    class: RetentionClassV1,
    age_days: u32,
    replay_critical: bool,
) -> RetentionDecisionV1 {
    if replay_critical {
        return RetentionDecisionV1 {
            class,
            replay_critical,
            allow_delete: false,
            reason: "replay-critical evidence cannot be deleted by retention policy".to_string(),
        };
    }
    let allow_delete = match class {
        RetentionClassV1::Ephemeral => age_days > 7,
        RetentionClassV1::Operational => age_days > 30,
        RetentionClassV1::Audit => age_days > 365,
        RetentionClassV1::Release => age_days > 730,
        RetentionClassV1::Scientific => age_days > 1825,
    };
    let reason = if allow_delete {
        "retention policy allows deletion by class age threshold".to_string()
    } else {
        "retention policy keeps artifact within class age threshold".to_string()
    };
    RetentionDecisionV1 { class, replay_critical, allow_delete, reason }
}

/// Cache GC candidate row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheGcCandidateV1 {
    pub cache_key: String,
    pub referenced_by_run_evidence: bool,
    pub safe_by_policy: bool,
}

/// Cache GC dry-run entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheGcDryRunEntryV1 {
    pub cache_key: String,
    pub would_remove: bool,
    pub reason: String,
}

/// Cache GC dry-run report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheGcDryRunReportV1 {
    pub entries: Vec<CacheGcDryRunEntryV1>,
}

/// Build evidence-safe cache garbage collection dry-run report.
pub fn build_cache_gc_dry_run(candidates: &[CacheGcCandidateV1]) -> CacheGcDryRunReportV1 {
    let entries = candidates
        .iter()
        .map(|candidate| {
            let (would_remove, reason) = if candidate.referenced_by_run_evidence {
                (false, "cache entry is referenced by run evidence".to_string())
            } else if !candidate.safe_by_policy {
                (false, "cache entry is blocked by policy".to_string())
            } else {
                (true, "cache entry is unreferenced and policy-safe".to_string())
            };
            CacheGcDryRunEntryV1 {
                cache_key: candidate.cache_key.clone(),
                would_remove,
                reason,
            }
        })
        .collect();
    CacheGcDryRunReportV1 { entries }
}

/// Portable cache entry descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachePortableEntryV1 {
    pub cache_key: String,
    pub schema_fingerprint: String,
    pub policy_fingerprint: String,
    pub integrity_verified: bool,
}

/// Portable cache bundle descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachePortableBundleV1 {
    pub bundle_id: String,
    pub entries: Vec<CachePortableEntryV1>,
}

/// Validate imported portable cache bundle for safe reuse.
pub fn validate_cache_import_bundle(
    bundle: &CachePortableBundleV1,
    expected_schema_fingerprint: &str,
    expected_policy_fingerprint: &str,
) -> Result<(), String> {
    if bundle.bundle_id.trim().is_empty() {
        return Err("cache portable bundle must include bundle_id".to_string());
    }
    if bundle.entries.is_empty() {
        return Err("cache portable bundle must include entries".to_string());
    }
    for entry in &bundle.entries {
        if entry.cache_key.trim().is_empty() {
            return Err("cache portable entry must include cache_key".to_string());
        }
        if !entry.integrity_verified {
            return Err(format!(
                "cache entry '{}' import refused: integrity is not verified",
                entry.cache_key
            ));
        }
        if entry.schema_fingerprint != expected_schema_fingerprint {
            return Err(format!(
                "cache entry '{}' import refused: schema fingerprint mismatch",
                entry.cache_key
            ));
        }
        if entry.policy_fingerprint != expected_policy_fingerprint {
            return Err(format!(
                "cache entry '{}' import refused: policy fingerprint mismatch",
                entry.cache_key
            ));
        }
    }
    Ok(())
}

/// Artifact lineage record for query index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactLineageRecordV1 {
    pub artifact_id: String,
    pub producer_node: String,
    pub consumers: Vec<String>,
    pub parent_artifacts: Vec<String>,
    pub cache_source: Option<String>,
    pub replay_source: Option<String>,
}

/// Queryable lineage index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactLineageQueryIndexV1 {
    pub records: Vec<ArtifactLineageRecordV1>,
    pub producer_by_artifact: BTreeMap<String, String>,
}

/// Build query index for producer/consumer/ancestor/descendant queries.
pub fn build_artifact_lineage_query_index(
    records: Vec<ArtifactLineageRecordV1>,
) -> ArtifactLineageQueryIndexV1 {
    let producer_by_artifact = records
        .iter()
        .map(|record| (record.artifact_id.clone(), record.producer_node.clone()))
        .collect();
    ArtifactLineageQueryIndexV1 { records, producer_by_artifact }
}

/// Find ancestor artifacts for an artifact id.
pub fn lineage_ancestors(index: &ArtifactLineageQueryIndexV1, artifact_id: &str) -> Vec<String> {
    let by_id = index
        .records
        .iter()
        .map(|record| (record.artifact_id.as_str(), record))
        .collect::<BTreeMap<_, _>>();
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::from([artifact_id.to_string()]);
    while let Some(current) = queue.pop_front() {
        if let Some(record) = by_id.get(current.as_str()) {
            for parent in &record.parent_artifacts {
                if visited.insert(parent.clone()) {
                    queue.push_back(parent.clone());
                }
            }
        }
    }
    visited.into_iter().collect()
}

/// Find descendant artifacts for an artifact id.
pub fn lineage_descendants(index: &ArtifactLineageQueryIndexV1, artifact_id: &str) -> Vec<String> {
    let mut children = BTreeMap::<String, Vec<String>>::new();
    for record in &index.records {
        for parent in &record.parent_artifacts {
            children.entry(parent.clone()).or_default().push(record.artifact_id.clone());
        }
    }
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::from([artifact_id.to_string()]);
    while let Some(current) = queue.pop_front() {
        if let Some(next) = children.get(&current) {
            for child in next {
                if visited.insert(child.clone()) {
                    queue.push_back(child.clone());
                }
            }
        }
    }
    visited.into_iter().collect()
}

/// Safe artifact preview output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactPreviewV1 {
    pub preview_kind: String,
    pub preview: String,
}

/// Build safe preview for JSON/text artifacts with size and redaction guards.
pub fn build_safe_artifact_preview(
    media_type: &str,
    bytes: &[u8],
    max_preview_bytes: usize,
) -> ArtifactPreviewV1 {
    if bytes.len() > max_preview_bytes {
        return ArtifactPreviewV1 {
            preview_kind: "summary".to_string(),
            preview: format!("artifact too large for inline preview ({} bytes)", bytes.len()),
        };
    }
    let is_textual = media_type.starts_with("text/") || media_type == "application/json";
    if !is_textual {
        return ArtifactPreviewV1 {
            preview_kind: "summary".to_string(),
            preview: format!("binary artifact preview suppressed for media type {}", media_type),
        };
    }
    let mut text = String::from_utf8_lossy(bytes).to_string();
    for marker in ["secret=", "token=", "password="] {
        if let Some(pos) = text.to_lowercase().find(marker) {
            let end = (pos + marker.len() + 8).min(text.len());
            text.replace_range(pos..end, "[REDACTED]");
        }
    }
    ArtifactPreviewV1 { preview_kind: "inline".to_string(), preview: text }
}

/// Artifact index migration request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactIndexMigrationRequestV1 {
    pub from_schema: String,
    pub to_schema: String,
    pub semantic_loss_detected: bool,
}

/// Artifact index migration decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactIndexMigrationDecisionV1 {
    pub migrated: bool,
    pub reason: String,
}

/// Evaluate artifact index migration safety.
pub fn evaluate_artifact_index_migration(
    request: &ArtifactIndexMigrationRequestV1,
) -> ArtifactIndexMigrationDecisionV1 {
    if request.from_schema.trim().is_empty() || request.to_schema.trim().is_empty() {
        return ArtifactIndexMigrationDecisionV1 {
            migrated: false,
            reason: "migration refused: schema identifiers must not be empty".to_string(),
        };
    }
    if request.semantic_loss_detected {
        return ArtifactIndexMigrationDecisionV1 {
            migrated: false,
            reason: "migration refused: semantic-loss migration is not allowed".to_string(),
        };
    }
    ArtifactIndexMigrationDecisionV1 {
        migrated: true,
        reason: format!(
            "migration accepted from '{}' to '{}'",
            request.from_schema, request.to_schema
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        evaluate_artifact_index_migration, ArtifactIndexMigrationRequestV1,
        build_safe_artifact_preview,
        build_artifact_lineage_query_index, lineage_ancestors, lineage_descendants,
        ArtifactLineageRecordV1,
        build_cache_gc_dry_run, CacheGcCandidateV1,
        validate_cache_import_bundle, CachePortableBundleV1, CachePortableEntryV1,
        enforce_retention_class, RetentionClassV1,
        validate_artifact_lifecycle_transition, validate_artifact_schema_descriptor,
        ArtifactLifecycleStateV1, ArtifactSchemaDescriptorV1,
    };

    #[test]
    fn g151_artifact_schema_descriptor_requires_media_role_version_verifier_and_contract() {
        let descriptor = ArtifactSchemaDescriptorV1 {
            artifact_id: "artifact://run-151/node-a/output.json".to_string(),
            media_type: "application/json".to_string(),
            role: "analysis-report".to_string(),
            schema_version: "report-schema/v3".to_string(),
            verifier: "bijux verify report-schema/v3".to_string(),
            producer_contract: "node-a@attempt-1".to_string(),
        };
        validate_artifact_schema_descriptor(&descriptor).expect("valid descriptor");

        let mut invalid = descriptor;
        invalid.media_type = "json".to_string();
        let error = validate_artifact_schema_descriptor(&invalid).expect_err("invalid media type");
        assert!(error.contains("type/subtype"));
    }

    #[test]
    fn g152_artifact_lifecycle_transitions_are_explicit_and_queryable() {
        use ArtifactLifecycleStateV1::{Archived, Draft, Exported, Retained, Verified};
        validate_artifact_lifecycle_transition(Draft, Verified).expect("draft->verified");
        validate_artifact_lifecycle_transition(Verified, Retained).expect("verified->retained");
        validate_artifact_lifecycle_transition(Retained, Archived).expect("retained->archived");
        validate_artifact_lifecycle_transition(Archived, Exported).expect("archived->exported");
        let error =
            validate_artifact_lifecycle_transition(Draft, Exported).expect_err("must reject skip transition");
        assert!(error.contains("illegal artifact lifecycle transition"));
    }

    #[test]
    fn g153_retention_class_policy_never_deletes_replay_critical_evidence() {
        let replay_critical =
            enforce_retention_class(RetentionClassV1::Ephemeral, 400, true);
        assert!(!replay_critical.allow_delete);
        assert!(replay_critical.reason.contains("replay-critical"));

        let non_critical = enforce_retention_class(RetentionClassV1::Operational, 31, false);
        assert!(non_critical.allow_delete);
    }

    #[test]
    fn g154_cache_gc_dry_run_explains_evidence_safe_removals() {
        let report = build_cache_gc_dry_run(&[
            CacheGcCandidateV1 {
                cache_key: "keep-evidence".to_string(),
                referenced_by_run_evidence: true,
                safe_by_policy: true,
            },
            CacheGcCandidateV1 {
                cache_key: "remove-safe".to_string(),
                referenced_by_run_evidence: false,
                safe_by_policy: true,
            },
        ]);
        assert_eq!(report.entries.len(), 2);
        let keep = report
            .entries
            .iter()
            .find(|entry| entry.cache_key == "keep-evidence")
            .expect("keep entry");
        assert!(!keep.would_remove);
        let remove = report
            .entries
            .iter()
            .find(|entry| entry.cache_key == "remove-safe")
            .expect("remove entry");
        assert!(remove.would_remove);
        assert!(remove.reason.contains("unreferenced"));
    }

    #[test]
    fn g155_cache_import_bundle_refuses_unsafe_entries() {
        let bundle = CachePortableBundleV1 {
            bundle_id: "cache-bundle-155".to_string(),
            entries: vec![CachePortableEntryV1 {
                cache_key: "key-a".to_string(),
                schema_fingerprint: "schema-v1".to_string(),
                policy_fingerprint: "policy-v1".to_string(),
                integrity_verified: true,
            }],
        };
        validate_cache_import_bundle(&bundle, "schema-v1", "policy-v1").expect("safe import");

        let mut unsafe_bundle = bundle;
        unsafe_bundle.entries[0].integrity_verified = false;
        let error = validate_cache_import_bundle(&unsafe_bundle, "schema-v1", "policy-v1")
            .expect_err("must refuse unsafe cache entry");
        assert!(error.contains("integrity is not verified"));
    }

    #[test]
    fn g156_artifact_lineage_query_answers_ancestors_descendants_and_producers() {
        let index = build_artifact_lineage_query_index(vec![
            ArtifactLineageRecordV1 {
                artifact_id: "a1".to_string(),
                producer_node: "n1".to_string(),
                consumers: vec!["n2".to_string()],
                parent_artifacts: vec![],
                cache_source: None,
                replay_source: None,
            },
            ArtifactLineageRecordV1 {
                artifact_id: "a2".to_string(),
                producer_node: "n2".to_string(),
                consumers: vec!["n3".to_string()],
                parent_artifacts: vec!["a1".to_string()],
                cache_source: Some("cache://a2".to_string()),
                replay_source: None,
            },
            ArtifactLineageRecordV1 {
                artifact_id: "a3".to_string(),
                producer_node: "n3".to_string(),
                consumers: vec![],
                parent_artifacts: vec!["a2".to_string()],
                cache_source: None,
                replay_source: Some("run-older/a3".to_string()),
            },
        ]);
        assert_eq!(index.producer_by_artifact.get("a3"), Some(&"n3".to_string()));
        assert_eq!(lineage_ancestors(&index, "a3"), vec!["a1".to_string(), "a2".to_string()]);
        assert_eq!(lineage_descendants(&index, "a1"), vec!["a2".to_string(), "a3".to_string()]);
    }

    #[test]
    fn g157_artifact_preview_is_safe_for_large_or_binary_content() {
        let binary = build_safe_artifact_preview("application/octet-stream", &[0, 1, 2, 3], 1024);
        assert_eq!(binary.preview_kind, "summary");
        assert!(binary.preview.contains("binary artifact preview suppressed"));

        let large_text = build_safe_artifact_preview("text/plain", &vec![b'a'; 2000], 256);
        assert_eq!(large_text.preview_kind, "summary");
        assert!(large_text.preview.contains("too large"));

        let redacted = build_safe_artifact_preview(
            "text/plain",
            b"password=abc12345\nok",
            1024,
        );
        assert_eq!(redacted.preview_kind, "inline");
        assert!(redacted.preview.contains("[REDACTED]"));
    }

    #[test]
    fn g158_artifact_index_migration_refuses_semantic_loss_paths() {
        let refused = evaluate_artifact_index_migration(&ArtifactIndexMigrationRequestV1 {
            from_schema: "artifact-index/v1".to_string(),
            to_schema: "artifact-index/v2".to_string(),
            semantic_loss_detected: true,
        });
        assert!(!refused.migrated);
        assert!(refused.reason.contains("semantic-loss"));

        let accepted = evaluate_artifact_index_migration(&ArtifactIndexMigrationRequestV1 {
            from_schema: "artifact-index/v1".to_string(),
            to_schema: "artifact-index/v2".to_string(),
            semantic_loss_detected: false,
        });
        assert!(accepted.migrated);
    }
}
