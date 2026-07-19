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

use bijux_dag_runtime::{LocalWorkerExecution, LocalWorkerPool, LocalWorkerState};
use std::sync::{mpsc, Arc, Barrier};

#[test]
fn local_worker_pool_enforces_fixed_capacity_before_completion() {
    let mut pool = LocalWorkerPool::<&'static str>::new(2);
    let barrier = Arc::new(Barrier::new(3));

    for node_id in ["a", "b"] {
        let barrier = Arc::clone(&barrier);
        pool.submit(
            node_id.to_string(),
            Box::new(move || {
                barrier.wait();
                LocalWorkerExecution { started_unix_ms: 1, finished_unix_ms: 2, result: "done" }
            }),
        )
        .expect("submit worker job");
    }

    let error = pool
        .submit(
            "c".to_string(),
            Box::new(|| LocalWorkerExecution {
                started_unix_ms: 1,
                finished_unix_ms: 2,
                result: "extra",
            }),
        )
        .expect_err("bounded capacity");
    assert!(error.contains("no idle local worker available"));
    assert_eq!(pool.available_workers(), 0);
    assert!(pool.has_running());

    barrier.wait();

    let first = pool.wait_for_completion().expect("first completion");
    let second = pool.wait_for_completion().expect("second completion");
    assert_ne!(first.node_id, second.node_id);
    assert_eq!(pool.available_workers(), 2);
    assert!(!pool.has_running());
}

#[test]
fn local_worker_pool_reports_completion_identity_and_releases_worker() {
    let mut pool = LocalWorkerPool::<String>::new(1);

    let assignment = pool
        .submit(
            "report".to_string(),
            Box::new(|| LocalWorkerExecution {
                started_unix_ms: 10,
                finished_unix_ms: 20,
                result: "ok".to_string(),
            }),
        )
        .expect("submit");
    let completion = pool.wait_for_completion().expect("completion");

    assert_eq!(completion.worker_id, assignment.worker_id);
    assert_eq!(completion.node_id, assignment.node_id);
    assert_eq!(completion.started_unix_ms, 10);
    assert_eq!(completion.finished_unix_ms, 20);
    assert_eq!(completion.result, "ok");
    assert_eq!(pool.available_workers(), 1);
    assert_eq!(pool.status()[0].state, LocalWorkerState::Idle);
}

#[test]
fn local_worker_pool_blocks_new_submissions_after_cancellation_request() {
    let mut pool = LocalWorkerPool::<&'static str>::new(1);
    let (release_tx, release_rx) = mpsc::channel();

    pool.submit(
        "running".to_string(),
        Box::new(move || {
            release_rx.recv().expect("release running worker");
            LocalWorkerExecution { started_unix_ms: 4, finished_unix_ms: 9, result: "done" }
        }),
    )
    .expect("submit running worker");

    pool.request_cancellation();
    assert_eq!(
        pool.status()[0].state,
        LocalWorkerState::CancelRequested { node_id: "running".to_string() }
    );

    let error = pool
        .submit(
            "blocked".to_string(),
            Box::new(|| LocalWorkerExecution {
                started_unix_ms: 10,
                finished_unix_ms: 11,
                result: "blocked",
            }),
        )
        .expect_err("cancellation closes submissions");
    assert!(error.contains("closed to new submissions"));

    release_tx.send(()).expect("release completion");
    let completion = pool.wait_for_completion().expect("completion after cancellation");
    assert_eq!(completion.node_id, "running");
    assert_eq!(pool.status()[0].state, LocalWorkerState::Idle);
}
