use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn collect_markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_markdown_files_inner(root, &mut files);
    files.sort();
    files
}

fn collect_markdown_files_inner(root: &Path, files: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(root).unwrap_or_else(|err| {
        panic!("failed to read markdown directory {}: {err}", root.display())
    });
    for entry in entries {
        let entry = entry.expect("markdown directory entry");
        let path = entry.path();
        if entry.file_type().expect("markdown file type").is_dir() {
            collect_markdown_files_inner(&path, files);
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "md") {
            files.push(path);
        }
    }
}

fn extract_inline_code_references(markdown: &str) -> Vec<(usize, String)> {
    let mut references = Vec::new();
    let mut in_fenced_block = false;

    for (line_idx, line) in markdown.lines().enumerate() {
        if line.trim_start().starts_with("```") {
            in_fenced_block = !in_fenced_block;
            continue;
        }
        if in_fenced_block {
            continue;
        }

        let mut cursor = 0;
        while let Some(open) = line[cursor..].find('`') {
            let start = cursor + open + 1;
            let Some(close) = line[start..].find('`') else {
                break;
            };
            let end = start + close;
            let candidate = line[start..end].trim();
            if !candidate.is_empty() {
                references.push((line_idx + 1, candidate.to_string()));
            }
            cursor = end + 1;
        }
    }

    references
}

fn extract_markdown_links(markdown: &str) -> Vec<(usize, String)> {
    let mut links = Vec::new();
    let mut in_fenced_block = false;

    for (line_idx, line) in markdown.lines().enumerate() {
        if line.trim_start().starts_with("```") {
            in_fenced_block = !in_fenced_block;
            continue;
        }
        if in_fenced_block {
            continue;
        }

        let mut cursor = 0;
        while let Some(open) = line[cursor..].find('[') {
            let label_start = cursor + open;
            if label_start > 0 && line.as_bytes()[label_start - 1] == b'!' {
                cursor = label_start + 1;
                continue;
            }

            let Some(close_rel) = line[label_start + 1..].find(']') else {
                break;
            };
            let label_end = label_start + 1 + close_rel;
            if !line[label_end + 1..].starts_with('(') {
                cursor = label_end + 1;
                continue;
            }

            let target_start = label_end + 2;
            let Some(target_end_rel) = line[target_start..].find(')') else {
                break;
            };
            let target_end = target_start + target_end_rel;
            let candidate = line[target_start..target_end].trim();
            if !candidate.is_empty() {
                links.push((line_idx + 1, candidate.to_string()));
            }
            cursor = target_end + 1;
        }
    }

    links
}

fn extract_document_anchors(markdown: &str) -> BTreeSet<String> {
    let mut anchors = BTreeSet::new();
    let mut in_fenced_block = false;

    for line in markdown.lines() {
        if line.trim_start().starts_with("```") {
            in_fenced_block = !in_fenced_block;
            continue;
        }
        if in_fenced_block {
            continue;
        }

        if let Some(title) = line
            .trim_start()
            .strip_prefix('#')
            .map(str::trim_start)
            .filter(|title| !title.is_empty())
        {
            anchors.insert(github_anchor_slug(title));
        }
        anchors.extend(extract_html_anchor_ids(line));
    }

    anchors
}

fn extract_html_anchor_ids(line: &str) -> Vec<String> {
    let mut anchors = Vec::new();
    let mut remainder = line;

    while let Some(anchor_start) = remainder.find("<a") {
        remainder = &remainder[anchor_start + 2..];
        for attribute in ["id=\"", "id='", "name=\"", "name='"] {
            if let Some(value_start) = remainder.find(attribute) {
                let content = &remainder[value_start + attribute.len()..];
                let terminator = if attribute.ends_with('"') { '"' } else { '\'' };
                if let Some(value_end) = content.find(terminator) {
                    let candidate = content[..value_end].trim();
                    if !candidate.is_empty() {
                        anchors.push(candidate.to_string());
                    }
                }
            }
        }
    }

    anchors
}

fn github_anchor_slug(title: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;

    for ch in title.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            slug.push(ch);
            last_was_dash = false;
        } else if ch.is_whitespace() && !last_was_dash && !slug.is_empty() {
            slug.push('-');
            last_was_dash = true;
        }
    }

    slug.trim_matches('-').to_string()
}

fn looks_like_path_reference(candidate: &str) -> bool {
    if candidate.contains(' ') || candidate.contains("://") || candidate.starts_with('#') {
        return false;
    }
    if candidate.starts_with("artifacts/")
        || candidate.starts_with("./artifacts/")
        || candidate.starts_with("../artifacts/")
    {
        return false;
    }

    let exact_root_files = [
        "Cargo.toml",
        "Makefile",
        "README.md",
        "mkdocs.yml",
        "mkdocs.shared.yml",
        "PROJECT_TREE.md",
        "TOOLING.md",
        "simulated_platform.rs",
    ];
    if exact_root_files.contains(&candidate) {
        return true;
    }

    let path_prefixes = [
        ".github/",
        "analysis/",
        "adapters/",
        "backend/",
        "build/",
        "builtins/",
        "configs/",
        "contracts/",
        "crates/",
        "diagnostics/",
        "docs/",
        "error/",
        "graph/",
        "internal/",
        "makes/",
        "pipeline/",
        "planner/",
        "src/",
        "templates/",
        "tools/",
    ];
    path_prefixes.iter().any(|prefix| candidate.starts_with(prefix))
}

fn resolution_roots(doc: &Path, repo_root: &Path) -> Vec<PathBuf> {
    let mut roots =
        vec![doc.parent().expect("markdown parent").to_path_buf(), repo_root.to_path_buf()];
    let repo_relative = doc.strip_prefix(repo_root).expect("repo-relative doc path");

    if repo_relative.starts_with("docs/bijux-cli") {
        roots.push(repo_root.join("crates/bijux-cli"));
    }
    if repo_relative == Path::new("docs/reports/foundation/RUNTIME_NON_KERNEL_MODULES_REPORT.md") {
        roots.push(repo_root.join("crates/bijux-dag-runtime/src"));
    }
    if repo_relative.starts_with("docs/bijux-dev") {
        roots.push(repo_root.join("crates/bijux-dev"));
    }

    roots
}

fn normalize_markdown_target(target: &str) -> &str {
    target.trim().trim_start_matches('<').trim_end_matches('>')
}

fn reference_resolves(doc: &Path, repo_root: &Path, reference: &str) -> bool {
    let path_text =
        reference.split_once('#').map_or(reference, |(path, _)| path).trim_end_matches('/');
    if path_text.is_empty()
        || path_text.contains('*')
        || path_text.contains('{')
        || path_text.contains('}')
        || path_text.contains('?')
        || path_text.contains("...")
    {
        return false;
    }

    resolution_roots(doc, repo_root).into_iter().any(|root| root.join(path_text).exists())
}

fn markdown_target_resolves(doc: &Path, repo_root: &Path, target: &str) -> Result<(), String> {
    let canonical_root = repo_root
        .canonicalize()
        .unwrap_or_else(|err| panic!("failed to canonicalize {}: {err}", repo_root.display()));
    let normalized = normalize_markdown_target(target);
    if normalized.is_empty()
        || normalized.starts_with("http://")
        || normalized.starts_with("https://")
        || normalized.starts_with("mailto:")
    {
        return Ok(());
    }

    let (path_text, anchor) = normalized
        .split_once('#')
        .map_or((normalized, None), |(path, anchor)| (path, Some(anchor)));

    let destination = if path_text.is_empty() {
        doc.to_path_buf()
    } else {
        doc.parent()
            .expect("markdown parent")
            .join(path_text)
            .canonicalize()
            .map_err(|_| format!("missing target {normalized}"))?
    };

    if !destination.starts_with(&canonical_root) {
        return Err(format!("target escapes repository: {normalized}"));
    }
    if !destination.exists() {
        return Err(format!("missing target {normalized}"));
    }

    if let Some(anchor_name) = anchor {
        if destination.extension().is_some_and(|ext| ext == "md") {
            let anchors =
                extract_document_anchors(&fs::read_to_string(&destination).unwrap_or_else(|err| {
                    panic!("failed to read {}: {err}", destination.display())
                }));
            if !anchors.contains(anchor_name) {
                return Err(format!("missing anchor {normalized}"));
            }
        }
    }

    Ok(())
}

fn assert_source_references_resolve(
    markdown_files: impl IntoIterator<Item = PathBuf>,
    label: &str,
) {
    let root = repo_root();
    let mut failures = Vec::new();

    for doc in markdown_files {
        let text = fs::read_to_string(&doc)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", doc.display()));
        for (line, reference) in extract_inline_code_references(&text) {
            if !looks_like_path_reference(&reference) {
                continue;
            }
            if !reference_resolves(&doc, &root, &reference) {
                let rel = doc.strip_prefix(&root).expect("repo-relative doc path");
                failures.push(format!("{}:{} `{}`", rel.display(), line, reference));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{}",
        format!("{label} contain stale source references:\n{}", failures.join("\n"))
    );
}

fn assert_markdown_links_resolve(markdown_files: impl IntoIterator<Item = PathBuf>, label: &str) {
    let root = repo_root();
    let mut failures = Vec::new();

    for doc in markdown_files {
        let text = fs::read_to_string(&doc)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", doc.display()));
        for (line, target) in extract_markdown_links(&text) {
            if let Err(reason) = markdown_target_resolves(&doc, &root, &target) {
                let rel = doc.strip_prefix(&root).expect("repo-relative doc path");
                failures.push(format!("{}:{} {}", rel.display(), line, reason));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{}",
        format!("{label} contain broken markdown links:\n{}", failures.join("\n"))
    );
}

fn source_reference_markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut markdown_files = collect_markdown_files(&root.join("docs/spec"));
    markdown_files.extend(collect_markdown_files(&root.join("docs/reports")));
    markdown_files.push(root.join("README.md"));
    markdown_files.push(root.join("docs/index.md"));
    markdown_files.extend([
        root.join("crates/bijux-cli/README.md"),
        root.join("crates/bijux-cli-python/README.md"),
        root.join("crates/bijux-dag-app/README.md"),
        root.join("crates/bijux-dag-artifacts/README.md"),
        root.join("crates/bijux-dag-cli/README.md"),
        root.join("crates/bijux-dag-core/README.md"),
        root.join("crates/bijux-dag-runtime/README.md"),
        root.join("crates/bijux-dag-testkit/README.md"),
        root.join("crates/bijux-dev/README.md"),
        root.join("crates/bijux-dag-app/CONTRACT.md"),
        root.join("crates/bijux-dag-artifacts/CONTRACT.md"),
        root.join("crates/bijux-dag-cli/CONTRACT.md"),
        root.join("crates/bijux-dag-core/CONTRACT.md"),
        root.join("crates/bijux-dag-runtime/CONTRACT.md"),
        root.join("crates/bijux-dag-testkit/CONTRACT.md"),
        root.join("crates/bijux-cli/CHANGELOG.md"),
        root.join("crates/bijux-cli-python/CHANGELOG.md"),
    ]);
    markdown_files.extend(collect_markdown_files(&root.join("docs/bijux-core")));
    markdown_files.extend(collect_markdown_files(&root.join("docs/bijux-cli")));
    markdown_files.extend(collect_markdown_files(&root.join("docs/bijux-dag")));
    markdown_files.extend(collect_markdown_files(&root.join("docs/bijux-dev")));
    markdown_files.sort();
    markdown_files.dedup();
    markdown_files
}

#[test]
fn workspace_handbook_source_references_resolve() {
    let root = repo_root();
    assert_source_references_resolve(
        collect_markdown_files(&root.join("docs/bijux-core")),
        "workspace handbook pages",
    );
}

#[test]
fn cli_handbook_source_references_resolve() {
    let root = repo_root();
    assert_source_references_resolve(
        collect_markdown_files(&root.join("docs/bijux-cli")),
        "CLI handbook pages",
    );
}

#[test]
fn dag_handbook_source_references_resolve() {
    let root = repo_root();
    assert_source_references_resolve(
        collect_markdown_files(&root.join("docs/bijux-dag")),
        "DAG handbook pages",
    );
}

#[test]
fn maintainer_handbook_source_references_resolve() {
    let root = repo_root();
    assert_source_references_resolve(
        collect_markdown_files(&root.join("docs/bijux-dev")),
        "maintainer handbook pages",
    );
}

#[test]
fn repository_specs_and_reports_source_references_resolve() {
    let root = repo_root();
    let mut markdown_files = collect_markdown_files(&root.join("docs/spec"));
    markdown_files.extend(collect_markdown_files(&root.join("docs/reports")));
    markdown_files.push(root.join("README.md"));
    markdown_files.push(root.join("docs/index.md"));
    assert_source_references_resolve(markdown_files, "repository specification and report pages");
}

#[test]
fn package_readme_source_references_resolve() {
    let root = repo_root();
    let markdown_files = [
        root.join("crates/bijux-cli/README.md"),
        root.join("crates/bijux-cli-python/README.md"),
        root.join("crates/bijux-dag-app/README.md"),
        root.join("crates/bijux-dag-artifacts/README.md"),
        root.join("crates/bijux-dag-cli/README.md"),
        root.join("crates/bijux-dag-core/README.md"),
        root.join("crates/bijux-dag-runtime/README.md"),
        root.join("crates/bijux-dag-testkit/README.md"),
        root.join("crates/bijux-dev/README.md"),
    ];
    assert_source_references_resolve(markdown_files, "package readme pages");
}

#[test]
fn dag_crate_contract_source_references_resolve() {
    let root = repo_root();
    let markdown_files = [
        root.join("crates/bijux-dag-app/CONTRACT.md"),
        root.join("crates/bijux-dag-artifacts/CONTRACT.md"),
        root.join("crates/bijux-dag-cli/CONTRACT.md"),
        root.join("crates/bijux-dag-core/CONTRACT.md"),
        root.join("crates/bijux-dag-runtime/CONTRACT.md"),
        root.join("crates/bijux-dag-testkit/CONTRACT.md"),
    ];
    assert_source_references_resolve(markdown_files, "DAG crate contract pages");
}

#[test]
fn package_changelog_source_references_resolve() {
    let root = repo_root();
    let markdown_files = [
        root.join("crates/bijux-cli/CHANGELOG.md"),
        root.join("crates/bijux-cli-python/CHANGELOG.md"),
    ];
    assert_source_references_resolve(markdown_files, "package changelog pages");
}

#[test]
fn markdown_links_and_anchors_resolve() {
    let root = repo_root();
    assert_markdown_links_resolve(
        source_reference_markdown_files(&root),
        "repository markdown pages",
    );
}
