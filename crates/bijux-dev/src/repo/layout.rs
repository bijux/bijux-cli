use std::path::Path;

pub fn required_contract_files() -> &'static [&'static str] {
    &[
        "docs/06-specification/01-dag-model.md",
        "docs/06-specification/02-run-model.md",
        "docs/06-specification/03-artifact-model.md",
        "docs/06-specification/07-replay-semantics.md",
    ]
}

pub fn all_required_present(root: &Path) -> bool {
    required_contract_files().iter().all(|file| root.join(file).exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_contract_file_list_is_not_empty() {
        assert!(!required_contract_files().is_empty());
    }

    #[test]
    fn workspace_root_contains_required_contract_files() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        assert!(all_required_present(&root));
    }
}
