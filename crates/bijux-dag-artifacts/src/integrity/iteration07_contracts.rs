use serde::{Deserialize, Serialize};

/// Canonical run directory layout contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunDirectoryLayoutContractV1 {
    pub manifest_path: String,
    pub plan_path: String,
    pub run_log_path: String,
    pub traces_root: String,
    pub nodes_root: String,
    pub outputs_index_path: String,
    pub cache_root: String,
    pub replay_root: String,
    pub summaries_root: String,
}

/// Build predictable canonical run directory layout.
pub fn build_run_directory_layout_contract(run_id: &str) -> Result<RunDirectoryLayoutContractV1, String> {
    let normalized = run_id.trim();
    if normalized.is_empty() {
        return Err("run_id must not be empty".to_string());
    }
    if normalized.contains('/') || normalized.contains('\\') || normalized.contains("..") {
        return Err("run_id must be a normalized identifier".to_string());
    }
    let root = format!("run-{normalized}");
    Ok(RunDirectoryLayoutContractV1 {
        manifest_path: format!("{root}/manifest.json"),
        plan_path: format!("{root}/plan.json"),
        run_log_path: format!("{root}/run.log.jsonl"),
        traces_root: format!("{root}/traces"),
        nodes_root: format!("{root}/nodes"),
        outputs_index_path: format!("{root}/outputs/index.json"),
        cache_root: format!("{root}/cache"),
        replay_root: format!("{root}/replay"),
        summaries_root: format!("{root}/summaries"),
    })
}

#[cfg(test)]
mod tests {
    use super::build_run_directory_layout_contract;

    #[test]
    fn g061_run_directory_layout_contract_is_predictable() {
        let layout = build_run_directory_layout_contract("20260501-abc")
            .expect("layout should build");
        assert_eq!(layout.manifest_path, "run-20260501-abc/manifest.json");
        assert_eq!(layout.outputs_index_path, "run-20260501-abc/outputs/index.json");
        assert_eq!(layout.replay_root, "run-20260501-abc/replay");
    }
}
