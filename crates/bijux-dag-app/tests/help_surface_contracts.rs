use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_app::dag_command;
use std::path::Path;

#[test]
fn root_help_mentions_shipped_operator_surfaces() {
    let mut help = Vec::new();
    dag_command().write_long_help(&mut help).expect("help");
    let rendered = String::from_utf8(help).expect("utf8");
    for token in [
        "validate",
        "plan",
        "run",
        "replay",
        "runs",
        "diff",
        "prove",
        "verify",
        "export",
        "import",
        "capabilities",
    ] {
        assert!(rendered.contains(token), "missing shipped command token {token}");
    }
}

#[test]
fn root_help_does_not_claim_modeled_platform_features() {
    let mut help = Vec::new();
    dag_command().write_long_help(&mut help).expect("help");
    let rendered = String::from_utf8(help).expect("utf8").to_lowercase();
    for forbidden in [
        "control-plane api",
        "federated scheduler",
        "geo federation",
        "tenant control plane",
        "ai operator autopilot",
    ] {
        assert!(
            !rendered.contains(forbidden),
            "help should not claim modeled-only surface: {forbidden}"
        );
    }
}

#[test]
fn help_does_not_list_modeled_only_runtime_modules() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root");
    let modeled = std::fs::read_to_string(
        root.join("docs/reports/foundation/runtime_modeled_only_surfaces.md"),
    )
    .expect("read modeled surfaces report")
    .to_lowercase();
    let mut help = Vec::new();
    dag_command().write_long_help(&mut help).expect("help");
    let rendered = String::from_utf8(help).expect("utf8").to_lowercase();
    for line in modeled.lines() {
        if !line.trim_start().starts_with("- `") {
            continue;
        }
        let token = line
            .trim()
            .trim_start_matches("- `")
            .trim_end_matches('`')
            .rsplit('/')
            .next()
            .unwrap_or("")
            .trim_end_matches(".rs")
            .replace('_', " ");
        if token.is_empty() {
            continue;
        }
        assert!(
            !rendered.contains(&token),
            "help should not present modeled-only surface token: {token}"
        );
    }
}
