#![forbid(unsafe_code)]
//! Registry fuzz regression suite replaying minimized registry-fuzz cases.
//! test_type: registry-fuzz-regression

use bijux_cli_routing as _;
use std::fs;
use std::path::Path;

use bijux_cli_plugin::{load_registry, PluginError};
use semver as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use thiserror as _;

#[test]
fn minimized_registry_cases_replay_without_unexpected_errors() {
    let dir = Path::new("tests/fuzz/registry_minimized_cases");
    let mut files: Vec<_> = fs::read_dir(dir)
        .expect("minimized registry cases directory must exist")
        .map(|entry| entry.expect("entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "minimized registry cases must be retained");

    let temp = std::env::temp_dir().join("bijux-registry-fuzz-replay.json");
    for path in files {
        let sample = fs::read_to_string(&path).expect("sample should be readable");
        fs::write(&temp, sample).expect("write sample");

        let first = load_registry(&temp);
        let second = load_registry(&temp);
        assert_eq!(first.is_ok(), second.is_ok());

        if let Err(err) = first {
            assert!(matches!(err, PluginError::RegistryCorrupted | PluginError::Io(_)));
        }
    }
}
