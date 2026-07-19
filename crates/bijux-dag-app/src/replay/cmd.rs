use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct ReplayCommandResponse {
    pub run_dir: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run_plan: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replay_proof: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream_artifact_verification: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_rerun_diff: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_surface: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_surface: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_scope: Option<serde_json::Value>,
}
