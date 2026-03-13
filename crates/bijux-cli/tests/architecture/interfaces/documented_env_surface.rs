#![forbid(unsafe_code)]
//! Public documentation checks for supported environment-variable surfaces.

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn read_doc(relative: &str) -> String {
    fs::read_to_string(workspace_root().join(relative)).expect("doc must be readable")
}

#[test]
fn state_environment_reference_lists_only_supported_public_variables() {
    let doc = read_doc("docs/06-reference/state-and-environment.md");

    for supported in [
        "`BIJUXCLI_FORMAT`",
        "`BIJUXCLI_LOG_LEVEL`",
        "`BIJUXCLI_COLOR`",
        "`BIJUXCLI_CONFIG`",
        "`BIJUXCLI_HISTORY_FILE`",
        "`BIJUXCLI_PLUGINS_DIR`",
        "`BIJUX_BIN`",
        "`NO_COLOR=1`",
    ] {
        assert!(doc.contains(supported), "{supported} must remain documented");
    }

    for unsupported in [
        "`BIJUX_DEV_CLI_BIN`",
        "`BIJUXCLI_ALLOWED_PRODUCT_BINS`",
        "`BIJUXCLI_PRODUCT_BIN_DIR`",
        "`BIJUXCLI_PRODUCT_BIN_DIRS`",
        "`BIJUXCLI_PRODUCT_BIN_PRECEDENCE`",
        "`BIJUXCLI_ENFORCE_PRODUCT_MAJOR_MATCH`",
    ] {
        assert!(
            !doc.contains(unsupported),
            "{unsupported} is not implemented and must not be documented as public"
        );
    }
}

#[test]
fn routed_runtime_reference_matches_current_binary_resolution_contract() {
    let doc = read_doc("docs/06-reference/integrations-and-routed-runtimes.md");

    assert!(doc.contains("bijux-dev-cli"), "maintainer binary documentation must stay explicit");
    assert!(
        doc.contains("`bijux-dev-<tool>`"),
        "product control-plane binaries must remain documented explicitly"
    );
    assert!(
        doc.contains("`PATH`"),
        "product binary documentation must describe PATH-based discovery"
    );

    for unsupported in [
        "bijux dev ",
        "BIJUXCLI_ALLOWED_PRODUCT_BINS",
        "BIJUXCLI_PRODUCT_BIN_DIR",
        "BIJUXCLI_PRODUCT_BIN_DIRS",
        "BIJUXCLI_PRODUCT_BIN_PRECEDENCE",
        "BIJUXCLI_ENFORCE_PRODUCT_MAJOR_MATCH",
        "BIJUX_DEV_CLI_BIN",
    ] {
        assert!(
            !doc.contains(unsupported),
            "{unsupported} is not part of the current routing contract"
        );
    }
}
