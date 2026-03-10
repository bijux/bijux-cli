#![forbid(unsafe_code)]
//! Plugin manifest fuzz regression replay for retained minimized cases.
//! test_type: plugin-manifest-fuzz-regression

use std::fs;
use std::path::Path;

use bijux_cli_plugin::{parse_manifest_v1, validate_manifest};
use bijux_cli_routing as _;
use semver as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use thiserror as _;

#[test]
fn minimized_plugin_manifest_cases_replay_deterministically() {
    let dir = Path::new("tests/fuzz/plugin_manifest_minimized_cases");
    let mut files: Vec<_> = fs::read_dir(dir)
        .expect("manifest minimized corpus must exist")
        .map(|e| e.expect("entry").path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "manifest minimized corpus must not be empty");

    for path in files {
        let sample = fs::read_to_string(&path).expect("read sample");
        let a = parse_manifest_v1(&sample);
        let b = parse_manifest_v1(&sample);
        assert_eq!(a.is_ok(), b.is_ok(), "parse determinism for {}", path.display());

        if let Ok(manifest) = a {
            let va = validate_manifest(manifest.clone(), "0.1.0", &[]);
            let vb = validate_manifest(manifest, "0.1.0", &[]);
            assert_eq!(va.is_ok(), vb.is_ok(), "validate determinism for {}", path.display());
        }
    }
}
