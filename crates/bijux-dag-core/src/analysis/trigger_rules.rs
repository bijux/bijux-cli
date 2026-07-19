use crate::TriggerRule;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UpstreamTerminalOutcome {
    Success,
    Cached,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TriggerRuleEvaluation {
    pub trigger_rule: TriggerRule,
    pub satisfied: bool,
    pub reason: String,
    pub parent_outcomes: Vec<UpstreamTerminalOutcome>,
}

pub fn evaluate_trigger_rule(
    trigger_rule: &TriggerRule,
    parent_outcomes: &[UpstreamTerminalOutcome],
) -> TriggerRuleEvaluation {
    let success = parent_outcomes
        .iter()
        .filter(|outcome| {
            matches!(outcome, UpstreamTerminalOutcome::Success | UpstreamTerminalOutcome::Cached)
        })
        .count();
    let failed = parent_outcomes
        .iter()
        .filter(|outcome| matches!(outcome, UpstreamTerminalOutcome::Failed))
        .count();
    let total = parent_outcomes.len();

    let (satisfied, reason) = match trigger_rule {
        TriggerRule::AllSuccess => {
            (success == total, "requires every upstream to complete in success or cached status")
        }
        TriggerRule::AnySuccess => {
            (success > 0, "requires at least one upstream to complete in success or cached status")
        }
        TriggerRule::AllDone => (true, "accepts any terminal upstream status"),
        TriggerRule::NoneFailed => (failed == 0, "requires upstream completion without failures"),
    };

    TriggerRuleEvaluation {
        trigger_rule: trigger_rule.clone(),
        satisfied,
        reason: reason.to_string(),
        parent_outcomes: parent_outcomes.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::{evaluate_trigger_rule, UpstreamTerminalOutcome};
    use crate::TriggerRule;

    #[test]
    fn all_success_accepts_cached_and_rejects_skipped() {
        let cached = evaluate_trigger_rule(
            &TriggerRule::AllSuccess,
            &[UpstreamTerminalOutcome::Success, UpstreamTerminalOutcome::Cached],
        );
        assert!(cached.satisfied);

        let skipped = evaluate_trigger_rule(
            &TriggerRule::AllSuccess,
            &[UpstreamTerminalOutcome::Success, UpstreamTerminalOutcome::Skipped],
        );
        assert!(!skipped.satisfied);
    }

    #[test]
    fn any_success_requires_a_success_like_parent() {
        let satisfied = evaluate_trigger_rule(
            &TriggerRule::AnySuccess,
            &[UpstreamTerminalOutcome::Failed, UpstreamTerminalOutcome::Cached],
        );
        assert!(satisfied.satisfied);

        let blocked = evaluate_trigger_rule(
            &TriggerRule::AnySuccess,
            &[UpstreamTerminalOutcome::Skipped, UpstreamTerminalOutcome::Failed],
        );
        assert!(!blocked.satisfied);
    }

    #[test]
    fn all_done_accepts_failed_and_skipped_upstreams() {
        let evaluation = evaluate_trigger_rule(
            &TriggerRule::AllDone,
            &[UpstreamTerminalOutcome::Failed, UpstreamTerminalOutcome::Skipped],
        );
        assert!(evaluation.satisfied);
    }

    #[test]
    fn none_failed_rejects_failed_and_accepts_skipped() {
        let allowed = evaluate_trigger_rule(
            &TriggerRule::NoneFailed,
            &[UpstreamTerminalOutcome::Skipped, UpstreamTerminalOutcome::Cached],
        );
        assert!(allowed.satisfied);

        let blocked = evaluate_trigger_rule(
            &TriggerRule::NoneFailed,
            &[UpstreamTerminalOutcome::Success, UpstreamTerminalOutcome::Failed],
        );
        assert!(!blocked.satisfied);
    }
}
