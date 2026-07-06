use std::path::{Path, PathBuf};

pub(crate) fn node_trace_path(run_dir: &Path, node_id: &str) -> PathBuf {
    run_dir.join("nodes").join(node_id).join("trace.json")
}

pub(crate) fn node_outputs_index_path(run_dir: &Path, node_id: &str) -> PathBuf {
    run_dir.join("nodes").join(node_id).join("outputs").join("index.json")
}

pub(crate) fn node_inputs_index_path(run_dir: &Path, node_id: &str) -> PathBuf {
    run_dir.join("nodes").join(node_id).join("inputs").join("index.json")
}

pub(crate) fn node_resolved_params_path(run_dir: &Path, node_id: &str) -> PathBuf {
    run_dir.join("nodes").join(node_id).join("resolved_params.json")
}

pub(crate) fn node_attempts_path(run_dir: &Path, node_id: &str) -> PathBuf {
    run_dir.join("nodes").join(node_id).join("attempts.json")
}

pub(crate) fn node_stdout_path(run_dir: &Path, node_id: &str) -> PathBuf {
    run_dir.join("nodes").join(node_id).join("stdout.log")
}

pub(crate) fn node_stderr_path(run_dir: &Path, node_id: &str) -> PathBuf {
    run_dir.join("nodes").join(node_id).join("stderr.log")
}

pub(crate) fn manifest_path(run_dir: &Path) -> PathBuf {
    run_dir.join("manifest.json")
}

#[cfg(test)]
mod tests {
    use super::{
        manifest_path, node_attempts_path, node_inputs_index_path, node_outputs_index_path,
        node_resolved_params_path, node_stderr_path, node_stdout_path, node_trace_path,
    };
    use std::path::Path;

    #[test]
    fn resolves_node_paths() {
        let run_dir = Path::new("/runs/r1");
        assert_eq!(node_trace_path(run_dir, "n1"), Path::new("/runs/r1/nodes/n1/trace.json"));
        assert_eq!(
            node_inputs_index_path(run_dir, "n1"),
            Path::new("/runs/r1/nodes/n1/inputs/index.json")
        );
        assert_eq!(
            node_outputs_index_path(run_dir, "n1"),
            Path::new("/runs/r1/nodes/n1/outputs/index.json")
        );
        assert_eq!(
            node_resolved_params_path(run_dir, "n1"),
            Path::new("/runs/r1/nodes/n1/resolved_params.json")
        );
        assert_eq!(node_attempts_path(run_dir, "n1"), Path::new("/runs/r1/nodes/n1/attempts.json"));
        assert_eq!(node_stdout_path(run_dir, "n1"), Path::new("/runs/r1/nodes/n1/stdout.log"));
        assert_eq!(node_stderr_path(run_dir, "n1"), Path::new("/runs/r1/nodes/n1/stderr.log"));
    }

    #[test]
    fn resolves_manifest_path() {
        assert_eq!(manifest_path(Path::new("/runs/r2")), Path::new("/runs/r2/manifest.json"));
    }
}
