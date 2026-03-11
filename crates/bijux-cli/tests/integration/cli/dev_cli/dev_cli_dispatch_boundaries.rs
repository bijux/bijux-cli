#![forbid(unsafe_code)]
//! Ensures binary entrypoint remains a thin dispatcher without dev workflow ownership.

#[test]
fn main_entrypoint_stays_thin_and_route_agnostic() {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bin/bijux.rs"))
        .expect("read bijux.rs");

    assert!(
        source.contains("bijux_cli::api::runtime::run_cli_from_env"),
        "main entrypoint must delegate command execution to core app"
    );
    assert!(
        !source.contains("dev cli"),
        "main entrypoint must not hardcode maintainer command routing"
    );
    assert!(
        !source.contains("match normalized_path"),
        "main entrypoint must not own command dispatch match arms"
    );
}
