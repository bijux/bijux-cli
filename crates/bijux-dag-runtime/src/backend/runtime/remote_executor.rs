use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteExecutionRequest {
    pub run_id: String,
    pub node_id: String,
    pub contract_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteExecutionReceipt {
    pub submission_id: String,
    pub accepted: bool,
    pub reason: Option<String>,
}

pub trait RemoteExecutorSubmitter: Send + Sync {
    fn submit(&self, request: RemoteExecutionRequest) -> Result<RemoteExecutionReceipt, String>;
}
