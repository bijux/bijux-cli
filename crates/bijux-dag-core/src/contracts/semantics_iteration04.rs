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

/// Explicit upstream terminal states for trigger-rule reasoning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpstreamTerminalStateV1 {
    Success,
    Failed,
    Skipped,
    Cancelled,
}

/// Per-node terminal state summary captured in run evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeTerminalStateRecordV1 {
    /// Node identifier.
    pub node_id: String,
    /// Final terminal state.
    pub state: UpstreamTerminalStateV1,
    /// Whether an execution attempt happened.
    pub executed: bool,
}

/// Trigger readiness derived from explicit parent states.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerReadinessFromStatesV1 {
    /// Trigger rule identifier.
    pub trigger_rule: String,
    /// Whether node is runnable.
    pub runnable: bool,
    /// Explanation for decision.
    pub reason: String,
}

/// Evaluate trigger readiness with skipped state modeled explicitly.
pub fn evaluate_trigger_readiness_from_states(
    trigger_rule: &str,
    parent_states: &[UpstreamTerminalStateV1],
) -> Result<TriggerReadinessFromStatesV1, String> {
    if parent_states.is_empty() {
        return Ok(TriggerReadinessFromStatesV1 {
            trigger_rule: trigger_rule.to_string(),
            runnable: true,
            reason: "no parents".to_string(),
        });
    }
    let success = parent_states
        .iter()
        .filter(|state| **state == UpstreamTerminalStateV1::Success)
        .count();
    let failed = parent_states
        .iter()
        .filter(|state| **state == UpstreamTerminalStateV1::Failed)
        .count();
    let cancelled = parent_states
        .iter()
        .filter(|state| **state == UpstreamTerminalStateV1::Cancelled)
        .count();
    let total = parent_states.len();

    let result = match trigger_rule {
        "all_success" => (
            success == total,
            "requires every parent in success and treats skipped as non-success",
        ),
        "all_done" => (success + failed + cancelled <= total, "all terminal states accepted"),
        "any_success" => (success > 0, "requires at least one successful parent"),
        "none_failed" => (failed == 0, "requires zero failed parents"),
        _ => return Err("unsupported trigger_rule".to_string()),
    };

    Ok(TriggerReadinessFromStatesV1 {
        trigger_rule: trigger_rule.to_string(),
        runnable: result.0,
        reason: result.1.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        build_branch_decision_artifact, evaluate_trigger_readiness_from_states,
        UpstreamTerminalStateV1,
    };

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

    #[test]
    fn g032_skipped_state_is_explicit_and_affects_trigger_readiness() {
        let readiness = evaluate_trigger_readiness_from_states(
            "all_success",
            &[UpstreamTerminalStateV1::Success, UpstreamTerminalStateV1::Skipped],
        )
        .expect("trigger readiness should evaluate");
        assert!(!readiness.runnable);
        assert!(readiness.reason.contains("skipped"));
    }
}
