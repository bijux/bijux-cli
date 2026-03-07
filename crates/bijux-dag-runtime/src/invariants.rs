//! Runtime invariants for manifest and trace consistency.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RunNodeCounts {
    pub success: u32,
    pub failed: u32,
    pub skipped: u32,
    pub cached: u32,
}

pub fn run_summary_invariant_ok(manifest: RunNodeCounts, traces: &[crate::NodeStatus]) -> bool {
    let mut observed = RunNodeCounts::default();
    for status in traces {
        match status {
            crate::NodeStatus::Success => observed.success += 1,
            crate::NodeStatus::Failed => observed.failed += 1,
            crate::NodeStatus::Skipped => observed.skipped += 1,
            crate::NodeStatus::Cached => observed.cached += 1,
        }
    }
    manifest == observed
}
