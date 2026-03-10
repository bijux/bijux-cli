#![forbid(unsafe_code)]
//! Source-level invariants for dev-cli dispatch and failure handling.

#[test]
fn dev_cli_dispatch_uses_shared_envelope_and_exit_mapping() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/interface/cli/dispatch.rs"
    ))
    .expect("read dispatch source");
    assert!(source.contains("render_value("), "core app must use shared report envelope renderer");
    assert!(source.contains("AppRunResult"), "core app must return a normalized run envelope");
}

#[test]
fn dev_cli_dispatch_remains_core_only_and_bin_stays_thin() {
    let core_source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/interface/cli/dispatch.rs"
    ))
    .expect("read core dispatch source");
    let bin_source =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bin/bijux-rs.rs"))
            .expect("read core bin source");

    assert!(core_source.contains("developer_runtime_handlers::try_handle"));
    assert!(
        core_source.contains("owns_dev_cli_path"),
        "core route target classification must use bijux-dev-cli ownership helper"
    );
    assert!(!bin_source.contains("dev cli"), "bin must not own dev cli dispatch");
}
