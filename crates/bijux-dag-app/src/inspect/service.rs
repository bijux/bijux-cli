use crate::run_views::{
    doctor_run, explain_failure, inspect_summary, resolve_run_dir, run_timeline, run_tree,
};
use serde_json::Value;
use std::path::Path;
use std::process::ExitCode;

pub(crate) fn run_summary_for_id(root: &Path, run_id: &str) -> Result<Value, ExitCode> {
    let run_dir = resolve_run_dir(root, run_id);
    inspect_summary(&run_dir).map_err(|_| ExitCode::from(3))
}

pub(crate) fn run_tree_for_id(root: &Path, run_id: &str) -> Result<Value, ExitCode> {
    let run_dir = resolve_run_dir(root, run_id);
    run_tree(&run_dir).map_err(|_| ExitCode::from(3))
}

pub(crate) fn run_timeline_for_id(root: &Path, run_id: &str) -> Result<Value, ExitCode> {
    let run_dir = resolve_run_dir(root, run_id);
    run_timeline(&run_dir).map_err(|_| ExitCode::from(3))
}

pub(crate) fn doctor_for_run_id(root: &Path, run_id: &str) -> Value {
    let run_dir = resolve_run_dir(root, run_id);
    doctor_run(&run_dir)
}

pub(crate) fn explain_failure_for_run_id(root: &Path, run_id: &str) -> Result<Value, ExitCode> {
    let run_dir = resolve_run_dir(root, run_id);
    explain_failure(&run_dir).map_err(|_| ExitCode::from(3))
}
