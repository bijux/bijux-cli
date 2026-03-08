use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile as _;

#[derive(Debug, Clone)]
struct FileCoverage {
    covered_lines: u64,
    instrumented_lines: u64,
}

impl FileCoverage {
    fn pct(&self) -> f64 {
        if self.instrumented_lines == 0 {
            100.0
        } else {
            (self.covered_lines as f64 / self.instrumented_lines as f64) * 100.0
        }
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn to_repo_relative(root: &Path, raw: &str) -> String {
    let path = Path::new(raw);
    if path.is_absolute() {
        path.strip_prefix(root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| raw.replace('\\', "/"))
    } else {
        raw.replace('\\', "/")
    }
}

fn parse_lcov(root: &Path, lcov_path: &Path) -> Result<BTreeMap<String, FileCoverage>, String> {
    let content = fs::read_to_string(lcov_path)
        .map_err(|err| format!("failed to read {}: {err}", lcov_path.display()))?;

    let mut map = BTreeMap::<String, FileCoverage>::new();
    let mut current_file: Option<String> = None;
    let mut covered = 0u64;
    let mut total = 0u64;

    for line in content.lines() {
        if let Some(sf) = line.strip_prefix("SF:") {
            if let Some(file) = current_file.take() {
                map.insert(
                    file,
                    FileCoverage {
                        covered_lines: covered,
                        instrumented_lines: total,
                    },
                );
            }
            current_file = Some(to_repo_relative(root, sf));
            covered = 0;
            total = 0;
            continue;
        }
        if let Some(da) = line.strip_prefix("DA:") {
            let mut parts = da.split(',');
            let _line_no = parts.next();
            let hits = parts
                .next()
                .ok_or_else(|| format!("invalid DA row in lcov: {line}"))?
                .parse::<u64>()
                .map_err(|err| format!("invalid DA hit count in `{line}`: {err}"))?;
            total += 1;
            if hits > 0 {
                covered += 1;
            }
            continue;
        }
        if line == "end_of_record" {
            if let Some(file) = current_file.take() {
                map.insert(
                    file,
                    FileCoverage {
                        covered_lines: covered,
                        instrumented_lines: total,
                    },
                );
            }
            covered = 0;
            total = 0;
        }
    }

    if let Some(file) = current_file.take() {
        map.insert(
            file,
            FileCoverage {
                covered_lines: covered,
                instrumented_lines: total,
            },
        );
    }

    Ok(map)
}

fn rust_crate_source_files<'a>(
    entries: impl Iterator<Item = (&'a String, &'a FileCoverage)>,
) -> Vec<(String, FileCoverage)> {
    entries
        .filter(|(path, _)| {
            path.starts_with("crates/") && path.ends_with(".rs") && path.contains("/src/")
        })
        .map(|(path, cov)| (path.clone(), cov.clone()))
        .collect()
}

fn render_threshold_report(
    title: &str,
    subtitle: &str,
    rows: &[(String, FileCoverage)],
    threshold: f64,
) -> String {
    let mut lines = vec![
        format!("# {title}"),
        String::new(),
        subtitle.to_string(),
        String::new(),
        "| file | covered_lines | instrumented_lines | line_coverage_pct |".to_string(),
        "| --- | ---: | ---: | ---: |".to_string(),
    ];
    for (path, cov) in rows {
        if cov.pct() < threshold {
            lines.push(format!(
                "| {path} | {} | {} | {:.2} |",
                cov.covered_lines,
                cov.instrumented_lines,
                cov.pct()
            ));
        }
    }
    if lines.len() == 6 {
        lines.push("| (none) | 0 | 0 | 100.00 |".to_string());
    }
    lines.push(String::new());
    lines.push(
        "_Generated from `artifacts/coverage/lcov.info` by `generate_line_coverage_reports`._"
            .to_string(),
    );
    lines.push(String::new());
    lines.join("\n")
}

fn protected_zero_files(rows: &[(String, FileCoverage)]) -> BTreeSet<String> {
    rows.iter()
        .filter(|(path, cov)| {
            cov.instrumented_lines > 0
                && cov.covered_lines == 0
                && (path.starts_with("crates/bijux-dag-core/src/")
                    || path.starts_with("crates/bijux-dag-runtime/src/")
                    || path.starts_with("crates/bijux-dag-app/src/")
                    || path.starts_with("crates/bijux-dag-artifacts/src/"))
        })
        .map(|(path, _)| path.clone())
        .collect()
}

fn read_allowlist(path: &Path) -> Result<BTreeSet<String>, String> {
    let payload = fs::read_to_string(path)
        .map_err(|err| format!("failed to read allowlist {}: {err}", path.display()))?;
    let parsed: serde_json::Value =
        serde_json::from_str(&payload).map_err(|err| format!("invalid allowlist json: {err}"))?;
    let arr = parsed["protected_zero_coverage_allowlist"]
        .as_array()
        .ok_or_else(|| {
            "allowlist must contain `protected_zero_coverage_allowlist` array".to_string()
        })?;
    let mut out = BTreeSet::new();
    for item in arr {
        let s = item
            .as_str()
            .ok_or_else(|| "allowlist entries must be strings".to_string())?;
        out.insert(s.to_string());
    }
    Ok(out)
}

fn write_text(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(path, content).map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn main() -> Result<(), String> {
    let root = repo_root();
    let lcov_path = root.join("artifacts/coverage/lcov.info");
    let allowlist_path = root.join("configs/policy/protected_zero_coverage_allowlist.json");
    let out_under_50 = root.join("docs/reports/foundation/line_coverage_under_50_report.md");
    let out_under_25 = root.join("docs/reports/foundation/line_coverage_under_25_report.md");
    let out_zero = root.join("docs/reports/foundation/line_coverage_zero_direct_report.md");

    if !lcov_path.exists() {
        let msg = "# Coverage report unavailable\n\n`artifacts/coverage/lcov.info` was not found. Run `make coverage` first.\n";
        write_text(&out_under_50, msg)?;
        write_text(&out_under_25, msg)?;
        write_text(&out_zero, msg)?;
        println!("coverage input missing; wrote placeholder reports");
        return Ok(());
    }

    let parsed = parse_lcov(&root, &lcov_path)?;
    let mut rows = rust_crate_source_files(parsed.iter());
    rows.sort_by(|a, b| a.0.cmp(&b.0));

    write_text(
        &out_under_50,
        &render_threshold_report(
            "Line Coverage Under 50% Report",
            "Files in crate `src/` trees below 50% line coverage.",
            &rows,
            50.0,
        ),
    )?;
    write_text(
        &out_under_25,
        &render_threshold_report(
            "Line Coverage Under 25% Report",
            "Files in crate `src/` trees below 25% line coverage.",
            &rows,
            25.0,
        ),
    )?;

    let mut zero_lines = vec![
        "# Zero Direct Line Coverage Report".to_string(),
        String::new(),
        "Files in crate `src/` trees with 0% line coverage.".to_string(),
        String::new(),
        "| file | covered_lines | instrumented_lines | line_coverage_pct |".to_string(),
        "| --- | ---: | ---: | ---: |".to_string(),
    ];
    let zero_rows: Vec<_> = rows
        .iter()
        .filter(|(_, cov)| cov.instrumented_lines > 0 && cov.covered_lines == 0)
        .collect();
    for (path, cov) in &zero_rows {
        zero_lines.push(format!(
            "| {path} | {} | {} | 0.00 |",
            cov.covered_lines, cov.instrumented_lines
        ));
    }
    if zero_rows.is_empty() {
        zero_lines.push("| (none) | 0 | 0 | 100.00 |".to_string());
    }
    zero_lines.push(String::new());
    zero_lines.push(
        "_Generated from `artifacts/coverage/lcov.info` by `generate_line_coverage_reports`._"
            .to_string(),
    );
    zero_lines.push(String::new());
    write_text(&out_zero, &zero_lines.join("\n"))?;

    let protected_zeros = protected_zero_files(&rows);
    let allowlist = read_allowlist(&allowlist_path)?;
    let new_protected: Vec<_> = protected_zeros.difference(&allowlist).cloned().collect();
    if !new_protected.is_empty() {
        let mut msg = String::from("new protected-crate 0%-coverage files detected:\n");
        for file in new_protected {
            msg.push_str(&format!("- {file}\n"));
        }
        msg.push_str(
            "\nIf intentional, update configs/policy/protected_zero_coverage_allowlist.json.\n",
        );
        return Err(msg);
    }

    println!("generated coverage threshold reports");
    Ok(())
}
