use std::fs;
use std::path::PathBuf;
use std::process::Command;

const DAG_PRODUCT_SENTENCE: &str = "bijux-dag v0.4.0 is a local-first DAG runtime for reproducible workflows with explicit graph contracts, deterministic execution records, verified artifacts, cache explanation, and replayable run bundles.";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn normalize_whitespace(text: &str) -> String {
    text.replace('`', "").split_whitespace().collect::<Vec<_>>().join(" ")
}

fn read_repo_file(path: &str) -> String {
    let absolute = repo_root().join(path);
    fs::read_to_string(&absolute)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", absolute.display()))
}

fn assert_sentence_in_file(path: &str) {
    let normalized = normalize_whitespace(&read_repo_file(path));
    assert!(
        normalized.contains(DAG_PRODUCT_SENTENCE),
        "{path} must contain the canonical dag product sentence"
    );
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

#[test]
fn public_dag_readmes_and_handbook_entrypoints_share_the_same_product_sentence() {
    for path in [
        "README.md",
        "crates/bijux-dag-core/README.md",
        "crates/bijux-dag-artifacts/README.md",
        "crates/bijux-dag-runtime/README.md",
        "crates/bijux-dag-app/README.md",
        "crates/bijux-dag-cli/README.md",
        "docs/bijux-dag/index.md",
        "docs/bijux-dag/interfaces/entrypoints-and-examples.md",
        "docs/bijux-dag/operations/index.md",
    ] {
        assert_sentence_in_file(path);
    }
}

#[test]
fn dag_package_pages_keep_the_product_sentence_and_owned_boundary() {
    for path in [
        "docs/bijux-dag/packages/index.md",
        "docs/bijux-dag/packages/bijux-dag-core.md",
        "docs/bijux-dag/packages/bijux-dag-artifacts.md",
        "docs/bijux-dag/packages/bijux-dag-runtime.md",
        "docs/bijux-dag/packages/bijux-dag-app.md",
        "docs/bijux-dag/packages/bijux-dag-cli.md",
    ] {
        assert_sentence_in_file(path);
    }
}

#[test]
fn dag_product_sentence_stays_backed_by_explicit_proof_maps() {
    let handbook = read_repo_file("docs/bijux-dag/index.md");
    assert!(handbook.contains("## Product Proof Map"));
    assert!(handbook.contains("explicit graph contracts"));
    assert!(handbook.contains("deterministic execution records"));
    assert!(handbook.contains("verified artifacts"));
    assert!(handbook.contains("cache explanation"));
    assert!(handbook.contains("replayable run bundles"));

    let entrypoints = read_repo_file("docs/bijux-dag/interfaces/entrypoints-and-examples.md");
    assert!(entrypoints.contains("## Proof Map"));
    assert!(entrypoints.contains("retained node traces under `artifacts/`"));
    assert!(entrypoints.contains("the cache-behavior workflow"));
    assert!(entrypoints.contains("the reproducibility model for replay identity"));

    let first_hour = read_repo_file("docs/bijux-dag/operations/first-hour-with-bijux-dag.md");
    assert!(first_hour.contains("That first hour proves the product sentence in order:"));

    let first_run = read_repo_file("docs/bijux-dag/operations/first-run-tutorial.md");
    assert!(first_run.contains("shortest proof path for the `bijux-dag` product promise"));
}

#[test]
fn dag_root_help_uses_the_same_product_sentence() {
    let normalized = normalize_whitespace(&dag_root_help());
    assert!(
        normalized.contains(DAG_PRODUCT_SENTENCE),
        "bijux-dag --help must contain the canonical dag product sentence"
    );
}
