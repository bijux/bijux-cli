#![forbid(unsafe_code)]
//! Route case replay suite for retained minimized route registry cases.
//! `test_type`: route-case-replay

use proptest as _;
use serde as _;
use serde_json as _;
use std::fs;
use std::path::Path;

use bijux_cli::api::routing::registry::RouteRegistry;
use clap as _;
use schemars as _;
use semver as _;
use thiserror as _;

fn load_cases(dir: &Path) -> Vec<Vec<String>> {
    let mut files: Vec<_> = fs::read_dir(dir)
        .expect("minimized route cases directory must exist")
        .map(|entry| entry.expect("entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "txt"))
        .collect();
    files.sort();

    let mut cases = Vec::new();
    for path in files {
        let text = fs::read_to_string(&path).expect("minimized route case should be readable");
        let parts: Vec<String> = text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(str::to_string)
            .collect();
        cases.push(parts);
    }
    cases
}

#[test]
fn minimized_route_cases_do_not_crash_and_are_deterministic() {
    let cases = load_cases(Path::new("tests/fuzz/routing/route_minimized_cases"));
    assert!(!cases.is_empty(), "minimized route cases must be retained");

    for namespaces in cases {
        let mut left = RouteRegistry::default();
        let mut right = RouteRegistry::default();
        for ns in &namespaces {
            let _ = left.register_plugin_namespace(ns);
        }
        for ns in namespaces.iter().rev() {
            let _ = right.register_plugin_namespace(ns);
        }

        assert_eq!(left.route_tree(), right.route_tree());
        assert_eq!(left.render_command_tree(), right.render_command_tree());
    }
}
