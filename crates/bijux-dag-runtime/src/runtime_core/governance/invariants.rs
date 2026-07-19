//! Runtime invariants for manifest and trace consistency.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvariantDefinition {
    pub id: &'static str,
    pub title: &'static str,
    pub enforcement: &'static str,
}

pub const INVARIANT_REGISTRY: &[InvariantDefinition] = &[
    InvariantDefinition {
        id: "INV-GRAPH-SHAPE-001",
        title: "canonical graph shape remains acyclic, unique, and structurally deterministic",
        enforcement: "dag-core graph identity property contracts + runtime formal invariant property contracts",
    },
    InvariantDefinition {
        id: "INV-PLAN-SHAPE-001",
        title: "execution plan shape remains deterministic for equivalent graph structure",
        enforcement: "runtime formal invariant property contracts + planner lowering contracts",
    },
    InvariantDefinition {
        id: "INV-RUN-COUNTS-001",
        title: "manifest node counts match observed terminal node statuses",
        enforcement: "runtime::invariants::run_summary_invariant_ok + app verify_run",
    },
    InvariantDefinition {
        id: "INV-RUN-TERMINAL-001",
        title: "terminal run requires at least one terminal node status",
        enforcement: "app verify_run",
    },
    InvariantDefinition {
        id: "INV-TRACE-TIME-001",
        title: "trace finished_unix_ms is greater than or equal to started_unix_ms",
        enforcement: "app verify_run deep mode",
    },
    InvariantDefinition {
        id: "INV-SCHED-READY-001",
        title: "a node must not be admitted to ready queue twice",
        enforcement: "scheduler contract tests",
    },
    InvariantDefinition {
        id: "INV-PLAN-DEPENDENCY-001",
        title: "planned dependency counts remain stable for equivalent lowered graphs",
        enforcement: "planner and scheduler contract tests",
    },
    InvariantDefinition {
        id: "INV-CACHE-PROOF-001",
        title: "cache hit requires proof metadata compatibility",
        enforcement: "cache evolution contract tests",
    },
    InvariantDefinition {
        id: "INV-ARTIFACT-REF-001",
        title: "artifact references remain attributable, present, and integrity-checkable",
        enforcement: "app smoke artifact inspect contracts + import or export artifact corruption contracts",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RunNodeCounts {
    pub success: u32,
    pub failed: u32,
    pub skipped: u32,
    pub cached: u32,
    pub cancelled: u32,
}

pub fn run_summary_invariant_ok(manifest: RunNodeCounts, traces: &[crate::NodeStatus]) -> bool {
    let mut observed = RunNodeCounts::default();
    for status in traces {
        match status {
            crate::NodeStatus::Success => observed.success += 1,
            crate::NodeStatus::Failed => observed.failed += 1,
            crate::NodeStatus::Skipped => observed.skipped += 1,
            crate::NodeStatus::Cached => observed.cached += 1,
            crate::NodeStatus::Cancelled => observed.cancelled += 1,
        }
    }
    manifest == observed
}

pub fn terminal_run_has_terminal_node(statuses: &[crate::NodeStatus]) -> bool {
    statuses.iter().any(|s| {
        matches!(
            s,
            crate::NodeStatus::Success
                | crate::NodeStatus::Failed
                | crate::NodeStatus::Cached
                | crate::NodeStatus::Skipped
                | crate::NodeStatus::Cancelled
        )
    })
}

pub fn trace_time_order_ok(started_unix_ms: u64, finished_unix_ms: u64) -> bool {
    finished_unix_ms >= started_unix_ms
}
