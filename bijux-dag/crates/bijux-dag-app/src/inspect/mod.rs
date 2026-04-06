use std::fs;
use std::path::{Path, PathBuf};

pub fn node_trace_path(run_dir: &Path, node_id: &str) -> PathBuf {
    run_dir.join("nodes").join(node_id).join("trace.json")
}

pub fn read_trace(run_dir: &Path, node_id: &str) -> Result<String, std::io::Error> {
    fs::read_to_string(node_trace_path(run_dir, node_id))
}
