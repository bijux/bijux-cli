use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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

/// Content-based artifact identity for file or directory artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactContentIdentityV1 {
    pub artifact_path: String,
    pub artifact_kind: String,
    pub content_hash: String,
}

/// Build content-based identity for a file artifact.
pub fn content_identity_for_file(path: &str, bytes: &[u8]) -> Result<ArtifactContentIdentityV1, String> {
    if path.trim().is_empty() {
        return Err("artifact path must not be empty".to_string());
    }
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash = format!("{:x}", hasher.finalize());
    Ok(ArtifactContentIdentityV1 {
        artifact_path: path.to_string(),
        artifact_kind: "file".to_string(),
        content_hash: hash,
    })
}

/// Build deterministic identity for a directory artifact from sorted file hash entries.
pub fn content_identity_for_directory(
    path: &str,
    entries: Vec<(String, String)>,
) -> Result<ArtifactContentIdentityV1, String> {
    if path.trim().is_empty() {
        return Err("artifact path must not be empty".to_string());
    }
    if entries.is_empty() {
        return Err("directory entries must not be empty".to_string());
    }
    let mut canonical = entries;
    canonical.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    let material = canonical
        .into_iter()
        .map(|(entry_path, entry_hash)| format!("{entry_path}:{entry_hash}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut hasher = Sha256::new();
    hasher.update(material.as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    Ok(ArtifactContentIdentityV1 {
        artifact_path: path.to_string(),
        artifact_kind: "directory".to_string(),
        content_hash: hash,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        build_run_directory_layout_contract, content_identity_for_directory,
        content_identity_for_file,
    };

    #[test]
    fn g061_run_directory_layout_contract_is_predictable() {
        let layout = build_run_directory_layout_contract("20260501-abc")
            .expect("layout should build");
        assert_eq!(layout.manifest_path, "run-20260501-abc/manifest.json");
        assert_eq!(layout.outputs_index_path, "run-20260501-abc/outputs/index.json");
        assert_eq!(layout.replay_root, "run-20260501-abc/replay");
    }

    #[test]
    fn g062_artifact_identity_is_content_based_for_files_and_directories() {
        let file_a = content_identity_for_file("outputs/a.json", br#"{"a":1}"#)
            .expect("file identity");
        let file_b = content_identity_for_file("outputs/a.json", br#"{"a":2}"#)
            .expect("file identity");
        assert_ne!(file_a.content_hash, file_b.content_hash);

        let dir_a = content_identity_for_directory(
            "outputs/dir",
            vec![
                ("a.txt".to_string(), file_a.content_hash.clone()),
                ("b.txt".to_string(), file_b.content_hash.clone()),
            ],
        )
        .expect("dir identity");
        let dir_b = content_identity_for_directory(
            "outputs/dir",
            vec![
                ("b.txt".to_string(), file_b.content_hash),
                ("a.txt".to_string(), file_a.content_hash),
            ],
        )
        .expect("dir identity");
        assert_eq!(dir_a.content_hash, dir_b.content_hash);
    }
}
