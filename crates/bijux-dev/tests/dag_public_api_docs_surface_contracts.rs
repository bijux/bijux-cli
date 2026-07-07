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
fn runtime_modeled_and_future_exports_stay_hidden_from_root_docs() {
    let decls = root_use_decls("crates/bijux-dag-runtime/src/lib.rs");
    let expected_hidden = [
        "backend::fake",
        "backend_cluster",
        "batch_execution",
        "extension_catalog",
        "formal_verification",
        "kubernetes_execution",
        "observability_deep",
        "recovery",
        "remote_execution_model",
        "remote_executor",
        "slurm_execution",
        "task_contract",
        "task_types",
        "upgrade_compatibility",
    ];

    for source in expected_hidden {
        let decl = decls
            .iter()
            .find(|decl| decl.source == source)
            .unwrap_or_else(|| panic!("expected runtime root export group `{source}`"));
        assert!(
            decl.doc_hidden,
            "runtime root export group `{source}` must stay hidden from the visible docs surface"
        );
    }
}
