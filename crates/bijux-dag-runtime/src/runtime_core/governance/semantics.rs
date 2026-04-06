use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadyNode {
    pub node_id: String,
    pub priority: u8,
    pub attempt: u32,
    pub ready_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryPolicySemantics {
    pub max_attempts: u32,
    pub initial_backoff_ms: u64,
    pub exponential: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeFailureClass {
    AdapterFailure,
    Timeout,
    Cancelled,
    DependencyFailure,
    PolicyViolation,
    CacheInvalid,
    ArtifactCorruption,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheValidationInput {
    pub fingerprint_matches: bool,
    pub schema_matches: bool,
    pub proof_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestVerificationInput {
    pub has_run_header: bool,
    pub has_trace_index: bool,
    pub has_outputs_index: bool,
    pub totals_consistent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryInput {
    pub has_checkpoint: bool,
    pub terminal_state_seen: bool,
    pub partial_artifacts_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeAuditEvent {
    pub event_id: String,
    pub run_id: String,
    pub node_id: Option<String>,
    pub category: String,
    pub details: BTreeMap<String, String>,
}

pub fn deterministic_schedule_order(
    mut ready: Vec<ReadyNode>,
    starvation_ticks: &BTreeMap<String, u32>,
) -> Vec<ReadyNode> {
    ready.sort_by(|a, b| {
        let a_starve = starvation_ticks.get(&a.node_id).copied().unwrap_or(0);
        let b_starve = starvation_ticks.get(&b.node_id).copied().unwrap_or(0);
        b_starve
            .cmp(&a_starve)
            .then_with(|| b.priority.cmp(&a.priority))
            .then_with(|| a.attempt.cmp(&b.attempt))
            .then_with(|| a.ready_unix_ms.cmp(&b.ready_unix_ms))
            .then_with(|| a.node_id.cmp(&b.node_id))
    });
    ready
}

pub fn fairness_is_satisfied(
    dispatch_order: &[ReadyNode],
    starvation_threshold: u32,
    starvation_ticks: &BTreeMap<String, u32>,
) -> bool {
    if dispatch_order.is_empty() {
        return true;
    }
    let dispatched: BTreeSet<&str> = dispatch_order.iter().map(|n| n.node_id.as_str()).collect();
    starvation_ticks
        .iter()
        .filter(|(node, ticks)| {
            **ticks >= starvation_threshold && dispatched.contains(node.as_str())
        })
        .count()
        >= starvation_ticks
            .iter()
            .filter(|(_, ticks)| **ticks >= starvation_threshold)
            .count()
            .min(1)
}

pub fn retry_allowed(attempt: u32, policy: &RetryPolicySemantics) -> bool {
    attempt < policy.max_attempts
}

pub fn timeout_triggered(start_unix_ms: u128, now_unix_ms: u128, timeout_ms: Option<u64>) -> bool {
    timeout_ms.is_some_and(|limit| now_unix_ms.saturating_sub(start_unix_ms) > limit as u128)
}

pub fn cancellation_is_terminal(cancel_requested: bool, node_terminal: bool) -> bool {
    cancel_requested && node_terminal
}

pub fn dependency_resolution_is_complete(
    required: &[String],
    succeeded: &BTreeSet<String>,
) -> bool {
    required.iter().all(|dep| succeeded.contains(dep))
}

pub fn artifact_commit_guaranteed(
    temp_written: bool,
    manifest_updated: bool,
    fsync_complete: bool,
) -> bool {
    temp_written && manifest_updated && fsync_complete
}

pub fn cache_entry_valid(input: &CacheValidationInput) -> bool {
    input.fingerprint_matches && input.schema_matches && input.proof_present
}

pub fn cache_entry_invalidated(
    policy_changed: bool,
    adapter_version_changed: bool,
    output_schema_changed: bool,
) -> bool {
    policy_changed || adapter_version_changed || output_schema_changed
}

pub fn replay_equivalent(expected_fingerprint: &str, observed_fingerprint: &str) -> bool {
    expected_fingerprint == observed_fingerprint
}

pub fn run_manifest_valid(input: &ManifestVerificationInput) -> bool {
    input.has_run_header
        && input.has_trace_index
        && input.has_outputs_index
        && input.totals_consistent
}

pub fn recovery_action_required(input: &RecoveryInput) -> bool {
    input.partial_artifacts_present || (!input.terminal_state_seen && input.has_checkpoint)
}

pub fn artifact_lineage_complete(
    outputs: &[String],
    lineage_index: &BTreeMap<String, String>,
) -> bool {
    outputs
        .iter()
        .all(|output| lineage_index.contains_key(output))
}

pub fn classify_failure(
    timeout: bool,
    cancelled: bool,
    dependency_failed: bool,
    policy_violation: bool,
    cache_invalid: bool,
    artifact_corruption: bool,
) -> RuntimeFailureClass {
    if timeout {
        RuntimeFailureClass::Timeout
    } else if cancelled {
        RuntimeFailureClass::Cancelled
    } else if dependency_failed {
        RuntimeFailureClass::DependencyFailure
    } else if policy_violation {
        RuntimeFailureClass::PolicyViolation
    } else if cache_invalid {
        RuntimeFailureClass::CacheInvalid
    } else if artifact_corruption {
        RuntimeFailureClass::ArtifactCorruption
    } else {
        RuntimeFailureClass::AdapterFailure
    }
}

pub fn append_audit_event(events: &mut Vec<RuntimeAuditEvent>, event: RuntimeAuditEvent) {
    events.push(event);
}

pub fn trace_event_count_by_category(events: &[RuntimeAuditEvent]) -> BTreeMap<String, usize> {
    let mut by_category = BTreeMap::new();
    for event in events {
        *by_category.entry(event.category.clone()).or_insert(0) += 1;
    }
    by_category
}
