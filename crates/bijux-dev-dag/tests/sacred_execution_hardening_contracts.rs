use bijux_dag_testkit as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn sacred_execution_contract_and_report_exist() {
    let root = repo_root();
    for required in [
        "docs/spec/SACRED_EXECUTION_FLOW.md",
        "docs/architecture/runtime-execution-flow.md",
        "docs/reports/foundation/sacred_execution_hardening_report.md",
        "crates/bijux-dag-runtime/src/runtime_core/governance/sacred_execution.rs",
        "crates/bijux-dag-runtime/src/runtime_core/execution/context.rs",
        "crates/bijux-dag-runtime/tests/sacred_execution_flow_contracts.rs",
    ] {
        assert!(
            root.join(required).exists(),
            "missing sacred execution hardening surface: {required}"
        );
    }
}

#[test]
fn sacred_execution_spec_documents_canonical_context_and_pipeline() {
    let root = repo_root();
    let text = fs::read_to_string(root.join("docs/spec/SACRED_EXECUTION_FLOW.md"))
        .expect("sacred execution spec should exist");

    for token in [
        "plan",
        "schedule",
        "execute",
        "collect",
        "persist",
        "advance",
        "ExecutionContext",
        "NodeExecutionContext",
        "NodeResult",
        "Side-channel execution prohibition",
    ] {
        assert!(
            text.contains(token),
            "sacred execution spec missing token `{token}`"
        );
    }
}

#[test]
fn engine_keeps_sacred_hooks_and_forbids_direct_bypass_calls() {
    let root = repo_root();
    let source = fs::read_to_string(
        root.join("crates/bijux-dag-runtime/src/runtime_core/execution/engine.rs"),
    )
    .expect("engine source should exist");

    for required in [
        "sacred_execution::run_materialize_inputs",
        "sacred_execution::run_cache_lookup",
        "sacred_execution::run_retry_logic",
        "sacred_execution::run_write_trace",
        "sacred_execution::run_cache_write",
        "sacred_execution::resolve_dependencies",
    ] {
        assert!(
            source.contains(required),
            "engine missing sacred hook `{required}`"
        );
    }

    for forbidden in [
        "crate::try_cache_read(",
        "crate::try_cache_write(",
        "crate::write_trace(",
        "crate::execute_with_retries(",
    ] {
        assert!(
            !source.contains(forbidden),
            "engine bypasses sacred hook with direct call `{forbidden}`"
        );
    }
}
