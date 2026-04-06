#![forbid(unsafe_code)]
//! Replay minimized config-corruption campaign cases.

use std::fs;
use std::path::Path;
use std::process::Command;

use bijux_cli as _;
use libc as _;
use serde_json as _;
use shlex as _;
use thiserror as _;

fn run_case(case_file: &Path) {
    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(case_file).expect("read case"))
            .expect("parse case json");

    let args: Vec<String> = json["args"]
        .as_array()
        .expect("args array")
        .iter()
        .map(|v| v.as_str().expect("arg str").to_owned())
        .collect();

    let temp = std::env::temp_dir().join(format!(
        "bijux-config-campaign-repro-{}-{}",
        case_file.file_stem().and_then(|s| s.to_str()).unwrap_or("case"),
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp).expect("mkdir temp");

    let config = temp.join("active.env");
    let source = temp.join("source.env");
    let export = temp.join("export.env");
    fs::write(&config, json["config_text"].as_str().unwrap_or_default()).expect("write config");
    fs::write(&source, json["source_text"].as_str().unwrap_or_default()).expect("write source");

    let mut expanded = Vec::<String>::new();
    for arg in args {
        match arg.as_str() {
            "<CONFIG_PATH>" => expanded.push(config.display().to_string()),
            "<SOURCE_PATH>" => expanded.push(source.display().to_string()),
            "<EXPORT_PATH>" => expanded.push(export.display().to_string()),
            _ => expanded.push(arg),
        }
    }

    let run_once =
        || Command::new(env!("CARGO_BIN_EXE_bijux")).args(&expanded).output().expect("run case");

    let first = run_once();
    let second = run_once();

    for out in [&first, &second] {
        assert!(
            matches!(out.status.code(), Some(0) | Some(1) | Some(2)),
            "case {} crashed with status {:?}",
            case_file.display(),
            out.status.code()
        );
        if out.status.success() {
            assert!(
                out.stderr.is_empty(),
                "successful case must keep stderr empty: {}",
                case_file.display()
            );
            assert!(
                !out.stdout.is_empty(),
                "successful case must emit stdout: {}",
                case_file.display()
            );
        } else {
            assert!(
                out.stdout.is_empty(),
                "failed case must keep stdout empty: {}",
                case_file.display()
            );
            assert!(
                !out.stderr.is_empty(),
                "failed case must emit stderr: {}",
                case_file.display()
            );
        }
    }

    assert_eq!(
        first.status.code(),
        second.status.code(),
        "exit code drift in {}",
        case_file.display()
    );
    assert_eq!(first.stdout, second.stdout, "stdout drift in {}", case_file.display());
    assert_eq!(first.stderr, second.stderr, "stderr drift in {}", case_file.display());
}

#[test]
fn minimized_config_corruption_campaign_cases_replay_without_crashing() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fuzz/minimized_cases/config_corruption_minimized_cases");
    let mut files: Vec<_> = fs::read_dir(dir)
        .expect("read case directory")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "must retain at least one minimized campaign case");

    for case_file in files {
        run_case(&case_file);
    }
}
