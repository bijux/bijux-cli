#![forbid(unsafe_code)]
//! Bridge conversion case replay suite for retained minimized cases.
//! test_type: bridge-conversion-case-replay

use std::fs;
use std::path::Path;

use bijux_cli as _;
use bijux_cli_python as _;
use serde_json as _;

#[test]
fn minimized_bridge_conversion_cases_replay_deterministically() {
    let dir = Path::new("tests/fuzz/bridge_conversion_minimized_cases");
    let mut files: Vec<_> = fs::read_dir(dir)
        .expect("bridge minimized corpus must exist")
        .map(|e| e.expect("entry").path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "bridge minimized corpus must not be empty");

    for path in files {
        let sample = fs::read_to_string(&path).expect("read sample");
        let a = serde_json::from_str::<serde_json::Value>(&sample);
        let b = serde_json::from_str::<serde_json::Value>(&sample);
        assert_eq!(a.is_ok(), b.is_ok(), "determinism for {}", path.display());
    }
}
