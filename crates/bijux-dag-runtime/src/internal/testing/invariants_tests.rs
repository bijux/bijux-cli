use crate::invariants::{
    run_summary_invariant_ok, terminal_run_has_terminal_node, trace_time_order_ok, RunNodeCounts,
    INVARIANT_REGISTRY,
};
use crate::NodeStatus;

#[test]
fn run_summary_invariant_matches_trace_totals() {
    let manifest = RunNodeCounts { success: 2, failed: 1, skipped: 1, cached: 0, cancelled: 0 };
    let traces =
        [NodeStatus::Success, NodeStatus::Success, NodeStatus::Failed, NodeStatus::Skipped];
    assert!(run_summary_invariant_ok(manifest, &traces));
}

#[test]
fn run_summary_invariant_detects_mismatch() {
    let manifest = RunNodeCounts { success: 1, failed: 0, skipped: 0, cached: 0, cancelled: 0 };
    let traces = [NodeStatus::Success, NodeStatus::Failed];
    assert!(!run_summary_invariant_ok(manifest, &traces));
}

#[test]
fn terminal_run_invariant_detects_absent_terminal_nodes() {
    assert!(!terminal_run_has_terminal_node(&[]));
    assert!(terminal_run_has_terminal_node(&[NodeStatus::Success]));
}

#[test]
fn trace_time_invariant_requires_monotonic_timestamps() {
    assert!(trace_time_order_ok(10, 10));
    assert!(trace_time_order_ok(10, 11));
    assert!(!trace_time_order_ok(11, 10));
}

#[test]
fn invariant_registry_ids_are_stable_and_unique() {
    let mut ids = std::collections::BTreeSet::new();
    for inv in INVARIANT_REGISTRY {
        assert!(inv.id.starts_with("INV-"));
        assert!(ids.insert(inv.id), "duplicate invariant id {}", inv.id);
        assert!(!inv.enforcement.trim().is_empty());
    }
}
