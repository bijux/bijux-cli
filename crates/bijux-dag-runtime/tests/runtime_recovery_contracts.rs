use bijux_dag_runtime::{recovery_action_required, RecoveryInput};

#[test]
fn recovery_required_for_checkpoint_without_terminal_completion() {
    assert!(recovery_action_required(&RecoveryInput {
        has_checkpoint: true,
        terminal_state_seen: false,
        partial_artifacts_present: false,
    }));
}
