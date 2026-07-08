use base64 as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_app::{dag_command, dag_run};

fn run_without_panic(args: &[&str]) -> bool {
    std::panic::catch_unwind(|| {
        let matches = dag_command().get_matches_from(args.iter().copied());
        let _ = dag_run(&matches);
    })
    .is_ok()
}

#[test]
fn malformed_route_entrypoints_do_not_panic() {
    let cases: &[&[&str]] = &[
        &["bijux-dag", "validate", "/no/such/file.json"],
        &["bijux-dag", "plan", "explain", "/no/such/file.json"],
        &["bijux-dag", "run", "/no/such/file.json", "--out", "/tmp/nowhere"],
        &["bijux-dag", "replay", "/no/such/run", "--out", "/tmp/replay"],
        &["bijux-dag", "runs", "inspect", "missing-run", "--root", "/no/such/root"],
        &["bijux-dag", "diff", "/no/such/run-a", "/no/such/run-b"],
        &["bijux-dag", "prove", "/no/such/run"],
        &["bijux-dag", "export", "--out", "/tmp/bundle.json"],
        &["bijux-dag", "import", "/no/such/bundle.json", "--verify-only"],
        &["bijux-dag", "cache", "verify", "--cache-dir", "/no/such/cache"],
        &["bijux-dag", "explain", "/no/such/run"],
        &["bijux-dag", "artifact-inspect", "/no/such/run", "node1:out"],
        &["bijux-dag", "capabilities", "--backend", "unknown"],
    ];
    for case in cases {
        assert!(run_without_panic(case), "command panicked for malformed input: {case:?}");
    }
}
