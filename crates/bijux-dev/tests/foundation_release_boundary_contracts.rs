use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct DagReleaseTruthTable {
    schema_version: String,
    release: String,
    owner: String,
    stable_operator_surface: SurfaceEntry,
    experimental_operator_surface: SurfaceEntry,
    simulated_surface: SurfaceEntry,
    internal_surface: SurfaceEntry,
    future_surface: FutureSurfaceEntry,
}

#[derive(Debug, Deserialize)]
struct SurfaceEntry {
    summary: String,
    root_commands: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct FutureSurfaceEntry {
    summary: String,
    capabilities: Vec<String>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_truth_table() -> DagReleaseTruthTable {
    let path = repo_root().join("contracts/foundation/dag_release_truth_table.v1.json");
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|err| panic!("invalid json {}: {err}", path.display()))
}

fn read_repo_file(path: &str) -> String {
    let absolute = repo_root().join(path);
    fs::read_to_string(&absolute)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", absolute.display()))
}

fn dag_root_help() -> String {
    let output = Command::new("cargo")
        .args(["run", "-q", "-p", "bijux-dag-cli", "--bin", "bijux-dag", "--", "--help"])
        .current_dir(repo_root())
        .output()
        .expect("run bijux-dag --help");
    assert!(output.status.success(), "bijux-dag --help failed");
    String::from_utf8(output.stdout).expect("help output must be utf8")
}

fn parse_root_help_commands(help: &str) -> BTreeSet<String> {
    let mut commands = BTreeSet::new();
    let mut in_commands = false;

    for line in help.lines() {
        if line == "Commands:" {
            in_commands = true;
            continue;
        }
        if !in_commands {
            continue;
        }
        if line == "Options:" {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let command = trimmed.split_whitespace().next().expect("command token").to_string();
        commands.insert(command);
    }

    commands
}

fn assert_contains_all(text: &str, needles: &[&str], context: &str) {
    for needle in needles {
        assert!(text.contains(needle), "{context} must contain `{needle}`");
    }
}

#[test]
fn dag_release_truth_table_contract_is_current() {
    let truth_table = read_truth_table();
    assert_eq!(truth_table.schema_version, "foundation-dag-release-truth-table/v1");
    assert_eq!(truth_table.release, "v0.4.0");
    assert_eq!(truth_table.owner, "bijux-dag");
    assert!(!truth_table.stable_operator_surface.summary.trim().is_empty());
    assert!(!truth_table.experimental_operator_surface.summary.trim().is_empty());
    assert!(!truth_table.simulated_surface.summary.trim().is_empty());
    assert!(!truth_table.internal_surface.summary.trim().is_empty());
    assert!(!truth_table.future_surface.summary.trim().is_empty());
    assert!(!truth_table.future_surface.capabilities.is_empty());
}

#[test]
fn dag_root_help_matches_stable_release_boundary() {
    let truth_table = read_truth_table();
    let help = dag_root_help();
    let commands = parse_root_help_commands(&help);

    let expected = truth_table
        .stable_operator_surface
        .root_commands
        .into_iter()
        .chain(std::iter::once("help".to_string()))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        commands, expected,
        "visible bijux-dag --help command inventory drifted from the stable release boundary"
    );

    assert_contains_all(
        &help,
        &[
            "v0.4.0 surface truth table:",
            "stable: validate, plan, run, replay, runs ..., artifact, artifact-inspect, diff, explain, verify, doctor, cache, version, commands",
            "Use `bijux-dag commands --all` to inventory repository-owned non-stable routes.",
        ],
        "bijux-dag --help",
    );
}

#[test]
fn dag_release_boundary_docs_and_examples_stay_honest() {
    let readme = read_repo_file("README.md");
    let handbook = read_repo_file("docs/bijux-dag/index.md");
    let cli_surface = read_repo_file("docs/bijux-dag/interfaces/cli-surface.md");
    let release_boundary = read_repo_file("docs/bijux-dag/foundation/release-boundary.md");

    assert_contains_all(
        &readme,
        &[
            "### `bijux-dag` v0.4.0 Surface Truth Table",
            "contracts/foundation/dag_release_truth_table.v1.json",
        ],
        "README.md",
    );
    assert_contains_all(
        &handbook,
        &["## v0.4.0 Surface Truth Table", "[Release Boundary](foundation/release-boundary.md)"],
        "docs/bijux-dag/index.md",
    );
    assert_contains_all(
        &cli_surface,
        &["## v0.4.0 Surface Truth Table", "../foundation/release-boundary.md"],
        "docs/bijux-dag/interfaces/cli-surface.md",
    );
    assert_contains_all(
        &release_boundary,
        &["| stable |", "| experimental |", "| simulated |", "| internal |", "| future |"],
        "docs/bijux-dag/foundation/release-boundary.md",
    );

    for path in [
        "docs/bijux-dag/interfaces/entrypoints-and-examples.md",
        "docs/bijux-dag/interfaces/operator-workflows.md",
        "docs/bijux-dag/operations/installation-and-setup.md",
        "docs/bijux-dag/operations/common-workflows.md",
        "docs/bijux-dag/operations/failure-recovery.md",
    ] {
        let content = read_repo_file(path);
        assert!(
            !content.contains("bijux-dag status"),
            "{path} must not advertise hidden experimental `status` as a stable example"
        );
        assert!(
            !content.contains("bijux-dag inspect"),
            "{path} must not advertise a nonexistent root `inspect` command"
        );
    }

    let recipes = read_repo_file("docs/bijux-dag/interfaces/executable-recipes.md");
    assert_contains_all(
        &recipes,
        &["experimental explicit-path routes", "bijux-dag explain --json ${RUN_DIR}"],
        "docs/bijux-dag/interfaces/executable-recipes.md",
    );

    let release_binary = read_repo_file("docs/spec/RELEASE_BINARY_VERIFICATION.md");
    assert_contains_all(
        &release_binary,
        &["internal probe (`capabilities`)", "bijux-dag explain --json ${RUN_DIR}"],
        "docs/spec/RELEASE_BINARY_VERIFICATION.md",
    );
}
