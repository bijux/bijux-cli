use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::Path;

fn workspace_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root")
}

fn read_repo_file(path: &str) -> String {
    let absolute = workspace_root().join(path);
    fs::read_to_string(&absolute)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", absolute.display()))
}

#[test]
fn reproducibility_reference_covers_identity_layers_cache_and_replay_bundle_boundary() {
    let reference = read_repo_file("docs/bijux-dag/interfaces/reference/reproducibility-model.md");

    for token in [
        "# Reproducibility Model",
        "## Graph Fingerprint",
        "## Plan Fingerprint",
        "## Execution Fingerprint",
        "## Environment Fingerprint",
        "## Output Fingerprint",
        "## Cache Key",
        "## Cache Verification",
        "## Replay Bundle",
        "## Replay Limitations",
        "planner_fingerprint",
        "declared_environment_fingerprint",
        "outputs/index.json",
        "export-bundle/v0.1",
        "dag-diagnostics-bundle/v0.1",
        "with-files",
        "manifest-only",
        "without-artifacts",
        "why-cache-missed",
        "docs/spec/REPLAY_CONTRACT.md",
        "docs/spec/IMPORT_EXPORT_CONTRACT.md",
    ] {
        assert!(reference.contains(token), "reproducibility reference missing token: {token}");
    }
}

#[test]
fn handbook_package_pages_and_readmes_route_identity_questions_to_reproducibility_reference() {
    for path in [
        "README.md",
        "docs/bijux-dag/index.md",
        "docs/bijux-dag/interfaces/artifact-contracts.md",
        "docs/bijux-dag/interfaces/cli-surface.md",
        "docs/bijux-dag/interfaces/operator-workflows.md",
        "docs/bijux-dag/packages/index.md",
        "docs/bijux-dag/packages/bijux-dag-core.md",
        "docs/bijux-dag/packages/bijux-dag-artifacts.md",
        "docs/bijux-dag/packages/bijux-dag-runtime.md",
        "docs/bijux-dag/packages/bijux-dag-app.md",
        "docs/bijux-dag/packages/bijux-dag-cli.md",
        "crates/bijux-dag-core/README.md",
        "crates/bijux-dag-artifacts/README.md",
        "crates/bijux-dag-runtime/README.md",
        "crates/bijux-dag-app/README.md",
        "crates/bijux-dag-cli/README.md",
    ] {
        let content = read_repo_file(path);
        assert!(
            content.contains("reproducibility-model.md")
                || content.contains("Reproducibility Model")
                || content.contains("reproducibility-model/"),
            "{path} must route identity and replay-boundary questions to the reproducibility reference"
        );
    }
}

#[test]
fn replay_and_import_export_specs_distinguish_replay_bundles_from_diagnostics_bundles() {
    let replay = read_repo_file("docs/spec/REPLAY_CONTRACT.md");
    let import_export = read_repo_file("docs/spec/IMPORT_EXPORT_CONTRACT.md");
    let bundle_rulebook = read_repo_file("docs/spec/EXPORT_BUNDLE_EVOLUTION_RULEBOOK.md");

    for token in [
        "## Replay bundle boundary",
        "IMPORT_EXPORT_CONTRACT.md",
        "runs diagnostics-bundle",
        "artifact-bearing replay bundle",
        "not an importable replay contract",
    ] {
        assert!(replay.contains(token), "replay contract missing token: {token}");
    }

    for token in [
        "portable replay bundle mode",
        "with-files",
        "manifest-only",
        "without-artifacts",
        "diagnostics bundles are a separate operator-inspection surface",
    ] {
        assert!(import_export.contains(token), "import/export contract missing token: {token}");
    }

    for token in ["replay-bundle", "diagnostics bundle versions must evolve independently"] {
        assert!(bundle_rulebook.contains(token), "export bundle rulebook missing token: {token}");
    }
}
