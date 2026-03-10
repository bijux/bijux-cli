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

#[test]
fn core_app_env_contracts_parity_status_delegate_to_dev_cli() {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/app.rs"))
        .expect("read core app source");

    assert!(
        source.contains("dev_env::build_report"),
        "core app must delegate dev cli env to bijux-dev-cli"
    );
    assert!(
        source.contains("dev_contracts::build_report"),
        "core app must delegate dev cli contracts to bijux-dev-cli"
    );
    assert!(
        source.contains("dev_parity::build_report"),
        "core app must delegate dev cli parity to bijux-dev-cli"
    );
    assert!(
        source.contains("dev_status::build_report"),
        "core app must delegate dev cli status to bijux-dev-cli"
    );
    assert!(
        source.contains("dev_script_audit::build_report"),
        "core app must delegate dev cli script-audit to bijux-dev-cli"
    );
    assert!(
        source.contains("dev_docs_audit::build_report"),
        "core app must delegate dev cli docs-audit to bijux-dev-cli"
    );
    assert!(
        source.contains("dev_crate_health::build_report"),
        "core app must delegate dev cli crate-health to bijux-dev-cli"
    );
    assert!(
        source.contains("dev_runtime_identity::build_report"),
        "core app must delegate dev cli runtime-identity to bijux-dev-cli"
    );
    assert!(
        source.contains("dev_package_health::build_report"),
        "core app must delegate dev cli package-health to bijux-dev-cli"
    );
    assert!(
        source.contains("dev_state_audit::build_report"),
        "core app must delegate dev cli state-audit to bijux-dev-cli"
    );
    assert!(
        source.contains("dev_state_audit::build_doctor_report"),
        "core app must delegate dev cli state-doctor to bijux-dev-cli"
    );
    assert!(
        !source.contains("artifacts/parity/command_parity_matrix.json"),
        "core app must not assemble dev cli parity artifact payloads directly"
    );
    assert!(
        !source.contains("status_dev_cli_subcommands.json"),
        "core app must not assemble dev cli status report payloads directly"
    );
    assert!(
        !source.contains("active_path_is_canonical_name"),
        "core app must not assemble runtime identity payload fields directly"
    );
    assert!(
        !source.contains("docs_audit\": docs_audit"),
        "core app must not assemble docs-audit presentation payload directly"
    );
    assert!(
        !source.contains("crate_metrics\": metrics"),
        "core app must not assemble crate-health presentation payload directly"
    );
    assert!(
        !source.contains("remaining_script_only_behaviors"),
        "core app must not assemble script-audit presentation payload directly"
    );
}
