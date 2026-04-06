use crate::infrastructure::{BackendExecutionCompletion, BackendExecutionRequest, ExecutorBackend};
use crate::{NodeResult, RuntimeError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdapterCapabilities {
    pub supports_async: bool,
    pub supports_streaming_logs: bool,
    pub supported_backends: Vec<ExecutorBackend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterContext {
    pub run_id: String,
    pub node_id: String,
    pub params: Value,
    pub env: BTreeMap<String, String>,
}

pub trait AdapterPlugin: Send + Sync {
    fn adapter_name(&self) -> &str;
    fn adapter_version(&self) -> &str;
    fn capabilities(&self) -> AdapterCapabilities;
    fn execute(&self, context: &AdapterContext) -> Result<NodeResult, RuntimeError>;
}

pub trait BackendPlugin: Send + Sync {
    fn backend_kind(&self) -> ExecutorBackend;
    fn submit(&self, request: &BackendExecutionRequest) -> Result<(), String>;
    fn poll(
        &self,
        run_id: &str,
        node_id: &str,
    ) -> Result<Option<BackendExecutionCompletion>, String>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginManifest {
    pub plugin_name: String,
    pub plugin_version: String,
    pub plugin_type: String,
    pub contract_version: String,
}
