use std::path::Path;

pub fn required_contract_files() -> &'static [&'static str] {
    &[
        "docs/spec/WORKSPACE_CONTRACT.md",
        "docs/spec/BOUNDARY_RULES.md",
        "docs/spec/CRATE_OWNERSHIP.md",
        "docs/spec/EVIDENCE_MODEL.md",
    ]
}

pub fn all_required_present(root: &Path) -> bool {
    required_contract_files()
        .iter()
        .all(|file| root.join(file).exists())
}
