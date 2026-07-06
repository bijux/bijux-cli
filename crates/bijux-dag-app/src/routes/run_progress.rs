use bijux_dag_runtime::{ExecutionCheckpoint, RunSnapshot};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompactRunProgressFailure {
    pub node_id: String,
    pub status: String,
    pub reason: Option<String>,
    pub failure_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompactRunProgressSnapshot {
    pub elapsed: Duration,
    pub total_nodes: usize,
    pub completed_nodes: usize,
    pub ready_count: usize,
    pub running_count: usize,
    pub blocked_count: usize,
    pub success_count: usize,
    pub failed_count: usize,
    pub skipped_count: usize,
    pub cached_count: usize,
    pub cancelled_count: usize,
    pub cache_hits: usize,
    pub active_nodes: Vec<String>,
    pub latest_failure: Option<CompactRunProgressFailure>,
    pub finished: bool,
}

#[derive(Debug, Default)]
pub(crate) struct ProgressEventCursor {
    offset: u64,
    pending_fragment: String,
}

#[derive(Debug)]
pub(crate) struct CompactRunProgressState {
    fallback_total_nodes: usize,
    selected_node_count: Option<usize>,
    ready_count: usize,
    blocked_count: usize,
    checkpoint_active_nodes: Option<BTreeSet<String>>,
    event_active_nodes: BTreeSet<String>,
    terminal_statuses: BTreeMap<String, String>,
    cache_hit_nodes: BTreeSet<String>,
    latest_failure: Option<CompactRunProgressFailure>,
    started: bool,
    finished: bool,
}

impl CompactRunProgressState {
    pub(crate) fn new(fallback_total_nodes: usize) -> Self {
        Self {
            fallback_total_nodes,
            selected_node_count: None,
            ready_count: 0,
            blocked_count: 0,
            checkpoint_active_nodes: None,
            event_active_nodes: BTreeSet::new(),
            terminal_statuses: BTreeMap::new(),
            cache_hit_nodes: BTreeSet::new(),
            latest_failure: None,
            started: false,
            finished: false,
        }
    }

    pub(crate) fn refresh_from_staging_dir(
        &mut self,
        cursor: &mut ProgressEventCursor,
        staging_path: &Path,
        started_at: Instant,
    ) -> Option<CompactRunProgressSnapshot> {
        self.refresh_selected_node_count(staging_path);
        self.refresh_checkpoint(staging_path);
        for event in cursor.read_new_events(&staging_path.join("run.log.jsonl")) {
            self.apply_event(&event);
        }
        if !self.started
            && self.checkpoint_active_nodes.is_none()
            && self.terminal_statuses.is_empty()
            && self.ready_count == 0
            && self.blocked_count == 0
        {
            return None;
        }

        let active_nodes = self
            .checkpoint_active_nodes
            .as_ref()
            .cloned()
            .unwrap_or_else(|| self.event_active_nodes.clone())
            .into_iter()
            .collect::<Vec<_>>();
        let (success_count, failed_count, skipped_count, cached_count, cancelled_count) =
            summarize_terminal_statuses(&self.terminal_statuses);
        let completed_nodes =
            success_count + failed_count + skipped_count + cached_count + cancelled_count;
        let total_nodes = self
            .selected_node_count
            .unwrap_or(self.fallback_total_nodes)
            .max(completed_nodes + active_nodes.len() + self.ready_count);

        Some(CompactRunProgressSnapshot {
            elapsed: started_at.elapsed(),
            total_nodes,
            completed_nodes,
            ready_count: self.ready_count,
            running_count: active_nodes.len(),
            blocked_count: self.blocked_count,
            success_count,
            failed_count,
            skipped_count,
            cached_count,
            cancelled_count,
            cache_hits: self.cache_hit_nodes.len().max(cached_count),
            active_nodes,
            latest_failure: self.latest_failure.clone(),
            finished: self.finished,
        })
    }

    fn refresh_selected_node_count(&mut self, staging_path: &Path) {
        if self.selected_node_count.is_some() {
            return;
        }
        let path = staging_path.join("run.snapshot.json");
        let Ok(raw) = std::fs::read_to_string(path) else {
            return;
        };
        let Ok(snapshot) = serde_json::from_str::<RunSnapshot>(&raw) else {
            return;
        };
        self.selected_node_count = Some(snapshot.selected_nodes.len());
    }

    fn refresh_checkpoint(&mut self, staging_path: &Path) {
        let path = staging_path.join("scheduler.checkpoint.json");
        let Ok(raw) = std::fs::read_to_string(path) else {
            return;
        };
        let Ok(checkpoint) = serde_json::from_str::<ExecutionCheckpoint>(&raw) else {
            return;
        };
        self.ready_count = checkpoint.ready_queue_depth;
        self.blocked_count = checkpoint.blocked_by_budget.len();
        self.checkpoint_active_nodes = Some(checkpoint.inflight.iter().cloned().collect());
        for (node_id, status) in checkpoint.completed_statuses {
            self.event_active_nodes.remove(&node_id);
            self.terminal_statuses.insert(node_id.clone(), status.clone());
            if status == "cached" {
                self.cache_hit_nodes.insert(node_id);
            }
        }
    }

    fn apply_event(&mut self, event: &Value) {
        let Some(name) = event.get("event").and_then(Value::as_str) else {
            return;
        };
        match name {
            "run_started" => {
                self.reset_for_new_attempt();
                self.started = true;
            }
            "run_finished" => {
                self.finished = true;
                self.ready_count = 0;
                self.blocked_count = 0;
                self.event_active_nodes.clear();
                self.checkpoint_active_nodes = Some(BTreeSet::new());
            }
            "cache_hit" => {
                if let Some(node_id) = event.get("node_id").and_then(Value::as_str) {
                    self.cache_hit_nodes.insert(node_id.to_string());
                }
            }
            "node_started" => {
                if let Some(node_id) = event.get("node_id").and_then(Value::as_str) {
                    self.event_active_nodes.insert(node_id.to_string());
                }
            }
            "node_skipped" => {
                self.record_terminal_event(event, "skipped");
            }
            "node_finished" => {
                let status = event.get("status").and_then(Value::as_str).unwrap_or("unknown");
                self.record_terminal_event(event, status);
            }
            _ => {}
        }
    }

    fn record_terminal_event(&mut self, event: &Value, status: &str) {
        let Some(node_id) = event.get("node_id").and_then(Value::as_str) else {
            return;
        };
        let node_id = node_id.to_string();
        self.event_active_nodes.remove(&node_id);
        self.terminal_statuses.insert(node_id.clone(), status.to_string());
        if status == "cached" {
            self.cache_hit_nodes.insert(node_id.clone());
        }
        if is_failure_status(status) {
            self.latest_failure = Some(CompactRunProgressFailure {
                node_id,
                status: status.to_string(),
                reason: event.get("reason").and_then(Value::as_str).map(ToOwned::to_owned),
                failure_code: event
                    .get("failure_code")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
            });
        }
    }

    fn reset_for_new_attempt(&mut self) {
        self.ready_count = 0;
        self.blocked_count = 0;
        self.checkpoint_active_nodes = None;
        self.event_active_nodes.clear();
        self.terminal_statuses.clear();
        self.cache_hit_nodes.clear();
        self.latest_failure = None;
        self.finished = false;
    }
}

pub(crate) fn format_compact_run_progress(snapshot: &CompactRunProgressSnapshot) -> String {
    format!(
        "progress elapsed={} done={}/{} ready={} running={} success={} failed={} skipped={} cached={} cancelled={} blocked={} cache_hits={} active=[{}] latest_failure={}",
        format_elapsed(snapshot.elapsed),
        snapshot.completed_nodes,
        snapshot.total_nodes,
        snapshot.ready_count,
        snapshot.running_count,
        snapshot.success_count,
        snapshot.failed_count,
        snapshot.skipped_count,
        snapshot.cached_count,
        snapshot.cancelled_count,
        snapshot.blocked_count,
        snapshot.cache_hits,
        format_active_nodes(&snapshot.active_nodes),
        format_failure(snapshot.latest_failure.as_ref()),
    )
}

fn summarize_terminal_statuses(
    terminal_statuses: &BTreeMap<String, String>,
) -> (usize, usize, usize, usize, usize) {
    let mut success_count = 0;
    let mut failed_count = 0;
    let mut skipped_count = 0;
    let mut cached_count = 0;
    let mut cancelled_count = 0;
    for status in terminal_statuses.values() {
        match status.as_str() {
            "success" => success_count += 1,
            "failed" | "timed_out" => failed_count += 1,
            "skipped" => skipped_count += 1,
            "cached" => cached_count += 1,
            "cancelled" => cancelled_count += 1,
            _ => {}
        }
    }
    (success_count, failed_count, skipped_count, cached_count, cancelled_count)
}

fn is_failure_status(status: &str) -> bool {
    matches!(status, "failed" | "timed_out" | "cancelled")
}

fn format_elapsed(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    if hours > 0 {
        return format!("{hours:02}:{minutes:02}:{seconds:02}");
    }
    format!("{minutes:02}:{seconds:02}")
}

fn format_active_nodes(active_nodes: &[String]) -> String {
    if active_nodes.is_empty() {
        return "-".to_string();
    }
    let visible = active_nodes.iter().take(3).cloned().collect::<Vec<_>>();
    if active_nodes.len() <= visible.len() {
        return visible.join(", ");
    }
    format!("{}, +{} more", visible.join(", "), active_nodes.len() - visible.len())
}

fn format_failure(failure: Option<&CompactRunProgressFailure>) -> String {
    let Some(failure) = failure else {
        return "-".to_string();
    };
    let mut rendered = format!("{}:{}", failure.node_id, failure.status);
    if let Some(reason) = failure.reason.as_deref() {
        rendered.push('/');
        rendered.push_str(reason);
    }
    if let Some(code) = failure.failure_code.as_deref() {
        rendered.push('[');
        rendered.push_str(code);
        rendered.push(']');
    }
    rendered
}

impl ProgressEventCursor {
    fn read_new_events(&mut self, run_log_path: &Path) -> Vec<Value> {
        let Ok(mut file) = File::open(run_log_path) else {
            return Vec::new();
        };
        let Ok(metadata) = file.metadata() else {
            return Vec::new();
        };
        if metadata.len() < self.offset {
            self.offset = 0;
            self.pending_fragment.clear();
        }
        if file.seek(SeekFrom::Start(self.offset)).is_err() {
            return Vec::new();
        }
        let mut chunk = String::new();
        if file.read_to_string(&mut chunk).is_err() {
            return Vec::new();
        }
        self.offset += chunk.len() as u64;
        if chunk.is_empty() {
            return Vec::new();
        }
        let mut raw = std::mem::take(&mut self.pending_fragment);
        raw.push_str(&chunk);
        let mut events = Vec::new();
        for line in raw.split_inclusive('\n') {
            if !line.ends_with('\n') {
                self.pending_fragment.push_str(line);
                continue;
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
                events.push(value);
            }
        }
        if !raw.ends_with('\n') && self.pending_fragment.is_empty() {
            self.pending_fragment = raw
                .rsplit_once('\n')
                .map(|(_, tail)| tail.to_string())
                .unwrap_or(raw);
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::{
        format_compact_run_progress, CompactRunProgressFailure, CompactRunProgressSnapshot,
        CompactRunProgressState, ProgressEventCursor,
    };
    use serde_json::json;
    use std::fs;
    use std::time::{Duration, Instant};

    #[test]
    fn compact_progress_refresh_reads_checkpoint_log_and_selected_nodes() {
        let dir = tempfile::tempdir().expect("tmp");
        fs::write(
            dir.path().join("run.snapshot.json"),
            serde_json::to_vec_pretty(&json!({
                "run_id":"run-live",
                "graph_snapshot_path":"graph.snapshot.json",
                "planner_config":"default",
                "scheduler_config":"local",
                "policy_config":"runtime-policy-v0.1",
                "provenance":"provenance.json",
                "submission_source":"cli",
                "trigger_source":"manual",
                "operator":"bijux-dag",
                "labels":[],
                "parent_run_id":null,
                "requested_selectors":[],
                "selected_nodes":["extract","train","publish"],
                "dependency_closure_enabled":false,
                "replay_source_run_id":null
            }))
            .expect("snapshot"),
        )
        .expect("write snapshot");
        fs::write(
            dir.path().join("scheduler.checkpoint.json"),
            serde_json::to_vec_pretty(&json!({
                "loop_index":4,
                "ready_queue_depth":1,
                "ready_queue":["publish"],
                "inflight":["train"],
                "scheduled":["train"],
                "blocked_by_budget":["publish"],
                "blocked_reasons":{"publish":"cpu_budget"},
                "completed_statuses":{"extract":"cached"},
                "failure_propagation_mode":"continue_independent",
                "dependency_closure_enabled":false,
                "generated_unix_ms":44
            }))
            .expect("checkpoint"),
        )
        .expect("write checkpoint");
        fs::write(
            dir.path().join("run.log.jsonl"),
            concat!(
                "{\"event\":\"run_started\",\"ts\":1}\n",
                "{\"event\":\"cache_hit\",\"ts\":2,\"node_id\":\"extract\"}\n",
                "{\"event\":\"node_started\",\"ts\":3,\"node_id\":\"train\"}\n",
                "{\"event\":\"node_finished\",\"ts\":4,\"node_id\":\"evaluate\",\"status\":\"failed\",\"reason\":\"timeout\",\"failure_code\":\"TIMEOUT\"}\n"
            ),
        )
        .expect("write run log");

        let mut state = CompactRunProgressState::new(8);
        let mut cursor = ProgressEventCursor::default();
        let started_at = Instant::now() - Duration::from_secs(65);
        let snapshot = state
            .refresh_from_staging_dir(&mut cursor, dir.path(), started_at)
            .expect("snapshot");

        assert_eq!(snapshot.total_nodes, 3);
        assert_eq!(snapshot.completed_nodes, 2);
        assert_eq!(snapshot.ready_count, 1);
        assert_eq!(snapshot.running_count, 1);
        assert_eq!(snapshot.blocked_count, 1);
        assert_eq!(snapshot.cached_count, 1);
        assert_eq!(snapshot.failed_count, 1);
        assert_eq!(snapshot.cache_hits, 1);
        assert_eq!(snapshot.active_nodes, vec!["train".to_string()]);
        assert_eq!(
            snapshot.latest_failure,
            Some(CompactRunProgressFailure {
                node_id: "evaluate".to_string(),
                status: "failed".to_string(),
                reason: Some("timeout".to_string()),
                failure_code: Some("TIMEOUT".to_string()),
            })
        );
    }

    #[test]
    fn compact_progress_resets_state_when_new_attempt_starts() {
        let dir = tempfile::tempdir().expect("tmp");
        fs::write(
            dir.path().join("run.log.jsonl"),
            concat!(
                "{\"event\":\"run_started\",\"ts\":1}\n",
                "{\"event\":\"node_finished\",\"ts\":2,\"node_id\":\"old\",\"status\":\"failed\",\"reason\":\"boom\"}\n",
                "{\"event\":\"run_started\",\"ts\":3}\n",
                "{\"event\":\"node_started\",\"ts\":4,\"node_id\":\"fresh\"}\n"
            ),
        )
        .expect("write run log");

        let mut state = CompactRunProgressState::new(2);
        let mut cursor = ProgressEventCursor::default();
        let snapshot = state
            .refresh_from_staging_dir(&mut cursor, dir.path(), Instant::now())
            .expect("snapshot");

        assert_eq!(snapshot.failed_count, 0);
        assert_eq!(snapshot.active_nodes, vec!["fresh".to_string()]);
        assert!(snapshot.latest_failure.is_none());
    }

    #[test]
    fn compact_progress_formatter_renders_compact_status_line() {
        let rendered = format_compact_run_progress(&CompactRunProgressSnapshot {
            elapsed: Duration::from_secs(3723),
            total_nodes: 9,
            completed_nodes: 5,
            ready_count: 2,
            running_count: 2,
            blocked_count: 1,
            success_count: 3,
            failed_count: 1,
            skipped_count: 0,
            cached_count: 1,
            cancelled_count: 0,
            cache_hits: 1,
            active_nodes: vec![
                "extract".to_string(),
                "train".to_string(),
                "publish".to_string(),
                "notify".to_string(),
            ],
            latest_failure: Some(CompactRunProgressFailure {
                node_id: "score".to_string(),
                status: "failed".to_string(),
                reason: Some("timeout".to_string()),
                failure_code: Some("TIMEOUT".to_string()),
            }),
            finished: false,
        });

        assert!(rendered.contains("elapsed=01:02:03"));
        assert!(rendered.contains("done=5/9"));
        assert!(rendered.contains("active=[extract, train, publish, +1 more]"));
        assert!(rendered.contains("latest_failure=score:failed/timeout[TIMEOUT]"));
    }
}
