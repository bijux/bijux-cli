use std::fs;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn runtime_does_not_depend_on_clap_or_cli_surfaces() {
    let runtime_lib = fs::read_to_string(root().join("crates/bijux-dag-runtime/src/lib.rs"))
        .expect("read runtime lib");
    assert!(!runtime_lib.contains("use clap"));
    assert!(!runtime_lib.contains("bijux_dag_cli"));
}

#[test]
fn cli_crate_remains_dispatch_thin() {
    let cli_main = fs::read_to_string(root().join("crates/bijux-dag-cli/src/main.rs"))
        .expect("read cli main");
    assert!(cli_main.contains("dag_run("));
    assert!(!cli_main.contains("build_plan("));
    assert!(!cli_main.contains("execute_with_retries("));
}

#[test]
fn app_crate_avoids_runtime_scheduler_internal_types() {
    let app_lib = fs::read_to_string(root().join("crates/bijux-dag-app/src/lib.rs"))
        .expect("read app lib");
    assert!(!app_lib.contains("SchedulerPolicy {"));
    assert!(!app_lib.contains("ReadyQueue::"));
}
