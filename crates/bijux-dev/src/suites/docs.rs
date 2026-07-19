pub const IDS: &[&str] = &["handbook-indexes", "guarantee-evidence"];

fn is_guarantee_claim(line: &str) -> bool {
    let trimmed = line.trim();
    let lower = trimmed.to_ascii_lowercase();
    if !lower.contains("guarantee") {
        return false;
    }

    let is_heading = trimmed.starts_with('#');
    is_heading
        || lower.contains("these guarantees")
        || lower.contains("must preserve")
        || lower.contains("complete guarantee")
        || lower.contains("complete `v")
}

fn has_proof_reference(markdown: &str) -> bool {
    [
        "docs/spec/",
        "../spec/",
        "../../spec/",
        "/tests/",
        "`tests/",
        "contracts/",
        "benchmarks/",
        "artifacts/benchmarks/",
        "artifacts/memory/",
    ]
    .iter()
    .any(|marker| markdown.contains(marker))
}

pub fn guarantee_claims_without_evidence(markdown: &str) -> Vec<usize> {
    let claims = markdown
        .lines()
        .enumerate()
        .filter_map(|(index, line)| is_guarantee_claim(line).then_some(index + 1))
        .collect::<Vec<_>>();

    if claims.is_empty() || has_proof_reference(markdown) {
        Vec::new()
    } else {
        claims
    }
}

#[cfg(test)]
mod tests {
    use super::guarantee_claims_without_evidence;

    #[test]
    fn negative_and_future_boundary_language_is_not_a_guarantee_claim() {
        let markdown = "\
Route recognition does not guarantee local availability.

Removal requires rollback guarantees.
";
        assert!(guarantee_claims_without_evidence(markdown).is_empty());
    }

    #[test]
    fn guarantee_sections_require_document_evidence() {
        let markdown = "\
## Execution Guarantees

- one final exit code
";
        assert_eq!(guarantee_claims_without_evidence(markdown), vec![1]);
    }

    #[test]
    fn executable_contracts_can_support_an_earlier_guarantee_section() {
        let markdown = "\
## Execution Guarantees

- one final exit code

## Executable Contracts

- `crates/bijux-cli/tests/integration.rs`
";
        assert!(guarantee_claims_without_evidence(markdown).is_empty());
    }

    #[test]
    fn strong_inline_guarantees_require_document_evidence() {
        let markdown = "Source-run write protection is the complete guarantee.\n";
        assert_eq!(guarantee_claims_without_evidence(markdown), vec![1]);
    }
}
