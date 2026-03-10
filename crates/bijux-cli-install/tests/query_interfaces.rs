#![forbid(unsafe_code)]
//! Query interface shape and determinism checks for install-owned dev bridge data.

use bijux_cli_install::query::runtime_identity_query;

#[test]
fn runtime_identity_query_shape_is_stable() {
    let query = runtime_identity_query("", None, None, "0.1.0");
    assert_eq!(query.path_binaries.len(), 0);
    assert!(!query.has_path_shadowing);
    assert!(!query.has_duplicate_installs);
}

#[test]
fn runtime_identity_query_is_deterministic_for_same_inputs() {
    let first = runtime_identity_query("", Some("/tmp/bijux"), Some("0.1.0"), "0.1.0");
    let second = runtime_identity_query("", Some("/tmp/bijux"), Some("0.1.0"), "0.1.0");
    assert_eq!(first, second);
}
