use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchJobMetadata {
    pub scheduler_id: String,
    pub submission_time_unix_ms: u128,
    pub run_id: String,
    pub node_id: String,
    pub attempt_id: String,
    pub resource_request: String,
    pub status_mapping: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchLifecycleEvent {
    pub scheduler_id: String,
    pub status: String,
    pub unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchAttemptState {
    pub metadata: BatchJobMetadata,
    pub events: Vec<BatchLifecycleEvent>,
    pub cancelled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchHeartbeat {
    pub scheduler_id: String,
    pub unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchModeReport {
    pub implemented: Vec<String>,
    pub simulated: Vec<String>,
    pub aspirational: Vec<String>,
}

pub fn validate_batch_metadata(meta: &BatchJobMetadata) -> Result<(), String> {
    for value in [
        &meta.scheduler_id,
        &meta.run_id,
        &meta.node_id,
        &meta.attempt_id,
        &meta.resource_request,
        &meta.status_mapping,
    ] {
        if value.trim().is_empty() {
            return Err("batch job metadata fields must be non-empty".to_string());
        }
    }
    Ok(())
}

pub fn retry_attempt(previous: &BatchJobMetadata, new_attempt_id: &str) -> BatchJobMetadata {
    BatchJobMetadata {
        scheduler_id: previous.scheduler_id.clone(),
        submission_time_unix_ms: previous.submission_time_unix_ms.saturating_add(1),
        run_id: previous.run_id.clone(),
        node_id: previous.node_id.clone(),
        attempt_id: new_attempt_id.to_string(),
        resource_request: previous.resource_request.clone(),
        status_mapping: previous.status_mapping.clone(),
    }
}

pub fn cancel_batch_attempt(state: &mut BatchAttemptState) {
    state.cancelled = true;
    state.events.push(BatchLifecycleEvent {
        scheduler_id: state.metadata.scheduler_id.clone(),
        status: "cancel-requested".to_string(),
        unix_ms: state.events.last().map(|e| e.unix_ms.saturating_add(1)).unwrap_or(1),
    });
}

pub fn heartbeat_stale(last: &BatchHeartbeat, now_unix_ms: u128, max_age_ms: u128) -> bool {
    now_unix_ms.saturating_sub(last.unix_ms) > max_age_ms
}

pub fn duplicate_status_delivery_detected(events: &[BatchLifecycleEvent]) -> bool {
    let mut seen = BTreeMap::new();
    for event in events {
        let key = (event.scheduler_id.clone(), event.status.clone(), event.unix_ms);
        let count = seen.entry(key).or_insert(0usize);
        *count += 1;
        if *count > 1 {
            return true;
        }
    }
    false
}

pub fn restart_recovery_supported() -> bool {
    false
}

pub fn execution_mode_report() -> BatchModeReport {
    BatchModeReport {
        implemented: vec!["local".to_string(), "subprocess".to_string(), "container".to_string()],
        simulated: vec![
            "remote-contract".to_string(),
            "batch-contract".to_string(),
            "fake-batch-backend".to_string(),
            "slurm-backend".to_string(),
        ],
        aspirational: vec!["kubernetes-backend".to_string(), "pbs-backend".to_string()],
    }
}
