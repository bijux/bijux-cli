use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunSummaryCounters {
    pub success: u32,
    pub failed: u32,
    pub skipped: u32,
    pub cached: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceWriteRecord {
    pub node_id: String,
    pub trace_path: PathBuf,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeCoordinationSnapshot {
    pub summary: RunSummaryCounters,
    pub trace_writes: Vec<TraceWriteRecord>,
    pub cache_claimed_fingerprints: Vec<String>,
    pub latest_link_updates: Vec<PathBuf>,
}

#[derive(Clone, Default)]
pub struct RuntimeCoordinationState {
    summary: Arc<Mutex<RunSummaryCounters>>,
    trace_writes: Arc<Mutex<Vec<TraceWriteRecord>>>,
    cache_claims: Arc<Mutex<BTreeSet<String>>>,
    latest_link_updates: Arc<Mutex<Vec<PathBuf>>>,
    in_progress_runs: Arc<Mutex<BTreeSet<String>>>,
}

impl RuntimeCoordinationState {
    pub fn mark_success(&self) {
        if let Ok(mut summary) = self.summary.lock() {
            summary.success += 1;
        }
    }

    pub fn mark_failed(&self) {
        if let Ok(mut summary) = self.summary.lock() {
            summary.failed += 1;
        }
    }

    pub fn mark_skipped(&self) {
        if let Ok(mut summary) = self.summary.lock() {
            summary.skipped += 1;
        }
    }

    pub fn mark_cached(&self) {
        if let Ok(mut summary) = self.summary.lock() {
            summary.cached += 1;
        }
    }

    pub fn register_trace_write(&self, node_id: &str, trace_path: PathBuf) {
        if let Ok(mut writes) = self.trace_writes.lock() {
            writes.push(TraceWriteRecord {
                node_id: node_id.to_string(),
                trace_path,
            });
        }
    }

    pub fn claim_cache_fingerprint(&self, fingerprint: &str) -> bool {
        if let Ok(mut claims) = self.cache_claims.lock() {
            return claims.insert(fingerprint.to_string());
        }
        false
    }

    pub fn register_latest_link_update(&self, link_path: PathBuf) {
        if let Ok(mut links) = self.latest_link_updates.lock() {
            links.push(link_path);
        }
    }

    pub fn begin_run(&self, run_id: &str) -> bool {
        if let Ok(mut set) = self.in_progress_runs.lock() {
            return set.insert(run_id.to_string());
        }
        false
    }

    pub fn end_run(&self, run_id: &str) {
        if let Ok(mut set) = self.in_progress_runs.lock() {
            set.remove(run_id);
        }
    }

    pub fn reject_read_during_active_run(&self, run_id: &str) -> Result<(), String> {
        let set = self
            .in_progress_runs
            .lock()
            .map_err(|_| "coordination lock poisoned".to_string())?;
        if set.contains(run_id) {
            Err(format!(
                "run directory access rejected while run is in progress: {run_id}"
            ))
        } else {
            Ok(())
        }
    }

    pub fn snapshot(&self) -> RuntimeCoordinationSnapshot {
        let summary = self.summary.lock().ok().map(|v| v.clone()).unwrap_or_default();
        let trace_writes = self
            .trace_writes
            .lock()
            .ok()
            .map(|v| v.clone())
            .unwrap_or_default();
        let cache_claimed_fingerprints = self
            .cache_claims
            .lock()
            .ok()
            .map(|v| v.iter().cloned().collect())
            .unwrap_or_default();
        let latest_link_updates = self
            .latest_link_updates
            .lock()
            .ok()
            .map(|v| v.clone())
            .unwrap_or_default();
        RuntimeCoordinationSnapshot {
            summary,
            trace_writes,
            cache_claimed_fingerprints,
            latest_link_updates,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadSafetyAuditRecord {
    pub surface: String,
    pub owner: String,
    pub synchronization: String,
}

pub fn thread_safety_audit() -> Vec<ThreadSafetyAuditRecord> {
    vec![
        ThreadSafetyAuditRecord {
            surface: "scheduler_state".to_string(),
            owner: "SchedulerState".to_string(),
            synchronization: "single_owner_mutation".to_string(),
        },
        ThreadSafetyAuditRecord {
            surface: "run_summary".to_string(),
            owner: "RuntimeCoordinationState".to_string(),
            synchronization: "mutex".to_string(),
        },
        ThreadSafetyAuditRecord {
            surface: "trace_write_ledger".to_string(),
            owner: "RuntimeCoordinationState".to_string(),
            synchronization: "mutex".to_string(),
        },
        ThreadSafetyAuditRecord {
            surface: "cache_claim_map".to_string(),
            owner: "RuntimeCoordinationState".to_string(),
            synchronization: "mutex".to_string(),
        },
    ]
}

pub fn merge_timeout_and_exit_events(
    timed_out_nodes: &[String],
    exited_nodes: &[String],
) -> BTreeMap<String, String> {
    let mut merged = BTreeMap::new();
    for node in timed_out_nodes {
        merged.insert(node.clone(), "timed_out".to_string());
    }
    for node in exited_nodes {
        merged.entry(node.clone()).or_insert("exited".to_string());
    }
    merged
}
