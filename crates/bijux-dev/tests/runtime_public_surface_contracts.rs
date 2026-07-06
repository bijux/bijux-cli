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

fn runtime_root_use_decls() -> Vec<RootUseDecl> {
    let path = repo_root().join("crates/bijux-dag-runtime/src/lib.rs");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
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
            let source = rest
                .split_once("::{")
                .map_or_else(|| rest.trim_end_matches(';').to_string(), |(prefix, _)| prefix.to_string());
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
fn modeled_and_future_backend_exports_stay_hidden_from_runtime_root_docs() {
    let decls = runtime_root_use_decls();
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
        "task_contract",
        "task_types",
        "slurm_execution",
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
