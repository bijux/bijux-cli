use std::fs;
use std::path::Path;

fn line_count(path: &Path) -> usize {
    fs::read_to_string(path)
        .expect("read file")
        .lines()
        .count()
}

#[test]
fn source_files_stay_under_size_budget() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let max_lines = 2200usize;
    let allowlist = [
        "crates/bijux-dag-runtime/src/lib.rs",
        "crates/bijux-dag-app/src/lib.rs",
    ];

    let mut violations = Vec::new();
    for entry in [
        "crates/bijux-dag-runtime/src/engine.rs",
        "crates/bijux-dag-runtime/src/planner.rs",
        "crates/bijux-dag-runtime/src/external_adapter.rs",
        "crates/bijux-dag-app/src/diff.rs",
        "crates/bijux-dev-dag/src/main.rs",
    ] {
        let path = root.join(entry);
        let count = line_count(&path);
        if count > max_lines {
            violations.push(format!("{entry} has {count} lines"));
        }
    }

    for entry in allowlist {
        let path = root.join(entry);
        assert!(path.exists(), "allowlisted path missing: {entry}");
    }

    assert!(violations.is_empty(), "{}", violations.join(", "));
}
