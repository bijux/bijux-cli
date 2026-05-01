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
            return Err(format!(
                "artifact '{}' has empty producer_app",
                entry.artifact_id
            ));
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

#[cfg(test)]
mod tests {
    use super::{
        validate_domain_neutral_artifact_roles, verify_sample_identity_propagation,
        ArtifactRoleKindV1, ArtifactRoleMetadataV1, ArtifactSampleIdentityV1,
        SampleIdentityMismatchActionV1, SampleIdentityPolicyV1,
    };
    use std::collections::BTreeMap;

    #[test]
    fn g171_domain_neutral_artifact_roles_are_typed_and_enforceable() {
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
    fn g172_sample_identity_propagation_supports_warn_and_refuse_policies() {
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
}
