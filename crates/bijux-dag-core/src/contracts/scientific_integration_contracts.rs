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

#[cfg(test)]
mod tests {
    use super::{
        validate_domain_neutral_artifact_roles, ArtifactRoleKindV1, ArtifactRoleMetadataV1,
    };

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
}
