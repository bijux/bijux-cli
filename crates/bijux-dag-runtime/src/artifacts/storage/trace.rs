//! Runtime trace boundary surface.

pub(crate) use crate::transition_cause_for_status;
use crate::NodeStatus;

pub fn trace_status_label(status: &NodeStatus) -> &'static str {
    match status {
        NodeStatus::Success => "success",
        NodeStatus::Failed => "failed",
        NodeStatus::Skipped => "skipped",
        NodeStatus::Cached => "cached",
    }
}
