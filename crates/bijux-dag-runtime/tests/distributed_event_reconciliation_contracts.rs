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

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
enum RemoteStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
}

#[derive(Clone, Debug)]
struct RemoteEvent {
    node_id: String,
    seq: u64,
    status: RemoteStatus,
}

#[derive(Default)]
struct FakeDistributedEventSource {
    events: Vec<RemoteEvent>,
}

impl FakeDistributedEventSource {
    fn new(events: Vec<RemoteEvent>) -> Self {
        Self { events }
    }

    fn poll(&self) -> Vec<RemoteEvent> {
        self.events.clone()
    }
}

#[derive(Default)]
struct ReconciledState {
    highest_seq: BTreeMap<String, u64>,
    status: BTreeMap<String, RemoteStatus>,
    completed: BTreeSet<String>,
}

impl ReconciledState {
    fn apply_event(&mut self, event: RemoteEvent) {
        let current_seq = self.highest_seq.get(&event.node_id).copied().unwrap_or(0);
        if event.seq < current_seq {
            return;
        }
        if event.seq == current_seq && self.status.get(&event.node_id) == Some(&event.status) {
            return;
        }

        let terminal_seen = self.completed.contains(&event.node_id);
        if terminal_seen {
            return;
        }

        let is_terminal = matches!(event.status, RemoteStatus::Succeeded | RemoteStatus::Failed);
        if is_terminal {
            self.completed.insert(event.node_id.clone());
        }
        self.highest_seq.insert(event.node_id.clone(), event.seq);
        self.status.insert(event.node_id, event.status);
    }
}

fn ev(node_id: &str, seq: u64, status: RemoteStatus) -> RemoteEvent {
    RemoteEvent { node_id: node_id.to_string(), seq, status }
}

#[test]
fn out_of_order_remote_events_do_not_revert_status() {
    let source = FakeDistributedEventSource::new(vec![
        ev("n1", 2, RemoteStatus::Running),
        ev("n1", 1, RemoteStatus::Queued),
        ev("n1", 3, RemoteStatus::Succeeded),
    ]);
    let mut state = ReconciledState::default();
    for event in source.poll() {
        state.apply_event(event);
    }
    assert_eq!(state.status.get("n1"), Some(&RemoteStatus::Succeeded));
}

#[test]
fn duplicate_remote_events_are_idempotent() {
    let source = FakeDistributedEventSource::new(vec![
        ev("n1", 1, RemoteStatus::Running),
        ev("n1", 1, RemoteStatus::Running),
        ev("n1", 2, RemoteStatus::Succeeded),
        ev("n1", 2, RemoteStatus::Succeeded),
    ]);
    let mut state = ReconciledState::default();
    for event in source.poll() {
        state.apply_event(event);
    }
    assert_eq!(state.highest_seq.get("n1"), Some(&2));
    assert_eq!(state.status.get("n1"), Some(&RemoteStatus::Succeeded));
}

#[test]
fn missing_completion_event_keeps_node_non_terminal() {
    let source = FakeDistributedEventSource::new(vec![ev("n1", 1, RemoteStatus::Running)]);
    let mut state = ReconciledState::default();
    for event in source.poll() {
        state.apply_event(event);
    }
    assert_eq!(state.status.get("n1"), Some(&RemoteStatus::Running));
    assert!(!state.completed.contains("n1"));
}

#[test]
fn inconsistent_snapshot_after_terminal_is_ignored() {
    let source = FakeDistributedEventSource::new(vec![
        ev("n1", 1, RemoteStatus::Running),
        ev("n1", 2, RemoteStatus::Succeeded),
        ev("n1", 3, RemoteStatus::Running),
    ]);
    let mut state = ReconciledState::default();
    for event in source.poll() {
        state.apply_event(event);
    }
    assert_eq!(state.status.get("n1"), Some(&RemoteStatus::Succeeded));
}

#[test]
fn controller_restart_reconciles_partially_completed_remote_state() {
    let pre_restart = vec![ev("n1", 1, RemoteStatus::Running), ev("n2", 1, RemoteStatus::Failed)];
    let post_restart =
        vec![ev("n1", 2, RemoteStatus::Succeeded), ev("n2", 2, RemoteStatus::Running)];

    let mut state = ReconciledState::default();
    for event in pre_restart {
        state.apply_event(event);
    }
    assert_eq!(state.status.get("n1"), Some(&RemoteStatus::Running));
    assert_eq!(state.status.get("n2"), Some(&RemoteStatus::Failed));

    for event in post_restart {
        state.apply_event(event);
    }

    assert_eq!(state.status.get("n1"), Some(&RemoteStatus::Succeeded));
    assert_eq!(state.status.get("n2"), Some(&RemoteStatus::Failed));
}
