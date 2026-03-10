#![forbid(unsafe_code)]
//! Prevents maintainer route/registry report shaping from drifting back into core.

#[test]
fn core_app_routes_and_registry_delegate_to_dev_cli() {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/app.rs"))
        .expect("read core app source");

    assert!(
        source.contains("dev_routes::build_report"),
        "core app must delegate dev cli routes to bijux-dev-cli"
    );
    assert!(
        source.contains("dev_registry::build_report"),
        "core app must delegate dev cli registry to bijux-dev-cli"
    );
    assert!(
        !source.contains("routes_report(&registry)"),
        "core app must not shape dev cli routes directly"
    );
    assert!(
        !source.contains("registry_report(&registry)"),
        "core app must not shape dev cli registry directly"
    );
}
