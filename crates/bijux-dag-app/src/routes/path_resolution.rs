use std::path::{Path, PathBuf};

pub(crate) fn node_trace_path(run_dir: &Path, node_id: &str) -> PathBuf {
    run_dir.join("nodes").join(node_id).join("trace.json")
}

pub(crate) fn node_outputs_index_path(run_dir: &Path, node_id: &str) -> PathBuf {
    run_dir.join("nodes").join(node_id).join("outputs").join("index.json")
}

pub(crate) fn manifest_path(run_dir: &Path) -> PathBuf {
    run_dir.join("manifest.json")
}

#[cfg(test)]
mod tests {
    use super::{manifest_path, node_outputs_index_path, node_trace_path};
    use std::path::Path;

    #[test]
    fn resolves_node_paths() {
        let run_dir = Path::new("/runs/r1");
        assert_eq!(node_trace_path(run_dir, "n1"), Path::new("/runs/r1/nodes/n1/trace.json"));
        assert_eq!(
            node_outputs_index_path(run_dir, "n1"),
            Path::new("/runs/r1/nodes/n1/outputs/index.json")
        );
    }

    #[test]
    fn resolves_manifest_path() {
        assert_eq!(manifest_path(Path::new("/runs/r2")), Path::new("/runs/r2/manifest.json"));
    }
}
