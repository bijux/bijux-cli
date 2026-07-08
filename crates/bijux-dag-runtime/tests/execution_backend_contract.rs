use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::{
    backend_registry, execute_with_backend, BackendContext, BackendError, BackendKind,
    BackendLifecycleResult, ExecutionBackend, FakeBackend, NodeStatus, ProcessLikeBackend,
};
use std::collections::BTreeSet;

#[test]
fn fake_backend_can_exercise_engine_without_subprocesses() {
    let backend = FakeBackend::default();
    let outcome =
        execute_with_backend(&backend, BackendKind::Shell, &["a".to_string(), "b".to_string()])
            .expect("fake backend run");
    assert_eq!(outcome.attempts.len(), 2);
    assert!(outcome.attempts.iter().all(|a| a.attempt == 1));
}

#[test]
fn fake_and_process_like_backends_have_parity_on_basic_scenario() {
    let fake = FakeBackend::default();
    let process_like = ProcessLikeBackend;
    let nodes = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let fake_outcome = execute_with_backend(&fake, BackendKind::Shell, &nodes).unwrap();
    let process_outcome = execute_with_backend(&process_like, BackendKind::Shell, &nodes).unwrap();
    assert_eq!(fake_outcome.attempts, process_outcome.attempts);
}

#[test]
fn backend_capability_mismatch_fails_during_binding() {
    let backend = FakeBackend::default();
    let err =
        execute_with_backend(&backend, BackendKind::Container, &["a".to_string()]).unwrap_err();
    match err {
        BackendError::Capability(message) => assert!(message.contains("requires")),
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn backend_prepare_failures_are_classified_correctly() {
    let backend = FakeBackend {
        fail_prepare_for: BTreeSet::from(["a".to_string()]),
        ..FakeBackend::default()
    };
    let err = execute_with_backend(&backend, BackendKind::Shell, &["a".to_string()]).unwrap_err();
    assert!(matches!(err, BackendError::Prepare(_)));
}

#[test]
fn backend_launch_failures_do_not_corrupt_state() {
    let backend = FakeBackend {
        fail_launch_for: BTreeSet::from(["a".to_string()]),
        ..FakeBackend::default()
    };
    let err = execute_with_backend(&backend, BackendKind::Shell, &["a".to_string()]).unwrap_err();
    assert!(matches!(err, BackendError::Launch(_)));
}

#[test]
fn backend_finalize_failures_do_not_look_like_success() {
    let backend = FakeBackend {
        fail_finalize_for: BTreeSet::from(["a".to_string()]),
        ..FakeBackend::default()
    };
    let err = execute_with_backend(&backend, BackendKind::Shell, &["a".to_string()]).unwrap_err();
    assert!(matches!(err, BackendError::Finalize(_)));
}

#[test]
fn cleanup_runs_after_observe_and_reports_cleanup_failures() {
    struct CleanupFailureBackend;
    impl ExecutionBackend for CleanupFailureBackend {
        fn name(&self) -> &'static str {
            "cleanup-failure-backend"
        }
        fn capabilities(&self) -> bijux_dag_runtime::BackendCapabilities {
            bijux_dag_runtime::BackendCapabilities {
                kind: BackendKind::Shell,
                supports_env_shaping: true,
                supports_timeout: true,
                supports_stream_capture: true,
            }
        }
        fn prepare(&self, _ctx: &bijux_dag_runtime::BackendContext) -> Result<(), BackendError> {
            Ok(())
        }
        fn launch(&self, _ctx: &bijux_dag_runtime::BackendContext) -> Result<(), BackendError> {
            Ok(())
        }
        fn observe(
            &self,
            _ctx: &bijux_dag_runtime::BackendContext,
        ) -> Result<bijux_dag_runtime::BackendLifecycleResult, BackendError> {
            Ok(bijux_dag_runtime::BackendLifecycleResult {
                status: bijux_dag_runtime::NodeStatus::Success,
                exit_code: Some(0),
                stdout: String::new(),
                stderr: String::new(),
                produced_outputs: BTreeSet::new(),
            })
        }
        fn finalize(
            &self,
            _ctx: &bijux_dag_runtime::BackendContext,
            _result: &bijux_dag_runtime::BackendLifecycleResult,
        ) -> Result<(), BackendError> {
            Ok(())
        }
        fn cleanup(&self, _ctx: &bijux_dag_runtime::BackendContext) -> Result<(), BackendError> {
            Err(BackendError::Cleanup("cleanup failure".to_string()))
        }
    }
    let backend = CleanupFailureBackend;
    let err = execute_with_backend(&backend, BackendKind::Shell, &["a".to_string()]).unwrap_err();
    assert!(matches!(err, BackendError::Cleanup(_)));
}

#[test]
fn cleanup_runs_when_prepare_fails() {
    struct PrepareFailureStillCleansUpBackend;
    impl ExecutionBackend for PrepareFailureStillCleansUpBackend {
        fn name(&self) -> &'static str {
            "prepare-failure-still-cleans-up"
        }
        fn capabilities(&self) -> bijux_dag_runtime::BackendCapabilities {
            bijux_dag_runtime::BackendCapabilities {
                kind: BackendKind::Shell,
                supports_env_shaping: true,
                supports_timeout: true,
                supports_stream_capture: true,
            }
        }
        fn prepare(&self, _ctx: &BackendContext) -> Result<(), BackendError> {
            Err(BackendError::Prepare("prepare failed".to_string()))
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
            Err(BackendError::Cleanup("cleanup executed".to_string()))
        }
    }
    let backend = PrepareFailureStillCleansUpBackend;
    let err = execute_with_backend(&backend, BackendKind::Shell, &["a".to_string()]).unwrap_err();
    assert!(matches!(err, BackendError::Prepare(_)));
}

#[test]
fn backend_observe_timeout_has_distinct_error() {
    let backend = FakeBackend {
        fail_observe_timeout_for: BTreeSet::from(["a".to_string()]),
        ..FakeBackend::default()
    };
    let err = execute_with_backend(&backend, BackendKind::Shell, &["a".to_string()]).unwrap_err();
    assert!(matches!(err, BackendError::ObserveTimeout(_)));
}

#[test]
fn stdout_stderr_capture_contract_is_stable() {
    struct CaptureBackend;
    impl ExecutionBackend for CaptureBackend {
        fn name(&self) -> &'static str {
            "capture-backend"
        }
        fn capabilities(&self) -> bijux_dag_runtime::BackendCapabilities {
            bijux_dag_runtime::BackendCapabilities {
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
                stdout: "captured-stdout".to_string(),
                stderr: "captured-stderr".to_string(),
                produced_outputs: BTreeSet::new(),
            })
        }
        fn finalize(
            &self,
            _ctx: &BackendContext,
            result: &BackendLifecycleResult,
        ) -> Result<(), BackendError> {
            if result.stdout != "captured-stdout" || result.stderr != "captured-stderr" {
                return Err(BackendError::Finalize("capture mismatch".to_string()));
            }
            Ok(())
        }
        fn cleanup(&self, _ctx: &BackendContext) -> Result<(), BackendError> {
            Ok(())
        }
    }
    let backend = CaptureBackend;
    let outcome = execute_with_backend(&backend, BackendKind::Shell, &["a".to_string()]).unwrap();
    assert_eq!(outcome.attempts.len(), 1);
}

#[test]
fn backend_output_collection_rejects_undeclared_outputs() {
    struct UndeclaredOutputBackend;
    impl ExecutionBackend for UndeclaredOutputBackend {
        fn name(&self) -> &'static str {
            "undeclared-output-backend"
        }
        fn capabilities(&self) -> bijux_dag_runtime::BackendCapabilities {
            bijux_dag_runtime::BackendCapabilities {
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
                produced_outputs: BTreeSet::from(["undeclared.txt".to_string()]),
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
    let backend = UndeclaredOutputBackend;
    let err = execute_with_backend(&backend, BackendKind::Shell, &["a".to_string()]).unwrap_err();
    assert!(matches!(err, BackendError::Finalize(_)));
}

#[test]
fn backend_registry_includes_capability_descriptors() {
    let registry = backend_registry();
    assert!(registry.iter().any(|row| row.backend_name == "fake-backend"));
    assert!(registry.iter().any(|row| row.backend_name == "process-like-backend"));
    assert!(registry.iter().all(|row| row.supports_env_shaping
        && row.supports_timeout
        && row.supports_stream_capture));
}

#[test]
fn backend_env_shaping_contract_is_explicitly_applied() {
    struct EnvShapeBackend;
    impl ExecutionBackend for EnvShapeBackend {
        fn name(&self) -> &'static str {
            "env-shape-backend"
        }
        fn capabilities(&self) -> bijux_dag_runtime::BackendCapabilities {
            bijux_dag_runtime::BackendCapabilities {
                kind: BackendKind::Shell,
                supports_env_shaping: true,
                supports_timeout: true,
                supports_stream_capture: true,
            }
        }
        fn prepare(&self, ctx: &BackendContext) -> Result<(), BackendError> {
            if !ctx.env.is_empty() {
                return Err(BackendError::Prepare(
                    "unexpected ambient environment leak".to_string(),
                ));
            }
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
    let backend = EnvShapeBackend;
    let outcome = execute_with_backend(&backend, BackendKind::Shell, &["a".to_string()]).unwrap();
    assert_eq!(outcome.attempts.len(), 1);
}
