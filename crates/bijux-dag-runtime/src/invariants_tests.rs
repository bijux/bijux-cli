#![cfg(test)]

use crate::invariants::{run_summary_invariant_ok, RunNodeCounts};
use crate::NodeStatus;

#[test]
fn run_summary_invariant_matches_trace_totals() {
    let manifest = RunNodeCounts {
        success: 2,
        failed: 1,
        skipped: 1,
        cached: 0,
    };
    let traces = [
        NodeStatus::Success,
        NodeStatus::Success,
        NodeStatus::Failed,
        NodeStatus::Skipped,
    ];
    assert!(run_summary_invariant_ok(manifest, &traces));
}

#[test]
fn run_summary_invariant_detects_mismatch() {
    let manifest = RunNodeCounts {
        success: 1,
        failed: 0,
        skipped: 0,
        cached: 0,
    };
    let traces = [NodeStatus::Success, NodeStatus::Failed];
    assert!(!run_summary_invariant_ok(manifest, &traces));
}
