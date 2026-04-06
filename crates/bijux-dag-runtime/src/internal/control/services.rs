//! Runtime crate service interfaces.

use bijux_dag_core::Graph;
use std::path::{Path, PathBuf};

pub trait RuntimeExecutionService {
    fn execute_graph(
        &self,
        graph: &Graph,
        out_dir: &Path,
        config: &crate::RuntimeConfig,
    ) -> Result<PathBuf, crate::RuntimeError>;
}

pub trait RuntimeArtifactService {
    fn persist_run_artifacts(
        &self,
        run_dir: &Path,
        manifest: &bijux_dag_artifacts::Manifest,
    ) -> Result<(), crate::RuntimeError>;
}
