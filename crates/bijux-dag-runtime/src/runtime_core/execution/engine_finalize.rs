use bijux_dag_artifacts::{NodeCounts, RunSummary};

pub fn summarize_counts(counts: &NodeCounts) -> RunSummary {
    RunSummary {
        total_nodes: counts.success + counts.failed + counts.skipped + counts.cached,
        success: counts.success,
        failed: counts.failed,
        skipped: counts.skipped,
        cached: counts.cached,
    }
}
