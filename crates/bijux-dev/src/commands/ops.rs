use super::*;

pub(super) fn run_status(cmd: &str, args: &[&str]) -> Result<(), String> {
    exec_run_status_in_dir(&repo_root()?, cmd, args)
}

pub(super) fn run_status_in_dir(dir: &Path, cmd: &str, args: &[&str]) -> Result<(), String> {
    exec_run_status_in_dir(dir, cmd, args)
}

pub(super) fn run_with_root(root: &Path, cmd: &str, args: &[&str]) -> Result<(), String> {
    exec_run_with_root(root, cmd, args)
}

pub(super) fn run_status_and_json(root: &Path, args: &[&str]) -> Result<Value, String> {
    exec_run_status_and_json(root, args)
}

pub(super) fn run_stdout_and_json(root: &Path, cmd: &str, args: &[&str]) -> Result<String, String> {
    exec_run_stdout_and_json(root, cmd, args)
}

pub(crate) fn repo_root() -> Result<PathBuf, String> {
    let mut dir = env::current_dir().map_err(|err| err.to_string())?;
    loop {
        if dir.join("Cargo.toml").exists() && dir.join("crates").is_dir() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    Err("could not locate repo root".to_string())
}

pub(super) fn run_evidence_taxonomy_report() -> Result<(), String> {
    let root = repo_root()?;
    let content =
        fs::read_to_string(root.join("evidence/taxonomy.md")).map_err(|err| err.to_string())?;
    println!("{content}");
    Ok(())
}

pub(super) fn run_evidence_ledger_report() -> Result<(), String> {
    let root = repo_root()?;
    let content = fs::read_to_string(root.join("evidence/ownership/evidence_ledger.json"))
        .map_err(|err| err.to_string())?;
    println!("{content}");
    Ok(())
}

pub(super) fn run_evidence_directory_map(out: &Path, create_missing: bool) -> Result<(), String> {
    let root = repo_root()?;
    let structure_payload =
        fs::read_to_string(root.join("configs/dag/policy/evidence_structure.json"))
            .map_err(|err| err.to_string())?;
    let structure: Value =
        serde_json::from_str(&structure_payload).map_err(|err| err.to_string())?;
    let required_dirs = structure["required_directories"]
        .as_array()
        .ok_or_else(|| "required_directories must be an array".to_string())?;

    let mut map_entries = Vec::new();
    for dir in required_dirs {
        let rel =
            dir.as_str().ok_or_else(|| "required directory entry must be a string".to_string())?;
        let full = root.join(rel);
        if create_missing && !full.exists() {
            fs::create_dir_all(&full).map_err(|err| err.to_string())?;
        }
        map_entries.push(json!({
            "path": rel,
            "exists": full.is_dir(),
        }));
    }

    let payload = json!({
        "version": structure["version"].as_str().unwrap_or("1"),
        "source_policy": "configs/dag/policy/evidence_structure.json",
        "entries": map_entries
    });
    let out_path = if out.is_absolute() { out.to_path_buf() } else { root.join(out) };
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let text = serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?;
    fs::write(&out_path, text).map_err(|err| err.to_string())?;
    println!("{}", out_path.display());
    Ok(())
}

pub(super) fn run_evidence_metadata_validate() -> Result<(), String> {
    let root = repo_root()?;
    let policy_payload =
        fs::read_to_string(root.join("configs/dag/policy/evidence_governance.json"))
            .map_err(|err| err.to_string())?;
    let policy: Value = serde_json::from_str(&policy_payload).map_err(|err| err.to_string())?;
    let ledger_payload = fs::read_to_string(root.join("evidence/ownership/evidence_ledger.json"))
        .map_err(|err| err.to_string())?;
    let ledger: Value = serde_json::from_str(&ledger_payload).map_err(|err| err.to_string())?;
    let path_policy_payload =
        fs::read_to_string(root.join("configs/dag/policy/evidence_path_policy.json"))
            .map_err(|err| err.to_string())?;
    let path_policy: Value =
        serde_json::from_str(&path_policy_payload).map_err(|err| err.to_string())?;

    let required_fields: BTreeSet<String> = policy["required_metadata_fields"]
        .as_array()
        .ok_or_else(|| "required_metadata_fields must be an array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "required metadata field must be a string".to_string())
                .map(ToOwned::to_owned)
        })
        .collect::<Result<_, _>>()?;
    let allowed_classes: BTreeSet<String> = policy["allowed_evidence_classes"]
        .as_array()
        .ok_or_else(|| "allowed_evidence_classes must be an array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "allowed evidence class must be a string".to_string())
                .map(ToOwned::to_owned)
        })
        .collect::<Result<_, _>>()?;
    let allowed_decisions: BTreeSet<String> = policy["allowed_decisions"]
        .as_array()
        .ok_or_else(|| "allowed_decisions must be an array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "allowed decision must be a string".to_string())
                .map(ToOwned::to_owned)
        })
        .collect::<Result<_, _>>()?;
    let allowed_impl_status: BTreeSet<String> = policy["allowed_implementation_statuses"]
        .as_array()
        .ok_or_else(|| "allowed_implementation_statuses must be an array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "allowed implementation status must be a string".to_string())
                .map(ToOwned::to_owned)
        })
        .collect::<Result<_, _>>()?;
    let forbidden_globs: Vec<String> = policy["forbidden_globs"]
        .as_array()
        .ok_or_else(|| "forbidden_globs must be an array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "forbidden glob must be a string".to_string())
                .map(ToOwned::to_owned)
        })
        .collect::<Result<_, _>>()?;

    let entries = ledger["entries"]
        .as_array()
        .ok_or_else(|| "evidence ledger entries must be an array".to_string())?;
    for entry in entries {
        let map = entry
            .as_object()
            .ok_or_else(|| "evidence ledger entry must be an object".to_string())?;
        for field in &required_fields {
            if !map.contains_key(field) {
                return Err(format!("evidence ledger entry missing required field `{field}`"));
            }
        }
        let path = entry["path"].as_str().ok_or_else(|| "entry path must be string".to_string())?;
        let owner =
            entry["owner"].as_str().ok_or_else(|| format!("owner must be string for {path}"))?;
        let class = entry["evidence_class"]
            .as_str()
            .ok_or_else(|| format!("evidence_class must be string for {path}"))?;
        let decision = entry["decision"]
            .as_str()
            .ok_or_else(|| format!("decision must be string for {path}"))?;
        let implementation_status = entry["implementation_status"]
            .as_str()
            .ok_or_else(|| format!("implementation_status must be string for {path}"))?;
        let canonical_location = entry["canonical_location"]
            .as_str()
            .ok_or_else(|| format!("canonical_location must be string for {path}"))?;
        let trust_property = entry["trust_property"]
            .as_str()
            .ok_or_else(|| format!("trust_property must be string for {path}"))?;
        let trust_properties_protected = entry["trust_properties_protected"]
            .as_array()
            .ok_or_else(|| format!("trust_properties_protected must be array for {path}"))?;
        let consumer_surfaces = entry["consumer_surfaces"]
            .as_array()
            .ok_or_else(|| format!("consumer_surfaces must be array for {path}"))?;
        if owner.trim().is_empty() {
            return Err(format!("owner is empty for {path}"));
        }
        if canonical_location.trim().is_empty() {
            return Err(format!("canonical_location is empty for {path}"));
        }
        if trust_property.trim().is_empty() {
            return Err(format!("trust_property is empty for {path}"));
        }
        if trust_properties_protected.is_empty() {
            return Err(format!("trust_properties_protected is empty for {path}"));
        }
        if consumer_surfaces.is_empty() {
            return Err(format!("consumer_surfaces is empty for {path}"));
        }
        if !allowed_classes.contains(class) {
            return Err(format!("invalid evidence_class `{class}` for {path}"));
        }
        if !allowed_decisions.contains(decision) {
            return Err(format!("invalid decision `{decision}` for {path}"));
        }
        if !allowed_impl_status.contains(implementation_status) {
            return Err(format!(
                "invalid implementation_status `{implementation_status}` for {path}"
            ));
        }
        if !root.join(path).exists() {
            return Err(format!("ledger path does not exist: {path}"));
        }
    }

    let governed_roots: Vec<String> = path_policy["governed_roots"]
        .as_array()
        .ok_or_else(|| "governed_roots must be an array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "governed root must be string".to_string())
                .map(ToOwned::to_owned)
        })
        .collect::<Result<_, _>>()?;
    let schema_fixture_roots: Vec<String> = path_policy["schema_fixture_roots"]
        .as_array()
        .ok_or_else(|| "schema_fixture_roots must be an array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "schema fixture root must be string".to_string())
                .map(ToOwned::to_owned)
        })
        .collect::<Result<_, _>>()?;
    let legacy_scenario_roots: Vec<String> = path_policy["legacy_scenario_roots"]
        .as_array()
        .ok_or_else(|| "legacy_scenario_roots must be an array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "legacy scenario root must be string".to_string())
                .map(ToOwned::to_owned)
        })
        .collect::<Result<_, _>>()?;
    let helper_allowlist: Vec<String> = path_policy["helper_allowlist"]
        .as_array()
        .ok_or_else(|| "helper_allowlist must be an array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "helper allowlist entry must be string".to_string())
                .map(ToOwned::to_owned)
        })
        .collect::<Result<_, _>>()?;

    for file in repository_files_with_extension(&root, "json")? {
        let rel = file
            .strip_prefix(&root)
            .map_err(|err| err.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        let in_governed_root = governed_roots.iter().any(|governed_root| {
            rel == *governed_root || rel.starts_with(&format!("{governed_root}/"))
        });
        let in_schema_fixture_root = schema_fixture_roots
            .iter()
            .any(|schema_root| rel == *schema_root || rel.starts_with(&format!("{schema_root}/")));
        let in_helper_allowlist =
            helper_allowlist.iter().any(|pattern| wildcard_match(pattern, &rel));
        let in_legacy_scenario_root = legacy_scenario_roots
            .iter()
            .any(|legacy_root| rel == *legacy_root || rel.starts_with(&format!("{legacy_root}/")));
        if in_governed_root
            || in_schema_fixture_root
            || in_helper_allowlist
            || in_legacy_scenario_root
        {
            continue;
        }
        let is_scenario_like = rel.ends_with(".dag.json")
            || rel.contains("/scenarios/")
            || rel.contains("/fixtures/")
            || rel.starts_with("examples/");
        if is_scenario_like {
            return Err(format!(
                "scenario-like json path outside evidence-governed roots is forbidden: {rel}"
            ));
        }
        if forbidden_globs.iter().any(|pattern| wildcard_match(pattern, &rel)) {
            return Err(format!("path is forbidden by evidence governance freeze policy: {rel}"));
        }
        if rel.starts_with("tests/authoring/examples/") || rel.starts_with("tests/authoring/bad/") {
            return Err(format!(
                "authoring evidence outside evidence/authoring is forbidden: {rel}"
            ));
        }
    }

    println!("evidence metadata validation passed");
    Ok(())
}

pub(super) fn resolve_under_root(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

pub(super) fn write_report(path: &Path, body: &str) -> Result<(), String> {
    crate::report::write::write_text_report(path, body)
}

pub(super) fn run_repo_hotspot_reports(
    file_out: &Path,
    function_out: &Path,
    api_out: &Path,
    dep_out: &Path,
) -> Result<(), String> {
    let root = repo_root()?;
    let crates_dir = root.join("crates");
    let mut files = Vec::new();
    collect_all_files(&crates_dir, &mut files)?;
    let rust_files: Vec<PathBuf> = files
        .into_iter()
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"))
        .collect();

    let mut file_counts = Vec::new();
    let mut long_functions = Vec::new();
    let mut api_counts: BTreeMap<String, usize> = BTreeMap::new();

    for file in &rust_files {
        let rel = file
            .strip_prefix(&root)
            .map_err(|err| err.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        let body = fs::read_to_string(file).map_err(|err| err.to_string())?;
        let lines: Vec<&str> = body.lines().collect();
        file_counts.push((lines.len(), rel.clone()));

        let mut fn_starts = Vec::new();
        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("fn ")
                || trimmed.starts_with("pub fn ")
                || trimmed.starts_with("pub(crate) fn ")
            {
                fn_starts.push(idx + 1);
            }
            if trimmed.starts_with("pub ") || trimmed.starts_with("pub(crate) ") {
                *api_counts.entry(rel.clone()).or_insert(0) += 1;
            }
        }
        for (pos, start) in fn_starts.iter().enumerate() {
            let end = if pos + 1 < fn_starts.len() { fn_starts[pos + 1] - 1 } else { lines.len() };
            let len = end.saturating_sub(*start) + 1;
            if len >= 60 {
                long_functions.push((len, rel.clone(), *start));
            }
        }
    }

    file_counts.sort_by(|a, b| b.cmp(a));
    long_functions.sort_by(|a, b| b.cmp(a));
    let mut api_rows: Vec<(usize, String)> = api_counts.into_iter().map(|(f, c)| (c, f)).collect();
    api_rows.sort_by(|a, b| b.cmp(a));

    let mut file_report =
        String::from("# File Size Hotspot Report\n\nGenerated from Rust source line counts.\n\n");
    for (lines, path) in file_counts.into_iter().take(40) {
        file_report.push_str(&format!("- {lines} lines :: {path}\n"));
    }

    let mut function_report = String::from(
        "# Long Function Hotspot Report\n\nHeuristic report for functions exceeding 60 lines in Rust files.\n\n",
    );
    for (len, path, start) in long_functions.into_iter().take(80) {
        function_report.push_str(&format!("- {len} lines :: {path}:{start}\n"));
    }

    let mut public_api_report =
        String::from("# Public API Hotspot Report\n\nTop files by count of public items.\n\n");
    for (count, path) in api_rows.into_iter().take(40) {
        public_api_report.push_str(&format!("- {count} public items :: {path}\n"));
    }

    let dep_status = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(&root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    let mut dep_report = String::from(
        "# Dependency Cycle Report\n\nRust crate graph cycles are expected to be absent.\n\n",
    );
    dep_report.push_str("- check: cargo metadata package graph\n");
    if dep_status {
        dep_report.push_str(
            "- status: no crate-level dependency cycles detected by Cargo package resolution\n",
        );
    } else {
        dep_report.push_str("- status: metadata resolution failed\n");
    }
    dep_report.push_str(
        "- note: module-level cycles are prevented by Rust module system and compile checks\n",
    );

    write_report(&resolve_under_root(&root, file_out), &file_report)?;
    write_report(&resolve_under_root(&root, function_out), &function_report)?;
    write_report(&resolve_under_root(&root, api_out), &public_api_report)?;
    write_report(&resolve_under_root(&root, dep_out), &dep_report)?;
    Ok(())
}

pub(super) fn run_repo_schema_changelog(out: &Path, schema_root: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let schema_path = resolve_under_root(&root, schema_root);
    let mut files = Vec::new();
    collect_all_files(&schema_path, &mut files)?;
    let mut schema_files: Vec<String> = files
        .into_iter()
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .map(|p| {
            p.strip_prefix(&root)
                .map(|v| v.to_string_lossy().replace('\\', "/"))
                .map_err(|err| err.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    schema_files.sort();

    let mut report = String::from(
        "# Schema Changelog\n\nGenerated from files under `configs/dag/schema`.\n\n## Schemas\n",
    );
    for rel in schema_files {
        let full = root.join(&rel);
        let bytes = fs::read(&full).map_err(|err| err.to_string())?;
        let sum = format!("{:x}", sha2::Sha256::digest(bytes));
        report.push_str(&format!("- {rel} :: {sum}\n"));
    }

    write_report(&resolve_under_root(&root, out), &report)?;
    Ok(())
}

pub(super) fn collect_markdown_files_under(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    collect_all_files(root, &mut files)?;
    Ok(files
        .into_iter()
        .filter(|p| p.extension().and_then(|ext| ext.to_str()) == Some("md"))
        .collect())
}

pub(super) fn markdown_contains_path(
    rel_path: &str,
    markdown_files: &[PathBuf],
    root: &Path,
) -> bool {
    markdown_files.iter().any(|doc| {
        fs::read_to_string(doc).map(|body| body.contains(rel_path)).unwrap_or(false)
            && doc.strip_prefix(root).is_ok()
    })
}

pub(super) fn collect_public_item_count(src_dir: &Path) -> Result<usize, String> {
    let mut files = Vec::new();
    collect_all_files(src_dir, &mut files)?;
    let mut count = 0usize;
    for file in files {
        if file.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let body = fs::read_to_string(&file).map_err(|err| err.to_string())?;
        count += body
            .lines()
            .filter(|line| {
                let trimmed = line.trim_start();
                trimmed.starts_with("pub ") || trimmed.starts_with("pub(")
            })
            .count();
    }
    Ok(count)
}

pub(super) fn run_repo_runtime_scope_reports(
    kernel_out: &Path,
    non_kernel_out: &Path,
    contract_backing_out: &Path,
    operator_surface_out: &Path,
    core_api_out: &Path,
    runtime_api_out: &Path,
) -> Result<(), String> {
    let root = repo_root()?;
    let scope_payload = fs::read_to_string(root.join("configs/dag/policy/runtime_scope_v2.json"))
        .map_err(|err| err.to_string())?;
    let scope: Value = serde_json::from_str(&scope_payload).map_err(|err| err.to_string())?;
    let entries = scope["module_entries"]
        .as_array()
        .ok_or_else(|| "runtime_scope_v2.module_entries must be an array".to_string())?;

    let mut kernel_modules = Vec::new();
    let mut non_kernel_modules = Vec::new();
    for entry in entries {
        let module = entry["module"]
            .as_str()
            .ok_or_else(|| "runtime scope entry missing module".to_string())?;
        let decision = entry["decision"].as_str().unwrap_or("keep");
        let classification = entry["classification"].as_str().unwrap_or("support");
        let rationale = entry["rationale"].as_str().unwrap_or("");
        let rel = format!("crates/bijux-dag-runtime/src/{module}");
        let is_kernel = module.starts_with("runtime_core/")
            || module.starts_with("artifacts/")
            || module.starts_with("cache/")
            || module.starts_with("replay/")
            || module.starts_with("policy/");
        if is_kernel && decision == "keep" {
            kernel_modules.push((
                module.to_string(),
                classification.to_string(),
                rationale.to_string(),
            ));
        } else {
            non_kernel_modules.push((
                module.to_string(),
                classification.to_string(),
                decision.to_string(),
                rationale.to_string(),
                rel,
            ));
        }
    }
    kernel_modules.sort();
    non_kernel_modules.sort();

    let mut kernel_report = String::from(
        "# Kernel Owned Runtime Modules\n\nGenerated from `configs/dag/policy/runtime_scope_v2.json`.\n\n",
    );
    kernel_report.push_str(
        "Performance-related classifications on this page must stay backed by `bijux-dev-dag performance-evidence-report` and `evidence/perf/metadata.json`.\n\n",
    );
    kernel_report.push_str("## Runtime kernel-owned module set\n\n");
    for (module, classification, rationale) in &kernel_modules {
        kernel_report.push_str(&format!("- `{module}` ({classification}): {rationale}\n"));
    }
    kernel_report.push_str(&format!("\nTotal: `{}` modules.\n", kernel_modules.len()));

    let mut non_kernel_report = String::from(
        "# Runtime Non-Kernel Modules\n\nGenerated from `configs/dag/policy/runtime_scope_v2.json`.\n\n",
    );
    non_kernel_report.push_str(
        "Performance-related classifications on this page must stay backed by `bijux-dev-dag performance-evidence-report` and `evidence/perf/metadata.json`.\n\n",
    );
    non_kernel_report.push_str("## Runtime modules outside kernel ownership\n\n");
    for (module, classification, decision, rationale, _) in &non_kernel_modules {
        non_kernel_report.push_str(&format!(
            "- `{module}` ({classification}, decision `{decision}`): {rationale}\n"
        ));
    }
    non_kernel_report.push_str(&format!("\nTotal: `{}` modules.\n", non_kernel_modules.len()));

    let runtime_tests = root.join("crates/bijux-dag-runtime/tests");
    let dev_tests = root.join("crates/bijux-dev/tests");
    let docs_spec = root.join("docs/spec");
    let docs_arch = root.join("docs/bijux-core/architecture");
    let docs_runtime_package = root.join("docs/bijux-dag/packages");
    let mut test_files = Vec::new();
    collect_all_files(&runtime_tests, &mut test_files)?;
    collect_all_files(&dev_tests, &mut test_files)?;
    let mut documentation_files = Vec::new();
    for doc_root in [&docs_spec, &docs_arch, &docs_runtime_package] {
        if doc_root.exists() {
            documentation_files.extend(collect_markdown_files_under(doc_root)?);
        }
    }

    let mut contract_backed = Vec::new();
    let mut documented_only = Vec::new();
    let mut unclassified = Vec::new();
    for (_, _, _, _, rel) in &non_kernel_modules {
        let referenced_by_tests = test_files
            .iter()
            .any(|file| fs::read_to_string(file).map(|body| body.contains(rel)).unwrap_or(false));
        let referenced_by_docs = markdown_contains_path(rel, &documentation_files, &root);
        if referenced_by_tests {
            contract_backed.push(rel.clone());
        } else if referenced_by_docs {
            documented_only.push(rel.clone());
        } else {
            unclassified.push(rel.clone());
        }
    }
    contract_backed.sort();
    documented_only.sort();
    unclassified.sort();

    let mut contract_backing_report = String::from(
        "# Runtime Contract Backing Report\n\nGenerated by scanning runtime module paths in policy against runtime/dev tests and architecture/spec docs.\n\n",
    );
    contract_backing_report.push_str(
        "Performance-related classifications on this page must stay backed by `bijux-dev-dag performance-evidence-report` and `evidence/perf/metadata.json`.\n\n",
    );
    contract_backing_report.push_str("## Contract-backed modules\n\n");
    for rel in &contract_backed {
        contract_backing_report.push_str(&format!("- `{rel}`\n"));
    }
    contract_backing_report.push_str("\n## Documented-only modules\n\n");
    for rel in &documented_only {
        contract_backing_report.push_str(&format!("- `{rel}`\n"));
    }
    contract_backing_report.push_str("\n## Unclassified modules\n\n");
    for rel in &unclassified {
        contract_backing_report.push_str(&format!("- `{rel}`\n"));
    }

    let mut operator_facing = Vec::new();
    let mut internal_only = Vec::new();
    for (_, _, _, _, rel) in &non_kernel_modules {
        let rel_mod = rel.strip_prefix("crates/bijux-dag-runtime/src/").unwrap_or(rel.as_str());
        let is_operator_facing = rel_mod.starts_with("diagnostics/runtime/")
            || rel_mod.starts_with("replay/")
            || rel_mod == "artifacts/verifier.rs"
            || rel_mod == "artifacts/storage/recovery.rs";
        if is_operator_facing {
            operator_facing.push(rel.clone());
        } else {
            internal_only.push(rel.clone());
        }
    }
    operator_facing.sort();
    internal_only.sort();

    let mut operator_surface_report = String::from(
        "# Runtime Operator Surface Report\n\nGenerated from runtime scope policy path classification.\n\n",
    );
    operator_surface_report.push_str(
        "Performance-related classifications on this page must stay backed by `bijux-dev-dag performance-evidence-report` and `evidence/perf/metadata.json`.\n\n",
    );
    operator_surface_report.push_str("## Operator-facing modules\n\n");
    for rel in &operator_facing {
        operator_surface_report.push_str(&format!("- `{rel}`\n"));
    }
    operator_surface_report.push_str("\n## Internal-only modules\n\n");
    for rel in &internal_only {
        operator_surface_report.push_str(&format!("- `{rel}`\n"));
    }

    let core_src = root.join("crates/bijux-dag-core/src");
    let runtime_src = root.join("crates/bijux-dag-runtime/src");
    let core_pub = collect_public_item_count(&core_src)?;
    let runtime_pub = collect_public_item_count(&runtime_src)?;
    let core_api_report = format!(
        r#"# Core Public API Surface

This report is generated by `bijux-dev-dag repo runtime-scope-reports` from a
source scan of `crates/bijux-dag-core/src/`.

## Observation

- crate: `bijux-dag-core`
- public item declarations: `{core_pub}`

## Interpretation

The count is an architecture signal, not a semantic-versioning verdict. It
includes source declarations found by the report scanner and does not replace
`cargo public-api`, rustdoc reachability, or compatibility review. Changes
require inspection of whether graph and planning APIs are intentional or
incidental exposure.

## Review Condition

Review this report when core modules or exports change. A count increase is
acceptable only when the new public ownership is deliberate and documented; a
decrease still requires compatibility review.
"#
    );
    let runtime_api_report = format!(
        r#"# Runtime Public API Surface

This report is generated by `bijux-dev-dag repo runtime-scope-reports` from a
source scan of `crates/bijux-dag-runtime/src/`.

## Observation

- crate: `bijux-dag-runtime`
- public item declarations: `{runtime_pub}`

## Interpretation

The count is an architecture signal, not a semantic-versioning verdict. It
includes source declarations found by the report scanner and does not replace
`cargo public-api`, rustdoc reachability, or compatibility review. Changes
require inspection of whether execution-kernel APIs are intentional and whether
modeled or platform-specific surfaces remain isolated.

## Review Condition

Review this report when runtime modules or exports change. A count increase is
acceptable only when the new public ownership is deliberate and documented; a
decrease still requires compatibility review.
"#
    );

    write_report(&resolve_under_root(&root, kernel_out), &kernel_report)?;
    write_report(&resolve_under_root(&root, non_kernel_out), &non_kernel_report)?;
    write_report(&resolve_under_root(&root, contract_backing_out), &contract_backing_report)?;
    write_report(&resolve_under_root(&root, operator_surface_out), &operator_surface_report)?;
    write_report(&resolve_under_root(&root, core_api_out), &core_api_report)?;
    write_report(&resolve_under_root(&root, runtime_api_out), &runtime_api_report)?;
    Ok(())
}

pub(super) fn run_repo_planner_hardening_report(out: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let planner_fixtures_dir = root.join("crates/bijux-dag-core/tests/fixtures/planner");
    let mut fixtures = Vec::new();
    collect_all_files(&planner_fixtures_dir, &mut fixtures)?;
    fixtures.retain(|path| {
        path.extension().and_then(|ext| ext.to_str()) == Some("json")
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".dag.json"))
    });
    fixtures.sort();

    let schema_path = root.join("configs/dag/schema/execution_plan.schema.json");
    let required = required_schema_fields(&schema_path)?;
    let mut rows = Vec::new();

    for fixture in fixtures {
        let fixture_rel = fixture
            .strip_prefix(&root)
            .map_err(|err| err.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        let payload = fs::read_to_string(&fixture).map_err(|err| err.to_string())?;
        let graph = bijux_dag_core::parse_graph_strict(&payload)
            .map_err(|err| format!("planner fixture parse failed for {fixture_rel}: {err}"))?;
        let first = bijux_dag_core::lower_graph_to_execution_plan(
            &graph,
            bijux_dag_core::PlanOptions::default(),
        )
        .map_err(|err| format!("planner lower failed for {fixture_rel}: {err}"))?;
        let second = bijux_dag_core::lower_graph_to_execution_plan(
            &graph,
            bijux_dag_core::PlanOptions::default(),
        )
        .map_err(|err| format!("planner re-lower failed for {fixture_rel}: {err}"))?;
        let stable_dump = serde_json::to_string_pretty(&first).map_err(|err| err.to_string())?
            == serde_json::to_string_pretty(&second).map_err(|err| err.to_string())?;
        let plan_value = serde_json::to_value(&first).map_err(|err| err.to_string())?;
        let schema_ok = required.iter().all(|field| plan_value.get(field).is_some());
        rows.push((fixture_rel, first.nodes.len(), first.edges.len(), stable_dump, schema_ok));
    }

    let mut report = String::from(
        "# Planner Hardening Report\n\n## Purpose\n\nThis report records the repository surfaces that currently harden planner behavior and keep lowering claims tied to executable proof.\n\n## Guarded surfaces\n\n- contract: `docs/spec/PLANNER_CONTRACT.md`\n- battle trust properties: `docs/spec/BATTLE_TRUST_PROPERTIES.md`\n- schema: `configs/dag/schema/execution_plan.schema.json`\n- trust map: `configs/dag/policy/trust_property_test_map.json`\n- battle trust policy: `configs/dag/policy/battle_trust_properties.json`\n- core planner tests: `crates/bijux-dag-core/tests/planner_contract.rs`, `crates/bijux-dag-core/tests/planner_fixture_contracts.rs`, `crates/bijux-dag-core/tests/planner_validation_edge_case_contracts.rs`\n- runtime lowering tests: `crates/bijux-dag-runtime/tests/planner_lowering_contracts.rs`, `crates/bijux-dag-runtime/tests/engine_correctness_contracts.rs`\n- maintainer guard: `crates/bijux-dev/tests/planner_hardening_contracts.rs`\n- maintainer command surface: `dag plan-dump`\n\nGenerated from execution-plan lowering against canonical graph fixtures in `crates/bijux-dag-core/tests/fixtures/planner`.\n\n## Fixture results\n\n",
    );
    for (fixture, nodes, edges, stable_dump, schema_ok) in &rows {
        report.push_str(&format!(
            "- `{fixture}` :: nodes=`{nodes}` edges=`{edges}` stable_dump=`{stable_dump}` schema_required_fields=`{schema_ok}`\n"
        ));
    }
    report.push_str("\n## Guardrails\n\n");
    report.push_str("- deterministic lowering across repeated runs for each fixture\n");
    report.push_str("- schema-required field presence from `execution_plan.schema.json`\n");
    report.push_str(
        "- `tp_plan_truth` remains covered by planner lowering and engine correctness tests\n",
    );
    report.push_str("- planner diagnostics such as `P4021` stay visible through `dag plan-dump`\n");
    report.push_str(
        "- fixture corpus includes linear/fan/diamond/resource/retry/replay-oriented shapes\n",
    );

    write_report(&resolve_under_root(&root, out), &report)
}

pub(super) fn run_repo_artifact_capability_reports(
    matrix_out: &Path,
    model_out: &Path,
) -> Result<(), String> {
    use bijux_dag_artifacts::store::{
        ArtifactStoreBackend, FilesystemArtifactStore, ObjectArtifactStore,
    };

    let root = repo_root()?;
    let fs_store = FilesystemArtifactStore::new(".");
    let object_store =
        ObjectArtifactStore { bucket: "modeled".to_string(), prefix: "artifacts/".to_string() };

    let fs_caps = fs_store.capabilities();
    let object_caps = object_store.capabilities();

    let support_label = |implemented: bool| {
        if implemented {
            "implemented"
        } else {
            "modeled"
        }
    };

    let matrix = format!(
        "# Artifact Store Capability Matrix\n\nGenerated from `crates/bijux-dag-artifacts/src/io/store.rs` backend capability declarations.\n\n| capability | filesystem store | object store model |\n|---|---|---|\n| write artifact payload | {} | {} |\n| read artifact payload | {} | {} |\n| runtime-backed execution | {} | {} |\n\nNotes:\n- Runtime source-of-truth currently implements filesystem storage semantics.\n- Object-store surface remains declared capability only and must not be presented as implemented runtime behavior.\n",
        support_label(fs_caps.can_write_bytes),
        support_label(object_caps.can_write_bytes),
        support_label(fs_caps.can_read_bytes),
        support_label(object_caps.can_read_bytes),
        support_label(matches!(fs_caps.support_level, bijux_dag_artifacts::store::ArtifactStoreSupportLevel::Implemented)),
        support_label(matches!(object_caps.support_level, bijux_dag_artifacts::store::ArtifactStoreSupportLevel::Implemented)),
    );

    let model = String::from(
        "# Content Addressed Storage Model\n\nGenerated from artifact store implementation capability declarations.\n\n## Identity primitives\n\n- `artifact_sha256` identifies content bytes.\n- `artifact_id` identifies logical artifact identity (`<node_id>:<file_name>`).\n- Durable provenance joins `artifact_sha256` with `run_id`, `node_id`, and `node_fingerprint`.\n\n## Implementation status\n\n- Filesystem backend: implemented read/write payload persistence.\n- Object backend: modeled-only surface; runtime rejects read/write calls.\n\n## Safety rules\n\n- Artifact identity is content + provenance; identical bytes can legitimately appear under distinct provenance chains.\n- Garbage collection decisions must remain lineage-aware and dry-run explainable.\n",
    );

    write_report(&resolve_under_root(&root, matrix_out), &matrix)?;
    write_report(&resolve_under_root(&root, model_out), &model)?;
    Ok(())
}

pub(super) fn run_evidence_ownership_verify() -> Result<(), String> {
    run_evidence_metadata_validate()
}

pub(super) fn required_schema_fields(schema_path: &Path) -> Result<BTreeSet<String>, String> {
    let schema: Value =
        serde_json::from_str(&fs::read_to_string(schema_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("schema missing required array: {}", schema_path.display()))?;
    let mut fields = BTreeSet::new();
    for field in required {
        let name = field.as_str().ok_or_else(|| {
            format!("schema required entry must be string: {}", schema_path.display())
        })?;
        fields.insert(name.to_string());
    }
    Ok(fields)
}

pub(super) fn run_evidence_schema_verify() -> Result<(), String> {
    let root = repo_root()?;

    let schema_files = [
        "configs/dag/schema/evidence_asset.schema.json",
        "configs/dag/schema/evidence_family.schema.json",
        "configs/dag/schema/evidence_cache_metadata.schema.json",
        "configs/dag/schema/evidence_battle_metadata.schema.json",
        "configs/dag/schema/evidence_perf_metadata.schema.json",
        "configs/dag/schema/evidence_compare_metadata.schema.json",
        "configs/dag/schema/evidence_compat_metadata.schema.json",
        "configs/dag/schema/evidence_fault_metadata.schema.json",
        "configs/dag/schema/evidence_authoring_metadata.schema.json",
    ];
    for rel in schema_files {
        let path = root.join(rel);
        if !path.exists() {
            return Err(format!("required evidence schema is missing: {rel}"));
        }
        let parsed: Value =
            serde_json::from_str(&fs::read_to_string(&path).map_err(|err| err.to_string())?)
                .map_err(|err| err.to_string())?;
        if parsed.get("type").and_then(Value::as_str) != Some("object") {
            return Err(format!("evidence schema must declare object type: {rel}"));
        }
    }

    let asset_required =
        required_schema_fields(&root.join("configs/dag/schema/evidence_asset.schema.json"))?;
    let family_required =
        required_schema_fields(&root.join("configs/dag/schema/evidence_family.schema.json"))?;

    let ledger: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/ownership/evidence_ledger.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;

    let entries = ledger["entries"]
        .as_array()
        .ok_or_else(|| "evidence ledger entries must be an array".to_string())?;
    for entry in entries {
        let map =
            entry.as_object().ok_or_else(|| "evidence ledger entry must be object".to_string())?;
        let id = entry
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "evidence entry missing id".to_string())?;
        for field in &asset_required {
            if !map.contains_key(field) {
                return Err(format!("evidence entry `{id}` missing required field `{field}`"));
            }
        }

        let kind = entry
            .get("kind")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("evidence entry `{id}` has invalid kind"))?;
        let owner = entry
            .get("owner")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("evidence entry `{id}` has invalid owner"))?;
        let canonical_path = entry
            .get("canonical_path")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("evidence entry `{id}` has invalid canonical_path"))?;
        let consumers = entry
            .get("consumers")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("evidence entry `{id}` has invalid consumers"))?;
        let release_blocking = entry
            .get("release_blocking")
            .and_then(Value::as_bool)
            .ok_or_else(|| format!("evidence entry `{id}` has invalid release_blocking"))?;
        let trust_properties = entry
            .get("trust_properties")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("evidence entry `{id}` has invalid trust_properties"))?;
        let duplicate_of = entry
            .get("duplicate_of")
            .ok_or_else(|| format!("evidence entry `{id}` missing duplicate_of"))?;
        let derived_from = entry
            .get("derived_from")
            .ok_or_else(|| format!("evidence entry `{id}` missing derived_from"))?;

        if owner.trim().is_empty() {
            return Err(format!("evidence entry `{id}` has empty owner"));
        }
        if canonical_path.trim().is_empty() {
            return Err(format!("evidence entry `{id}` has empty canonical_path"));
        }
        if !root.join(canonical_path).exists() {
            return Err(format!(
                "evidence entry `{id}` canonical_path does not exist: {canonical_path}"
            ));
        }
        if consumers.is_empty() {
            return Err(format!("evidence entry `{id}` has empty consumers"));
        }
        let allowed_kinds =
            ["authoring", "battle", "cache", "compat", "fault", "operator", "perf", "compare"];
        if !allowed_kinds.contains(&kind) {
            return Err(format!("evidence entry `{id}` has unknown kind `{kind}`"));
        }
        if release_blocking && trust_properties.is_empty() {
            return Err(format!(
                "evidence entry `{id}` is release_blocking but has no trust_properties"
            ));
        }
        if !duplicate_of.is_null()
            && duplicate_of.as_str().map_or(true, |value| value.trim().is_empty())
        {
            return Err(format!(
                "evidence entry `{id}` duplicate_of must be non-empty string or null"
            ));
        }
        if !derived_from.is_null()
            && derived_from.as_str().map_or(true, |value| value.trim().is_empty())
        {
            return Err(format!(
                "evidence entry `{id}` derived_from must be non-empty string or null"
            ));
        }
    }

    let families = ledger["asset_families"]
        .as_array()
        .ok_or_else(|| "evidence ledger asset_families must be an array".to_string())?;
    for family in families {
        let map = family.as_object().ok_or_else(|| "asset family must be object".to_string())?;
        let family_id = family
            .get("family_id")
            .and_then(Value::as_str)
            .ok_or_else(|| "asset family missing family_id".to_string())?;
        for field in &family_required {
            if !map.contains_key(field) {
                return Err(format!("asset family `{family_id}` missing required field `{field}`"));
            }
        }
    }

    let metadata_files = [
        ("evidence/cache/metadata.json", "cache metadata"),
        ("evidence/battle/metadata.json", "battle metadata"),
        ("evidence/perf/metadata.json", "perf metadata"),
        ("evidence/compare/metadata.json", "compare metadata"),
        ("evidence/compat/metadata.json", "compat metadata"),
        ("evidence/fault/metadata.json", "fault metadata"),
        ("evidence/authoring/metadata.json", "authoring metadata"),
    ];
    for (rel, label) in metadata_files {
        let path = root.join(rel);
        if !path.exists() {
            return Err(format!("missing {label}: {rel}"));
        }
        let payload: Value =
            serde_json::from_str(&fs::read_to_string(path).map_err(|err| err.to_string())?)
                .map_err(|err| err.to_string())?;
        if !payload.is_object() {
            return Err(format!("{label} is not an object: {rel}"));
        }
    }

    println!("evidence schema validation passed");
    Ok(())
}

pub(super) fn run_evidence_domain_verify(
    domain: &str,
    required_paths: &[&str],
) -> Result<(), String> {
    let root = repo_root()?;
    run_evidence_metadata_validate()?;
    for rel in required_paths {
        if !root.join(rel).exists() {
            return Err(format!("required evidence surface missing for `{domain}`: {rel}"));
        }
    }
    let ledger: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/ownership/evidence_ledger.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let entries = ledger["entries"]
        .as_array()
        .ok_or_else(|| "evidence ledger entries must be an array".to_string())?;
    let prefix = format!("evidence/{domain}/");
    let has_entries = entries.iter().any(|entry| {
        entry.get("path").and_then(Value::as_str).is_some_and(|path| path.starts_with(&prefix))
    });
    if !has_entries {
        return Err(format!("evidence ledger has no entries for governed domain `{domain}`"));
    }
    Ok(())
}

pub(super) fn parse_string_set(value: &Value, label: &str) -> Result<BTreeSet<String>, String> {
    let items = value.as_array().ok_or_else(|| format!("{label} must be an array"))?;
    let mut out = BTreeSet::new();
    for item in items {
        let name = item.as_str().ok_or_else(|| format!("{label} entry must be a string"))?;
        if name.trim().is_empty() {
            return Err(format!("{label} contains empty string"));
        }
        out.insert(name.to_string());
    }
    Ok(out)
}

pub(super) fn run_evidence_family_boundary_verify() -> Result<(), String> {
    let root = repo_root()?;
    let assets = load_registry_assets(&root)?;

    for asset in &assets {
        let path = asset.canonical_path.as_str();
        let kind = asset.kind.as_str();
        let expected_kind = if path.starts_with("evidence/cache/") {
            Some("cache")
        } else if path.starts_with("evidence/compat/") {
            Some("compat")
        } else if path.starts_with("evidence/fault/") {
            Some("fault")
        } else {
            None
        };
        if let Some(expected) = expected_kind {
            if kind != expected {
                return Err(format!(
                    "mixed-family misuse: `{path}` is classified as `{kind}` but must be `{expected}`"
                ));
            }
        }
    }

    let cache_metadata: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/cache/metadata.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let cache_allowed = parse_string_set(
        &cache_metadata["consumer_boundaries"]["cache_allowed_consumers"],
        "cache consumer_boundaries.cache_allowed_consumers",
    )?;
    let replay_allowed = parse_string_set(
        &cache_metadata["consumer_boundaries"]["replay_allowed_consumers"],
        "cache consumer_boundaries.replay_allowed_consumers",
    )?;

    for asset in &assets {
        let path = asset.canonical_path.as_str();
        if !path.starts_with("evidence/cache/") {
            continue;
        }
        let allowed = if path.starts_with("evidence/cache/replay/") {
            &replay_allowed
        } else {
            &cache_allowed
        };
        for consumer in &asset.consumers {
            if !allowed.contains(consumer) {
                return Err(format!(
                    "cache/replay consumer misuse: `{path}` uses consumer `{consumer}` outside allowed set"
                ));
            }
        }
    }

    let compat_metadata: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/compat/metadata.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let decision_matrix = compat_metadata["decision_matrix"]
        .as_object()
        .ok_or_else(|| "compat metadata decision_matrix must be an object".to_string())?;
    for (path, entry) in decision_matrix {
        if !path.starts_with("evidence/compat/") {
            return Err(format!("compat decision matrix contains out-of-family asset: {path}"));
        }
        if !root.join(path).exists() {
            return Err(format!("compat decision matrix path does not exist: {path}"));
        }
        let decision = entry
            .get("decision")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("compat decision matrix entry missing decision: {path}"))?;
        let allowed =
            ["supported", "unsupported_newer_version", "unsupported_older_version", "corrupt"];
        if !allowed.contains(&decision) {
            return Err(format!(
                "compat decision matrix has unknown decision `{decision}` for `{path}`"
            ));
        }
    }

    let fault_metadata: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/fault/metadata.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let fault_expectations = fault_metadata["fault_expectations"]
        .as_object()
        .ok_or_else(|| "fault metadata fault_expectations must be an object".to_string())?;
    let fault_profiles = fault_metadata["fault_profiles"]
        .as_object()
        .ok_or_else(|| "fault metadata fault_profiles must be an object".to_string())?;
    for fault_class in fault_expectations.keys() {
        if !fault_profiles.contains_key(fault_class) {
            return Err(format!(
                "fault metadata missing fault_profiles entry for fault class `{fault_class}`"
            ));
        }
    }
    for (fault_class, profile) in fault_profiles {
        let expected_fault_class =
            profile.get("expected_fault_class").and_then(Value::as_str).ok_or_else(|| {
                format!("fault profile missing expected_fault_class: `{fault_class}`")
            })?;
        let expected_reaction =
            profile.get("expected_system_reaction").and_then(Value::as_str).ok_or_else(|| {
                format!("fault profile missing expected_system_reaction: `{fault_class}`")
            })?;
        if expected_fault_class.trim().is_empty() || expected_reaction.trim().is_empty() {
            return Err(format!(
                "fault profile contains empty fields for fault class `{fault_class}`"
            ));
        }
    }

    Ok(())
}

pub(super) fn run_evidence_authoring_verify() -> Result<(), String> {
    run_evidence_domain_verify(
        "authoring",
        &[
            "evidence/authoring/metadata.json",
            "evidence/authoring/examples",
            "evidence/authoring/patterns",
            "evidence/authoring/negative",
        ],
    )?;
    run_validate_all_authoring()
}

pub(super) fn run_evidence_battle_verify() -> Result<(), String> {
    run_evidence_domain_verify(
        "battle",
        &[
            "evidence/battle/workflows",
            "evidence/battle/metadata.json",
            "evidence/battle/registries/scenario_registry.json",
            "evidence/battle/registries/trust_property_registry.json",
        ],
    )?;
    run_battle_scenario_mapping_validate()
}

pub(super) fn run_evidence_cache_verify() -> Result<(), String> {
    run_evidence_domain_verify(
        "cache",
        &[
            "evidence/cache/corrupt",
            "evidence/cache/scenarios",
            "evidence/cache/replay",
            "evidence/cache/metadata.json",
        ],
    )?;
    run_evidence_family_boundary_verify()
}

pub(super) fn run_evidence_replay_verify() -> Result<(), String> {
    run_evidence_domain_verify(
        "cache",
        &[
            "evidence/cache/replay/match_case.json",
            "evidence/cache/replay/mismatch_case.json",
            "evidence/cache/replay/corruption_case.json",
            "evidence/cache/replay/unsupported_version_case.json",
        ],
    )?;
    run_replay_contract_guard()
}

pub(super) fn run_evidence_compat_verify() -> Result<(), String> {
    run_evidence_domain_verify(
        "compat",
        &[
            "evidence/compat/graph_schema",
            "evidence/compat/export_bundle",
            "evidence/compat/run_dir",
            "evidence/compat/scenarios",
            "evidence/compat/metadata.json",
        ],
    )?;
    run_evidence_family_boundary_verify()
}

pub(super) fn run_evidence_fault_verify() -> Result<(), String> {
    run_evidence_domain_verify(
        "fault",
        &["evidence/fault/classes", "evidence/fault/corrupt_runs", "evidence/fault/metadata.json"],
    )?;
    run_evidence_family_boundary_verify()
}

pub(super) fn run_evidence_perf_verify() -> Result<(), String> {
    run_evidence_domain_verify(
        "perf",
        &["evidence/perf/scenarios", "evidence/perf/baselines", "evidence/perf/metadata.json"],
    )?;
    run_perf_evidence_policy_verify()
}

pub(super) fn run_evidence_compare_verify() -> Result<(), String> {
    run_evidence_domain_verify(
        "compare",
        &[
            "evidence/compare/scenarios",
            "evidence/compare/baselines",
            "evidence/compare/metadata.json",
            "evidence/reports/comparison_fact_vs_interpretation.md",
        ],
    )?;
    run_compare_evidence_policy_verify()
}

pub(super) fn run_evidence_registry_verify_foundation_gate() -> Result<(), String> {
    run_evidence_registry_verify()?;
    Ok(())
}

pub(super) struct EvidenceFoundationStep {
    id: &'static str,
    description: &'static str,
    evidence_scope: &'static [&'static str],
    run: fn() -> Result<(), String>,
}

pub(super) fn run_evidence_foundation_reports_presence_verify() -> Result<(), String> {
    let root = repo_root()?;
    for rel in [
        "evidence/reports/evidence_audit_2026-03-07.md",
        "evidence/reports/evidence_topology_before_after.md",
        "evidence/reports/evidence_root_consolidation_report.md",
        "evidence/reports/release_evidence_strength_before_after.md",
        "evidence/reports/evidence_architecture_freeze_review_cycle.md",
        "evidence/reports/evidence_roast_memo_2026-03-07.md",
        "evidence/reports/evidence_root_contract_report.md",
        "evidence/reports/root_topology_before_after.md",
        "evidence/reports/evidence_verification_summary.md",
    ] {
        if !root.join(rel).exists() {
            return Err(format!(
                "foundation report required for evidence governance is missing: {rel}"
            ));
        }
    }
    Ok(())
}

pub(super) fn write_evidence_foundation_summary_report(
    root: &Path,
    rows: &[String],
    total_steps: usize,
) -> Result<PathBuf, String> {
    let out_rel = Path::new("artifacts/reports/evidence-foundation-verification-summary.md");
    let out_path = root.join(out_rel);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let mut markdown = String::new();
    markdown.push_str("# Evidence Foundation Verification Summary\n\n");
    markdown.push_str("This report describes each verification check executed by `verify evidence-foundation`, including the evidence surfaces validated by each check.\n\n");
    markdown.push_str("## Result\n\n");
    markdown.push_str(&format!("- Status: PASS\n- Checks executed: {}\n\n", total_steps));
    markdown.push_str("## Verification Matrix\n\n");
    markdown.push_str("| Step | What was verified | Evidence surfaces |\n");
    markdown.push_str("| --- | --- | --- |\n");
    for row in rows {
        markdown.push_str(row);
        markdown.push('\n');
    }

    fs::write(&out_path, markdown).map_err(|err| err.to_string())?;
    Ok(out_path)
}

pub(super) fn run_evidence_foundation_verify() -> Result<(), String> {
    let root = repo_root()?;
    let steps: [EvidenceFoundationStep; 16] = [
        EvidenceFoundationStep {
            id: "suite-policy",
            description: "suite policy contract and verify-command mapping",
            evidence_scope: &["configs/dag/policy/evidence_suite_policy.json"],
            run: run_evidence_suite_policy_verify,
        },
        EvidenceFoundationStep {
            id: "schema",
            description: "schema validity for governed evidence JSON assets",
            evidence_scope: &["configs/dag/schema/**", "evidence/**.json"],
            run: run_evidence_schema_verify,
        },
        EvidenceFoundationStep {
            id: "registry",
            description: "registry normalization, drift, orphan, and missing-file integrity",
            evidence_scope: &[
                "evidence/_meta/registries/evidence_registry.json",
                "evidence/ownership/evidence_ledger.json",
                "evidence/**",
            ],
            run: run_evidence_registry_verify_foundation_gate,
        },
        EvidenceFoundationStep {
            id: "ownership",
            description: "ledger ownership and consumer coverage contracts",
            evidence_scope: &["evidence/ownership/evidence_ledger.json"],
            run: run_evidence_ownership_verify,
        },
        EvidenceFoundationStep {
            id: "drift",
            description: "legacy-root drift prevention for evidence asset placement",
            evidence_scope: &[
                "repository-wide json surfaces",
                "configs/dag/policy/evidence_path_policy.json",
            ],
            run: run_evidence_drift_verify,
        },
        EvidenceFoundationStep {
            id: "consumers",
            description: "evidence consumer integrity and registry access boundaries",
            evidence_scope: &[
                "evidence/_meta/registries/evidence_registry.json",
                "crates/**/tests",
            ],
            run: run_evidence_consumers_verify,
        },
        EvidenceFoundationStep {
            id: "authoring",
            description: "authoring evidence domain contracts",
            evidence_scope: &["evidence/authoring/**", "evidence/authoring/metadata.json"],
            run: run_evidence_authoring_verify,
        },
        EvidenceFoundationStep {
            id: "battle",
            description: "battle scenario mapping and trust-property governance",
            evidence_scope: &["evidence/battle/**", "evidence/battle/metadata.json"],
            run: run_evidence_battle_verify,
        },
        EvidenceFoundationStep {
            id: "cache",
            description: "cache correctness and corruption evidence contracts",
            evidence_scope: &["evidence/cache/**", "evidence/cache/metadata.json"],
            run: run_evidence_cache_verify,
        },
        EvidenceFoundationStep {
            id: "replay",
            description: "replay-equivalence evidence contracts",
            evidence_scope: &["evidence/cache/replay/**", "evidence/cache/metadata.json"],
            run: run_evidence_replay_verify,
        },
        EvidenceFoundationStep {
            id: "compat",
            description: "compatibility evidence domain contracts",
            evidence_scope: &["evidence/compat/**", "evidence/compat/metadata.json"],
            run: run_evidence_compat_verify,
        },
        EvidenceFoundationStep {
            id: "fault",
            description: "fault evidence domain contracts",
            evidence_scope: &["evidence/fault/**", "evidence/fault/metadata.json"],
            run: run_evidence_fault_verify,
        },
        EvidenceFoundationStep {
            id: "perf",
            description: "performance evidence domain contracts and policy",
            evidence_scope: &["evidence/perf/**", "evidence/perf/metadata.json"],
            run: run_evidence_perf_verify,
        },
        EvidenceFoundationStep {
            id: "compare",
            description: "comparison evidence domain contracts and policy",
            evidence_scope: &["evidence/compare/**", "evidence/compare/metadata.json"],
            run: run_evidence_compare_verify,
        },
        EvidenceFoundationStep {
            id: "release-set",
            description: "release evidence set membership and classification integrity",
            evidence_scope: &[
                "evidence/release/release_evidence_set.json",
                "evidence/_meta/registries/evidence_registry.json",
            ],
            run: run_evidence_release_set_verify,
        },
        EvidenceFoundationStep {
            id: "required-reports",
            description: "required foundation governance reports are present",
            evidence_scope: &["evidence/reports/*.md"],
            run: run_evidence_foundation_reports_presence_verify,
        },
    ];

    println!("evidence foundation verification summary");
    let mut rows = Vec::new();
    for (index, step) in steps.iter().enumerate() {
        println!("  [{}/{}] {}: {}", index + 1, steps.len(), step.id, step.description);
        if let Err(err) = (step.run)() {
            return Err(format!("evidence foundation step `{}` failed: {}", step.id, err));
        }
        let surfaces = step.evidence_scope.join(", ");
        rows.push(format!("| `{}` | {} | `{}` |", step.id, step.description, surfaces));
    }

    let report_path = write_evidence_foundation_summary_report(&root, &rows, steps.len())?;
    println!("evidence foundation verification report: {}", report_path.display());
    Ok(())
}

pub(super) fn run_evidence_drift_verify() -> Result<(), String> {
    let root = repo_root()?;
    let legacy_roots = [
        "examples",
        "benchmarks/scenarios",
        "benchmarks/baselines",
        "comparisons/scenarios",
        "comparisons/bijux/baselines",
        "tests/e2e/fixtures",
        "tests/e2e/replay/fixtures",
        "tests/e2e/compat",
        "tests/e2e/container",
    ];
    let mut violations = Vec::new();
    for legacy_root in legacy_roots {
        let base = root.join(legacy_root);
        if !base.exists() {
            continue;
        }
        let mut files = Vec::new();
        collect_all_files(&base, &mut files)?;
        for file in files {
            let rel = file
                .strip_prefix(&root)
                .map_err(|err| err.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            if rel.ends_with(".json") || rel.ends_with(".dag.json") {
                violations.push(rel);
            }
        }
    }
    if let Err(err) = run_evidence_family_boundary_verify() {
        violations.push(format!("family-boundary-drift: {err}"));
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!("evidence drift violations detected: {}", violations.join(", ")))
    }
}

pub(super) fn run_evidence_resolve_by_id(asset_id: &str) -> Result<(), String> {
    let root = repo_root()?;
    let assets = load_registry_assets(&root)?;
    let asset = resolve_asset_by_id(&assets, asset_id)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&evidence_assets_as_json(&[asset]))
            .map_err(|err| err.to_string())?
    );
    Ok(())
}

pub(super) fn run_evidence_resolve_by_family(family: &str) -> Result<(), String> {
    let root = repo_root()?;
    let assets = load_registry_assets(&root)?;
    let resolved = resolve_assets_by_family(&assets, family);
    println!(
        "{}",
        serde_json::to_string_pretty(&evidence_assets_as_json(&resolved))
            .map_err(|err| err.to_string())?
    );
    Ok(())
}

pub(super) fn run_evidence_resolve_by_trust_property(trust_property: &str) -> Result<(), String> {
    let root = repo_root()?;
    let assets = load_registry_assets(&root)?;
    let resolved = resolve_assets_by_trust_property(&assets, trust_property);
    println!(
        "{}",
        serde_json::to_string_pretty(&evidence_assets_as_json(&resolved))
            .map_err(|err| err.to_string())?
    );
    Ok(())
}

pub(super) fn run_evidence_resolve_by_consumer(consumer: &str) -> Result<(), String> {
    let root = repo_root()?;
    let assets = load_registry_assets(&root)?;
    let resolved = resolve_assets_by_consumer(&assets, consumer);
    println!(
        "{}",
        serde_json::to_string_pretty(&evidence_assets_as_json(&resolved))
            .map_err(|err| err.to_string())?
    );
    Ok(())
}

pub(super) fn run_evidence_consumer_reports(
    assets_out: &Path,
    consumers_out: &Path,
) -> Result<(), String> {
    let root = repo_root()?;
    let assets = load_registry_assets(&root)?;
    let assets_report = render_assets_to_consumers_report(&assets);
    let consumers_report = render_consumers_to_families_report(&assets);
    fs::write(root.join(assets_out), assets_report).map_err(|err| err.to_string())?;
    fs::write(root.join(consumers_out), consumers_report).map_err(|err| err.to_string())?;
    println!(
        "{}",
        json!({"assets_report": assets_out.to_string_lossy(), "consumers_report": consumers_out.to_string_lossy()})
    );
    Ok(())
}

pub(super) fn run_evidence_consumers_verify() -> Result<(), String> {
    let root = repo_root()?;
    verify_registry_access_bypass(&root)?;
    let path_policy_payload =
        fs::read_to_string(root.join("configs/dag/policy/evidence_path_policy.json"))
            .map_err(|err| err.to_string())?;
    let path_policy: Value =
        serde_json::from_str(&path_policy_payload).map_err(|err| err.to_string())?;
    let restricted_patterns: Vec<String> = path_policy["legacy_scenario_roots"]
        .as_array()
        .ok_or_else(|| "legacy_scenario_roots must be an array".to_string())?
        .iter()
        .chain(
            path_policy["legacy_scenario_paths"]
                .as_array()
                .ok_or_else(|| "legacy_scenario_paths must be an array".to_string())?
                .iter(),
        )
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "legacy scenario pattern must be a string".to_string())
                .map(ToOwned::to_owned)
        })
        .collect::<Result<_, _>>()?;
    let mut violations = Vec::new();
    let mut files = Vec::new();
    for extension in ["rs", "md", "json", "toml"] {
        files.extend(repository_files_with_extension(&root, extension)?);
    }
    files.sort();
    files.dedup();
    let ignore_paths = [
        "crates/bijux-dev/src/commands/ops.rs",
        "crates/bijux-dev/src/commands/mod.rs",
        "crates/bijux-dev/tests/evidence_consumer_integrity_contracts.rs",
        "crates/bijux-dev/tests/evidence_access_contracts.rs",
        "docs/spec/TEST_EVIDENCE_CONSUMER_CONTRACT.md",
        "configs/dag/policy/evidence_governance.json",
        "configs/dag/policy/evidence_path_policy.json",
        "configs/dag/policy/release_evidence_policy.json",
    ];
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .map_err(|err| err.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        let ignore_prefixes = ["configs/dag/policy/", "evidence/", "evidence/dag/"];
        if ignore_paths.iter().any(|path| rel == *path)
            || ignore_prefixes.iter().any(|prefix| rel.starts_with(prefix))
            || rel.contains("/tests/")
            || rel.ends_with("/README.md")
        {
            continue;
        }
        let text = fs::read_to_string(&file).map_err(|err| err.to_string())?;
        for pattern in &restricted_patterns {
            if text.contains(pattern) {
                violations.push(format!("{rel}: contains legacy scenario reference `{pattern}`"));
            }
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(" | "))
    }
}

pub(super) fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

pub(crate) fn now_millis() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0)
}

#[derive(Debug, Deserialize)]
pub(super) struct TestTaxonomyPolicy {
    allowed_suffixes: Vec<String>,
    allowed_exact_names: Vec<String>,
    descriptive_path_segments: Vec<String>,
    shellout_allowed_suffixes: Vec<String>,
    shellout_allowed_path_segments: Vec<String>,
    required_families: Vec<TestTaxonomyFamily>,
}

#[derive(Debug, Deserialize)]
pub(super) struct TestTaxonomyFamily {
    name: String,
    #[serde(default)]
    prefix: Option<String>,
    #[serde(default)]
    suffix: Option<String>,
    #[serde(default)]
    exact_name: Option<String>,
    #[serde(default)]
    path_segment: Option<String>,
}

#[derive(Debug)]
struct TestFile {
    path: PathBuf,
    rel: String,
    name: String,
}

fn collect_test_files(root: &Path) -> Result<Vec<TestFile>, String> {
    let mut files = Vec::new();
    let mut dirs = vec![root.join("crates"), root.join("tests")];
    while let Some(dir) = dirs.pop() {
        if !dir.exists() {
            continue;
        }
        for entry in fs::read_dir(&dir).map_err(|err| err.to_string())? {
            let entry = entry.map_err(|err| err.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
                continue;
            }
            if path.extension().and_then(|v| v.to_str()) != Some("rs") {
                continue;
            }
            let rel = path
                .strip_prefix(root)
                .map_err(|err| err.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            if !rel.contains("/tests/") && !rel.starts_with("tests/") {
                continue;
            }
            let name = path.file_name().and_then(|v| v.to_str()).unwrap_or_default().to_string();
            files.push(TestFile { path, rel, name });
        }
    }
    Ok(files)
}

fn test_name_matches_suffix(name: &str, suffixes: &[String]) -> bool {
    suffixes.iter().any(|suffix| name.ends_with(suffix))
}

fn test_path_matches_segment(rel: &str, segments: &[String]) -> bool {
    segments.iter().any(|segment| rel.contains(segment))
}

fn test_taxonomy_family_matches(family: &TestTaxonomyFamily, rel: &str, name: &str) -> bool {
    family.prefix.as_ref().is_some_and(|prefix| name.starts_with(prefix))
        || family.suffix.as_ref().is_some_and(|suffix| name.ends_with(suffix))
        || family.exact_name.as_ref().is_some_and(|exact_name| name == exact_name)
        || family.path_segment.as_ref().is_some_and(|segment| rel.contains(segment))
}

fn read_first_existing_repo_file(root: &Path, candidates: &[&str]) -> Result<String, String> {
    for candidate in candidates {
        let path = root.join(candidate);
        if path.exists() {
            return fs::read_to_string(&path).map_err(|err| err.to_string());
        }
    }
    Err(format!("none of the expected coverage files exist: {}", candidates.join(", ")))
}

pub(super) fn run_test_taxonomy_guard() -> Result<(), String> {
    let root = repo_root()?;
    let policy_path = root.join("configs/dag/policy/test_taxonomy.json");
    let policy_text = fs::read_to_string(&policy_path).map_err(|err| err.to_string())?;
    let policy: TestTaxonomyPolicy =
        serde_json::from_str(&policy_text).map_err(|err| err.to_string())?;

    let mut violations = Vec::new();
    let allowed_exact_names: BTreeSet<String> = policy.allowed_exact_names.into_iter().collect();
    let files = collect_test_files(&root)?;

    for file in files {
        let named_by_suffix = test_name_matches_suffix(&file.name, &policy.allowed_suffixes);
        let named_by_exact_name = allowed_exact_names.contains(&file.name);
        let named_by_suite_path = file.name == "mod.rs"
            || test_path_matches_segment(&file.rel, &policy.descriptive_path_segments);
        if !(named_by_suffix || named_by_exact_name || named_by_suite_path) {
            violations.push(format!(
                "test file must use governed taxonomy family or suite path: {}",
                file.rel
            ));
        }

        let content = fs::read_to_string(&file.path).map_err(|err| err.to_string())?;
        let shells_out = content.contains("-p\", \"bijux-dag-cli\"")
            || content.contains("Command::new(\"cargo\")")
            || content.contains("Command::new(\"bijux\")");
        let shellout_allowed =
            test_name_matches_suffix(&file.name, &policy.shellout_allowed_suffixes)
                || test_path_matches_segment(&file.rel, &policy.shellout_allowed_path_segments);
        if shells_out && !shellout_allowed {
            violations.push(format!(
                "test file shells out outside governed black-box families: {}",
                file.rel
            ));
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

pub(super) fn run_test_classification_report() -> Result<(), String> {
    let root = repo_root()?;
    let policy_path = root.join("configs/dag/policy/test_taxonomy.json");
    let policy_text = fs::read_to_string(&policy_path).map_err(|err| err.to_string())?;
    let policy: TestTaxonomyPolicy =
        serde_json::from_str(&policy_text).map_err(|err| err.to_string())?;
    let mut counts: BTreeMap<String, u64> =
        policy.required_families.iter().map(|family| (family.name.clone(), 0_u64)).collect();

    for file in collect_test_files(&root)? {
        for family in &policy.required_families {
            if test_taxonomy_family_matches(family, &file.rel, &file.name) {
                if let Some(value) = counts.get_mut(&family.name) {
                    *value += 1;
                }
            }
        }
    }

    let missing: Vec<String> = counts
        .iter()
        .filter(|(_, count)| **count == 0)
        .map(|(category, _)| category.clone())
        .collect();

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "counts": counts,
            "missing_categories": missing,
        }))
        .map_err(|err| err.to_string())?
    );

    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!("missing test categories: {}", missing.join(", ")))
    }
}

pub(super) fn run_test_policy_guard() -> Result<(), String> {
    let root = repo_root()?;
    let mut violations = Vec::new();

    let schema_fixtures_ok =
        root.join("configs/dag/schema/fixtures").join("v0.1").join("positive").exists()
            && root.join("configs/dag/schema/fixtures").join("v0.1").join("negative").exists();
    if !schema_fixtures_ok {
        violations.push("schema fixtures must have positive and negative coverage".to_string());
    }

    let state_text = read_first_existing_repo_file(
        &root,
        &[
            "crates/bijux-dag-runtime/tests/state_machine_transitions.rs",
            "crates/bijux-dag-runtime/src/state_machine_tests.rs",
        ],
    )?;
    for state in [
        "NodeState::Pending",
        "NodeState::Eligible",
        "NodeState::Queued",
        "NodeState::Running",
        "NodeState::Success",
        "NodeState::Failed",
        "NodeState::Skipped",
        "NodeState::Cached",
        "NodeState::Cancelled",
    ] {
        if !state_text.contains(state) {
            violations.push(format!("runtime transition coverage missing node state: {state}"));
        }
    }

    let cache_text = read_first_existing_repo_file(
        &root,
        &[
            "crates/bijux-dag-runtime/src/internal/testing/tests_runtime.in.rs",
            "crates/bijux-dag-runtime/src/tests_runtime.in.rs",
        ],
    )?;
    for mode in ["CacheMode::Off", "CacheMode::Read", "CacheMode::ReadWrite"] {
        if !cache_text.contains(mode) {
            violations.push(format!("cache mode coverage missing mode: {mode}"));
        }
    }

    let output_contract = root.join("crates/bijux-dag-app/tests/output_contract.rs");
    let cli_contract = root.join("crates/bijux-dag-app/tests/cli_contract.rs");
    if !(output_contract.exists() && cli_contract.exists()) {
        violations.push(
            "public command policy requires integration and error-path app command tests"
                .to_string(),
        );
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

pub(super) fn run_e2e_matrix() -> Result<(), String> {
    let root = repo_root()?;
    run_with_root(
        &root,
        "cargo",
        &["test", "-p", "bijux-dag-app", "--test", "e2e_integration_scenarios"],
    )
    .and_then(|_| {
        run_with_root(
            &root,
            "cargo",
            &[
                "run",
                "-p",
                "bijux-dag-cli",
                "--bin",
                "bijux-dag",
                "--",
                "validate",
                "evidence/authoring/examples/hello.dag.json",
            ],
        )
    })
}

pub(super) fn command_stdout(root: &Path, bin: &str, args: &[&str]) -> Result<String, String> {
    exec_command_stdout(root, bin, args)
}

#[derive(Debug, Deserialize)]
pub(super) struct FaultClassCatalog {
    fault_classes: Vec<FaultClassEntry>,
}

#[derive(Debug, Deserialize)]
pub(super) struct FaultClassEntry {
    id: String,
    tested_by: Vec<String>,
}

pub(super) fn run_fault_summary_report() -> Result<(), String> {
    let root = repo_root()?;
    let catalog_path = root.join("evidence/fault/classes/fault_classes.json");
    let metadata_path = root.join("evidence/fault/metadata.json");
    if !metadata_path.exists() {
        return Err("missing fault metadata: evidence/fault/metadata.json".to_string());
    }
    let payload = fs::read_to_string(&catalog_path).map_err(|err| err.to_string())?;
    let catalog: FaultClassCatalog =
        serde_json::from_str(&payload).map_err(|err| err.to_string())?;

    let mut tested = Vec::new();
    let mut missing = Vec::new();
    for entry in catalog.fault_classes {
        if entry.tested_by.is_empty() {
            missing.push(entry.id);
        } else {
            tested.push(json!({"id": entry.id, "tests": entry.tested_by}));
        }
    }

    let summary = json!({
        "tested_fault_classes": tested,
        "missing_fault_classes": missing,
    });
    println!("{}", serde_json::to_string_pretty(&summary).map_err(|err| err.to_string())?);
    if summary["missing_fault_classes"].as_array().is_some_and(|items| items.is_empty()) {
        Ok(())
    } else {
        Err("fault class catalog has missing tested_by mappings".to_string())
    }
}

pub(super) fn run_benchmark_compare(
    current: &Path,
    baseline: &Path,
    max_regression_ratio: f64,
) -> Result<(), String> {
    let root = repo_root()?;
    let current_path = root.join(current);
    let baseline_path = root.join(baseline);

    let current_json: Value =
        serde_json::from_str(&fs::read_to_string(current_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    let baseline_json: Value =
        serde_json::from_str(&fs::read_to_string(baseline_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;

    let current_items = current_json
        .get("scenario_results")
        .and_then(Value::as_array)
        .ok_or_else(|| "current benchmark report missing scenario_results".to_string())?;
    let baseline_items = baseline_json
        .get("scenario_results")
        .and_then(Value::as_array)
        .ok_or_else(|| "baseline benchmark report missing scenario_results".to_string())?;

    let mut base_map: std::collections::BTreeMap<String, f64> = std::collections::BTreeMap::new();
    for item in baseline_items {
        let scenario = item
            .get("scenario_id")
            .and_then(Value::as_str)
            .ok_or_else(|| "baseline item missing scenario_id".to_string())?;
        let elapsed = item
            .get("elapsed_ms")
            .and_then(Value::as_f64)
            .ok_or_else(|| "baseline item missing elapsed_ms".to_string())?;
        base_map.insert(scenario.to_string(), elapsed);
    }

    let mut regressions = Vec::new();
    for item in current_items {
        let scenario = item
            .get("scenario_id")
            .and_then(Value::as_str)
            .ok_or_else(|| "current item missing scenario_id".to_string())?;
        let elapsed = item
            .get("elapsed_ms")
            .and_then(Value::as_f64)
            .ok_or_else(|| "current item missing elapsed_ms".to_string())?;
        if let Some(base) = base_map.get(scenario) {
            if *base > 0.0 {
                let ratio = (elapsed - *base) / *base;
                if ratio > max_regression_ratio {
                    regressions.push(json!({
                        "scenario_id": scenario,
                        "baseline_ms": base,
                        "current_ms": elapsed,
                        "regression_ratio": ratio
                    }));
                }
            }
        }
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({"regressions": regressions}))
            .map_err(|err| err.to_string())?
    );

    if regressions.is_empty() {
        Ok(())
    } else {
        Err("benchmark regressions exceed threshold".to_string())
    }
}

pub(super) fn run_performance_claims_guard() -> Result<(), String> {
    let root = repo_root()?;
    let docs = root.join("docs");
    let evidence_markers = [
        "artifacts/benchmarks",
        "benchmarks/",
        "evidence/perf/",
        "evidence/compare/",
        "performance-evidence-report",
        "benchmark_report",
    ];
    let mut violations = Vec::new();
    let mut stack = vec![docs];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).map_err(|err| err.to_string())? {
            let entry = entry.map_err(|err| err.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|v| v.to_str()) != Some("md") {
                continue;
            }
            let rel = path
                .strip_prefix(&root)
                .map_err(|err| err.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            let text = fs::read_to_string(&path).map_err(|err| err.to_string())?;
            let has_evidence = evidence_markers.iter().any(|marker| text.contains(marker))
                || text.contains("PERFORMANCE_STRATEGY.md")
                || text.contains("PERFORMANCE_CONTRACT.md");

            let mut in_frontmatter = false;
            let mut in_code_block = false;
            let mut first_claim = None;
            for (index, line) in text.lines().enumerate() {
                let trimmed = line.trim();
                if index == 0 && trimmed == "---" {
                    in_frontmatter = true;
                    continue;
                }
                if in_frontmatter {
                    if trimmed == "---" {
                        in_frontmatter = false;
                    }
                    continue;
                }
                if trimmed.starts_with("```") {
                    in_code_block = !in_code_block;
                    continue;
                }
                if in_code_block
                    || trimmed.is_empty()
                    || trimmed.starts_with('#')
                    || trimmed.starts_with('|')
                    || trimmed.starts_with('<')
                    || trimmed.starts_with("- [")
                    || trimmed.starts_with("* [")
                    || trimmed.starts_with("[")
                    || trimmed.contains("`performance`")
                {
                    continue;
                }

                let lower = trimmed.to_ascii_lowercase();
                let claim = lower.contains("performance")
                    || lower.contains("latency")
                    || lower.contains("throughput")
                    || lower.contains("scaling")
                    || lower.contains("bounded-memory");
                if claim {
                    first_claim.get_or_insert_with(|| trimmed.to_string());
                }
            }

            if let Some(line) = first_claim.filter(|_| !has_evidence) {
                violations.push(format!("{rel}: performance claim without evidence link: {line}"));
            }
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

pub(super) fn run_resource_profile_summary(report: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let report_path = root.join(report);
    let payload = fs::read_to_string(&report_path).map_err(|err| err.to_string())?;
    let report_json: Value = serde_json::from_str(&payload).map_err(|err| err.to_string())?;

    let mut summary = json!({
        "measurement_quality": "approximate",
        "scenario_count": 0,
        "totals": {
            "wall_time_ms": 0.0,
            "artifact_bytes": 0,
            "trace_bytes": 0
        },
        "cost_split": {
            "product_execution_ms": 0.0,
            "harness_overhead_ms": 0.0
        }
    });

    if let Some(items) = report_json.get("scenario_results").and_then(Value::as_array) {
        let mut wall = 0.0_f64;
        for item in items {
            wall += item.get("elapsed_ms").and_then(Value::as_f64).unwrap_or(0.0);
        }
        summary["scenario_count"] = Value::from(items.len() as u64);
        summary["totals"]["wall_time_ms"] = Value::from(wall);
        summary["cost_split"]["product_execution_ms"] = Value::from(wall);
        summary["cost_split"]["harness_overhead_ms"] = Value::from(0.0_f64);
    }

    println!("{}", serde_json::to_string_pretty(&summary).map_err(|err| err.to_string())?);
    Ok(())
}

pub(super) fn run_resource_budget_check(report: &Path, gate: bool) -> Result<(), String> {
    let root = repo_root()?;
    let report_path = root.join(report);
    let budgets_path = root.join("evidence/perf/scenarios/resource_budgets.json");

    if !report_path.exists() {
        if gate {
            return Err(format!("missing resource budget report: {}", report_path.display()));
        }
        eprintln!(
            "resource-budget-warning: benchmark report unavailable at {}",
            report_path.display()
        );
        return Ok(());
    }

    let report_json: Value =
        serde_json::from_str(&fs::read_to_string(&report_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    let budgets_json: Value =
        serde_json::from_str(&fs::read_to_string(&budgets_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;

    let mut budget_map: std::collections::BTreeMap<String, Value> =
        std::collections::BTreeMap::new();
    if let Some(items) = budgets_json.get("scenarios").and_then(Value::as_array) {
        for item in items {
            if let Some(id) = item.get("scenario_id").and_then(Value::as_str) {
                budget_map.insert(id.to_string(), item.clone());
            }
        }
    }

    let mut warnings = Vec::new();
    if let Some(items) = report_json.get("scenario_results").and_then(Value::as_array) {
        for item in items {
            let scenario = item.get("scenario_id").and_then(Value::as_str).unwrap_or_default();
            let elapsed = item.get("elapsed_ms").and_then(Value::as_f64).unwrap_or(0.0);
            if let Some(budget) = budget_map.get(scenario) {
                let approx_budget_ms =
                    budget.get("max_manifest_bytes").and_then(Value::as_u64).unwrap_or(0) as f64;
                if approx_budget_ms > 0.0 && elapsed > approx_budget_ms {
                    warnings.push(format!(
                        "scenario {} exceeded approximate budget threshold (elapsed_ms={elapsed})",
                        scenario
                    ));
                }
            }
        }
    }

    if warnings.is_empty() {
        println!("resource budgets within thresholds");
        return Ok(());
    }

    for warning in &warnings {
        eprintln!("resource-budget-warning: {warning}");
    }
    if gate {
        Err("resource budget check failed in gate mode".to_string())
    } else {
        Ok(())
    }
}

pub(super) fn run_resource_trend_append(report: &Path, trend: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let report_json: Value = serde_json::from_str(
        &fs::read_to_string(root.join(report)).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;

    let trend_path = root.join(trend);
    let mut trend_json: Value = if trend_path.exists() {
        serde_json::from_str(&fs::read_to_string(&trend_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?
    } else {
        json!({"trend_format":"resource-trend/v1","series":[]})
    };

    let entry = json!({
        "commit_sha": report_json.get("commit_sha").cloned().unwrap_or(Value::from("unknown")),
        "timestamp_unix_ms": now_millis(),
        "scenario_results": report_json
            .get("scenario_results")
            .cloned()
            .unwrap_or(Value::Array(Vec::new()))
    });

    if let Some(series) = trend_json.get_mut("series").and_then(Value::as_array_mut) {
        series.push(entry);
    }

    fs::write(trend_path, serde_json::to_vec_pretty(&trend_json).map_err(|err| err.to_string())?)
        .map_err(|err| err.to_string())
}

pub(super) fn dir_size_bytes(path: &Path) -> Result<u64, String> {
    if !path.exists() {
        return Ok(0);
    }
    let mut total = 0_u64;
    let mut stack = vec![path.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).map_err(|err| err.to_string())? {
            let entry = entry.map_err(|err| err.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                total =
                    total.saturating_add(entry.metadata().map_err(|err| err.to_string())?.len());
            }
        }
    }
    Ok(total)
}

pub(super) fn estimate_trace_bytes(path: &Path) -> Result<u64, String> {
    if !path.exists() {
        return Ok(0);
    }
    let mut total = 0_u64;
    let mut stack = vec![path.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).map_err(|err| err.to_string())? {
            let entry = entry.map_err(|err| err.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .file_name()
                .and_then(|v| v.to_str())
                .is_some_and(|name| name == "trace.json")
            {
                total =
                    total.saturating_add(entry.metadata().map_err(|err| err.to_string())?.len());
            }
        }
    }
    Ok(total)
}

pub(super) fn run_runtime_semantics_guard() -> Result<(), String> {
    let root = repo_root()?;
    for rel in [
        "docs/spec/RUNTIME_SEMANTICS_CONTRACT.md",
        "crates/bijux-dag-runtime/src/runtime_core/governance/semantics.rs",
        "crates/bijux-dag-runtime/tests/runtime_semantics_contracts.rs",
        "crates/bijux-dag-runtime/tests/engine_correctness_contracts.rs",
    ] {
        if !root.join(rel).exists() {
            return Err(format!("missing runtime semantics artifact: {rel}"));
        }
    }
    Ok(())
}

pub(super) fn run_test_trust_foundation_guard() -> Result<(), String> {
    let root = repo_root()?;
    for rel in [
        "docs/spec/TEST_TRUST_CONTRACT.md",
        "docs/spec/TEST_PHILOSOPHY.md",
        "docs/bijux-core/governance/trust-evidence.md",
        "crates/bijux-dag-runtime/tests/fixtures/test_trust_catalog.json",
    ] {
        if !root.join(rel).exists() {
            return Err(format!("missing test trust artifact: {rel}"));
        }
    }

    let catalog: Value = serde_json::from_str(
        &fs::read_to_string(
            root.join("crates/bijux-dag-runtime/tests/fixtures/test_trust_catalog.json"),
        )
        .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let object = catalog
        .as_object()
        .ok_or_else(|| "test_trust_catalog.json must be an object".to_string())?;
    if object.is_empty() {
        return Err("test_trust_catalog.json must contain at least one class".to_string());
    }
    for (class, files) in object {
        let files =
            files.as_array().ok_or_else(|| format!("catalog class `{class}` must be an array"))?;
        if files.is_empty() {
            return Err(format!("catalog class `{class}` must not be empty"));
        }
        for file in files {
            let rel = file
                .as_str()
                .ok_or_else(|| format!("catalog class `{class}` contains non-string entry"))?;
            let full = root.join("crates/bijux-dag-runtime/tests").join(rel);
            if !full.exists() {
                return Err(format!(
                    "catalog references missing test file: crates/bijux-dag-runtime/tests/{rel}"
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn run_battle_suite_mandatory_guard() -> Result<(), String> {
    let root = repo_root()?;

    let policy_path = root.join("configs/dag/policy/battle_trust_properties.json");
    let metadata_path = root.join("evidence/battle/metadata.json");
    let harness_path =
        root.join("crates/bijux-dag-runtime/tests/battle_workflow_harness_contracts.rs");

    for required in [&policy_path, &metadata_path, &harness_path] {
        if !required.exists() {
            return Err(format!("missing battle suite artifact: {}", required.display()));
        }
    }

    let policy: Value =
        serde_json::from_str(&fs::read_to_string(&policy_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;

    let trust_properties = policy
        .get("trust_properties")
        .and_then(Value::as_array)
        .ok_or_else(|| "battle trust policy missing trust_properties".to_string())?;
    if trust_properties.len() < 12 {
        return Err("battle trust policy must define at least 12 trust properties".to_string());
    }
    let has_plan_truth = trust_properties.iter().any(|property| {
        property.get("id").and_then(Value::as_str).is_some_and(|id| id == "tp_plan_truth")
    });
    if !has_plan_truth {
        return Err("battle trust policy must include tp_plan_truth".to_string());
    }
    let has_state_machine_legality = trust_properties.iter().any(|property| {
        property
            .get("id")
            .and_then(Value::as_str)
            .is_some_and(|id| id == "tp_state_machine_legality")
    });
    if !has_state_machine_legality {
        return Err("battle trust policy must include tp_state_machine_legality".to_string());
    }
    let has_config_policy_determinism = trust_properties.iter().any(|property| {
        property
            .get("id")
            .and_then(Value::as_str)
            .is_some_and(|id| id == "tp_config_policy_determinism")
    });
    if !has_config_policy_determinism {
        return Err("battle trust policy must include tp_config_policy_determinism".to_string());
    }

    let required_scenarios = policy
        .get("required_scenarios")
        .and_then(Value::as_array)
        .ok_or_else(|| "battle trust policy missing required_scenarios".to_string())?;
    if required_scenarios.is_empty() {
        return Err("battle trust policy required_scenarios must not be empty".to_string());
    }
    let scenario_trust_map = policy
        .get("scenario_trust_map")
        .and_then(Value::as_object)
        .ok_or_else(|| "battle trust policy missing scenario_trust_map".to_string())?;
    let state_machine_mapped = scenario_trust_map.values().any(|value| {
        value.as_array().is_some_and(|ids| {
            ids.iter().any(|id| id.as_str().is_some_and(|v| v == "tp_state_machine_legality"))
        })
    });
    if !state_machine_mapped {
        return Err(
            "battle trust policy must map at least one scenario to tp_state_machine_legality"
                .to_string(),
        );
    }

    run_battle_scenario_mapping_validate()?;

    let metadata: Value =
        serde_json::from_str(&fs::read_to_string(&metadata_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    if metadata.get("scenarios").and_then(Value::as_object).is_none() {
        return Err("battle metadata must define scenario ownership".to_string());
    }

    Ok(())
}

pub(super) fn run_test_trust_maintenance_guard() -> Result<(), String> {
    let root = repo_root()?;
    let ledger = root.join("configs/dag/policy/test_trust_ledger.json");
    if !ledger.exists() {
        return Err("missing test trust ledger policy".to_string());
    }

    let docs = root.join("docs/spec/TEST_TRUST_LEDGER.md");
    if !docs.exists() {
        return Err("missing test trust ledger spec".to_string());
    }

    let report = root.join("docs/reports/foundation/TEST_TRUST_MAINTENANCE_REPORT.md");
    if !report.exists() {
        return Err("missing test trust maintenance report".to_string());
    }

    let policy: Value =
        serde_json::from_str(&fs::read_to_string(&ledger).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;

    let classes = policy
        .get("classification_rules")
        .and_then(Value::as_array)
        .ok_or_else(|| "test trust ledger missing classification_rules".to_string())?;
    if classes.is_empty() {
        return Err("classification_rules must not be empty".to_string());
    }

    let must_never_break = policy
        .get("must_never_break")
        .and_then(Value::as_array)
        .ok_or_else(|| "test trust ledger missing must_never_break".to_string())?;
    if must_never_break.is_empty() {
        return Err("must_never_break must not be empty".to_string());
    }

    Ok(())
}

pub(super) fn run_planner_alignment_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        bijux_dag_core::planner_alignment_required_doc(),
        bijux_dag_core::planner_alignment_required_schema(),
        bijux_dag_core::planner_alignment_required_test(),
        "crates/bijux-dag-runtime/tests/planner_lowering_contracts.rs",
        "crates/bijux-dev/tests/planner_hardening_contracts.rs",
        "docs/reports/foundation/PLANNER_HARDENING_REPORT.md",
        "docs/spec/BATTLE_TRUST_PROPERTIES.md",
        "configs/dag/policy/battle_trust_properties.json",
        "crates/bijux-dag-runtime/src/runtime_core/planning/planner.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!("planner alignment missing required surfaces: {}", missing.join(", ")));
    }

    let planner_contract =
        fs::read_to_string(root.join(bijux_dag_core::planner_alignment_required_doc()))
            .map_err(|err| err.to_string())?;
    for required_token in [
        "parsed graph",
        "validated graph",
        "canonical graph",
        "execution plan",
        "P4021",
        "dag plan-dump",
    ] {
        if !planner_contract.contains(required_token) {
            return Err(format!("planner contract missing required token: {required_token}"));
        }
    }

    let commands = fs::read_to_string(root.join("crates/bijux-dev/src/commands/mod.rs"))
        .map_err(|err| err.to_string())?;
    for required_command in ["DagCommand::PlanDump", "run_dag_plan_dump"] {
        if !commands.contains(required_command) {
            return Err(format!("planner alignment missing command surface: {required_command}"));
        }
    }

    let policy: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/dag/policy/battle_trust_properties.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let has_plan_truth =
        policy.get("trust_properties").and_then(Value::as_array).is_some_and(|entries| {
            entries.iter().any(|entry| {
                entry.get("id").and_then(Value::as_str).is_some_and(|id| id == "tp_plan_truth")
            })
        });
    if !has_plan_truth {
        return Err("planner alignment requires tp_plan_truth trust property".to_string());
    }

    Ok(())
}

pub(super) fn run_scheduler_invariants_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/SCHEDULER_CONTRACT.md",
        "docs/spec/SCHEDULER_STATE_TRANSITIONS.md",
        "docs/reports/foundation/SCHEDULER_HARDENING_REPORT.md",
        "crates/bijux-dag-runtime/tests/scheduler_contract.rs",
        "crates/bijux-dag-runtime/tests/runtime_scheduler_determinism_contracts.rs",
        "crates/bijux-dev/tests/scheduler_hardening_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "scheduler invariant coverage missing required surfaces: {}",
            missing.join(", ")
        ));
    }
    let commands = fs::read_to_string(root.join("crates/bijux-dev/src/commands/mod.rs"))
        .map_err(|err| err.to_string())?;
    for required in ["DagCommand::SchedulerTimeline", "run_dag_scheduler_timeline"] {
        if !commands.contains(required) {
            return Err(format!(
                "scheduler invariant coverage missing command surface: {required}"
            ));
        }
    }
    Ok(())
}

pub(super) fn run_state_machine_contract_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/STATE_MACHINE_CONTRACT.md",
        "docs/spec/STATE_MACHINE_VISUALIZATION.md",
        "docs/reports/foundation/STATE_MACHINE_HARDENING_REPORT.md",
        "crates/bijux-dag-runtime/tests/state_machine_transitions.rs",
        "crates/bijux-dag-runtime/tests/state_machine_contracts.rs",
        "crates/bijux-dag-runtime/tests/runtime_state_machine_contracts.rs",
        "crates/bijux-dag-runtime/tests/fixtures/state_machine/evolution_trace.json",
        "crates/bijux-dag-runtime/tests/fixtures/state_machine/cancellation_trace.json",
        "crates/bijux-dev/tests/state_machine_hardening_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "state machine contract missing required surfaces: {}",
            missing.join(", ")
        ));
    }

    let contract = fs::read_to_string(root.join("docs/spec/STATE_MACHINE_CONTRACT.md"))
        .map_err(|err| err.to_string())?;
    let node_states = [
        "pending",
        "eligible",
        "queued",
        "running",
        "success",
        "failed",
        "skipped",
        "cached",
        "cancelled",
    ];
    for state in node_states {
        if !contract.contains(&format!("- {}", state)) {
            return Err(format!(
                "state machine contract missing documented node state `{}`",
                state
            ));
        }
    }
    let run_states = [
        "submitted",
        "planning",
        "running",
        "paused",
        "interrupted",
        "cancelling",
        "cancelled",
        "failed",
        "succeeded",
    ];
    for state in run_states {
        if !contract.contains(&format!("- {}", state)) {
            return Err(format!("state machine contract missing documented run state `{}`", state));
        }
    }
    for token in [
        "INV-NODE-TRANSITION-*",
        "INV-NODE-TERMINAL-REVERT-001",
        "INV-RUN-TRANSITION-*",
        "INV-RUN-FAILED-CAUSAL-001",
    ] {
        if !contract.contains(token) {
            return Err(format!(
                "state machine contract missing documented invariant token `{}`",
                token
            ));
        }
    }

    let commands = fs::read_to_string(root.join("crates/bijux-dev/src/commands/mod.rs"))
        .map_err(|err| err.to_string())?;
    for required_surface in ["DagCommand::VerifyState", "run_dag_verify_state"] {
        if !commands.contains(required_surface) {
            return Err(format!(
                "state machine contract missing command surface `{}`",
                required_surface
            ));
        }
    }
    Ok(())
}

pub(super) fn run_concurrency_model_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/CONCURRENCY_MODEL.md",
        "docs/bijux-dag/architecture/runtime-concurrency-boundaries.md",
        "docs/reports/governance/concurrency-flake-ledger.md",
        "crates/bijux-dag-runtime/tests/concurrency_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!("concurrency model missing required surfaces: {}", missing.join(", ")))
    }
}

pub(super) fn run_runtime_unsafe_guard() -> Result<(), String> {
    let root = repo_root()?;
    let unsafe_pattern = r"\bunsafe\b(?=\s*(\{|fn\b|impl\b|trait\b|extern\b))";
    let output = Command::new("rg")
        .args(["-n", unsafe_pattern, "crates/bijux-dag-runtime/src"])
        .current_dir(&root)
        .output()
        .map_err(|err| err.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let findings = stdout.lines().filter(|line| !line.trim().is_empty()).collect::<Vec<_>>();
    if findings.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "runtime unsafe usage requires ADR and dedicated tests: {}",
            findings.join(" | ")
        ))
    }
}

pub(super) fn run_unsafe_audit_report() -> Result<(), String> {
    let root = repo_root()?;
    let unsafe_pattern = r"\bunsafe\b(?=\s*(\{|fn\b|impl\b|trait\b|extern\b))";
    let output = Command::new("rg")
        .args(["-n", unsafe_pattern, "crates"])
        .current_dir(&root)
        .output()
        .map_err(|err| err.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let entries = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let mut parts = line.splitn(3, ':');
            json!({
                "file": parts.next().unwrap_or(""),
                "line": parts.next().unwrap_or(""),
                "snippet": parts.next().unwrap_or("").trim(),
            })
        })
        .collect::<Vec<_>>();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "unsafe_entry_count": entries.len(),
            "entries": entries
        }))
        .map_err(|err| err.to_string())?
    );
    Ok(())
}

pub(super) fn run_backend_contract_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/BACKEND_CONTRACT.md",
        "docs/spec/EXECUTION_ENGINE_CONTRACT.md",
        "docs/spec/ATTEMPT_TRACE_SCHEMA_V0.1.md",
        "docs/reports/foundation/BACKEND_HARDENING_REPORT.md",
        "docs/bijux-dag/architecture/engine-backend-responsibilities.md",
        "crates/bijux-dag-runtime/src/backend/runtime/execution_backend.rs",
        "crates/bijux-dag-runtime/tests/execution_backend_contract.rs",
        "crates/bijux-dev/tests/backend_hardening_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!("backend contract missing required surfaces: {}", missing.join(", ")));
    }

    let test_path = root.join("crates/bijux-dag-runtime/tests/execution_backend_contract.rs");
    let payload = fs::read_to_string(&test_path).map_err(|err| err.to_string())?;
    if !payload.contains("fake_and_process_like_backends_have_parity_on_basic_scenario") {
        return Err("backend contract missing fake-backend parity test".to_string());
    }
    for required_test in [
        "backend_prepare_failures_are_classified_correctly",
        "backend_launch_failures_do_not_corrupt_state",
        "cleanup_runs_after_observe_and_reports_cleanup_failures",
        "cleanup_runs_when_prepare_fails",
        "backend_observe_timeout_has_distinct_error",
        "backend_env_shaping_contract_is_explicitly_applied",
        "backend_output_collection_rejects_undeclared_outputs",
        "backend_registry_includes_capability_descriptors",
    ] {
        if !payload.contains(required_test) {
            return Err(format!(
                "backend contract missing required conformance test `{}`",
                required_test
            ));
        }
    }
    let backend_src = fs::read_to_string(
        root.join("crates/bijux-dag-runtime/src/backend/runtime/execution_backend.rs"),
    )
    .map_err(|err| err.to_string())?;
    let implementation_count = backend_src.matches("impl ExecutionBackend for").count();
    if implementation_count > 2 {
        return Err(
            "new backend implementations are blocked until backend contract conformance remains explicit and passing"
                .to_string(),
        );
    }
    Ok(())
}

pub(super) fn run_backend_registry_report() -> Result<(), String> {
    let registry = bijux_dag_runtime::backend_registry();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "backend_count": registry.len(),
            "backends": registry
        }))
        .map_err(|err| err.to_string())?
    );
    Ok(())
}

pub(super) fn run_storage_boundary_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/STORAGE_CONTRACT.md",
        "docs/bijux-dag/architecture/storage-layout-ownership.md",
        "crates/bijux-dag-runtime/src/artifacts/storage/store.rs",
        "crates/bijux-dag-runtime/tests/storage_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "storage boundaries missing required surfaces: {}",
            missing.join(", ")
        ));
    }

    let runtime_src = root.join("crates/bijux-dag-runtime/src");
    let mut violations = Vec::new();
    let mut stack = vec![runtime_src];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).map_err(|err| err.to_string())? {
            let entry = entry.map_err(|err| err.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|v| v.to_str()) != Some("rs") {
                continue;
            }
            let rel = path
                .strip_prefix(&root)
                .map_err(|err| err.to_string())?
                .to_string_lossy()
                .to_string();
            if rel.ends_with("store.rs")
                || rel.ends_with("lib.rs")
                || rel.ends_with("engine.rs")
                || rel.contains("/internal/testing/")
                || rel.ends_with("internal/control/runtime_controls.rs")
                || rel.ends_with("diagnostics/runtime/observability_deep.rs")
            {
                continue;
            }
            let text = fs::read_to_string(&path).map_err(|err| err.to_string())?;
            if text.contains("staging_path().join(\"nodes\")")
                || text.contains("manifest.json")
                || text.contains("outputs.index.json")
            {
                violations.push(rel);
            }
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "runtime modules use ad-hoc storage paths outside approved modules: {}",
            violations.join(", ")
        ))
    }
}

pub(super) fn run_storage_health(run_dir: &Path, cache_dir: Option<&Path>) -> Result<(), String> {
    let root = repo_root()?;
    let run_path = root.join(run_dir);
    let mut anomalies = Vec::new();
    let manifest = run_path.join("manifest.json");
    if !manifest.exists() {
        anomalies.push("missing manifest.json".to_string());
    } else {
        let payload = fs::read_to_string(&manifest).map_err(|err| err.to_string())?;
        let parsed: Value = serde_json::from_str(&payload).map_err(|err| err.to_string())?;
        if parsed.get("run_id").is_none() {
            anomalies.push("manifest missing run_id".to_string());
        }
    }
    let outputs = run_path.join("outputs.index.json");
    if !outputs.exists() {
        anomalies.push("missing outputs.index.json".to_string());
    }
    if let Some(cache_path) = cache_dir {
        let cache_abs = root.join(cache_path);
        if cache_abs.exists() {
            for entry in fs::read_dir(&cache_abs).map_err(|err| err.to_string())? {
                let entry = entry.map_err(|err| err.to_string())?;
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let meta = path.join("meta.json");
                if !meta.exists() {
                    anomalies.push(format!("cache entry missing meta.json: {}", path.display()));
                    continue;
                }
                let payload = fs::read_to_string(&meta).map_err(|err| err.to_string())?;
                let parsed: Value =
                    serde_json::from_str(&payload).map_err(|err| err.to_string())?;
                if parsed.get("fingerprint").is_none() {
                    anomalies.push(format!("cache meta missing fingerprint: {}", meta.display()));
                }
            }
        }
    }
    let response = json!({
        "run_dir": run_dir,
        "cache_dir": cache_dir,
        "healthy": anomalies.is_empty(),
        "anomalies": anomalies
    });
    println!("{}", serde_json::to_string_pretty(&response).map_err(|err| err.to_string())?);
    Ok(())
}

pub(super) fn run_run_dir_audit(run_dir: &Path, strict: bool) -> Result<(), String> {
    let root = repo_root()?;
    let mode = if strict {
        bijux_dag_artifacts::VerificationMode::Strict
    } else {
        bijux_dag_artifacts::VerificationMode::Standard
    };
    let report = bijux_dag_artifacts::verify_run_dir(root.join(run_dir), mode)
        .map_err(|err| err.to_string())?;
    println!("{}", serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?);
    Ok(())
}

pub(super) fn run_artifact_hardening_guard() -> Result<(), String> {
    let root = repo_root()?;
    for rel in [
        "docs/spec/RUN_DIR_STORAGE_CONTRACT.md",
        "docs/spec/RUN_DIR_CONTRACT.md",
        "docs/spec/RUN_DIR_OWNERSHIP.md",
        "docs/spec/IMPORT_EXPORT_CONTRACT.md",
        "docs/reports/foundation/RUN_DIR_IMPORT_EXPORT_HARDENING_REPORT.md",
        "docs/spec/ARTIFACT_OWNERSHIP_TABLE.md",
        "docs/spec/ARTIFACT_LIFECYCLE.md",
        "configs/dag/schema/operator/run_verify_report.schema.json",
        "evidence/compat/export_bundle/v0_1_supported/bundle.json",
        "evidence/compat/export_bundle/unsupported_older_version/bundle.json",
        "crates/bijux-dag-app/tests/run_dir_import_export_contract.rs",
        "crates/bijux-dag-artifacts/src/storage/hardening.rs",
        "crates/bijux-dag-artifacts/tests/artifact_hardening_contracts.rs",
        "crates/bijux-dev/tests/run_dir_import_export_hardening_contracts.rs",
        "evidence/fault/corrupt_runs/missing_manifest_version.json",
        "evidence/fault/corrupt_runs/invalid_outputs_index.json",
    ] {
        if !root.join(rel).exists() {
            return Err(format!("missing artifact hardening artifact: {rel}"));
        }
    }
    let run_dir_contract = fs::read_to_string(root.join("docs/spec/RUN_DIR_CONTRACT.md"))
        .map_err(|err| err.to_string())?;
    for token in [
        "Required entries (authoritative)",
        "Optional entries",
        "Derived artifacts (non-authoritative)",
        "Verification behavior",
        "dag verify --strict",
    ] {
        if !run_dir_contract.contains(token) {
            return Err(format!("run-dir contract missing required section `{token}`"));
        }
    }
    let import_export_contract =
        fs::read_to_string(root.join("docs/spec/IMPORT_EXPORT_CONTRACT.md"))
            .map_err(|err| err.to_string())?;
    for token in [
        "Bundle versioning",
        "export-bundle/v0.1",
        "dag export --manifest-only",
        "dag export --with-files",
        "provenance.source",
    ] {
        if !import_export_contract.contains(token) {
            return Err(format!("import/export contract missing required section `{token}`"));
        }
    }
    let policy: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/dag/policy/battle_trust_properties.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let trust_properties = policy
        .get("trust_properties")
        .and_then(Value::as_array)
        .ok_or_else(|| "battle trust policy missing trust_properties".to_string())?;
    for required in ["tp_run_dir_resilience", "tp_import_export_compatibility"] {
        let present = trust_properties.iter().any(|property| {
            property.get("id").and_then(Value::as_str).is_some_and(|id| id == required)
        });
        if !present {
            return Err(format!("artifact hardening requires trust property `{required}`"));
        }
    }
    Ok(())
}

pub(super) fn run_observability_contract_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/OBSERVABILITY_CONTRACT.md",
        "docs/reports/governance/OBSERVABILITY_SURFACE_COVERAGE.md",
        "crates/bijux-dag-runtime/tests/observability_contracts.rs",
        "crates/bijux-dag-runtime/src/diagnostics/runtime/observability.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "observability contract missing required surfaces: {}",
            missing.join(", ")
        ));
    }
    let test_text =
        fs::read_to_string(root.join("crates/bijux-dag-runtime/tests/observability_contracts.rs"))
            .map_err(|err| err.to_string())?;
    if !test_text.contains("required_runtime_event_names_are_present_for_reference_sequence") {
        return Err(
            "observability contract test for required runtime event names is missing".to_string()
        );
    }
    Ok(())
}

pub(super) fn run_extensibility_contract_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/EXTENSIBILITY_CONTRACT.md",
        "docs/spec/INTERNAL_HOOK_PROMOTION_CHECKLIST.md",
        "configs/dag/schema/extension_descriptor.schema.json",
        "crates/bijux-dag-runtime/tests/extension_catalog_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "extensibility contract missing required surfaces: {}",
            missing.join(", ")
        ));
    }

    let report = json!({
        "extension_points": [
            { "name": "adapter", "stability": "stable", "owner": "bijux-dag-runtime" },
            { "name": "execution-backend", "stability": "experimental", "owner": "bijux-dag-runtime" },
            { "name": "internal-hook", "stability": "internal", "owner": "bijux-dag-runtime" }
        ],
        "source_contract": "docs/spec/EXTENSIBILITY_CONTRACT.md"
    });
    let report_dir = root.join("artifacts/reports");
    fs::create_dir_all(&report_dir).map_err(|err| err.to_string())?;
    fs::write(
        report_dir.join("extensibility_contract_report.json"),
        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

pub(super) fn run_security_model_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/SECURITY_MODEL.md",
        "docs/reports/governance/NON_HERMETIC_BEHAVIORS.md",
        "docs/reports/governance/SECURITY_DEBT_LEDGER.md",
        "crates/bijux-dag-runtime/tests/security_model_contracts.rs",
        "crates/bijux-dag-runtime/tests/security_policy_contracts.rs",
        "crates/bijux-dag-runtime/tests/secrets_security_contracts.rs",
        "crates/bijux-dag-runtime/src/internal/identity/security_env.rs",
        "crates/bijux-dag-runtime/src/artifacts/storage/path_authorization.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "security model contract missing required surfaces: {}",
            missing.join(", ")
        ));
    }

    let security_doc = fs::read_to_string(root.join("docs/spec/SECURITY_MODEL.md"))
        .map_err(|err| err.to_string())?;
    for required_section in [
        "## Threat model",
        "## Hermeticity model",
        "## Environment controls",
        "## Filesystem controls",
        "## Secret handling and redaction",
    ] {
        if !security_doc.contains(required_section) {
            return Err(format!("security model missing required section: {required_section}"));
        }
    }
    let security_tests =
        fs::read_to_string(root.join("crates/bijux-dag-runtime/tests/security_model_contracts.rs"))
            .map_err(|err| err.to_string())?;
    if !security_tests.contains("clean_env_and_allowlist_contract_is_deterministic")
        || !security_tests
            .contains("input_and_output_authorization_reject_path_traversal_and_symlink_escape")
    {
        return Err("security model tests missing required enforcement coverage".to_string());
    }
    Ok(())
}

pub(super) fn run_container_remote_boundary_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/CONTAINER_EXECUTION_CONTRACT.md",
        "docs/spec/REMOTE_EXECUTION_MODEL.md",
        "docs/bijux-dag/architecture/execution-mode-responsibilities.md",
        "crates/bijux-dag-runtime/tests/container_execution_contracts.rs",
        "crates/bijux-dag-runtime/tests/remote_execution_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "container/remote execution boundary missing required surfaces: {}",
            missing.join(", ")
        ));
    }

    let remote_doc = fs::read_to_string(root.join("docs/spec/REMOTE_EXECUTION_MODEL.md"))
        .map_err(|err| err.to_string())?;
    if !remote_doc.contains("Not implemented: production Kubernetes/HPC") {
        return Err(
            "remote execution model must explicitly declare kubernetes/hpc not implemented"
                .to_string(),
        );
    }
    let deployment_doc =
        fs::read_to_string(root.join("docs/bijux-dag/operations/deployment-boundaries.md"))
            .map_err(|err| err.to_string())?;
    if deployment_doc.contains("Kubernetes execution is production-ready")
        || deployment_doc.contains("HPC execution is production-ready")
    {
        return Err("deployment backend docs overclaim kubernetes/hpc maturity".to_string());
    }
    Ok(())
}

pub(super) fn run_batch_execution_boundary_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/BATCH_EXECUTION_MODEL.md",
        "docs/bijux-dag/architecture/execution-mode-responsibilities.md",
        "crates/bijux-dag-runtime/tests/batch_execution_contracts.rs",
        "crates/bijux-dag-runtime/tests/batch_backend_simulation_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "batch execution boundary missing required surfaces: {}",
            missing.join(", ")
        ));
    }
    let batch_doc = fs::read_to_string(root.join("docs/spec/BATCH_EXECUTION_MODEL.md"))
        .map_err(|err| err.to_string())?;
    if !batch_doc.contains("not implemented as") && !batch_doc.contains("not implemented") {
        return Err(
            "batch execution model must explicitly state not-implemented production boundary"
                .to_string(),
        );
    }
    if batch_doc.contains("production-ready") || batch_doc.contains("ga-ready") {
        return Err("batch execution model contains unsupported maturity claim".to_string());
    }
    Ok(())
}

pub(super) fn run_operator_ux_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/OPERATOR_UX_CONTRACT.md",
        "docs/spec/OPERATOR_INSPECTION_CONTRACT.md",
        "docs/bijux-dag/interfaces/generated-cli-reference.md",
        "docs/bijux-dag/interfaces/operator-command-index.md",
        "docs/bijux-dag/interfaces/gated-command-inventory.md",
        "crates/bijux-dag-app/tests/operator_ux_contract.rs",
        "evidence/operator/scenarios/inspection_only.json",
        "configs/dag/schema/operator/run_list.schema.json",
        "configs/dag/schema/operator/run_show.schema.json",
        "configs/dag/schema/operator/run_inspect.schema.json",
        "configs/dag/schema/operator/run_tree.schema.json",
        "configs/dag/schema/operator/run_timeline.schema.json",
        "configs/dag/schema/operator/run_explain_failure.schema.json",
        "configs/dag/schema/operator/run_doctor.schema.json",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "operator ux contract missing required surfaces: {}",
            missing.join(", ")
        ));
    }
    let index =
        fs::read_to_string(root.join("docs/bijux-dag/interfaces/operator-command-index.md"))
            .map_err(|err| err.to_string())?;
    for command in [
        "bijux-dag runs list",
        "bijux-dag runs show",
        "bijux-dag runs inspect",
        "bijux-dag runs tree",
        "bijux-dag runs timeline",
        "bijux-dag runs diff",
        "bijux-dag runs verify",
        "bijux-dag runs doctor",
        "bijux-dag runs explain-failure",
    ] {
        if !index.contains(command) {
            return Err(format!("operator command index missing `{command}`"));
        }
    }
    let tests = fs::read_to_string(root.join("crates/bijux-dag-app/tests/operator_ux_contract.rs"))
        .map_err(|err| err.to_string())?;
    for required_test in [
        "operator_inspection_supports_imported_runs",
        "operator_inspection_distinguishes_unsupported_runs",
        "operator_inspection_distinguishes_corrupt_runs",
        "operator_timing_summary_is_trace_coherent",
    ] {
        if !tests.contains(required_test) {
            return Err(format!(
                "operator ux test coverage missing required case `{}`",
                required_test
            ));
        }
    }
    Ok(())
}

pub(super) fn run_authoring_ux_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required_docs =
        ["docs/spec/AUTHORING_UX_CONTRACT.md", "docs/bijux-dag/interfaces/authoring-guide.md"];
    let required_examples = [
        "evidence/authoring/patterns/minimal.json",
        "evidence/authoring/patterns/medium.json",
        "evidence/authoring/patterns/pattern_chain.json",
        "evidence/authoring/patterns/pattern_diamond.json",
        "evidence/authoring/patterns/pattern_fanout.json",
        "evidence/authoring/patterns/pattern_aggregation.json",
        "evidence/authoring/patterns/pattern_cache_heavy.json",
        "evidence/authoring/patterns/pattern_replay_sensitive.json",
    ];
    let required_bad = [
        "evidence/authoring/negative/undeclared_outputs.json",
        "evidence/authoring/negative/invalid_refs.json",
        "evidence/authoring/negative/cycle.json",
        "evidence/authoring/negative/invalid_selectors.json",
        "evidence/authoring/negative/unsupported_adapter_payload.json",
    ];
    let mut missing = Vec::new();
    for rel in required_docs.iter().chain(required_examples.iter()).chain(required_bad.iter()) {
        if !root.join(rel).exists() {
            missing.push((*rel).to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!("authoring ux required surfaces missing: {}", missing.join(", ")));
    }

    let contract = fs::read_to_string(root.join("docs/spec/AUTHORING_UX_CONTRACT.md"))
        .map_err(|err| err.to_string())?;
    let guide = fs::read_to_string(root.join("docs/bijux-dag/interfaces/authoring-guide.md"))
        .map_err(|err| err.to_string())?;
    for rel in required_examples.iter().chain(required_bad.iter()) {
        if !contract.contains(rel) {
            return Err(format!("authoring contract must reference executable fixture: {rel}"));
        }
        if !guide.contains(rel) {
            return Err(format!("authoring user guide must reference executable fixture: {rel}"));
        }
    }

    for rel in required_examples {
        let payload = fs::read_to_string(root.join(rel)).map_err(|err| err.to_string())?;
        let graph = bijux_dag_core::parse_graph_strict(&payload).map_err(|err| err.to_string())?;
        let has_error = graph
            .validate_with_warnings()
            .iter()
            .any(|d| d.severity == bijux_dag_core::Severity::Error);
        if has_error {
            return Err(format!("authoring example must validate without errors: {rel}"));
        }
    }
    Ok(())
}

pub(super) fn run_versioning_compatibility_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required_docs = [
        "docs/spec/VERSIONING_MODEL.md",
        "docs/bijux-dag/interfaces/compatibility-matrix.md",
        "docs/spec/SCHEMA_EVOLUTION_RULEBOOK.md",
        "docs/spec/RUN_DIR_EVOLUTION_RULEBOOK.md",
        "docs/spec/EXPORT_BUNDLE_EVOLUTION_RULEBOOK.md",
        "docs/spec/MIGRATION_POLICY.md",
        "docs/spec/VERSION_COMPATIBILITY_DRIFT_POLICY.md",
    ];
    let required_fixtures = [
        "evidence/compat/metadata.json",
        "evidence/compat/graph_schema/v0_1_supported/minimal.dag.json",
        "evidence/compat/graph_schema/unsupported_newer_version/minimal.dag.json",
        "evidence/compat/graph_schema/unsupported_older_version/minimal.dag.json",
        "evidence/compat/run_dir/v0_1_supported/manifest.json",
        "evidence/compat/run_dir/unsupported_newer_version/manifest.json",
        "evidence/compat/export_bundle/v0_1_supported/bundle.json",
        "evidence/compat/export_bundle/unsupported_older_version/bundle.json",
    ];
    let mut missing = Vec::new();
    for rel in required_docs.iter().chain(required_fixtures.iter()) {
        if !root.join(rel).exists() {
            missing.push((*rel).to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!("versioning compatibility surfaces missing: {}", missing.join(", ")));
    }

    let matrix = fs::read_to_string(root.join("docs/bijux-dag/interfaces/compatibility-matrix.md"))
        .map_err(|err| err.to_string())?;
    for token in ["graph schema", "run-dir format", "export bundle"] {
        if !matrix.to_lowercase().contains(token) {
            return Err(format!("compatibility matrix missing required surface row: {token}"));
        }
    }
    Ok(())
}

pub(super) fn run_cache_evolution_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/CACHE_CONTRACT.md",
        "docs/spec/CACHE_EVOLUTION_MODEL.md",
        "docs/reports/foundation/CACHE_HARDENING_REPORT.md",
        "docs/spec/CACHE_PRUNE_POLICY.md",
        "docs/reports/governance/CACHE_CORRECTNESS_COVERAGE.md",
        "evidence/cache/metadata.json",
        "evidence/cache/corrupt/missing_meta.json",
        "evidence/cache/corrupt/hash_mismatch.json",
        "evidence/cache/corrupt/missing_manifest.json",
        "evidence/cache/corrupt/unsupported_metadata_version.json",
        "evidence/cache/corrupt/truncated_meta.json",
        "evidence/cache/corrupt/missing_outputs_proof.json",
        "evidence/cache/scenarios/warm_cold.json",
        "crates/bijux-dag-app/tests/cache_evolution_contract.rs",
        "crates/bijux-dag-runtime/tests/cache_contracts.rs",
        "crates/bijux-dev/tests/cache_hardening_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!("cache evolution required surfaces missing: {}", missing.join(", ")));
    }
    let cache_metadata: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/cache/metadata.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let corrupt_fixtures = cache_metadata
        .get("corrupt_fixtures")
        .and_then(Value::as_object)
        .ok_or_else(|| "cache metadata missing corrupt_fixtures object".to_string())?;
    for fixture in [
        "evidence/cache/corrupt/missing_meta.json",
        "evidence/cache/corrupt/hash_mismatch.json",
        "evidence/cache/corrupt/missing_manifest.json",
        "evidence/cache/corrupt/unsupported_metadata_version.json",
        "evidence/cache/corrupt/truncated_meta.json",
        "evidence/cache/corrupt/missing_outputs_proof.json",
    ] {
        if !corrupt_fixtures.contains_key(fixture) {
            return Err(format!("cache metadata missing corruption entry: {fixture}"));
        }
    }
    let model = fs::read_to_string(root.join("docs/spec/CACHE_EVOLUTION_MODEL.md"))
        .map_err(|err| err.to_string())?;
    for token in [
        "Intentional cache key inputs",
        "Metadata compatibility",
        "Cache lineage model",
        "Locality decision",
    ] {
        if !model.contains(token) {
            return Err(format!("cache evolution model missing section `{token}`"));
        }
    }
    let app_commands = fs::read_to_string(root.join("crates/bijux-dag-app/src/commands/mod.rs"))
        .map_err(|err| err.to_string())?;
    let cache_surface_count =
        ["Ls", "Pack", "Unpack", "Gc", "Verify", "Explain", "Stats", "PruneSimulate", "Diff"]
            .iter()
            .filter(|name| app_commands.contains(&format!("CacheCommands::{}", name)))
            .count();
    if cache_surface_count >= 9 {
        for test in [
            "crates/bijux-dag-app/tests/cache_evolution_contract.rs",
            "crates/bijux-dag-runtime/tests/cache_contracts.rs",
        ] {
            if !root.join(test).exists() {
                return Err(format!(
                    "cache command surface expanded without required cache coverage test: {}",
                    test
                ));
            }
        }
    }
    let policy: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/dag/policy/battle_trust_properties.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let trust_properties = policy
        .get("trust_properties")
        .and_then(Value::as_array)
        .ok_or_else(|| "battle trust policy missing trust_properties".to_string())?;
    let has_cache_integrity = trust_properties.iter().any(|property| {
        property.get("id").and_then(Value::as_str).is_some_and(|id| id == "tp_cache_integrity")
    });
    if !has_cache_integrity {
        return Err("cache evolution requires tp_cache_integrity trust property".to_string());
    }
    Ok(())
}

pub(super) fn run_replay_contract_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/REPLAY_CONTRACT.md",
        "docs/reports/foundation/REPLAY_HARDENING_REPORT.md",
        "configs/dag/schema/operator/replay_diff.schema.json",
        "evidence/cache/replay/match_case.json",
        "evidence/cache/replay/mismatch_case.json",
        "evidence/cache/replay/corruption_case.json",
        "evidence/cache/replay/unsupported_version_case.json",
        "crates/bijux-dag-app/tests/replay_contract.rs",
        "crates/bijux-dag-runtime/tests/replay_contract.rs",
        "crates/bijux-dag-runtime/tests/runtime_replay_contracts.rs",
        "crates/bijux-dev/tests/replay_hardening_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!("replay contract required surfaces missing: {}", missing.join(", ")));
    }
    let contract = fs::read_to_string(root.join("docs/spec/REPLAY_CONTRACT.md"))
        .map_err(|err| err.to_string())?;
    for token in [
        "## Replay definition",
        "## Authoritative inputs",
        "## Replay explain mode",
        "## What replay cannot prove",
    ] {
        if !contract.contains(token) {
            return Err(format!("replay contract missing section `{token}`"));
        }
    }
    let commands_src = fs::read_to_string(root.join("crates/bijux-dag-app/src/commands/mod.rs"))
        .map_err(|err| err.to_string())?;
    if !commands_src.contains("DiffModeArg::Semantic") {
        return Err("replay contract requires semantic diff mode in CLI surfaces".to_string());
    }
    let replay_battle = fs::read_to_string(
        root.join("evidence/battle/workflows/replay/replay_semantic_comparison.json"),
    )
    .map_err(|err| err.to_string())?;
    if !replay_battle.contains("replay_mandatory_proof") {
        return Err("replay battle scenario must assert replay_mandatory_proof".to_string());
    }
    let policy: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/dag/policy/battle_trust_properties.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let trust_properties = policy
        .get("trust_properties")
        .and_then(Value::as_array)
        .ok_or_else(|| "battle trust policy missing trust_properties".to_string())?;
    let has_replay_equivalence = trust_properties.iter().any(|property| {
        property.get("id").and_then(Value::as_str).is_some_and(|id| id == "tp_replay_equivalence")
    });
    if !has_replay_equivalence {
        return Err("replay contract requires tp_replay_equivalence trust property".to_string());
    }

    let mut violations = Vec::new();
    let docs_dir = root.join("docs");
    let mut stack = vec![docs_dir];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).map_err(|err| err.to_string())? {
            let entry = entry.map_err(|err| err.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|v| v.to_str()) != Some("md") {
                continue;
            }
            let rel = path
                .strip_prefix(&root)
                .map_err(|err| err.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            let text = fs::read_to_string(&path).map_err(|err| err.to_string())?;
            if text.to_ascii_lowercase().contains("replayable")
                && !text.contains("REPLAY_CONTRACT.md")
                && !text.contains("docs/spec/REPLAY_CONTRACT.md")
            {
                violations.push(rel);
            }
        }
    }
    if !violations.is_empty() {
        return Err(format!(
            "vague replayable claims must cite replay contract: {}",
            violations.join(" | ")
        ));
    }
    Ok(())
}

pub(super) fn run_multi_run_analytics_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/MULTI_RUN_ANALYTICS_CONTRACT.md",
        "docs/spec/HISTORY_RETENTION_POLICY.md",
        "docs/spec/ANALYTICS_EXACTNESS.md",
        "configs/dag/schema/operator/runs_analytics.schema.json",
        "crates/bijux-dag-app/tests/multi_run_analytics_contract.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "multi-run analytics required surfaces missing: {}",
            missing.join(", ")
        ));
    }
    let commands_src = fs::read_to_string(root.join("crates/bijux-dag-app/src/commands/mod.rs"))
        .map_err(|err| err.to_string())?;
    for token in ["Summary", "Compare", "Trend", "Failures", "Flakes"] {
        if !commands_src.contains(token) {
            return Err(format!("runs command surface missing analytics variant `{token}`"));
        }
    }
    let contract = fs::read_to_string(root.join("docs/spec/MULTI_RUN_ANALYTICS_CONTRACT.md"))
        .map_err(|err| err.to_string())?;
    if !contract.contains("never mutate authoritative run records") {
        return Err("multi-run analytics contract must assert non-mutation rule".to_string());
    }
    Ok(())
}

pub(super) fn run_distributed_coordination_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/DISTRIBUTED_COORDINATION_MODEL.md",
        "docs/bijux-dag/architecture/execution-mode-responsibilities.md",
        "crates/bijux-dag-runtime/tests/distributed_event_reconciliation_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "distributed coordination required surfaces missing: {}",
            missing.join(", ")
        ));
    }
    let model = fs::read_to_string(root.join("docs/spec/DISTRIBUTED_COORDINATION_MODEL.md"))
        .map_err(|err| err.to_string())?;
    for token in [
        "single-controller",
        "Single-writer rule",
        "Not implemented boundary",
        "planner, scheduler, and storage contracts",
    ] {
        if !model.contains(token) {
            return Err(format!("distributed coordination model missing section `{token}`"));
        }
    }
    Ok(())
}

pub(super) fn run_distributed_semantics_report() -> Result<(), String> {
    let payload = json!({
        "local_semantics": {
            "authoritative_writer": "controller",
            "run_state_writer_count": 1,
            "distributed_coordination_mode": "not_implemented"
        },
        "simulated_distributed_semantics": {
            "event_source": "fake_distributed_event_source",
            "reconciliation": ["out_of_order", "duplicate", "missing_completion", "restart_partial_state"],
            "authoritative_remote_state_writer": false
        }
    });
    println!("{}", serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?);
    Ok(())
}

pub(super) fn run_formal_invariants_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/FORMAL_INVARIANTS.md",
        "docs/reports/governance/INVARIANT_COVERAGE.md",
        "crates/bijux-dag-runtime/src/runtime_core/governance/invariants.rs",
        "crates/bijux-dag-runtime/src/internal/testing/invariants_tests.rs",
        "crates/bijux-dag-runtime/tests/formal_invariant_property_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!("formal invariants required surfaces missing: {}", missing.join(", ")));
    }
    let spec = fs::read_to_string(root.join("docs/spec/FORMAL_INVARIANTS.md"))
        .map_err(|err| err.to_string())?;
    for token in [
        "INV-GRAPH-SHAPE-001",
        "INV-PLAN-SHAPE-001",
        "INV-SCHED-READY-001",
        "INV-RUN-COUNTS-001",
        "INV-TRACE-TIME-001",
        "INV-CACHE-PROOF-001",
        "INV-ARTIFACT-REF-001",
    ] {
        if !spec.contains(token) {
            return Err(format!("formal invariants spec missing `{token}`"));
        }
    }
    let mut unchecked_guarantees = Vec::new();
    let rel = "docs/spec/FORMAL_INVARIANTS.md";
    let text = fs::read_to_string(root.join(rel)).map_err(|err| err.to_string())?;
    for (idx, line) in text.lines().enumerate() {
        let lower = line.to_ascii_lowercase();
        if (lower.contains("guarantee") || lower.contains("always") || lower.contains("never"))
            && !line.contains("INV-")
        {
            unchecked_guarantees.push(format!("{}:{} {}", rel, idx + 1, line.trim()));
        }
    }
    if !unchecked_guarantees.is_empty() {
        return Err(format!(
            "normative guarantee wording must cite invariant ids: {}",
            unchecked_guarantees.join(" | ")
        ));
    }
    Ok(())
}

pub(super) fn run_invariants_report() -> Result<(), String> {
    let root = repo_root()?;
    let registry_src = fs::read_to_string(
        root.join("crates/bijux-dag-runtime/src/runtime_core/governance/invariants.rs"),
    )
    .map_err(|err| err.to_string())?;
    let coverage = fs::read_to_string(root.join("docs/reports/governance/INVARIANT_COVERAGE.md"))
        .map_err(|err| err.to_string())?;

    let mut ids = Vec::new();
    for line in registry_src.lines() {
        if let Some(start) = line.find("id: \"INV-") {
            let slice = &line[start + 5..];
            if let Some(end) = slice.find('"') {
                ids.push(slice[..end].to_string());
            }
        }
    }
    ids.sort();
    ids.dedup();

    let mut missing_coverage = Vec::new();
    for id in &ids {
        if !coverage.contains(id) {
            missing_coverage.push(id.clone());
        }
    }

    let payload = json!({
        "invariant_ids": ids,
        "missing_coverage_entries": missing_coverage,
        "coverage_file": "docs/reports/governance/INVARIANT_COVERAGE.md"
    });
    println!("{}", serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?);
    if payload["missing_coverage_entries"].as_array().is_some_and(|a| a.is_empty()) {
        Ok(())
    } else {
        Err("invariant coverage file missing registry entries".to_string())
    }
}

pub(super) fn run_adoption_surfaces_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/ADOPTION_SURFACES.md",
        "docs/bijux-dag/operations/installation-and-setup.md",
        "docs/bijux-dag/operations/ci-integration.md",
        "docs/bijux-dag/operations/first-run-tutorial.md",
        "docs/bijux-dag/interfaces/support-matrix.md",
        "docs/spec/RELEASE_BINARY_VERIFICATION.md",
        "docs/bijux-dag/operations/security-isolation-truth.md",
        "evidence/authoring/examples/minimal_consumer.dag.json",
        "crates/bijux-dag-testkit/fixtures/minimal_consumer/README.md",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "adoption surfaces required docs/fixtures missing: {}",
            missing.join(", ")
        ));
    }

    let commands_src = fs::read_to_string(root.join("crates/bijux-dag-app/src/commands/mod.rs"))
        .map_err(|err| err.to_string())?;
    if !commands_src.contains("Capabilities") {
        return Err(
            "dag capabilities command is required for maintainer-only support probes".to_string()
        );
    }

    let onboarding =
        fs::read_to_string(root.join("docs/bijux-dag/operations/first-run-tutorial.md"))
            .map_err(|err| err.to_string())?;
    let install =
        fs::read_to_string(root.join("docs/bijux-dag/operations/installation-and-setup.md"))
            .map_err(|err| err.to_string())?;
    let ci = fs::read_to_string(root.join("docs/bijux-dag/operations/ci-integration.md"))
        .map_err(|err| err.to_string())?;
    let support_matrix =
        fs::read_to_string(root.join("docs/bijux-dag/interfaces/support-matrix.md"))
            .map_err(|err| err.to_string())?;
    for required_cmd in [
        "cargo build -p bijux-dag-cli --release",
        "cargo run -p bijux-dag-cli --bin bijux-dag -- version",
    ] {
        if !install.contains(required_cmd) {
            return Err(format!(
                "installation doc missing clean-environment command `{}`",
                required_cmd
            ));
        }
    }
    for forbidden in ["kubernetes", "hpc", "production-grade remote"] {
        if onboarding.to_ascii_lowercase().contains(forbidden) {
            return Err(format!(
                "first-run tutorial references unsupported surface `{}` as first-class",
                forbidden
            ));
        }
    }
    for required_cmd in [
        "cargo run -p bijux-dag-cli --bin bijux-dag -- commands",
        "Maintainer-only probes such as `capabilities` remain outside this",
    ] {
        if !onboarding.contains(required_cmd) {
            return Err(format!(
                "first-run tutorial missing public-boundary reminder `{}`",
                required_cmd
            ));
        }
    }
    if onboarding.contains("cargo run -p bijux-dag-cli --bin bijux-dag -- capabilities --json") {
        return Err(
            "first-run tutorial must not present `capabilities --json` as part of the operator lane"
                .to_string(),
        );
    }
    for required_cmd in [
        "cargo run -p bijux-dag-cli --bin bijux-dag -- commands",
        "BIJUX_DAG_ENABLE_INTERNAL=1 cargo run -p bijux-dag-cli --bin bijux-dag -- capabilities --json",
        "not part of the public operator boundary",
    ] {
        if !ci.contains(required_cmd) {
            return Err(format!(
                "ci integration doc missing support-boundary reminder `{}`",
                required_cmd
            ));
        }
    }
    for required_cmd in [
        "| `commands` | stable | visible CLI | route inventory for stable and non-stable command discovery |",
        "| `capabilities` | internal | `BIJUX_DAG_ENABLE_INTERNAL=1` | maintainer-only support probe outside the public operator lane |",
    ] {
        if !support_matrix.contains(required_cmd) {
            return Err(format!(
                "support matrix missing release-boundary classification `{}`",
                required_cmd
            ));
        }
    }
    Ok(())
}

pub(super) fn run_release_artifact_verification_suite() -> Result<(), String> {
    let root = repo_root()?;
    let commands_src = fs::read_to_string(root.join("crates/bijux-dag-app/src/commands/mod.rs"))
        .map_err(|err| err.to_string())?;
    for command in ["Version", "Capabilities", "Runs"] {
        if !commands_src.contains(command) {
            return Err(format!(
                "release artifact verification requires `{}` command surface",
                command
            ));
        }
    }
    let policy = fs::read_to_string(root.join("docs/spec/RELEASE_BINARY_VERIFICATION.md"))
        .map_err(|err| err.to_string())?;
    for token in ["bijux-dag version --json", "bijux-dag capabilities --json"] {
        if !policy.contains(token) {
            return Err(format!(
                "release binary verification doc missing required check `{}`",
                token
            ));
        }
    }
    let report = evaluate_distribution_delivery_goals(&root)?;
    if !report["ok"].as_bool().unwrap_or(false) {
        return Err(format!(
            "distribution delivery contract failed: {}",
            serde_json::to_string(&report).unwrap_or_else(|_| "invalid report".to_string())
        ));
    }
    Ok(())
}

pub(super) fn run_distribution_delivery_contract_report() -> Result<Value, String> {
    let root = repo_root()?;
    evaluate_distribution_delivery_goals(&root)
}

fn evaluate_distribution_delivery_goals(root: &Path) -> Result<Value, String> {
    let reports = vec![
        evaluate_publishable_crates(root)?,
        evaluate_python_bridge_distribution(root)?,
        evaluate_release_artifacts_runnable(root)?,
        evaluate_example_command_catalog(root)?,
        evaluate_executable_docs_recipes(root)?,
        evaluate_install_paths_predictable(root)?,
        evaluate_local_dev_loop_focus(root)?,
        evaluate_app_integration_documentation(root)?,
        evaluate_limitations_visibility(root)?,
        evaluate_production_candidate_suite(root)?,
    ];
    let failed_goals: Vec<String> = reports
        .iter()
        .filter(|report| !report["ok"].as_bool().unwrap_or(false))
        .map(|report| report["goal"].as_str().unwrap_or("unknown").to_string())
        .collect();
    Ok(json!({
        "ok": failed_goals.is_empty(),
        "goals": reports,
        "failed_goals": failed_goals,
    }))
}

pub(super) fn run_drift_dashboard() -> Result<(), String> {
    let root = repo_root()?;
    let payload = json!({
        "drift_classes": [
            {"name":"docs drift","severity":"blocker","check":"repo-docs"},
            {"name":"schema drift","severity":"blocker","check":"docs-schema-ref"},
            {"name":"contract drift","severity":"blocker","check":"docs-contract-ref"},
            {"name":"cli drift","severity":"blocker","check":"cli-freeze"},
            {"name":"test drift","severity":"warning","check":"contract-test-links"},
            {"name":"fixture drift","severity":"warning","check":"docs-coverage"},
            {"name":"benchmark drift","severity":"warning","check":"performance-claims"},
            {"name":"dependency drift","severity":"warning","check":"dependency-policy"}
        ],
        "dashboard_doc": "docs/reports/governance/DRIFT_DASHBOARD.md",
        "anti_drift_policy": root.join("docs/spec/ANTI_DRIFT_POLICY.md").exists()
    });
    println!("{}", serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?);
    Ok(())
}

pub(super) fn run_repo_trust_summary() -> Result<(), String> {
    let root = repo_root()?;
    let payload = json!({
        "contracts": {
            "invariants": root.join("docs/spec/FORMAL_INVARIANTS.md").exists(),
            "comparison_harness": root.join("docs/spec/COMPARISON_HARNESS_CONTRACT.md").exists(),
            "adoption_surfaces": root.join("docs/spec/ADOPTION_SURFACES.md").exists(),
            "anti_drift": root.join("docs/spec/ANTI_DRIFT_POLICY.md").exists()
        },
        "evidence": {
            "invariant_coverage": root.join("docs/reports/governance/INVARIANT_COVERAGE.md").exists(),
            "drift_dashboard": root.join("docs/reports/governance/DRIFT_DASHBOARD.md").exists()
        },
        "trust_model": root.join("docs/bijux-core/governance/trust-evidence.md").exists()
    });
    println!("{}", serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?);
    Ok(())
}

pub(super) fn run_anti_drift_governance_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/ANTI_DRIFT_POLICY.md",
        "docs/reports/governance/DRIFT_DASHBOARD.md",
        "docs/bijux-core/governance/trust-evidence.md",
        ".github/pull_request_template.md",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!("anti-drift governance surfaces missing: {}", missing.join(", ")));
    }
    let policy = fs::read_to_string(root.join("docs/spec/ANTI_DRIFT_POLICY.md"))
        .map_err(|err| err.to_string())?;
    for token in
        ["docs drift", "schema drift", "contract drift", "cli drift", "same-change alignment rule"]
    {
        if !policy.to_ascii_lowercase().contains(&token.to_ascii_lowercase()) {
            return Err(format!("anti-drift policy missing `{}`", token));
        }
    }

    let suite_ids = crate::suites::repo::IDS;
    for required_check in [
        "cli-freeze",
        "docs-schema-ref",
        "docs-contract-ref",
        "contract-test-links",
        "docs-coverage",
        "versioning-compatibility",
        "performance-claims",
    ] {
        if !suite_ids.contains(&required_check) {
            return Err(format!("anti-drift governance requires repo suite `{}`", required_check));
        }
    }

    let release_doc = fs::read_to_string(root.join("docs/spec/RELEASE_BINARY_VERIFICATION.md"))
        .map_err(|err| err.to_string())?;
    if !release_doc.contains("bijux-dag version --json")
        || !release_doc.contains("bijux-dag capabilities --json")
    {
        return Err(
            "release verification doc must define machine-readable artifact checks".to_string()
        );
    }

    let benchmark_scenarios = root.join("evidence/perf/scenarios");
    if !benchmark_scenarios.exists() {
        return Err(
            "benchmark scenario directory missing for anti-drift benchmark check".to_string()
        );
    }
    Ok(())
}

pub(super) fn run_runtime_module_triage_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/bijux-core/architecture/runtime-surfaces.md",
        "docs/bijux-dag/packages/bijux-dag-runtime.md",
        "configs/dag/policy/runtime_module_freeze.json",
        "configs/dag/policy/runtime_scope_v2.json",
        "docs/reports/foundation/KERNEL_OWNED_MODULES_REPORT.md",
        "docs/reports/foundation/RUNTIME_NON_KERNEL_MODULES_REPORT.md",
        "docs/reports/foundation/RUNTIME_CONTRACT_BACKING_REPORT.md",
        "docs/reports/foundation/RUNTIME_OPERATOR_SURFACE_REPORT.md",
        "docs/reports/foundation/core-public-api-surface.md",
        "docs/reports/foundation/runtime-public-api-surface.md",
        "crates/bijux-dag-runtime/src/lib.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!("runtime module triage surfaces missing: {}", missing.join(", ")));
    }

    let freeze_payload =
        fs::read_to_string(root.join("configs/dag/policy/runtime_module_freeze.json"))
            .map_err(|err| err.to_string())?;
    let freeze_json: Value =
        serde_json::from_str(&freeze_payload).map_err(|err| err.to_string())?;
    let scope_policy = freeze_json
        .get("scope_policy")
        .and_then(Value::as_str)
        .ok_or_else(|| "runtime_module_freeze.json missing scope_policy".to_string())?;
    if !root.join(scope_policy).exists() {
        return Err(format!("runtime module freeze scope policy missing: {scope_policy}"));
    }

    let allowed_dirs = freeze_json
        .get("allowed_top_level_dirs")
        .and_then(Value::as_array)
        .ok_or_else(|| "runtime_module_freeze.json missing allowed_top_level_dirs".to_string())?;
    let allowed_dir_set: BTreeSet<String> =
        allowed_dirs.iter().filter_map(Value::as_str).map(|s| s.to_string()).collect();
    let allowed_files = freeze_json
        .get("allowed_top_level_files")
        .and_then(Value::as_array)
        .ok_or_else(|| "runtime_module_freeze.json missing allowed_top_level_files".to_string())?;
    let allowed_file_set: BTreeSet<String> =
        allowed_files.iter().filter_map(Value::as_str).map(|s| s.to_string()).collect();

    let mut actual_dirs = BTreeSet::new();
    let mut actual_files = BTreeSet::new();
    for entry in
        fs::read_dir(root.join("crates/bijux-dag-runtime/src")).map_err(|err| err.to_string())?
    {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|v| v.to_str()) {
                actual_dirs.insert(name.to_string());
            }
            continue;
        }
        if let Some(name) = path.file_name().and_then(|v| v.to_str()) {
            actual_files.insert(name.to_string());
        }
    }
    let unexpected_dirs: Vec<String> = actual_dirs.difference(&allowed_dir_set).cloned().collect();
    let missing_dirs: Vec<String> = allowed_dir_set.difference(&actual_dirs).cloned().collect();
    let unexpected_files: Vec<String> =
        actual_files.difference(&allowed_file_set).cloned().collect();
    let missing_files: Vec<String> = allowed_file_set.difference(&actual_files).cloned().collect();

    if !unexpected_dirs.is_empty()
        || !missing_dirs.is_empty()
        || !unexpected_files.is_empty()
        || !missing_files.is_empty()
    {
        let mut violations = Vec::new();
        if !unexpected_dirs.is_empty() {
            violations.push(format!("unexpected top-level dirs: {}", unexpected_dirs.join(", ")));
        }
        if !missing_dirs.is_empty() {
            violations.push(format!("missing top-level dirs: {}", missing_dirs.join(", ")));
        }
        if !unexpected_files.is_empty() {
            violations.push(format!("unexpected top-level files: {}", unexpected_files.join(", ")));
        }
        if !missing_files.is_empty() {
            violations.push(format!("missing top-level files: {}", missing_files.join(", ")));
        }
        return Err(format!("runtime module freeze violated: {}", violations.join("; ")));
    }
    Ok(())
}

pub(super) fn run_sacred_execution_flow_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/SACRED_EXECUTION_FLOW.md",
        "docs/bijux-dag/architecture/runtime-execution-flow.md",
        "docs/reports/foundation/SACRED_EXECUTION_HARDENING_REPORT.md",
        "crates/bijux-dag-runtime/src/runtime_core/governance/sacred_execution.rs",
        "crates/bijux-dag-runtime/src/runtime_core/execution/context.rs",
        "crates/bijux-dag-runtime/src/runtime_core/execution/engine.rs",
        "crates/bijux-dag-runtime/tests/sacred_execution_flow_contracts.rs",
        "crates/bijux-dev/tests/sacred_execution_hardening_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "sacred execution flow required surfaces missing: {}",
            missing.join(", ")
        ));
    }

    let engine_src = fs::read_to_string(
        root.join("crates/bijux-dag-runtime/src/runtime_core/execution/engine.rs"),
    )
    .map_err(|err| err.to_string())?;
    for token in [
        "sacred_execution::run_materialize_inputs",
        "sacred_execution::run_cache_lookup",
        "sacred_execution::run_retry_logic",
        "sacred_execution::run_write_trace",
        "sacred_execution::run_cache_write",
        "sacred_execution::resolve_dependencies",
    ] {
        if !engine_src.contains(token) {
            return Err(format!("engine flow missing centralized hook `{}`", token));
        }
    }
    for forbidden in [
        "crate::try_cache_read(",
        "crate::try_cache_write(",
        "crate::write_trace(",
        "crate::execute_with_retries(",
    ] {
        if engine_src.contains(forbidden) {
            return Err(format!(
                "engine flow bypasses sacred hook with direct call `{}`",
                forbidden
            ));
        }
    }
    Ok(())
}

pub(super) fn run_crate_boundary_foundation_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/bijux-core/foundation/package-boundary.md",
        "docs/bijux-core/packages/index.md",
        "docs/bijux-dag/foundation/ownership-boundary.md",
        "docs/bijux-dag/architecture/integration-seams.md",
        "configs/dag/policy/forbidden_dependencies.json",
        "crates/bijux-dag-app/tests/crate_boundary_contract.rs",
        "crates/bijux-dag-runtime/src/internal/control/services.rs",
        "crates/bijux-dag-artifacts/src/storage/services.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "crate boundary foundation missing required surfaces: {}",
            missing.join(", ")
        ));
    }

    let policy_payload =
        fs::read_to_string(root.join("configs/dag/policy/forbidden_dependencies.json"))
            .map_err(|err| err.to_string())?;
    let policy: Value = serde_json::from_str(&policy_payload).map_err(|err| err.to_string())?;
    let edges = policy
        .get("forbidden_edges")
        .and_then(Value::as_array)
        .ok_or_else(|| "forbidden dependency policy missing forbidden_edges".to_string())?;
    for edge in edges {
        let from = edge.get("from").and_then(Value::as_str).unwrap_or_default();
        let to = edge.get("to").and_then(Value::as_str).unwrap_or_default();
        let cargo = fs::read_to_string(root.join(format!("crates/{}/Cargo.toml", from)))
            .map_err(|err| err.to_string())?;
        if cargo.contains(to) {
            return Err(format!("forbidden dependency edge detected: {} -> {}", from, to));
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct EffectiveConfigDump {
    jobs: Option<usize>,
    cache_mode: Option<String>,
    materialize_inputs: Option<String>,
    policy: Option<Value>,
    debug: Option<Value>,
}

pub(super) fn run_config_dump(config: Option<&Path>) -> Result<(), String> {
    let root = repo_root()?;
    let defaults_path = root.join("configs/dag/dev/default_runtime_config.json");
    let defaults_payload = fs::read_to_string(&defaults_path).map_err(|err| err.to_string())?;
    let defaults: Value = serde_json::from_str(&defaults_payload).map_err(|err| err.to_string())?;
    let mut merged = defaults;

    if let Ok(env_cache_dir) = env::var("BIJUX_DAG_CACHE_DIR") {
        merged["cache_dir"] = Value::String(env_cache_dir);
    }
    if let Ok(env_adapters_dir) = env::var("BIJUX_DAG_ADAPTERS_DIR") {
        merged["adapters_dir"] = Value::String(env_adapters_dir);
    }

    if let Some(path) = config {
        let full = if path.is_absolute() { path.to_path_buf() } else { root.join(path) };
        let payload = fs::read_to_string(full).map_err(|err| err.to_string())?;
        let parsed: Value = serde_json::from_str(&payload).map_err(|err| err.to_string())?;
        deep_merge_json(&mut merged, &parsed);
    }

    let _typed: EffectiveConfigDump =
        serde_json::from_value(merged.clone()).map_err(|err| err.to_string())?;
    println!("{}", serde_json::to_string_pretty(&merged).map_err(|err| err.to_string())?);
    Ok(())
}

pub(super) fn run_policy_audit(config: Option<&Path>) -> Result<(), String> {
    let root = repo_root()?;
    let defaults_path = root.join("configs/dag/dev/default_runtime_config.json");
    let defaults_payload = fs::read_to_string(&defaults_path).map_err(|err| err.to_string())?;
    let defaults: Value = serde_json::from_str(&defaults_payload).map_err(|err| err.to_string())?;
    let mut merged = defaults;
    if let Some(path) = config {
        let full = if path.is_absolute() { path.to_path_buf() } else { root.join(path) };
        let payload = fs::read_to_string(full).map_err(|err| err.to_string())?;
        let parsed: Value = serde_json::from_str(&payload).map_err(|err| err.to_string())?;
        deep_merge_json(&mut merged, &parsed);
    }
    let policy = merged.get("policy").and_then(Value::as_object).cloned().unwrap_or_default();
    let report = json!({
        "policy_controls": {
            "deny_network": policy.get("deny_network").cloned().unwrap_or(Value::Bool(false)),
            "deny_env": policy.get("deny_env").cloned().unwrap_or(Value::Bool(false)),
            "deny_clock": policy.get("deny_clock").cloned().unwrap_or(Value::Bool(false)),
            "clean_env": policy.get("clean_env").cloned().unwrap_or(Value::Bool(false))
        },
        "security_contract": "docs/spec/SECURITY_MODEL.md"
    });
    println!("{}", serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?);
    Ok(())
}

pub(super) fn run_execution_modes_report() -> Result<(), String> {
    let report = bijux_dag_runtime::execution_mode_report();
    println!("{}", serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?);
    Ok(())
}

pub(super) fn run_compatibility_report() -> Result<(), String> {
    let root = repo_root()?;
    let report = json!({
        "graph_schema": {
            "current": "0.1",
            "supported_fixtures": collect_fixture_count(&root.join("evidence/compat/graph_schema/v0_1_supported"))?,
            "unsupported_newer_version_fixtures": collect_fixture_count(&root.join("evidence/compat/graph_schema/unsupported_newer_version"))?,
            "unsupported_older_version_fixtures": collect_fixture_count(&root.join("evidence/compat/graph_schema/unsupported_older_version"))?
        },
        "run_dir": {
            "current": "run-manifest/v0.1",
            "supported_fixtures": collect_fixture_count(&root.join("evidence/compat/run_dir/v0_1_supported"))?,
            "unsupported_newer_version_fixtures": collect_fixture_count(&root.join("evidence/compat/run_dir/unsupported_newer_version"))?
        },
        "export_bundle": {
            "current": "export-bundle/v0.1",
            "supported_fixtures": collect_fixture_count(&root.join("evidence/compat/export_bundle/v0_1_supported"))?,
            "unsupported_older_version_fixtures": collect_fixture_count(&root.join("evidence/compat/export_bundle/unsupported_older_version"))?
        }
    });
    println!("{}", serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?);
    Ok(())
}

pub(super) fn run_cache_coverage_report() -> Result<(), String> {
    let root = repo_root()?;
    let report = json!({
        "cache_correctness": {
            "docs": {
                "contract": root.join("docs/spec/CACHE_CONTRACT.md").exists(),
                "model": root.join("docs/spec/CACHE_EVOLUTION_MODEL.md").exists(),
                "prune_policy": root.join("docs/spec/CACHE_PRUNE_POLICY.md").exists(),
                "coverage_ledger": root.join("docs/reports/governance/CACHE_CORRECTNESS_COVERAGE.md").exists()
            },
            "fixtures": {
                "corruption": collect_fixture_count(&root.join("evidence/cache/corrupt"))?,
                "warm_cold": collect_fixture_count(&root.join("evidence/cache/scenarios"))?
            },
            "tests": {
                "app_cache_evolution_contract": root.join("crates/bijux-dag-app/tests/cache_evolution_contract.rs").exists(),
                "runtime_cache_contracts": root.join("crates/bijux-dag-runtime/tests/cache_contracts.rs").exists()
            }
        }
    });
    println!("{}", serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?);
    Ok(())
}

pub(super) fn collect_fixture_count(dir: &Path) -> Result<usize, String> {
    if !dir.exists() {
        return Ok(0);
    }
    let count = fs::read_dir(dir)
        .map_err(|err| err.to_string())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .count();
    Ok(count)
}

pub(super) fn run_config_lint() -> Result<(), String> {
    let root = repo_root()?;
    let examples_dir = root.join("configs/dag/dev/examples");
    let mut violations = Vec::new();

    for entry in fs::read_dir(&examples_dir).map_err(|err| err.to_string())? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let payload = fs::read_to_string(&path).map_err(|err| err.to_string())?;
        let value: Value = serde_json::from_str(&payload).map_err(|err| err.to_string())?;
        let allowed_root = ["jobs", "cache_mode", "materialize_inputs", "policy", "debug"];
        if let Some(obj) = value.as_object() {
            for key in obj.keys() {
                if !allowed_root.contains(&key.as_str()) {
                    violations.push(format!("{} has unknown field `{}`", path.display(), key));
                }
                if key.starts_with("deprecated_") {
                    violations.push(format!(
                        "{} contains deprecated field `{}`",
                        path.display(),
                        key
                    ));
                }
            }
        } else {
            violations.push(format!("{} must be a JSON object", path.display()));
        }
        if value.get("jobs").and_then(|v| v.as_u64()).unwrap_or(0) == 0 {
            violations.push(format!("{} has invalid jobs", path.display()));
        }
        let cache_mode = value.get("cache_mode").and_then(|v| v.as_str()).unwrap_or("");
        if !["off", "read", "read-write"].contains(&cache_mode) {
            violations.push(format!("{} has invalid cache_mode", path.display()));
        }
        let materialize = value.get("materialize_inputs").and_then(|v| v.as_str()).unwrap_or("");
        if !["none", "direct", "all"].contains(&materialize) {
            violations.push(format!("{} has invalid materialize_inputs", path.display()));
        }
        if value.get("policy").is_none() {
            violations.push(format!("{} missing policy object", path.display()));
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

pub(super) fn run_config_precedence_drift_guard() -> Result<(), String> {
    let root = repo_root()?;
    let precedence_doc = fs::read_to_string(root.join("docs/spec/CONFIG_PRECEDENCE_CONTRACT.md"))
        .map_err(|err| err.to_string())?;
    let expected = "CLI > explicit config file > environment > defaults";
    if !precedence_doc.contains(expected) {
        return Err("docs/spec/CONFIG_PRECEDENCE_CONTRACT.md missing canonical precedence table"
            .to_string());
    }
    for token in ["dag config show-effective", "dag policy show-effective"] {
        if !precedence_doc.contains(token) {
            return Err(format!("config precedence contract missing command surface `{}`", token));
        }
    }

    let defaults = json!({"jobs": 1});
    let env_cfg = json!({"jobs": 2});
    let file_cfg = json!({"jobs": 3});
    let cli_cfg = json!({"jobs": 4});
    let mut merged = defaults;
    deep_merge_json(&mut merged, &env_cfg);
    deep_merge_json(&mut merged, &file_cfg);
    deep_merge_json(&mut merged, &cli_cfg);
    if merged.get("jobs").and_then(|v| v.as_u64()) != Some(4) {
        return Err("effective precedence behavior does not match documented order".to_string());
    }
    Ok(())
}

pub(super) fn run_config_policy_determinism_guard() -> Result<(), String> {
    let root = repo_root()?;
    for required in [
        "docs/spec/CONFIG_PRECEDENCE_CONTRACT.md",
        "docs/spec/POLICY_EVALUATION_TRACE.md",
        "docs/reports/foundation/CONFIG_POLICY_DETERMINISM_REPORT.md",
        "crates/bijux-dag-app/tests/config_precedence_contract.rs",
        "crates/bijux-dag-app/tests/config_validation_contract.rs",
        "crates/bijux-dag-app/tests/config_effective_command_contract.rs",
        "crates/bijux-dag-runtime/tests/security_model_contracts.rs",
    ] {
        if !root.join(required).exists() {
            return Err(format!("config/policy determinism missing required surface: {required}"));
        }
    }

    let contract = fs::read_to_string(root.join("docs/spec/CONFIG_PRECEDENCE_CONTRACT.md"))
        .map_err(|err| err.to_string())?;
    for token in [
        "CLI > explicit config file > environment > defaults",
        "Unknown fields in explicit config must fail before execution.",
        "Malformed config files must fail before execution.",
        "Policy evaluation trace must be available for operator/debug inspection.",
    ] {
        if !contract.contains(token) {
            return Err(format!("config precedence contract missing required token `{token}`"));
        }
    }

    let policy: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/dag/policy/battle_trust_properties.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let trust_properties = policy
        .get("trust_properties")
        .and_then(Value::as_array)
        .ok_or_else(|| "battle trust policy missing trust_properties".to_string())?;
    let has_config_policy = trust_properties.iter().any(|property| {
        property
            .get("id")
            .and_then(Value::as_str)
            .is_some_and(|id| id == "tp_config_policy_determinism")
    });
    if !has_config_policy {
        return Err(
            "battle trust policy must include tp_config_policy_determinism as release evidence"
                .to_string(),
        );
    }
    Ok(())
}

pub(super) fn run_ambient_env_guard() -> Result<(), String> {
    let root = repo_root()?;
    let policy: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/dag/policy/ambient_env_allowlist.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let rules = policy
        .get("rules")
        .and_then(Value::as_array)
        .ok_or_else(|| "ambient env allowlist missing rules array".to_string())?;
    let mut files = Vec::new();
    collect_files_with_extension(&root.join("crates"), "rs", &mut files)?;
    let mut violations = Vec::new();
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .map_err(|err| err.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        let mut allow_env_keys = BTreeSet::new();
        for rule in rules {
            let Some(path_glob) = rule.get("path_glob").and_then(Value::as_str) else {
                continue;
            };
            if !wildcard_match(path_glob, &rel) {
                continue;
            }
            for key in rule
                .get("allowed_env_keys")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                allow_env_keys.insert(key.to_string());
            }
        }
        let content = fs::read_to_string(&file).map_err(|err| err.to_string())?;
        if !(content.contains("std::env::var(") || content.contains("env::var(")) {
            continue;
        }
        for line in content.lines() {
            if rel.contains("/tests/") || rel.ends_with(".in.rs") {
                continue;
            }
            for key in ambient_env_var_keys(line) {
                if allow_env_keys.contains(key) {
                    continue;
                }
                violations.push(format!(
                    "{rel}: disallowed ambient env read `{}` via {key}",
                    line.trim()
                ));
            }
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

fn ambient_env_var_keys(line: &str) -> Vec<&str> {
    let mut keys = Vec::new();
    for needle in ["std::env::var(\"", "env::var(\""] {
        let mut offset = 0;
        while let Some(found) = line[offset..].find(needle) {
            let idx = offset + found;
            if needle == "env::var(\""
                && idx >= 5
                && line.get(idx - 5..idx).is_some_and(|prefix| prefix == "std::")
            {
                offset = idx + needle.len();
                continue;
            }
            if idx > 0 && line[..idx].chars().next_back().is_some_and(|ch| matches!(ch, '"' | '\''))
            {
                offset = idx + needle.len();
                continue;
            }
            let start = idx + needle.len();
            let rest = &line[start..];
            let Some(end) = rest.find('"') else {
                offset = idx + needle.len();
                continue;
            };
            let key = &rest[..end];
            if key.chars().all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_') {
                keys.push(key);
            }
            offset = start + end + 1;
        }
    }
    keys
}

pub(super) fn run_foundation_verification_guard() -> Result<(), String> {
    let root = repo_root()?;
    for rel in [
        "contracts/foundation/maintainer_command_surface.v1.json",
        "docs/bijux-core/foundation/package-boundary.md",
        "docs/bijux-core/foundation/module-surface-lanes.md",
        "docs/bijux-dev/operations/command-surface.md",
        "crates/bijux-dev/src/commands/mod.rs",
        "crates/bijux-dev/src/suites/repo.rs",
    ] {
        if !root.join(rel).exists() {
            return Err(format!("missing foundation artifact: {rel}"));
        }
    }
    for required in [
        "repo-docs",
        "repo-source",
        "root-directory-guard",
        "executable-guard",
        "docs-governance",
        "docs-links",
        "docs-schema-ref",
        "crate-boundary-foundation",
        "artifact-hardening",
        "performance-evidence",
        "test-trust-foundation",
        "test-trust-maintenance",
        "docs-config-reduction",
        "scheduler-invariants",
        "backend-contract",
        "cache-evolution",
        "replay-contract",
        "config-policy-determinism",
        "battle-suite-mandatory",
        "runtime-module-triage",
    ] {
        if !crate::suites::repo::IDS.contains(&required) {
            return Err(format!("foundation verification missing suite id: {required}"));
        }
    }
    Ok(())
}

pub(super) fn run_foundation_review_report() -> Result<(), String> {
    let root = repo_root()?;
    let runtime_src = root.join("crates/bijux-dag-runtime/src");
    let mut runtime_modules = Vec::new();
    collect_files_with_extension(&runtime_src, "rs", &mut runtime_modules)?;

    let docs_root = root.join("docs");
    let mut markdown = Vec::new();
    collect_markdown_files(&docs_root, &mut markdown)?;
    let docs_root_markdown_count =
        markdown.iter().filter(|path| path.parent() == Some(docs_root.as_path())).count();

    let report = json!({
        "runtime_module_count": runtime_modules.len(),
        "docs_root_markdown_count": docs_root_markdown_count,
        "repo_suite_count": crate::suites::repo::IDS.len(),
        "has_foundation_governance_posture": root.join("docs/reports/foundation/foundation-governance-posture.md").exists(),
    });
    println!("{}", serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?);
    Ok(())
}

pub(super) fn run_foundation_review_guard() -> Result<(), String> {
    let root = repo_root()?;
    for rel in [
        "docs/spec/FOUNDATION_READINESS_CRITERIA.md",
        "docs/spec/ARCHITECTURE_REVIEW_CHECKLIST.md",
        "docs/spec/FEATURE_DEVELOPMENT_FREEZE_POLICY.md",
        "docs/reports/foundation/repository_architecture_report.md",
        "docs/reports/foundation/runtime_module_ownership_report.md",
        "docs/reports/foundation/artifact_contract_report.md",
        "docs/reports/foundation/PERFORMANCE_EVIDENCE_REPORT.md",
        "docs/reports/foundation/TEST_TRUST_COVERAGE_REPORT.md",
        "docs/reports/foundation/RELEASE_EVIDENCE_REPORT.md",
        "docs/reports/foundation/repository-proof-statement.md",
        "docs/reports/foundation/foundation-governance-maintenance.md",
        "docs/reports/foundation/subsystem_strength_assessment.md",
        "docs/reports/foundation/foundation-governance-posture.md",
    ] {
        if !root.join(rel).exists() {
            return Err(format!("missing foundation review artifact: {rel}"));
        }
    }
    Ok(())
}

pub(super) fn run_control_plane_surfaces_guard() -> Result<(), String> {
    let root = repo_root()?;
    let commands = [
        "crates/bijux-dev/src/commands/cli.rs",
        "crates/bijux-dev/src/commands/cli_control_command.rs",
        "crates/bijux-dev/src/commands/cli_release_command.rs",
        "crates/bijux-dev/src/commands/mod.rs",
    ]
    .into_iter()
    .map(|path| {
        fs::read_to_string(root.join(path))
            .map_err(|err| format!("failed to read control-plane source {path}: {err}"))
    })
    .collect::<Result<Vec<_>, _>>()?
    .join("\n");
    for required in [
        "enum RepoCommand",
        "enum ReleaseCommand",
        "ArtifactVerify",
        "StorageHealth",
        "RunDirAudit",
        "Ci",
        "FoundationHardening",
        "ControlCommand::Run",
        "ReleaseCommand::Verify",
    ] {
        if !commands.contains(required) {
            return Err(format!("missing control-plane command surface: {required}"));
        }
    }
    let foundation = fs::read_to_string(root.join("docs/bijux-dev/operations/command-surface.md"))
        .map_err(|err| err.to_string())?;
    for required in [
        "contracts/foundation/maintainer_command_surface.v1.json",
        "`bijux-dev-dag` Root Surface",
        "`repo`",
        "`release`",
        "`verify`",
        "`dag`",
        "`foundation`",
        "`help`",
    ] {
        if !foundation.contains(required) {
            return Err(format!(
                "control-plane foundation doc missing required surface: {required}"
            ));
        }
    }
    Ok(())
}

pub(super) fn run_repo_hygiene_suite_guard() -> Result<(), String> {
    for required in [
        "repo-docs",
        "repo-source",
        "repo-manifests",
        "repo-api",
        "root-directory-guard",
        "executable-guard",
        "docs-governance",
        "docs-links",
        "docs-schema-ref",
        "config-lint",
        "config-drift",
        "ambient-env-guard",
        "evidence-authority",
    ] {
        if !crate::suites::repo::IDS.contains(&required) {
            return Err(format!("repo hygiene suite missing required guard: {required}"));
        }
    }
    Ok(())
}

fn evaluate_publishable_crates(root: &Path) -> Result<Value, String> {
    let metadata_json =
        command_stdout(root, "cargo", &["metadata", "--no-deps", "--format-version", "1"])?;
    let metadata: Value = serde_json::from_str(&metadata_json).map_err(|err| err.to_string())?;
    let packages = metadata["packages"]
        .as_array()
        .ok_or_else(|| "cargo metadata missing packages".to_string())?;

    let mut crate_reports = Vec::new();
    let mut violations = Vec::new();

    for package in packages {
        let publish = package.get("publish").cloned().unwrap_or(Value::Null);
        if matches!(publish, Value::Array(ref values) if values.is_empty()) {
            continue;
        }
        let Some(name) = package.get("name").and_then(Value::as_str) else {
            continue;
        };
        if !name.starts_with("bijux-") {
            continue;
        }
        let manifest_path = package
            .get("manifest_path")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("manifest_path missing for package {name}"))?;
        let manifest_path = PathBuf::from(manifest_path);
        let crate_dir = manifest_path
            .parent()
            .ok_or_else(|| format!("manifest has no parent: {}", manifest_path.display()))?
            .to_path_buf();

        let description_ok = package
            .get("description")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.trim().is_empty());
        if !description_ok {
            violations.push(format!("{name}: package description is required"));
        }
        let readme_rel = package
            .get("readme")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{name}: package readme is required"))?;
        let readme_path = crate_dir.join(readme_rel);
        if !readme_path.exists() {
            violations
                .push(format!("{name}: readme path does not exist: {}", readme_path.display()));
        }
        for (field, key) in [
            ("license", "license"),
            ("repository", "repository"),
            ("homepage", "homepage"),
            ("documentation", "documentation"),
        ] {
            if package.get(key).and_then(Value::as_str).is_none_or(|value| value.trim().is_empty())
            {
                violations.push(format!("{name}: package {field} is required"));
            }
        }
        if package.get("keywords").and_then(Value::as_array).is_none_or(|values| values.is_empty())
        {
            violations.push(format!("{name}: at least one keyword is required"));
        }
        if package
            .get("categories")
            .and_then(Value::as_array)
            .is_none_or(|values| values.is_empty())
        {
            violations.push(format!("{name}: at least one category is required"));
        }
        let targets = package.get("targets").and_then(Value::as_array).cloned().unwrap_or_default();
        let has_lib_or_bin = targets.iter().any(|target| {
            target
                .get("kind")
                .and_then(Value::as_array)
                .is_some_and(|kinds| kinds.iter().any(|kind| kind == "lib" || kind == "bin"))
        });
        if !has_lib_or_bin {
            violations.push(format!("{name}: publishable package must expose a lib or bin target"));
        }

        let package_list_status = Command::new("cargo")
            .args(["package", "-p", name, "--allow-dirty", "--list"])
            .current_dir(root)
            .status()
            .map_err(|err| format!("{name}: cargo package --list failed to start: {err}"))?;
        if !package_list_status.success() {
            violations.push(format!("{name}: cargo package --list failed"));
        }

        crate_reports.push(json!({
            "name": name,
            "manifest": manifest_path.strip_prefix(root).unwrap_or(&manifest_path).to_string_lossy(),
            "readme": readme_path.strip_prefix(root).unwrap_or(&readme_path).to_string_lossy(),
        }));
    }

    Ok(json!({
        "goal": "G191",
        "ok": violations.is_empty(),
        "publishable_crates": crate_reports,
        "violations": violations,
    }))
}

fn evaluate_python_bridge_distribution(root: &Path) -> Result<Value, String> {
    let bridge_root = root.join("crates/bijux-cli-python");
    let pyproject_path = bridge_root.join("pyproject.toml");
    let mut violations = Vec::new();

    let pyproject_text = fs::read_to_string(&pyproject_path)
        .map_err(|err| format!("failed to read {}: {err}", pyproject_path.display()))?;
    let pyproject: toml::Value = toml::from_str(&pyproject_text)
        .map_err(|err| format!("failed to parse pyproject: {err}"))?;
    let project = pyproject
        .get("project")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| "pyproject missing [project] table".to_string())?;
    if project.get("name").and_then(toml::Value::as_str) != Some("bijux-cli") {
        violations.push("pyproject [project].name must be `bijux-cli`".to_string());
    }
    if project
        .get("requires-python")
        .and_then(toml::Value::as_str)
        .is_none_or(|value| !value.contains("3.11"))
    {
        violations
            .push("pyproject [project].requires-python must advertise Python 3.11+".to_string());
    }
    let scripts = project
        .get("scripts")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| "pyproject missing [project.scripts] table".to_string())?;
    if scripts.get("bijux").and_then(toml::Value::as_str).is_none_or(str::is_empty) {
        violations.push("pyproject [project.scripts].bijux entry is required".to_string());
    }

    for rel in ["python/bijux_cli_py/__init__.py", "python/bijux_cli_py/cli.py", "README.md"] {
        if !bridge_root.join(rel).exists() {
            violations.push(format!("python bridge file missing: crates/bijux-cli-python/{rel}"));
        }
    }

    let metadata_json =
        command_stdout(root, "cargo", &["metadata", "--no-deps", "--format-version", "1"])?;
    let metadata: Value = serde_json::from_str(&metadata_json).map_err(|err| err.to_string())?;
    let packages = metadata["packages"]
        .as_array()
        .ok_or_else(|| "cargo metadata missing packages".to_string())?;
    let mut runtime_version = None::<String>;
    let mut bridge_version = None::<String>;
    for package in packages {
        let name = package.get("name").and_then(Value::as_str).unwrap_or_default();
        let version = package.get("version").and_then(Value::as_str).unwrap_or_default();
        if name == "bijux-cli" {
            runtime_version = Some(version.to_string());
        }
        if name == "bijux-cli-python" {
            bridge_version = Some(version.to_string());
        }
    }
    if runtime_version.is_none() || bridge_version.is_none() || runtime_version != bridge_version {
        violations.push(format!(
            "python bridge cargo crate version must match runtime crate version (runtime={:?}, bridge={:?})",
            runtime_version, bridge_version
        ));
    }

    let python_bin = if Command::new("python3").arg("--version").status().is_ok() {
        "python3"
    } else {
        "python"
    };
    let import_status = Command::new(python_bin)
        .args([
            "-c",
            "import pathlib,sys; sys.path.insert(0, str(pathlib.Path('crates/bijux-cli-python/python').resolve())); import bijux_cli_py, bijux_cli_py.cli; print(bijux_cli_py.__name__)",
        ])
        .current_dir(root)
        .status()
        .map_err(|err| format!("python import smoke failed to start: {err}"))?;
    if !import_status.success() {
        violations.push("python bridge import smoke failed".to_string());
    }

    let build_out_dir = root.join("artifacts/release/python-dist-smoke");
    fs::create_dir_all(&build_out_dir).map_err(|err| err.to_string())?;
    let build_status = Command::new(python_bin)
        .args([
            "-m",
            "build",
            "--sdist",
            "--wheel",
            "--outdir",
            build_out_dir.to_string_lossy().as_ref(),
            "crates/bijux-cli-python",
        ])
        .current_dir(root)
        .status()
        .map_err(|err| format!("python build smoke failed to start: {err}"))?;
    if !build_status.success() {
        violations.push("python bridge wheel/sdist build smoke failed".to_string());
    }

    Ok(json!({
        "goal": "G192",
        "ok": violations.is_empty(),
        "python_bin": python_bin,
        "output_dir": build_out_dir.strip_prefix(root).unwrap_or(&build_out_dir).to_string_lossy(),
        "violations": violations,
    }))
}

fn evaluate_release_artifacts_runnable(root: &Path) -> Result<Value, String> {
    let mut violations = Vec::new();
    let doc_rel = "docs/spec/RELEASE_BINARY_VERIFICATION.md";
    let scenario_rel = "configs/dag/release/release_smoke_scenarios.json";
    let policy = fs::read_to_string(root.join(doc_rel))
        .map_err(|err| format!("failed to read {doc_rel}: {err}"))?;
    for required_cmd in [
        "bijux --json doctor",
        "bijux --json cli paths",
        "bijux-dag validate --json evidence/authoring/examples/hello.dag.json",
        "bijux-dag validate --json evidence/authoring/examples/etl-constant-to-shell.dag.json",
        "bijux-dag run --json evidence/authoring/examples/hello.dag.json --out ${RUN_ROOT}",
        "bijux-dag run --json evidence/authoring/examples/etl-constant-to-shell.dag.json --out ${RUN_ROOT}",
        "bijux-dag explain --json ${RUN_DIR}",
    ] {
        if !policy.contains(required_cmd) {
            violations.push(format!("release binary verification doc missing `{required_cmd}`"));
        }
    }

    let scenarios_payload = fs::read_to_string(root.join(scenario_rel))
        .map_err(|err| format!("failed to read {scenario_rel}: {err}"))?;
    let scenarios: Value = serde_json::from_str(&scenarios_payload)
        .map_err(|err| format!("failed to parse {scenario_rel}: {err}"))?;
    let scenario_rows = scenarios["scenarios"]
        .as_array()
        .ok_or_else(|| "release smoke scenarios must contain `scenarios` array".to_string())?;
    if scenario_rows.len() < 2 {
        violations.push(
            "release smoke scenarios must include at least hello and shell ETL paths".to_string(),
        );
    }
    for row in scenario_rows {
        let id = row.get("id").and_then(Value::as_str).unwrap_or("<missing-id>");
        let graph = row
            .get("graph")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("scenario `{id}` is missing graph"))?;
        if !root.join(graph).exists() {
            violations.push(format!("scenario `{id}` graph does not exist: {graph}"));
        }
        let commands = row
            .get("required_commands")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("scenario `{id}` is missing required_commands"))?;
        if commands.is_empty() {
            violations.push(format!("scenario `{id}` must define at least one command"));
        }
    }

    Ok(json!({
        "goal": "G193",
        "ok": violations.is_empty(),
        "spec": doc_rel,
        "scenario_contract": scenario_rel,
        "scenario_count": scenario_rows.len(),
        "violations": violations,
    }))
}

fn evaluate_example_command_catalog(root: &Path) -> Result<Value, String> {
    let contract_rel = "configs/dag/release/example_command_catalog.json";
    let index_rel = "docs/reports/foundation/EXAMPLE_COMMAND_CATALOG.md";
    let contract_payload = fs::read_to_string(root.join(contract_rel))
        .map_err(|err| format!("failed to read {contract_rel}: {err}"))?;
    let contract: Value = serde_json::from_str(&contract_payload)
        .map_err(|err| format!("failed to parse {contract_rel}: {err}"))?;
    let rows = contract["commands"]
        .as_array()
        .ok_or_else(|| "example command catalog must contain `commands` array".to_string())?;

    let required = [
        "validate",
        "plan",
        "run",
        "replay",
        "diff",
        "cache",
        "artifact",
        "app-mount",
        "plugin",
        "bundle",
    ];
    let mut present = BTreeSet::new();
    let mut violations = Vec::new();
    for row in rows {
        let Some(command_id) = row.get("command_id").and_then(Value::as_str) else {
            violations.push("example command entry missing `command_id`".to_string());
            continue;
        };
        present.insert(command_id.to_string());
        let graph = row
            .get("graph")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("command `{command_id}` is missing graph"))?;
        if !root.join(graph).exists() {
            violations.push(format!("command `{command_id}` graph does not exist: {graph}"));
        }
        if row.get("command").and_then(Value::as_str).is_none_or(str::is_empty) {
            violations.push(format!("command `{command_id}` command is required"));
        }
    }
    for command_id in required {
        if !present.contains(command_id) {
            violations.push(format!("required command is missing from catalog: {command_id}"));
        }
    }

    let index_body = fs::read_to_string(root.join(index_rel))
        .map_err(|err| format!("failed to read {index_rel}: {err}"))?;
    for command_id in present {
        if !index_body.contains(&format!("`{command_id}`")) {
            violations.push(format!(
                "example command catalog markdown is missing command `{command_id}`"
            ));
        }
    }

    Ok(json!({
        "goal": "G194",
        "ok": violations.is_empty(),
        "contract": contract_rel,
        "index": index_rel,
        "task_count": rows.len(),
        "violations": violations,
    }))
}

fn evaluate_executable_docs_recipes(root: &Path) -> Result<Value, String> {
    let contract_rel = "configs/dag/release/executable_docs_recipes.json";
    let contract_payload = fs::read_to_string(root.join(contract_rel))
        .map_err(|err| format!("failed to read {contract_rel}: {err}"))?;
    let contract: Value = serde_json::from_str(&contract_payload)
        .map_err(|err| format!("failed to parse {contract_rel}: {err}"))?;
    let recipes = contract["recipes"]
        .as_array()
        .ok_or_else(|| "docs recipes contract must contain `recipes` array".to_string())?;

    let mut violations = Vec::new();
    let mut total_commands = 0usize;
    for recipe in recipes {
        let doc = recipe
            .get("doc")
            .and_then(Value::as_str)
            .ok_or_else(|| "recipe entry missing doc field".to_string())?;
        let commands = recipe
            .get("commands")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("recipe for `{doc}` missing commands array"))?;
        if commands.is_empty() {
            violations.push(format!("recipe for `{doc}` must include at least one command"));
            continue;
        }
        let body = fs::read_to_string(root.join(doc))
            .map_err(|err| format!("failed to read {doc}: {err}"))?;
        for command in commands {
            let Some(command) = command.as_str() else {
                violations.push(format!("recipe for `{doc}` contains non-string command entry"));
                continue;
            };
            total_commands += 1;
            if !(command.starts_with("bijux ")
                || command.starts_with("bijux-")
                || command.starts_with("python -m "))
            {
                violations.push(format!(
                    "recipe command must start with `bijux`, `bijux-*`, or `python -m`: {command}"
                ));
            }
            if !body.contains(command) {
                violations
                    .push(format!("documentation `{doc}` is missing recipe command `{command}`"));
            }
        }
    }

    if !root.join("crates/bijux-dag-app/tests/docs_executable_recipes_contract.rs").exists() {
        violations.push("missing docs executable recipe contract test".to_string());
    }

    Ok(json!({
        "goal": "G195",
        "ok": violations.is_empty(),
        "contract": contract_rel,
        "recipe_count": recipes.len(),
        "command_count": total_commands,
        "violations": violations,
    }))
}

fn evaluate_install_paths_predictable(root: &Path) -> Result<Value, String> {
    let contract_rel = "configs/dag/release/install_path_contract.json";
    let contract_payload = fs::read_to_string(root.join(contract_rel))
        .map_err(|err| format!("failed to read {contract_rel}: {err}"))?;
    let contract: Value = serde_json::from_str(&contract_payload)
        .map_err(|err| format!("failed to parse {contract_rel}: {err}"))?;

    let paths_output = command_stdout(
        root,
        "cargo",
        &["run", "-q", "-p", "bijux-cli", "--", "--json", "cli", "paths"],
    )?;
    let paths_payload: Value = serde_json::from_str(&paths_output)
        .map_err(|err| format!("cli paths output is not JSON: {err}"))?;
    let doctor_output =
        command_stdout(root, "cargo", &["run", "-q", "-p", "bijux-cli", "--", "--json", "doctor"])?;
    let doctor_payload: Value = serde_json::from_str(&doctor_output)
        .map_err(|err| format!("doctor output is not JSON: {err}"))?;

    let mut violations = Vec::new();
    for key in contract["required_keys"]
        .as_array()
        .ok_or_else(|| "install path contract missing required_keys".to_string())?
    {
        let Some(key) = key.as_str() else {
            violations.push("required_keys must contain strings".to_string());
            continue;
        };
        if paths_payload.get(key).is_none() {
            violations.push(format!("cli paths output missing required key `{key}`"));
            continue;
        }
        if matches!(paths_payload.get(key), Some(Value::String(value)) if value.trim().is_empty()) {
            violations.push(format!("cli paths key `{key}` must not be empty"));
        }
    }
    for section in contract["doctor_required_sections"]
        .as_array()
        .ok_or_else(|| "install path contract missing doctor_required_sections".to_string())?
    {
        let Some(section) = section.as_str() else {
            violations.push("doctor_required_sections must contain strings".to_string());
            continue;
        };
        if doctor_payload.get(section).is_none() {
            violations.push(format!("doctor output missing required section `{section}`"));
        }
    }

    Ok(json!({
        "goal": "G196",
        "ok": violations.is_empty(),
        "contract": contract_rel,
        "paths_command": contract["command"],
        "doctor_command": contract["doctor_command"],
        "violations": violations,
    }))
}

fn evaluate_local_dev_loop_focus(root: &Path) -> Result<Value, String> {
    let contract_rel = "configs/dag/release/change_impact_commands.json";
    let contract_payload = fs::read_to_string(root.join(contract_rel))
        .map_err(|err| format!("failed to read {contract_rel}: {err}"))?;
    let contract: Value = serde_json::from_str(&contract_payload)
        .map_err(|err| format!("failed to parse {contract_rel}: {err}"))?;
    let lanes = contract["lanes"]
        .as_array()
        .ok_or_else(|| "change-impact contract must contain `lanes` array".to_string())?;

    let mut violations = Vec::new();
    let mut lane_map: Vec<(String, Vec<String>, Vec<String>)> = Vec::new();
    for lane in lanes {
        let id = lane
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "change-impact lane missing id".to_string())?;
        let prefixes = lane["path_prefixes"]
            .as_array()
            .ok_or_else(|| format!("lane `{id}` missing path_prefixes"))?
            .iter()
            .filter_map(Value::as_str)
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        let commands = lane["commands"]
            .as_array()
            .ok_or_else(|| format!("lane `{id}` missing commands"))?
            .iter()
            .filter_map(Value::as_str)
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        if prefixes.is_empty() {
            violations.push(format!("lane `{id}` requires at least one path prefix"));
        }
        if commands.is_empty() {
            violations.push(format!("lane `{id}` requires at least one verification command"));
        }
        lane_map.push((id.to_string(), prefixes, commands));
    }

    let required = ["cli", "dag-core", "runtime", "artifacts", "app"];
    for id in required {
        if !lane_map.iter().any(|(lane_id, _, _)| lane_id == id) {
            violations.push(format!("change-impact contract missing required lane `{id}`"));
        }
    }

    let changed = command_stdout(root, "git", &["status", "--porcelain"])?;
    let changed_files = changed
        .lines()
        .filter_map(|line| {
            let path = line.get(3..)?.trim();
            if path.is_empty() {
                None
            } else {
                Some(path.to_string())
            }
        })
        .collect::<Vec<_>>();
    let mut selected_lanes = BTreeSet::new();
    let mut selected_commands = BTreeSet::new();
    for file in &changed_files {
        for (lane_id, prefixes, commands) in &lane_map {
            if prefixes.iter().any(|prefix| file.starts_with(prefix)) {
                selected_lanes.insert(lane_id.clone());
                for command in commands {
                    selected_commands.insert(command.clone());
                }
            }
        }
    }

    let report_path = root.join("artifacts/release/change_impact_runner_report.json");
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    write_pretty_json(
        &report_path,
        &json!({
            "contract": contract_rel,
            "changed_files": changed_files,
            "selected_lanes": selected_lanes,
            "selected_commands": selected_commands,
        }),
    )?;

    Ok(json!({
        "goal": "G197",
        "ok": violations.is_empty(),
        "contract": contract_rel,
        "changed_file_count": changed_files.len(),
        "selected_lane_count": selected_lanes.len(),
        "selected_command_count": selected_commands.len(),
        "report": report_path.strip_prefix(root).unwrap_or(&report_path).to_string_lossy(),
        "violations": violations,
    }))
}

fn evaluate_app_integration_documentation(root: &Path) -> Result<Value, String> {
    let contract_rel = "configs/dag/release/app_integration_scenario.json";
    let contract_payload = fs::read_to_string(root.join(contract_rel))
        .map_err(|err| format!("failed to read {contract_rel}: {err}"))?;
    let contract: Value = serde_json::from_str(&contract_payload)
        .map_err(|err| format!("failed to parse {contract_rel}: {err}"))?;
    let doc_rel = contract
        .get("doc")
        .and_then(Value::as_str)
        .ok_or_else(|| "app integration contract missing doc".to_string())?;
    let doc_body = fs::read_to_string(root.join(doc_rel))
        .map_err(|err| format!("failed to read {doc_rel}: {err}"))?;

    let mut violations = Vec::new();
    let required_assets = contract["required_assets"]
        .as_array()
        .ok_or_else(|| "app integration contract missing required_assets".to_string())?;
    for asset in required_assets {
        let Some(asset) = asset.as_str() else {
            violations.push("required_assets entries must be strings".to_string());
            continue;
        };
        if !root.join(asset).exists() {
            violations.push(format!("required app integration asset is missing: {asset}"));
        }
        if !doc_body.contains(asset) {
            violations.push(format!("app integration doc is missing asset reference: {asset}"));
        }
    }

    let required_commands = contract["required_commands"]
        .as_array()
        .ok_or_else(|| "app integration contract missing required_commands".to_string())?;
    let mut saw_apps = false;
    let mut saw_plugins = false;
    for command in required_commands {
        let Some(command) = command.as_str() else {
            violations.push("required_commands entries must be strings".to_string());
            continue;
        };
        saw_apps |= command.starts_with("bijux apps ");
        saw_plugins |= command.starts_with("bijux plugins ");
        if !doc_body.contains(command) {
            violations.push(format!("app integration doc missing required command: {command}"));
        }
    }
    if !saw_apps {
        violations.push(
            "app integration scenario must include at least one `bijux apps` command".to_string(),
        );
    }
    if !saw_plugins {
        violations.push(
            "app integration scenario must include at least one `bijux plugins` command"
                .to_string(),
        );
    }

    Ok(json!({
        "goal": "G198",
        "ok": violations.is_empty(),
        "contract": contract_rel,
        "doc": doc_rel,
        "required_command_count": required_commands.len(),
        "required_asset_count": required_assets.len(),
        "violations": violations,
    }))
}

fn evaluate_limitations_visibility(root: &Path) -> Result<Value, String> {
    let contract_rel = "configs/dag/release/limitations_visibility_contract.json";
    let contract_payload = fs::read_to_string(root.join(contract_rel))
        .map_err(|err| format!("failed to read {contract_rel}: {err}"))?;
    let contract: Value = serde_json::from_str(&contract_payload)
        .map_err(|err| format!("failed to parse {contract_rel}: {err}"))?;
    let commands = contract["commands"]
        .as_array()
        .ok_or_else(|| "limitations visibility contract must contain commands array".to_string())?;

    let mut violations = Vec::new();
    let mut command_reports = Vec::new();
    for command in commands {
        let id = command.get("id").and_then(Value::as_str).unwrap_or("<missing-id>");
        let run = command.get("run").and_then(Value::as_str).unwrap_or("");
        if run.trim().is_empty() {
            violations.push(format!("command `{id}` is missing run field"));
            continue;
        }
        let Some(program) = command.get("program").and_then(Value::as_str) else {
            violations.push(format!("command `{id}` is missing program field"));
            continue;
        };
        let Some(args) = command.get("args").and_then(Value::as_array) else {
            violations.push(format!("command `{id}` is missing args field"));
            continue;
        };
        let mut argv = Vec::with_capacity(args.len());
        let mut args_valid = true;
        for arg in args {
            let Some(arg) = arg.as_str() else {
                violations.push(format!("command `{id}` args entries must be strings"));
                args_valid = false;
                break;
            };
            argv.push(arg.to_string());
        }
        if !args_valid {
            continue;
        }
        let mut command_process = Command::new(program);
        command_process.current_dir(root).args(&argv);
        if let Some(env) = command.get("env").and_then(Value::as_object) {
            for (key, value) in env {
                let Some(value) = value.as_str() else {
                    violations.push(format!("command `{id}` env values must be strings"));
                    args_valid = false;
                    break;
                };
                command_process.env(key, value);
            }
        }
        if !args_valid {
            continue;
        }
        let command_output = command_process.output().map_err(|err| {
            format!("command `{id}` failed to spawn `{program}` from contract: {err}")
        })?;
        if !command_output.status.success() {
            violations.push(format!("command `{id}` failed: {} {}", program, argv.join(" ")));
            continue;
        }
        let output = String::from_utf8(command_output.stdout)
            .map_err(|err| format!("command `{id}` stdout was not utf-8: {err}"))?;
        if id != "dag-capabilities" && id != "root-doctor" {
            violations.push(format!("unknown limitations command id `{id}`"));
            continue;
        }
        let payload: Value = serde_json::from_str(&output)
            .map_err(|err| format!("command `{id}` did not emit JSON: {err}"))?;
        let rendered = serde_json::to_string(&payload).map_err(|err| err.to_string())?;
        for token in command["required_tokens"]
            .as_array()
            .ok_or_else(|| format!("command `{id}` missing required_tokens"))?
        {
            let Some(token) = token.as_str() else {
                violations.push(format!("command `{id}` required_tokens must be strings"));
                continue;
            };
            if !rendered.contains(token) {
                violations.push(format!("command `{id}` output missing required token `{token}`"));
            }
        }

        if id == "dag-capabilities" {
            let capabilities =
                payload["data"]["backend_capabilities"].as_array().cloned().unwrap_or_default();
            if capabilities.is_empty() {
                violations
                    .push("dag capabilities must return backend_capabilities entries".to_string());
            }
            for row in capabilities {
                if row.get("status").is_none() || row.get("production_ready").is_none() {
                    violations.push(
                        "dag capabilities backend entries must include status and production_ready"
                            .to_string(),
                    );
                    break;
                }
            }
        }
        command_reports.push(json!({"id": id, "ok": true}));
    }

    Ok(json!({
        "goal": "G199",
        "ok": violations.is_empty(),
        "contract": contract_rel,
        "commands_checked": command_reports,
        "violations": violations,
    }))
}

fn evaluate_production_candidate_suite(root: &Path) -> Result<Value, String> {
    let contract_rel = "configs/dag/release/production_candidate_suite.json";
    let contract_payload = fs::read_to_string(root.join(contract_rel))
        .map_err(|err| format!("failed to read {contract_rel}: {err}"))?;
    let contract: Value = serde_json::from_str(&contract_payload)
        .map_err(|err| format!("failed to parse {contract_rel}: {err}"))?;
    let required_steps = contract["required_steps"]
        .as_array()
        .ok_or_else(|| "production candidate suite must define required_steps".to_string())?;
    let required_set: BTreeSet<String> =
        required_steps.iter().filter_map(Value::as_str).map(ToOwned::to_owned).collect();

    let mut violations = Vec::new();
    for required in [
        "root.status",
        "root.doctor",
        "apps.list",
        "plugins.list",
        "dag.validate",
        "dag.plan",
        "dag.run",
        "dag.replay",
        "dag.diff",
        "dag.export",
        "dag.import",
    ] {
        if !required_set.contains(required) {
            violations
                .push(format!("production candidate suite missing required step `{required}`"));
        }
    }

    let tmp = tempfile::tempdir().map_err(|err| err.to_string())?;
    let run_root = tmp.path().join("runs");
    fs::create_dir_all(&run_root).map_err(|err| err.to_string())?;
    let graph = "evidence/authoring/examples/hello.dag.json";
    let export_bundle = tmp.path().join("hello.export.json");

    let mut executed = Vec::new();
    let mut run_step = |id: &str, cmd: &str, args: &[&str]| -> Result<(), String> {
        let status = Command::new(cmd)
            .args(args)
            .current_dir(root)
            .status()
            .map_err(|err| format!("step `{id}` failed to start: {err}"))?;
        executed.push(json!({
            "id": id,
            "command": format!("{} {}", cmd, args.join(" ")),
            "success": status.success(),
            "code": status.code(),
        }));
        if !status.success() {
            violations.push(format!("step `{id}` failed with status {status}"));
        }
        Ok(())
    };

    run_step("root.status", "cargo", &["run", "-q", "-p", "bijux-cli", "--", "--json", "status"])?;
    run_step("root.doctor", "cargo", &["run", "-q", "-p", "bijux-cli", "--", "--json", "doctor"])?;
    run_step(
        "apps.list",
        "cargo",
        &["run", "-q", "-p", "bijux-cli", "--", "--json", "apps", "list"],
    )?;
    run_step(
        "plugins.list",
        "cargo",
        &["run", "-q", "-p", "bijux-cli", "--", "--json", "plugins", "list"],
    )?;
    run_step(
        "dag.validate",
        "cargo",
        &[
            "run",
            "-q",
            "-p",
            "bijux-dag-cli",
            "--bin",
            "bijux-dag",
            "--",
            "validate",
            "--json",
            graph,
        ],
    )?;
    run_step(
        "dag.plan",
        "cargo",
        &[
            "run",
            "-q",
            "-p",
            "bijux-dag-cli",
            "--bin",
            "bijux-dag",
            "--",
            "plan",
            "explain",
            "--json",
            graph,
        ],
    )?;
    run_step(
        "dag.run",
        "cargo",
        &[
            "run",
            "-q",
            "-p",
            "bijux-dag-cli",
            "--bin",
            "bijux-dag",
            "--",
            "run",
            "--json",
            graph,
            "--out",
            run_root.to_string_lossy().as_ref(),
        ],
    )?;

    let first_run = newest_run(&run_root)?;
    run_step(
        "dag.replay",
        "cargo",
        &[
            "run",
            "-q",
            "-p",
            "bijux-dag-cli",
            "--bin",
            "bijux-dag",
            "--",
            "replay",
            "--json",
            first_run.to_string_lossy().as_ref(),
            "--out",
            run_root.to_string_lossy().as_ref(),
        ],
    )?;
    let replay_run = newest_run(&run_root)?;
    run_step(
        "dag.diff",
        "cargo",
        &[
            "run",
            "-q",
            "-p",
            "bijux-dag-cli",
            "--bin",
            "bijux-dag",
            "--",
            "diff",
            "--json",
            first_run.to_string_lossy().as_ref(),
            replay_run.to_string_lossy().as_ref(),
        ],
    )?;
    run_step(
        "dag.export",
        "cargo",
        &[
            "run",
            "-q",
            "-p",
            "bijux-dag-cli",
            "--bin",
            "bijux-dag",
            "--",
            "export",
            "--json",
            first_run.to_string_lossy().as_ref(),
            "--out",
            export_bundle.to_string_lossy().as_ref(),
        ],
    )?;
    run_step(
        "dag.import",
        "cargo",
        &[
            "run",
            "-q",
            "-p",
            "bijux-dag-cli",
            "--bin",
            "bijux-dag",
            "--",
            "import",
            "--json",
            "--verify-only",
            export_bundle.to_string_lossy().as_ref(),
        ],
    )?;

    let report_path = root.join("artifacts/release/production_candidate_bundle.json");
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let report = json!({
        "goal": "G200",
        "contract": contract_rel,
        "ok": violations.is_empty(),
        "executed_steps": executed,
        "violations": violations,
    });
    write_pretty_json(&report_path, &report)?;
    Ok(report)
}

pub(super) fn deep_merge_json(target: &mut Value, overlay: &Value) {
    match (target, overlay) {
        (Value::Object(dst), Value::Object(src)) => {
            for (key, value) in src {
                match dst.get_mut(key) {
                    Some(existing) => deep_merge_json(existing, value),
                    None => {
                        dst.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        (target, overlay) => {
            *target = overlay.clone();
        }
    }
}
