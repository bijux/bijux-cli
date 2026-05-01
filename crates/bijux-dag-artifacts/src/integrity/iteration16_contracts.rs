use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::{
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
}
