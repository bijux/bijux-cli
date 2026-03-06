use std::fs;
use std::path::Path;

#[test]
fn core_has_no_io_imports() {
    let root = Path::new("crates/bijux-dag-core/src");
    let mut offenders = Vec::new();
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let content = fs::read_to_string(&path).unwrap();
        for needle in ["std::fs", "std::process", "std::env", "std::time"] {
            if content.contains(needle) {
                offenders.push(format!("{} => {}", path.display(), needle));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "core must remain pure (no I/O). offenders:\n{}",
        offenders.join("\n")
    );
}
