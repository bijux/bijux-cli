#![forbid(unsafe_code)]
//! Prevents dev-cli route/registry presentation assembly from living in routing.

#[test]
fn routing_reports_keep_only_route_audit_presentation() {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/reports.rs"))
        .expect("read routing reports source");

    assert!(source.contains("route_audit_report"), "route audit report assembly must remain");
    assert!(
        !source.contains("pub fn routes_report"),
        "routing crate must not expose routes presentation report"
    );
    assert!(
        !source.contains("pub fn registry_report"),
        "routing crate must not expose registry presentation report"
    );
}
