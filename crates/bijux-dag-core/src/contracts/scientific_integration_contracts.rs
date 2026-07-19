use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Domain-neutral artifact role kinds declared by mounted scientific apps.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactRoleKindV1 {
    Fastq,
    Bam,
    Vcf,
    Tsv,
    Report,
    Reference,
    Index,
    Model,
    Binary,
}

/// Artifact role metadata attached by apps without embedding domain math in core.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRoleMetadataV1 {
    pub artifact_id: String,
    pub producer_app: String,
    pub role: ArtifactRoleKindV1,
}

/// Validation report for domain-neutral role declarations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRoleValidationReportV1 {
    pub artifact_count: usize,
    pub distinct_roles: Vec<ArtifactRoleKindV1>,
    pub counts_by_role: BTreeMap<String, u64>,
}

/// Validate and summarize app-supplied domain-neutral artifact roles.
pub fn validate_domain_neutral_artifact_roles(
    roles: &[ArtifactRoleMetadataV1],
) -> Result<ArtifactRoleValidationReportV1, String> {
    if roles.is_empty() {
        return Err("artifact role validation requires at least one artifact role".to_string());
    }

    let mut seen_artifacts = BTreeSet::new();
    let mut distinct_roles = BTreeSet::new();
    let mut counts_by_role = BTreeMap::<String, u64>::new();

    for entry in roles {
        if entry.artifact_id.trim().is_empty() {
            return Err("artifact role entry has empty artifact_id".to_string());
        }
        if entry.producer_app.trim().is_empty() {
            return Err(format!("artifact '{}' has empty producer_app", entry.artifact_id));
        }
        if !seen_artifacts.insert(entry.artifact_id.clone()) {
            return Err(format!(
                "artifact role mapping must be one-to-one, duplicate artifact_id '{}'",
                entry.artifact_id
            ));
        }
        distinct_roles.insert(entry.role.clone());
        let key = serde_json::to_string(&entry.role)
            .unwrap_or_else(|_| "\"unknown\"".to_string())
            .trim_matches('"')
            .to_string();
        *counts_by_role.entry(key).or_insert(0) += 1;
    }

    Ok(ArtifactRoleValidationReportV1 {
        artifact_count: roles.len(),
        distinct_roles: distinct_roles.into_iter().collect::<Vec<_>>(),
        counts_by_role,
    })
}

/// Policy for handling sample identity mismatches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SampleIdentityMismatchActionV1 {
    Warn,
    Refuse,
}

/// Declared sample identity fields and mismatch handling policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SampleIdentityPolicyV1 {
    pub required_fields: Vec<String>,
    pub mismatch_action: SampleIdentityMismatchActionV1,
}

/// Artifact-scoped identity values supplied by an app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactSampleIdentityV1 {
    pub artifact_id: String,
    pub values: BTreeMap<String, String>,
}

/// Identity propagation report across artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SampleIdentityPropagationReportV1 {
    pub mismatched_fields: Vec<String>,
    pub missing_fields: Vec<String>,
    pub warnings: Vec<String>,
    pub refusals: Vec<String>,
    pub admitted: bool,
}

/// Verify required sample identity field propagation across artifacts.
pub fn verify_sample_identity_propagation(
    policy: &SampleIdentityPolicyV1,
    artifacts: &[ArtifactSampleIdentityV1],
) -> Result<SampleIdentityPropagationReportV1, String> {
    if policy.required_fields.is_empty() {
        return Err("sample identity policy requires at least one required field".to_string());
    }
    if artifacts.is_empty() {
        return Err("sample identity propagation requires at least one artifact".to_string());
    }
    let baseline = artifacts
        .first()
        .ok_or_else(|| "sample identity propagation requires at least one artifact".to_string())?;

    let mut missing_fields = BTreeSet::<String>::new();
    let mut mismatched_fields = BTreeSet::<String>::new();
    let mut warnings = Vec::<String>::new();
    let mut refusals = Vec::<String>::new();

    for field in &policy.required_fields {
        if field.trim().is_empty() {
            return Err("sample identity policy has empty required field".to_string());
        }
        let Some(baseline_value) = baseline.values.get(field) else {
            missing_fields.insert(field.clone());
            continue;
        };
        if baseline_value.trim().is_empty() {
            missing_fields.insert(field.clone());
            continue;
        }
        for artifact in artifacts.iter().skip(1) {
            let current = artifact.values.get(field).cloned().unwrap_or_default();
            if current.trim().is_empty() {
                missing_fields.insert(field.clone());
                continue;
            }
            if current != *baseline_value {
                mismatched_fields.insert(field.clone());
            }
        }
    }

    let mismatched_fields = mismatched_fields.into_iter().collect::<Vec<_>>();
    let missing_fields = missing_fields.into_iter().collect::<Vec<_>>();

    if !missing_fields.is_empty() {
        refusals.push(format!(
            "sample identity propagation missing required fields: {}",
            missing_fields.join(", ")
        ));
    }
    if !mismatched_fields.is_empty() {
        let message = format!(
            "sample identity mismatch detected for fields: {}",
            mismatched_fields.join(", ")
        );
        match policy.mismatch_action {
            SampleIdentityMismatchActionV1::Warn => warnings.push(message),
            SampleIdentityMismatchActionV1::Refuse => refusals.push(message),
        }
    }

    let admitted = refusals.is_empty();
    Ok(SampleIdentityPropagationReportV1 {
        mismatched_fields,
        missing_fields,
        warnings,
        refusals,
        admitted,
    })
}

/// Alias resolution policy for reference identity metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceAliasPolicyV1 {
    ExactOnly,
    AliasAllowed,
}

/// Reference identity metadata attached to evidence for reference-sensitive workloads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceIdentityMetadataV1 {
    pub reference_id: String,
    pub build: String,
    pub checksum_sha256: String,
    pub alias_policy: ReferenceAliasPolicyV1,
    pub compatible_with: Vec<String>,
}

/// Validate reference identity metadata for attachment into core evidence.
pub fn validate_reference_identity_metadata(
    metadata: &ReferenceIdentityMetadataV1,
) -> Result<(), String> {
    for (field, value) in [
        ("reference_id", metadata.reference_id.as_str()),
        ("build", metadata.build.as_str()),
        ("checksum_sha256", metadata.checksum_sha256.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!("reference identity metadata requires {field}"));
        }
    }
    if !metadata.checksum_sha256.starts_with("sha256:") {
        return Err("reference identity checksum_sha256 must use sha256: prefix".to_string());
    }
    if metadata.compatible_with.iter().any(|value| value.trim().is_empty()) {
        return Err("reference identity compatible_with must not contain empty values".to_string());
    }
    Ok(())
}

/// Scientific finding class normalized across apps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScientificFindingClassV1 {
    Caveat,
    Uncertainty,
    Warning,
    Refusal,
    PromotedCheck,
}

/// Scientific finding severity semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScientificFindingModeV1 {
    Advisory,
    Enforced,
}

/// Standard scientific finding record emitted by mounted apps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScientificFindingV1 {
    pub finding_id: String,
    pub code: String,
    pub class: ScientificFindingClassV1,
    pub mode: ScientificFindingModeV1,
    pub message: String,
}

/// Normalized findings report with explicit blocking findings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScientificFindingReportV1 {
    pub finding_count: usize,
    pub blocking_codes: Vec<String>,
}

/// Normalize and validate scientific findings into a standard structure.
pub fn normalize_scientific_findings(
    findings: &[ScientificFindingV1],
) -> Result<ScientificFindingReportV1, String> {
    let mut seen_ids = BTreeSet::new();
    let mut blocking_codes = Vec::new();

    for finding in findings {
        for (field, value) in [
            ("finding_id", finding.finding_id.as_str()),
            ("code", finding.code.as_str()),
            ("message", finding.message.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(format!("scientific finding requires {field}"));
            }
        }
        if !seen_ids.insert(finding.finding_id.clone()) {
            return Err(format!(
                "duplicate scientific finding_id '{}' is not allowed",
                finding.finding_id
            ));
        }
        if matches!(finding.mode, ScientificFindingModeV1::Enforced)
            && matches!(
                finding.class,
                ScientificFindingClassV1::Refusal
                    | ScientificFindingClassV1::Warning
                    | ScientificFindingClassV1::Uncertainty
            )
        {
            blocking_codes.push(finding.code.clone());
        }
    }
    blocking_codes.sort();
    blocking_codes.dedup();

    Ok(ScientificFindingReportV1 { finding_count: findings.len(), blocking_codes })
}

/// Truth-set comparison record attached by apps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TruthSetComparisonV1 {
    pub comparison_id: String,
    pub truth_set_id: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub limitations: Vec<String>,
}

/// Evidence container for truth-set comparisons.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TruthSetEvidenceEnvelopeV1 {
    pub run_id: String,
    pub comparisons: Vec<TruthSetComparisonV1>,
}

/// Attach one truth-set comparison into run evidence with stable validation.
pub fn attach_truth_set_comparison(
    envelope: &mut TruthSetEvidenceEnvelopeV1,
    comparison: TruthSetComparisonV1,
) -> Result<(), String> {
    if envelope.run_id.trim().is_empty() {
        return Err("truth-set evidence envelope requires run_id".to_string());
    }
    for (field, value) in [
        ("comparison_id", comparison.comparison_id.as_str()),
        ("truth_set_id", comparison.truth_set_id.as_str()),
        ("metric_name", comparison.metric_name.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!("truth-set comparison requires {field}"));
        }
    }
    if !comparison.metric_value.is_finite() {
        return Err("truth-set comparison metric_value must be finite".to_string());
    }
    if envelope
        .comparisons
        .iter()
        .any(|existing| existing.comparison_id == comparison.comparison_id)
    {
        return Err(format!("duplicate truth-set comparison_id '{}'", comparison.comparison_id));
    }
    if comparison.limitations.iter().any(|limitation| limitation.trim().is_empty()) {
        return Err("truth-set comparison limitations must not contain empty entries".to_string());
    }

    envelope.comparisons.push(comparison);
    envelope.comparisons.sort_by(|left, right| left.comparison_id.cmp(&right.comparison_id));
    Ok(())
}

/// Scientific run trust class attached by mounted apps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScientificRunTrustClassV1 {
    Exploratory,
    Operational,
    Audit,
    Certification,
    PublicationCandidate,
}

/// Promotion decision for scientific run trust classes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScientificTrustPromotionDecisionV1 {
    pub class: ScientificRunTrustClassV1,
    pub promotable: bool,
    pub reason: String,
}

/// Evaluate whether a scientific run class can be promoted under evidence and policy gates.
pub fn evaluate_scientific_trust_promotion(
    class: ScientificRunTrustClassV1,
    evidence_complete: bool,
    app_policy_allows: bool,
) -> ScientificTrustPromotionDecisionV1 {
    let promotable = match class {
        ScientificRunTrustClassV1::Exploratory => true,
        ScientificRunTrustClassV1::Operational => app_policy_allows,
        ScientificRunTrustClassV1::Audit
        | ScientificRunTrustClassV1::Certification
        | ScientificRunTrustClassV1::PublicationCandidate => evidence_complete && app_policy_allows,
    };
    let reason = if promotable {
        "trust-class promotion requirements satisfied".to_string()
    } else {
        match class {
            ScientificRunTrustClassV1::Exploratory => "unexpected exploratory refusal".to_string(),
            ScientificRunTrustClassV1::Operational => {
                "operational promotion requires app policy approval".to_string()
            }
            ScientificRunTrustClassV1::Audit
            | ScientificRunTrustClassV1::Certification
            | ScientificRunTrustClassV1::PublicationCandidate => {
                "promotion requires complete evidence and app policy approval".to_string()
            }
        }
    };
    ScientificTrustPromotionDecisionV1 { class, promotable, reason }
}

/// Generic override category for scientific runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScientificOverrideTypeV1 {
    Reference,
    Sample,
    Qc,
    Prerequisite,
    EvidencePolicy,
}

/// Recorded scientific override action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScientificOverrideRecordV1 {
    pub override_id: String,
    pub override_type: ScientificOverrideTypeV1,
    pub actor: String,
    pub reason: String,
    pub risk_level: String,
}

/// Audit report for scientific overrides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScientificOverrideAuditReportV1 {
    pub override_count: usize,
    pub high_risk_override_ids: Vec<String>,
}

/// Audit overrides and expose high-risk records for run-history visibility.
pub fn audit_scientific_overrides(
    overrides: &[ScientificOverrideRecordV1],
) -> Result<ScientificOverrideAuditReportV1, String> {
    let mut high_risk_override_ids = Vec::new();
    let mut seen_ids = BTreeSet::new();
    for entry in overrides {
        for (field, value) in [
            ("override_id", entry.override_id.as_str()),
            ("actor", entry.actor.as_str()),
            ("reason", entry.reason.as_str()),
            ("risk_level", entry.risk_level.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(format!("scientific override requires {field}"));
            }
        }
        if !seen_ids.insert(entry.override_id.clone()) {
            return Err(format!(
                "duplicate scientific override_id '{}' is not allowed",
                entry.override_id
            ));
        }
        if entry.risk_level.eq_ignore_ascii_case("high") {
            high_risk_override_ids.push(entry.override_id.clone());
        }
    }
    high_risk_override_ids.sort();
    Ok(ScientificOverrideAuditReportV1 { override_count: overrides.len(), high_risk_override_ids })
}

/// Uncertainty state carried by scientific inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScientificUncertaintyStateV1 {
    Missing,
    Partial,
    Contradictory,
    Advisory,
    Clear,
}

/// Input uncertainty declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScientificUncertaintyInputV1 {
    pub field: String,
    pub state: ScientificUncertaintyStateV1,
    pub assumption_acknowledged: bool,
}

/// Uncertainty evaluation report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScientificUncertaintyReportV1 {
    pub blocking_fields: Vec<String>,
    pub advisory_fields: Vec<String>,
    pub admitted: bool,
}

/// Evaluate uncertainty declarations and refuse silent guesses.
pub fn evaluate_scientific_uncertainty(
    inputs: &[ScientificUncertaintyInputV1],
) -> Result<ScientificUncertaintyReportV1, String> {
    if inputs.is_empty() {
        return Err("scientific uncertainty evaluation requires at least one input".to_string());
    }
    let mut blocking_fields = Vec::<String>::new();
    let mut advisory_fields = Vec::<String>::new();
    for input in inputs {
        if input.field.trim().is_empty() {
            return Err("scientific uncertainty field must not be empty".to_string());
        }
        match input.state {
            ScientificUncertaintyStateV1::Clear => {}
            ScientificUncertaintyStateV1::Advisory => advisory_fields.push(input.field.clone()),
            ScientificUncertaintyStateV1::Missing
            | ScientificUncertaintyStateV1::Partial
            | ScientificUncertaintyStateV1::Contradictory => {
                if !input.assumption_acknowledged {
                    blocking_fields.push(input.field.clone());
                } else {
                    advisory_fields.push(input.field.clone());
                }
            }
        }
    }
    blocking_fields.sort();
    blocking_fields.dedup();
    advisory_fields.sort();
    advisory_fields.dedup();
    Ok(ScientificUncertaintyReportV1 {
        admitted: blocking_fields.is_empty(),
        blocking_fields,
        advisory_fields,
    })
}

/// Cross-app evidence lineage link.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossAppEvidenceLinkV1 {
    pub source_app: String,
    pub target_app: String,
    pub run_id: String,
    pub artifact_id: String,
    pub evidence_id: String,
}

/// Cross-app evidence linkage report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossAppEvidenceLinkReportV1 {
    pub link_count: usize,
    pub participating_apps: Vec<String>,
}

/// Validate cross-app evidence linkage through shared lineage kernel fields.
pub fn validate_cross_app_evidence_links(
    links: &[CrossAppEvidenceLinkV1],
) -> Result<CrossAppEvidenceLinkReportV1, String> {
    if links.is_empty() {
        return Err("cross-app evidence link validation requires at least one link".to_string());
    }
    let mut participating_apps = BTreeSet::<String>::new();
    let mut seen_link_keys = BTreeSet::<String>::new();

    for link in links {
        for (field, value) in [
            ("source_app", link.source_app.as_str()),
            ("target_app", link.target_app.as_str()),
            ("run_id", link.run_id.as_str()),
            ("artifact_id", link.artifact_id.as_str()),
            ("evidence_id", link.evidence_id.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(format!("cross-app evidence link requires {field}"));
            }
        }
        if link.source_app == link.target_app {
            return Err(
                "cross-app evidence link requires distinct source_app and target_app".to_string()
            );
        }
        participating_apps.insert(link.source_app.clone());
        participating_apps.insert(link.target_app.clone());
        let key = format!(
            "{}:{}:{}:{}:{}",
            link.source_app, link.target_app, link.run_id, link.artifact_id, link.evidence_id
        );
        if !seen_link_keys.insert(key) {
            return Err("duplicate cross-app evidence link is not allowed".to_string());
        }
    }

    Ok(CrossAppEvidenceLinkReportV1 {
        link_count: links.len(),
        participating_apps: participating_apps.into_iter().collect::<Vec<_>>(),
    })
}

/// Evidence strength class for scientific promotion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScientificEvidenceStrengthV1 {
    Simulated,
    Advisory,
    Operational,
    AuditVerified,
    CertificationGrade,
}

/// Strict promotion refusal decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScientificPromotionRefusalDecisionV1 {
    pub class: ScientificRunTrustClassV1,
    pub evidence_strength: ScientificEvidenceStrengthV1,
    pub promoted: bool,
    pub reason: String,
}

/// Enforce strict scientific promotion policy for high-trust classes.
pub fn enforce_strict_scientific_promotion_refusal(
    class: ScientificRunTrustClassV1,
    evidence_strength: ScientificEvidenceStrengthV1,
) -> ScientificPromotionRefusalDecisionV1 {
    let high_trust = matches!(
        class,
        ScientificRunTrustClassV1::Certification | ScientificRunTrustClassV1::PublicationCandidate
    );
    let promoted = if high_trust {
        matches!(
            evidence_strength,
            ScientificEvidenceStrengthV1::CertificationGrade
                | ScientificEvidenceStrengthV1::AuditVerified
        )
    } else {
        !matches!(evidence_strength, ScientificEvidenceStrengthV1::Simulated)
    };

    let reason = if promoted {
        "promotion allowed under strict scientific evidence policy".to_string()
    } else if high_trust {
        "certification/publication promotion refused: evidence is simulated or advisory".to_string()
    } else {
        "promotion refused: simulated evidence cannot be promoted".to_string()
    };

    ScientificPromotionRefusalDecisionV1 { class, evidence_strength, promoted, reason }
}

#[cfg(test)]
mod tests {
    use super::{
        attach_truth_set_comparison, audit_scientific_overrides,
        enforce_strict_scientific_promotion_refusal, evaluate_scientific_trust_promotion,
        evaluate_scientific_uncertainty, normalize_scientific_findings,
        validate_cross_app_evidence_links, validate_domain_neutral_artifact_roles,
        validate_reference_identity_metadata, verify_sample_identity_propagation,
        ArtifactRoleKindV1, ArtifactRoleMetadataV1, ArtifactSampleIdentityV1,
        CrossAppEvidenceLinkV1, ReferenceAliasPolicyV1, ReferenceIdentityMetadataV1,
        SampleIdentityMismatchActionV1, SampleIdentityPolicyV1, ScientificEvidenceStrengthV1,
        ScientificFindingClassV1, ScientificFindingModeV1, ScientificFindingV1,
        ScientificOverrideRecordV1, ScientificOverrideTypeV1, ScientificRunTrustClassV1,
        ScientificUncertaintyInputV1, ScientificUncertaintyStateV1, TruthSetComparisonV1,
        TruthSetEvidenceEnvelopeV1,
    };
    use std::collections::BTreeMap;

    #[test]
    fn domain_neutral_artifact_roles_are_typed_and_enforceable() {
        let report = validate_domain_neutral_artifact_roles(&[
            ArtifactRoleMetadataV1 {
                artifact_id: "artifact-fastq-1".to_string(),
                producer_app: "genomics".to_string(),
                role: ArtifactRoleKindV1::Fastq,
            },
            ArtifactRoleMetadataV1 {
                artifact_id: "artifact-bam-1".to_string(),
                producer_app: "genomics".to_string(),
                role: ArtifactRoleKindV1::Bam,
            },
            ArtifactRoleMetadataV1 {
                artifact_id: "artifact-report-1".to_string(),
                producer_app: "proteomics".to_string(),
                role: ArtifactRoleKindV1::Report,
            },
        ])
        .expect("role mapping should validate");
        assert_eq!(report.artifact_count, 3);
        assert_eq!(report.counts_by_role.get("fastq"), Some(&1));
        assert_eq!(report.counts_by_role.get("bam"), Some(&1));
        assert_eq!(report.counts_by_role.get("report"), Some(&1));
    }

    #[test]
    fn sample_identity_propagation_supports_warn_and_refuse_policies() {
        let artifacts = vec![
            ArtifactSampleIdentityV1 {
                artifact_id: "a-fastq".to_string(),
                values: BTreeMap::from([
                    ("sample_id".to_string(), "sample-7".to_string()),
                    ("subject_id".to_string(), "subject-a".to_string()),
                ]),
            },
            ArtifactSampleIdentityV1 {
                artifact_id: "a-bam".to_string(),
                values: BTreeMap::from([
                    ("sample_id".to_string(), "sample-8".to_string()),
                    ("subject_id".to_string(), "subject-a".to_string()),
                ]),
            },
        ];

        let warn_report = verify_sample_identity_propagation(
            &SampleIdentityPolicyV1 {
                required_fields: vec!["sample_id".to_string(), "subject_id".to_string()],
                mismatch_action: SampleIdentityMismatchActionV1::Warn,
            },
            &artifacts,
        )
        .expect("warn policy report");
        assert!(warn_report.admitted);
        assert_eq!(warn_report.mismatched_fields, vec!["sample_id".to_string()]);
        assert!(!warn_report.warnings.is_empty());

        let refuse_report = verify_sample_identity_propagation(
            &SampleIdentityPolicyV1 {
                required_fields: vec!["sample_id".to_string(), "subject_id".to_string()],
                mismatch_action: SampleIdentityMismatchActionV1::Refuse,
            },
            &artifacts,
        )
        .expect("refuse policy report");
        assert!(!refuse_report.admitted);
        assert!(!refuse_report.refusals.is_empty());
    }

    #[test]
    fn reference_identity_metadata_is_typed_and_checksum_guarded() {
        validate_reference_identity_metadata(&ReferenceIdentityMetadataV1 {
            reference_id: "grch38".to_string(),
            build: "grch38.p14".to_string(),
            checksum_sha256: "sha256:referenceabc".to_string(),
            alias_policy: ReferenceAliasPolicyV1::AliasAllowed,
            compatible_with: vec!["grch38".to_string(), "hg38".to_string()],
        })
        .expect("reference identity metadata should validate");

        let err = validate_reference_identity_metadata(&ReferenceIdentityMetadataV1 {
            reference_id: "grch38".to_string(),
            build: "grch38.p14".to_string(),
            checksum_sha256: "bad-checksum".to_string(),
            alias_policy: ReferenceAliasPolicyV1::ExactOnly,
            compatible_with: vec!["grch38".to_string()],
        })
        .expect_err("non-sha256 checksum must be rejected");
        assert!(err.contains("sha256"));
    }

    #[test]
    fn scientific_findings_use_standard_advisory_and_enforced_shapes() {
        let report = normalize_scientific_findings(&[
            ScientificFindingV1 {
                finding_id: "f-1".to_string(),
                code: "SCI_WARN_COVERAGE".to_string(),
                class: ScientificFindingClassV1::Warning,
                mode: ScientificFindingModeV1::Advisory,
                message: "coverage is below preferred threshold".to_string(),
            },
            ScientificFindingV1 {
                finding_id: "f-2".to_string(),
                code: "SCI_REFUSAL_REFERENCE".to_string(),
                class: ScientificFindingClassV1::Refusal,
                mode: ScientificFindingModeV1::Enforced,
                message: "reference build mismatch".to_string(),
            },
        ])
        .expect("findings should normalize");
        assert_eq!(report.finding_count, 2);
        assert_eq!(report.blocking_codes, vec!["SCI_REFUSAL_REFERENCE".to_string()]);
    }

    #[test]
    fn truth_set_comparisons_are_attachable_with_explicit_limitations() {
        let mut envelope =
            TruthSetEvidenceEnvelopeV1 { run_id: "run-18".to_string(), comparisons: Vec::new() };
        attach_truth_set_comparison(
            &mut envelope,
            TruthSetComparisonV1 {
                comparison_id: "cmp-1".to_string(),
                truth_set_id: "truth-small".to_string(),
                metric_name: "f1_score".to_string(),
                metric_value: 0.97,
                limitations: vec!["small cohort".to_string()],
            },
        )
        .expect("truth-set comparison should attach");
        assert_eq!(envelope.comparisons.len(), 1);
        assert_eq!(envelope.comparisons[0].metric_name, "f1_score");
    }

    #[test]
    fn scientific_trust_class_promotion_is_evidence_and_policy_gated() {
        let exploratory = evaluate_scientific_trust_promotion(
            ScientificRunTrustClassV1::Exploratory,
            false,
            false,
        );
        assert!(exploratory.promotable);

        let certification_refused = evaluate_scientific_trust_promotion(
            ScientificRunTrustClassV1::Certification,
            false,
            true,
        );
        assert!(!certification_refused.promotable);
        assert!(certification_refused.reason.contains("complete evidence"));

        let publication_allowed = evaluate_scientific_trust_promotion(
            ScientificRunTrustClassV1::PublicationCandidate,
            true,
            true,
        );
        assert!(publication_allowed.promotable);
    }

    #[test]
    fn scientific_overrides_are_audited_with_high_risk_visibility() {
        let report = audit_scientific_overrides(&[
            ScientificOverrideRecordV1 {
                override_id: "ovr-1".to_string(),
                override_type: ScientificOverrideTypeV1::Reference,
                actor: "operator-a".to_string(),
                reason: "patched reference alias mismatch".to_string(),
                risk_level: "high".to_string(),
            },
            ScientificOverrideRecordV1 {
                override_id: "ovr-2".to_string(),
                override_type: ScientificOverrideTypeV1::Qc,
                actor: "operator-b".to_string(),
                reason: "accepted lower depth under outage".to_string(),
                risk_level: "medium".to_string(),
            },
        ])
        .expect("override audit should succeed");
        assert_eq!(report.override_count, 2);
        assert_eq!(report.high_risk_override_ids, vec!["ovr-1".to_string()]);
    }

    #[test]
    fn uncertainty_is_first_class_and_blocks_silent_guesses() {
        let report = evaluate_scientific_uncertainty(&[
            ScientificUncertaintyInputV1 {
                field: "tumor_purity".to_string(),
                state: ScientificUncertaintyStateV1::Missing,
                assumption_acknowledged: false,
            },
            ScientificUncertaintyInputV1 {
                field: "panel_version".to_string(),
                state: ScientificUncertaintyStateV1::Advisory,
                assumption_acknowledged: true,
            },
            ScientificUncertaintyInputV1 {
                field: "reference_synonym".to_string(),
                state: ScientificUncertaintyStateV1::Contradictory,
                assumption_acknowledged: true,
            },
        ])
        .expect("uncertainty evaluation should succeed");
        assert!(!report.admitted);
        assert_eq!(report.blocking_fields, vec!["tumor_purity".to_string()]);
        assert_eq!(
            report.advisory_fields,
            vec!["panel_version".to_string(), "reference_synonym".to_string()]
        );
    }

    #[test]
    fn cross_app_evidence_links_share_core_lineage_kernel() {
        let report = validate_cross_app_evidence_links(&[
            CrossAppEvidenceLinkV1 {
                source_app: "genomics".to_string(),
                target_app: "proteomics".to_string(),
                run_id: "run-18".to_string(),
                artifact_id: "artifact-ref".to_string(),
                evidence_id: "evidence-a".to_string(),
            },
            CrossAppEvidenceLinkV1 {
                source_app: "proteomics".to_string(),
                target_app: "pollenomics".to_string(),
                run_id: "run-18".to_string(),
                artifact_id: "artifact-report".to_string(),
                evidence_id: "evidence-b".to_string(),
            },
        ])
        .expect("cross-app evidence links should validate");
        assert_eq!(report.link_count, 2);
        assert_eq!(
            report.participating_apps,
            vec!["genomics".to_string(), "pollenomics".to_string(), "proteomics".to_string(),]
        );
    }

    #[test]
    fn strict_promotion_refuses_simulated_or_advisory_for_certification_grade() {
        let refused = enforce_strict_scientific_promotion_refusal(
            ScientificRunTrustClassV1::Certification,
            ScientificEvidenceStrengthV1::Advisory,
        );
        assert!(!refused.promoted);
        assert!(refused.reason.contains("refused"));

        let allowed = enforce_strict_scientific_promotion_refusal(
            ScientificRunTrustClassV1::PublicationCandidate,
            ScientificEvidenceStrengthV1::CertificationGrade,
        );
        assert!(allowed.promoted);
    }
}
