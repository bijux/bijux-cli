use bijux_dag_artifacts::{NodeCounts, RunSummary};

pub fn summarize_counts(counts: &NodeCounts) -> RunSummary {
    RunSummary {
        total_nodes: counts.success
            + counts.failed
            + counts.skipped
            + counts.cached
            + counts.cancelled,
        success: counts.success,
        failed: counts.failed,
        skipped: counts.skipped,
        cached: counts.cached,
        cancelled: counts.cancelled,
        promoted_outputs: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::summarize_counts;
    use bijux_dag_artifacts::NodeCounts;

    #[test]
    fn summary_total_matches_component_counts() {
        let counts = NodeCounts { success: 3, failed: 2, skipped: 1, cached: 4, cancelled: 2 };
        let summary = summarize_counts(&counts);
        assert_eq!(summary.success, 3);
        assert_eq!(summary.failed, 2);
        assert_eq!(summary.skipped, 1);
        assert_eq!(summary.cached, 4);
        assert_eq!(summary.cancelled, 2);
        assert_eq!(summary.total_nodes, 12);
    }
}
