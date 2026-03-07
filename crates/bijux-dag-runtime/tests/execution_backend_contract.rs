use bijux_dag_runtime::{
    execute_with_backend, BackendError, BackendKind, ExecutionBackend, FakeBackend,
    ProcessLikeBackend,
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
    let err = execute_with_backend(&backend, BackendKind::Container, &["a".to_string()]).unwrap_err();
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
