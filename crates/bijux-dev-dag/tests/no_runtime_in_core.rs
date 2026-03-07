use std::fs;
use std::path::Path;

#[test]
fn core_has_no_runtime_deps() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = root.join("crates/bijux-dag-core/Cargo.toml");
    let content = fs::read_to_string(path).unwrap();
    assert!(
        !content.contains("bijux-dag-runtime"),
        "core must not depend on runtime"
    );
}
