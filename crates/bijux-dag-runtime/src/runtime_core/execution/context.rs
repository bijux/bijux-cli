//! Canonical execution contexts for run and node execution.

pub use crate::adapter::NodeCtx;
pub use crate::RunContext;

pub type ExecutionContext = RunContext;
pub type NodeExecutionContext<'a> = NodeCtx<'a>;
