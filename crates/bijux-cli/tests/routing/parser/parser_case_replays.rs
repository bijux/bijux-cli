#![forbid(unsafe_code)]
//! Parser case replay suite for retained minimized and interesting inputs.
//! test_type: parser-case-replay

use std::fs;
use std::path::Path;

use bijux_cli::api::routing::parser::parse_intent;
use bijux_cli::api::routing::registry::RouteRegistry;
use clap as _;
use proptest as _;
use schemars as _;
use semver as _;
use serde as _;
use serde_json as _;
use thiserror as _;

fn split_argv(line: &str) -> Vec<String> {
    line.split_whitespace().map(ToString::to_string).collect()
}

fn read_case_file(path: &Path) -> Vec<Vec<String>> {
    let text = fs::read_to_string(path).expect("case file must load");
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(split_argv)
        .collect()
}

#[test]
fn minimized_parser_cases_do_not_crash_and_are_deterministic() {
    let dir = Path::new("tests/fuzz/parser/parser_minimized_cases");
    let mut files = fs::read_dir(dir)
        .expect("minimized case directory must exist")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("argv"))
        .collect::<Vec<_>>();
    files.sort();

    assert!(!files.is_empty(), "minimized crash cases must be retained");

    for file in files {
        for argv in read_case_file(&file) {
            let first = parse_intent(&argv).expect("minimized case parse should not fail");
            let second = parse_intent(&argv).expect("second parse should not fail");
            assert_eq!(
                first.command_path, second.command_path,
                "command path drift for case {file:?}"
            );
            assert_eq!(
                first.normalized_path, second.normalized_path,
                "normalized path drift for case {file:?}"
            );
            assert_eq!(
                first.global_flags, second.global_flags,
                "global flags drift for case {file:?}"
            );
        }
    }
}

#[test]
fn interesting_corpus_cases_do_not_crash_or_corrupt_route_resolution() {
    let dir = Path::new("tests/fuzz/parser/parser_interesting_inputs");
    let mut files = fs::read_dir(dir)
        .expect("interesting corpus directory must exist")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("txt"))
        .collect::<Vec<_>>();
    files.sort();

    assert!(!files.is_empty(), "interesting parser corpus must be retained");

    let registry = RouteRegistry::default();
    for file in files {
        for argv in read_case_file(&file) {
            let parsed = parse_intent(&argv).expect("corpus case parse should not fail");
            if !parsed.normalized_path.is_empty() {
                let _ = registry.resolve(&parsed.normalized_path);
            }
        }
    }
}
