use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[derive(Debug)]
struct RootUseDecl {
    source: String,
    doc_hidden: bool,
}

fn read_crate_lib(relative_path: &str) -> String {
    let path = repo_root().join(relative_path);
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn root_use_decls(relative_path: &str) -> Vec<RootUseDecl> {
    let source = read_crate_lib(relative_path);
    let mut decls = Vec::new();
    let mut depth = 0usize;
    let mut next_doc_hidden = false;

    for line in source.lines() {
        let trimmed = line.trim_start();
        if depth == 0 && trimmed == "#[doc(hidden)]" {
            next_doc_hidden = true;
            continue;
        }
        if depth == 0 && trimmed.starts_with("pub use ") {
            let rest = trimmed.trim_start_matches("pub use ");
            let source = rest.split_once("::{").map_or_else(
                || rest.trim_end_matches(';').to_string(),
                |(prefix, _)| prefix.to_string(),
            );
            decls.push(RootUseDecl { source, doc_hidden: next_doc_hidden });
            next_doc_hidden = false;
        } else if depth == 0 && !trimmed.is_empty() && !trimmed.starts_with("//") {
            next_doc_hidden = false;
        }

        let opens = line.matches('{').count();
        let closes = line.matches('}').count();
        depth = depth.saturating_add(opens).saturating_sub(closes);
    }

    decls
}

fn assert_all_root_use_decls_hidden(crate_name: &str, relative_path: &str) {
    let decls = root_use_decls(relative_path);
    let visible: Vec<&str> =
        decls.iter().filter(|decl| !decl.doc_hidden).map(|decl| decl.source.as_str()).collect();

    assert!(
        visible.is_empty(),
        "{crate_name} must hide broad crate-root re-export groups from the primary docs lane; still visible: {}",
        visible.join(", ")
    );
}

#[test]
fn public_dag_crates_expose_curated_docs_lanes() {
    let crates = [
        ("bijux-dag-core", "crates/bijux-dag-core/src/lib.rs"),
        ("bijux-dag-artifacts", "crates/bijux-dag-artifacts/src/lib.rs"),
        ("bijux-dag-runtime", "crates/bijux-dag-runtime/src/lib.rs"),
        ("bijux-dag-app", "crates/bijux-dag-app/src/lib.rs"),
    ];

    for (crate_name, relative_path) in crates {
        let source = read_crate_lib(relative_path);
        assert!(
            source.contains("pub mod stable {"),
            "{crate_name} must expose a visible stable docs lane"
        );
        assert!(
            source.contains("pub mod prelude {"),
            "{crate_name} must expose a visible prelude docs lane"
        );
        assert!(
            source.contains("#[cfg(feature = \"experimental-public-api\")]")
                && source.contains("pub mod experimental {"),
            "{crate_name} must gate experimental docs lanes behind the experimental-public-api feature"
        );
    }
}

#[test]
fn core_root_reexports_stay_hidden_from_primary_docs_lane() {
    assert_all_root_use_decls_hidden("bijux-dag-core", "crates/bijux-dag-core/src/lib.rs");
}

#[test]
fn artifacts_root_reexports_stay_hidden_from_primary_docs_lane() {
    assert_all_root_use_decls_hidden(
        "bijux-dag-artifacts",
        "crates/bijux-dag-artifacts/src/lib.rs",
    );
}

#[test]
fn runtime_root_reexports_stay_hidden_from_primary_docs_lane() {
    assert_all_root_use_decls_hidden("bijux-dag-runtime", "crates/bijux-dag-runtime/src/lib.rs");
}
