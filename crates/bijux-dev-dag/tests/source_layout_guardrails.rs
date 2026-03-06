use std::path::Path;

#[test]
fn repository_layout_contains_required_roots() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let required = ["crates", "docs", "examples", "configs/nextest"];

    for rel in required {
        assert!(root.join(rel).exists(), "missing required path: {rel}");
    }
}
