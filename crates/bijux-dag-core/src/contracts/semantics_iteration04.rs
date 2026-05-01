use serde::{Deserialize, Serialize};

/// Branch decision evidence artifact with replay identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchDecisionArtifactV1 {
    /// Branch node identifier.
    pub branch_node_id: String,
    /// Serialized predicate input used for decisioning.
    pub predicate_input: String,
    /// Selected decision label.
    pub chosen_branch: String,
    /// Branches not selected.
    pub skipped_branches: Vec<String>,
    /// Stable replay identity for this decision event.
    pub replay_identity: String,
}

/// Build a first-class branch decision artifact.
pub fn build_branch_decision_artifact(
    branch_node_id: &str,
    predicate_input: &str,
    chosen_branch: &str,
    declared_branches: &[String],
    replay_identity: &str,
) -> Result<BranchDecisionArtifactV1, String> {
    for (name, value) in [
        ("branch_node_id", branch_node_id),
        ("predicate_input", predicate_input),
        ("chosen_branch", chosen_branch),
        ("replay_identity", replay_identity),
    ] {
        if value.trim().is_empty() {
            return Err(format!("{name} cannot be empty"));
        }
    }
    if !declared_branches.iter().any(|branch| branch == chosen_branch) {
        return Err("chosen_branch must be declared".to_string());
    }
    let mut skipped_branches = declared_branches
        .iter()
        .filter(|branch| branch.as_str() != chosen_branch)
        .cloned()
        .collect::<Vec<_>>();
    skipped_branches.sort();

    Ok(BranchDecisionArtifactV1 {
        branch_node_id: branch_node_id.to_string(),
        predicate_input: predicate_input.to_string(),
        chosen_branch: chosen_branch.to_string(),
        skipped_branches,
        replay_identity: replay_identity.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::build_branch_decision_artifact;

    #[test]
    fn g031_branch_decision_artifact_persists_chosen_and_skipped_branches() {
        let artifact = build_branch_decision_artifact(
            "branch.qc",
            r#"{"metric":"coverage","value":0.91}"#,
            "pass",
            &["pass".to_string(), "fail".to_string()],
            "run=demo;node=branch.qc;decision=pass",
        )
        .expect("artifact should build");
        assert_eq!(artifact.chosen_branch, "pass");
        assert_eq!(artifact.skipped_branches, vec!["fail".to_string()]);
    }
}
