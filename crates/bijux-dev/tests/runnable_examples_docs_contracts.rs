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
fn runnable_examples_catalog_covers_required_examples_with_expected_outputs() {
    let catalog = read_repo_file("docs/bijux-dag/interfaces/examples/index.md");

    for token in [
        "# Runnable Examples",
        "minimal hello DAG",
        "file-processing DAG",
        "cache demo",
        "failure demo",
        "replay demo",
        "branch demo",
        "container demo",
        "evidence/dag/authoring/examples/hello.dag.json",
        "evidence/dag/authoring/examples/file-processing-report.dag.json",
        "evidence/dag/authoring/examples/audience-branch-bulletin.dag.json",
        "evidence/dag/authoring/examples/release-note-bundle.dag.json",
        "Expected outputs:",
        "replay_proof",
        "why-cache-missed",
        "selected_lane: technical",
        "container-summary.json",
    ] {
        assert!(catalog.contains(token), "examples catalog missing token: {token}");
    }

    assert_eq!(
        catalog.matches("Expected outputs:").count(),
        7,
        "examples catalog must keep one expected-output section per required example"
    );
}

#[test]
fn public_dag_entry_surfaces_route_example_questions_to_the_catalog() {
    for path in [
        "README.md",
        "crates/bijux-dag-cli/README.md",
        "docs/bijux-dag/index.md",
        "docs/bijux-dag/interfaces/index.md",
        "docs/bijux-dag/interfaces/entrypoints-and-examples.md",
        "docs/bijux-dag/operations/common-workflows.md",
        "docs/bijux-dag/operations/index.md",
    ] {
        let content = read_repo_file(path);
        assert!(
            content.contains("interfaces/examples/index.md")
                || content.contains("Runnable Examples"),
            "{path} must route example questions to the runnable examples catalog"
        );
    }
}
