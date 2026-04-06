use crate::NodeStatus;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BackendKind {
    Shell,
    Process,
    Container,
    RemoteFuture,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendCapabilities {
    pub kind: BackendKind,
    pub supports_env_shaping: bool,
    pub supports_timeout: bool,
    pub supports_stream_capture: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionBackendCapabilityDescriptor {
    pub backend_name: String,
    pub kind: BackendKind,
    pub supports_env_shaping: bool,
    pub supports_timeout: bool,
    pub supports_stream_capture: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendBindingRequest {
    pub node_id: String,
    pub required_kind: BackendKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionAttemptRecord {
    pub node_id: String,
    pub attempt: u32,
    pub backend_kind: BackendKind,
    pub status: NodeStatus,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendContext {
    pub node_id: String,
    pub attempt: u32,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub declared_outputs: BTreeSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendLifecycleResult {
    pub status: NodeStatus,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub produced_outputs: BTreeSet<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("backend capability mismatch: {0}")]
    Capability(String),
    #[error("backend prepare failed: {0}")]
    Prepare(String),
    #[error("backend launch failed: {0}")]
    Launch(String),
    #[error("backend observe failed: {0}")]
    Observe(String),
    #[error("backend observe timed out: {0}")]
    ObserveTimeout(String),
    #[error("backend finalize failed: {0}")]
    Finalize(String),
    #[error("backend cleanup failed: {0}")]
    Cleanup(String),
}

pub trait ExecutionBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn capabilities(&self) -> BackendCapabilities;
    fn prepare(&self, _ctx: &BackendContext) -> Result<(), BackendError>;
    fn launch(&self, _ctx: &BackendContext) -> Result<(), BackendError>;
    fn observe(&self, _ctx: &BackendContext) -> Result<BackendLifecycleResult, BackendError>;
    fn finalize(
        &self,
        _ctx: &BackendContext,
        _result: &BackendLifecycleResult,
    ) -> Result<(), BackendError>;
    fn cleanup(&self, _ctx: &BackendContext) -> Result<(), BackendError>;
}

pub fn bind_backend_or_error(
    request: &BackendBindingRequest,
    backend: &dyn ExecutionBackend,
) -> Result<(), BackendError> {
    let caps = backend.capabilities();
    if caps.kind == request.required_kind {
        Ok(())
    } else {
        Err(BackendError::Capability(format!(
            "node '{}' requires {:?}, backend '{}' exposes {:?}",
            request.node_id,
            request.required_kind,
            backend.name(),
            caps.kind
        )))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EngineOutcome {
    pub attempts: Vec<ExecutionAttemptRecord>,
}

pub fn execute_with_backend(
    backend: &dyn ExecutionBackend,
    required_kind: BackendKind,
    nodes: &[String],
) -> Result<EngineOutcome, BackendError> {
    let mut attempts = Vec::new();
    for node_id in nodes {
        bind_backend_or_error(
            &BackendBindingRequest {
                node_id: node_id.clone(),
                required_kind: required_kind.clone(),
            },
            backend,
        )?;
        let ctx = BackendContext {
            node_id: node_id.clone(),
            attempt: 1,
            args: vec!["echo".to_string(), node_id.clone()],
            env: BTreeMap::new(),
            declared_outputs: BTreeSet::new(),
        };
        let lifecycle = (|| -> Result<BackendLifecycleResult, BackendError> {
            backend.prepare(&ctx)?;
            backend.launch(&ctx)?;
            let observed = backend.observe(&ctx)?;
            if !observed.produced_outputs.iter().all(|entry| ctx.declared_outputs.contains(entry)) {
                return Err(BackendError::Finalize(format!(
                    "backend produced undeclared outputs for {}",
                    ctx.node_id
                )));
            }
            backend.finalize(&ctx, &observed)?;
            Ok(observed)
        })();
        let cleanup = backend.cleanup(&ctx);
        let observed = match (lifecycle, cleanup) {
            (Err(primary), _) => return Err(primary),
            (Ok(_), Err(cleanup_error)) => return Err(cleanup_error),
            (Ok(observed), Ok(())) => observed,
        };
        attempts.push(ExecutionAttemptRecord {
            node_id: node_id.clone(),
            attempt: 1,
            backend_kind: backend.capabilities().kind,
            status: observed.status,
            exit_code: observed.exit_code,
        });
    }
    Ok(EngineOutcome { attempts })
}

pub fn backend_registry() -> Vec<ExecutionBackendCapabilityDescriptor> {
    let fake = FakeBackend::default();
    let process_like = ProcessLikeBackend;
    let backends: [&dyn ExecutionBackend; 2] = [&fake, &process_like];
    backends
        .iter()
        .map(|backend| {
            let caps = backend.capabilities();
            ExecutionBackendCapabilityDescriptor {
                backend_name: backend.name().to_string(),
                kind: caps.kind,
                supports_env_shaping: caps.supports_env_shaping,
                supports_timeout: caps.supports_timeout,
                supports_stream_capture: caps.supports_stream_capture,
            }
        })
        .collect()
}

#[derive(Default)]
pub struct FakeBackend {
    pub fail_prepare_for: BTreeSet<String>,
    pub fail_launch_for: BTreeSet<String>,
    pub fail_observe_timeout_for: BTreeSet<String>,
    pub fail_finalize_for: BTreeSet<String>,
    pub fail_cleanup_for: BTreeSet<String>,
}

impl ExecutionBackend for FakeBackend {
    fn name(&self) -> &'static str {
        "fake-backend"
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            kind: BackendKind::Shell,
            supports_env_shaping: true,
            supports_timeout: true,
            supports_stream_capture: true,
        }
    }

    fn prepare(&self, ctx: &BackendContext) -> Result<(), BackendError> {
        if self.fail_prepare_for.contains(&ctx.node_id) {
            return Err(BackendError::Prepare(format!("prepare failed for {}", ctx.node_id)));
        }
        Ok(())
    }

    fn launch(&self, ctx: &BackendContext) -> Result<(), BackendError> {
        if self.fail_launch_for.contains(&ctx.node_id) {
            return Err(BackendError::Launch(format!("launch failed for {}", ctx.node_id)));
        }
        Ok(())
    }

    fn observe(&self, ctx: &BackendContext) -> Result<BackendLifecycleResult, BackendError> {
        if self.fail_observe_timeout_for.contains(&ctx.node_id) {
            return Err(BackendError::ObserveTimeout(format!(
                "observe timeout for {}",
                ctx.node_id
            )));
        }
        Ok(BackendLifecycleResult {
            status: NodeStatus::Success,
            exit_code: Some(0),
            stdout: String::new(),
            stderr: String::new(),
            produced_outputs: BTreeSet::new(),
        })
    }

    fn finalize(
        &self,
        ctx: &BackendContext,
        _result: &BackendLifecycleResult,
    ) -> Result<(), BackendError> {
        if self.fail_finalize_for.contains(&ctx.node_id) {
            return Err(BackendError::Finalize(format!("finalize failed for {}", ctx.node_id)));
        }
        Ok(())
    }

    fn cleanup(&self, ctx: &BackendContext) -> Result<(), BackendError> {
        if self.fail_cleanup_for.contains(&ctx.node_id) {
            return Err(BackendError::Cleanup(format!("cleanup failed for {}", ctx.node_id)));
        }
        Ok(())
    }
}

pub struct ProcessLikeBackend;

impl ExecutionBackend for ProcessLikeBackend {
    fn name(&self) -> &'static str {
        "process-like-backend"
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            kind: BackendKind::Shell,
            supports_env_shaping: true,
            supports_timeout: true,
            supports_stream_capture: true,
        }
    }

    fn prepare(&self, _ctx: &BackendContext) -> Result<(), BackendError> {
        Ok(())
    }

    fn launch(&self, _ctx: &BackendContext) -> Result<(), BackendError> {
        Ok(())
    }

    fn observe(&self, _ctx: &BackendContext) -> Result<BackendLifecycleResult, BackendError> {
        Ok(BackendLifecycleResult {
            status: NodeStatus::Success,
            exit_code: Some(0),
            stdout: String::new(),
            stderr: String::new(),
            produced_outputs: BTreeSet::new(),
        })
    }

    fn finalize(
        &self,
        _ctx: &BackendContext,
        _result: &BackendLifecycleResult,
    ) -> Result<(), BackendError> {
        Ok(())
    }

    fn cleanup(&self, _ctx: &BackendContext) -> Result<(), BackendError> {
        Ok(())
    }
}
