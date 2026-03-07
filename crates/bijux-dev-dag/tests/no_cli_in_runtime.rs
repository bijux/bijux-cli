use std::fs;
use std::path::Path;

#[test]
fn runtime_has_no_cli_deps() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = root.join("crates/bijux-dag-runtime/Cargo.toml");
    let content = fs::read_to_string(path).unwrap();
    assert!(
        !content.contains("bijux-dag-app"),
        "runtime must not depend on bijux-dag-app"
    );
    assert!(
        !content.contains("bijux-dag-cli"),
        "runtime must not depend on bijux-dag-cli"
    );
}
