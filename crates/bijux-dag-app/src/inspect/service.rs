use crate::routes::selector_grammar::SelectorExpression;
use crate::run_views::{
    doctor_run, explain_failure, explain_run_id, inspect_summary, resolve_run_dir, run_timeline,
    run_tree, runs_history, runs_history_query_with_selectors,
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

pub(crate) fn run_history_for_root(root: &Path) -> Result<Value, ExitCode> {
    runs_history(root).map_err(|_| ExitCode::from(3))
}

pub(crate) fn run_history_query_for_root(
    root: &Path,
    status_filter: Option<&str>,
    source_filter: Option<&str>,
    pagination: Option<(usize, usize)>,
    selectors: Option<&[SelectorExpression]>,
) -> Result<Value, ExitCode> {
    runs_history_query_with_selectors(root, status_filter, source_filter, pagination, selectors)
        .map_err(|_| ExitCode::from(3))
}

pub(crate) fn run_id_explain_for_root(root: &Path, run_id: &str) -> Result<Value, ExitCode> {
    explain_run_id(root, run_id).map_err(|_| ExitCode::from(3))
}
