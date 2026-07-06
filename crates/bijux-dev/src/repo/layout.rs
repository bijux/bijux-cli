use std::path::Path;

pub fn required_contract_files() -> &'static [&'static str] {
    &[
        "contracts/schemas/output-envelope-v1.schema.json",
        "contracts/schemas/error-envelope-v1.schema.json",
        "contracts/schemas/plugin-manifest-v2.schema.json",
        "evidence/dag/CONTRACT.md",
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
