use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::{recovery_action_required, RecoveryInput};

#[test]
fn recovery_required_for_checkpoint_without_terminal_completion() {
    assert!(recovery_action_required(&RecoveryInput {
        has_checkpoint: true,
        terminal_state_seen: false,
        partial_artifacts_present: false,
    }));
}

#[test]
fn recovery_required_for_partial_artifact_or_interrupted_execution() {
    assert!(recovery_action_required(&RecoveryInput {
        has_checkpoint: false,
        terminal_state_seen: false,
        partial_artifacts_present: true,
    }));

    assert!(recovery_action_required(&RecoveryInput {
        has_checkpoint: true,
        terminal_state_seen: false,
        partial_artifacts_present: false,
    }));

    assert!(!recovery_action_required(&RecoveryInput {
        has_checkpoint: true,
        terminal_state_seen: true,
        partial_artifacts_present: false,
    }));
}
