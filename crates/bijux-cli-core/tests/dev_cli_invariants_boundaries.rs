#![forbid(unsafe_code)]
//! Source-level invariants for dev-cli dispatch and failure handling.

#[test]
fn dev_cli_dispatch_uses_shared_envelope_and_exit_mapping() {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/app.rs"))
        .expect("read app source");
    assert!(source.contains("render_value("), "core app must use shared report envelope renderer");
    assert!(source.contains("AppRunResult"), "core app must return a normalized run envelope");
}

#[test]
fn dev_cli_dispatch_remains_core_only_and_bin_stays_thin() {
    let core_source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/app.rs"))
        .expect("read core app source");
    let bin_source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/bin/bijux-rs.rs"
    ))
    .expect("read core bin source");

    assert!(core_source.contains("if a == \"dev\" && b == \"cli\""));
    assert!(!bin_source.contains("dev cli"), "bin must not own dev cli dispatch");
}
