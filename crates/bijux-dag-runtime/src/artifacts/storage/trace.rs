//! Runtime trace boundary surface.

use crate::NodeStatus;

pub fn trace_status_label(status: &NodeStatus) -> &'static str {
    match status {
        NodeStatus::Success => "success",
        NodeStatus::Failed => "failed",
        NodeStatus::Skipped => "skipped",
        NodeStatus::Cached => "cached",
        NodeStatus::Cancelled => "cancelled",
    }
}
