use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn dag_bin(cwd: &Path) -> Command {
    let cargo_bin = std::env::var("CARGO")
        .ok()
        .or_else(|| option_env!("CARGO").map(ToOwned::to_owned))
        .unwrap_or_else(|| "cargo".to_string());
    let mut command = Command::new(cargo_bin);
    command.current_dir(cwd).env("CARGO_TARGET_DIR", cwd.join("artifacts/target")).args([
        "run",
        "--quiet",
        "-p",
        "bijux-dag-cli",
        "--",
    ]);
    command
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root")
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/replay_bundles").join(name)
}

fn run_json(args: &[&str], cwd: &Path) -> (i32, Value) {
    let output = dag_bin(cwd).args(args).output().expect("run dag command");
    (
        output.status.code().unwrap_or(1),
        serde_json::from_slice(&output.stdout).expect("parse json output"),
    )
}

#[test]
#[allow(non_snake_case, reason = "nextest slow-tier namespace contract")]
fn slow__checked_in_replay_bundle_fixtures_import_with_expected_fidelity() {
    let root = repo_root();
    for (fixture, expected_level) in
        [("historic_manifest_only.json", "graded"), ("historic_with_files.json", "exact")]
    {
        let (code, payload) = run_json(
            &["--json", "import", "--verify-only", fixture_path(fixture).to_str().unwrap()],
            &root,
        );
        assert_eq!(code, 0, "fixture import failed for {fixture}");
        assert_eq!(payload["ok"], true);
        assert_eq!(payload["data"]["fidelity"]["level"], expected_level);
        assert_eq!(payload["data"]["bundle_version"], "export-bundle/v0.1");
    }
}

#[test]
fn replay_bundle_fixtures_are_checked_in_and_machine_readable() {
    for fixture in [
        "historic_manifest_only.json",
        "historic_with_files.json",
        "historic_unsupported_version.json",
    ] {
        let path = fixture_path(fixture);
        let raw = fs::read_to_string(&path).expect("read fixture bundle");
        let value: Value = serde_json::from_str(&raw).expect("parse fixture bundle");
        assert!(value.get("bundle_version").is_some(), "bundle_version missing for {fixture}");
    }
}

#[test]
#[allow(non_snake_case, reason = "nextest slow-tier namespace contract")]
fn slow__unsupported_historical_bundle_fixture_is_rejected() {
    let root = repo_root();
    let (code, payload) = run_json(
        &[
            "--json",
            "import",
            "--verify-only",
            fixture_path("historic_unsupported_version.json").to_str().unwrap(),
        ],
        &root,
    );
    assert_ne!(code, 0);
    assert_eq!(payload["ok"], false);
}
