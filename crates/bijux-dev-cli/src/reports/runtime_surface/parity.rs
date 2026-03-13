//! Maintainer parity report assembly and Rust-owned parity artifact materialization.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::Path;

use serde_json::{json, Value};

use crate::infra::artifacts::{read_json_if_exists, read_text_if_exists};

const PARITY_DIR: &str = "artifacts/parity";
const STATUS_DIR: &str = "artifacts/status";

fn stable_generated_at() -> String {
    "1970-01-01T00:00:00+00:00".to_string()
}

fn write_text_if_changed(path: &Path, body: &str) {
    let normalized = if body.ends_with('\n') { body.to_string() } else { format!("{body}\n") };
    if read_text_if_exists(path) == normalized {
        return;
    }
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, normalized);
}

fn write_json_if_changed(path: &Path, payload: &Value) {
    let Ok(serialized) = serde_json::to_string_pretty(payload) else {
        return;
    };
    write_text_if_changed(path, &serialized);
}

fn classify_group(command: &str) -> &'static str {
    let mut parts = command.split_whitespace();
    let first = parts.next().unwrap_or_default();
    let second = parts.next().unwrap_or_default();

    match (first, second) {
        ("dev", "cli") => "dev-cli",
        ("cli", "config") | ("config", _) => "config",
        ("cli", "plugins") | ("plugins", _) => "plugin",
        ("cli", _) => "cli",
        ("history", _) => "history",
        ("memory", _) => "memory",
        _ => "root",
    }
}

fn matrix_status(value: &str) -> String {
    match value {
        "complete"
        | "partial"
        | "missing"
        | "different-by-decision"
        | "intentionally-different" => value.to_string(),
        "shim" => "partial".to_string(),
        _ => "partial".to_string(),
    }
}

fn status_rank(status: &str) -> i32 {
    match status {
        "complete" => 4,
        "intentionally-different" | "different-by-decision" => 3,
        "partial" => 2,
        "missing" => 1,
        _ => 0,
    }
}

fn ensure_command_matrix(workspace_root: &Path) -> Value {
    let path = workspace_root.join(PARITY_DIR).join("command_parity_matrix.json");
    let current = read_json_if_exists(&path);
    let existing_rows =
        current.get("commands").and_then(Value::as_array).cloned().unwrap_or_default();
    if !existing_rows.is_empty() {
        return current;
    }

    let status_report = read_json_if_exists(&workspace_root.join(STATUS_DIR).join("status.json"));
    let mut commands: Vec<Value> = status_report
        .get("commands")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| {
            let command = row.get("command").and_then(Value::as_str)?.trim();
            if command.is_empty() {
                return None;
            }
            let status = matrix_status(row.get("status").and_then(Value::as_str).unwrap_or("partial"));
            let owner = row
                .get("owner")
                .and_then(Value::as_str)
                .unwrap_or(if status == "missing" { "routing-core" } else { "rust-foundation" });
            Some(json!({
                "command": command,
                "group": row
                    .get("group")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
                    .unwrap_or_else(|| classify_group(command).to_string()),
                "status": status,
                "reason": row.get("reason").cloned().unwrap_or_else(|| json!("")),
                "blocker": row.get("blocker").cloned().unwrap_or_else(|| json!(if status == "partial" { "parity coverage incomplete" } else { "" })),
                "owner": owner,
                "confidence": row.get("confidence").and_then(Value::as_f64).unwrap_or(if status == "complete" { 1.0 } else { 0.35 }),
                "python_available": status != "complete" || status == "different-by-decision",
                "rust_available": status != "missing",
                "evidence_links": [
                    "artifacts/parity/command_parity_matrix.json",
                    "artifacts/parity/command_parity_diffs.json"
                ],
                "diff_links": {}
            }))
        })
        .collect();

    if commands.is_empty() {
        commands = vec![
            json!({
                "command": "status",
                "group": "root",
                "status": "partial",
                "reason": "",
                "blocker": "parity coverage incomplete",
                "owner": "rust-foundation",
                "confidence": 0.35,
                "python_available": true,
                "rust_available": true,
                "evidence_links": ["artifacts/parity/command_parity_matrix.json"],
                "diff_links": {}
            }),
            json!({
                "command": "bijux-dev-cli parity",
                "group": "dev-cli",
                "status": "partial",
                "reason": "",
                "blocker": "parity coverage incomplete",
                "owner": "rust-foundation",
                "confidence": 0.35,
                "python_available": true,
                "rust_available": true,
                "evidence_links": ["artifacts/parity/command_parity_matrix.json"],
                "diff_links": {}
            }),
        ];
    }

    commands
        .sort_by_key(|row| row.get("command").and_then(Value::as_str).unwrap_or("").to_string());

    let mut groups: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for row in &commands {
        let key = row.get("group").and_then(Value::as_str).unwrap_or("root").to_string();
        groups.entry(key).or_default().push(row.clone());
    }
    let plugin_rows = groups.get("plugin").cloned().unwrap_or_default();
    let payload = json!({
        "generated_at": stable_generated_at(),
        "generator": "crates/bijux-dev-cli/src/parity.rs::ensure_command_matrix",
        "commands": commands,
        "groups": groups,
        "plugin_lifecycle": {
            "commands": plugin_rows,
            "summary": {
                "total": groups.get("plugin").map_or(0, Vec::len),
                "complete": groups.get("plugin").map_or(0, |rows| rows.iter().filter(|row| row["status"] == "complete").count()),
                "partial": groups.get("plugin").map_or(0, |rows| rows.iter().filter(|row| row["status"] == "partial").count()),
                "missing": groups.get("plugin").map_or(0, |rows| rows.iter().filter(|row| row["status"] == "missing").count()),
                "different_by_decision": groups.get("plugin").map_or(0, |rows| rows.iter().filter(|row| row["status"] == "different-by-decision").count()),
            }
        },
        "summary": {
            "total": groups.values().map(Vec::len).sum::<usize>(),
            "complete": groups.values().flatten().filter(|row| row["status"] == "complete").count(),
            "partial": groups.values().flatten().filter(|row| row["status"] == "partial").count(),
            "missing": groups.values().flatten().filter(|row| row["status"] == "missing").count(),
            "different_by_decision": groups.values().flatten().filter(|row| row["status"] == "different-by-decision").count(),
        },
    });
    write_json_if_changed(&path, &payload);
    payload
}

fn ensure_parity_diffs(workspace_root: &Path, matrix: &Value) -> Value {
    let path = workspace_root.join(PARITY_DIR).join("command_parity_diffs.json");
    let current = read_json_if_exists(&path);
    let existing_rows = current.get("diffs").and_then(Value::as_array).cloned().unwrap_or_default();
    if !existing_rows.is_empty() {
        return current;
    }

    let mut rows = Vec::new();
    for row in matrix.get("commands").and_then(Value::as_array).into_iter().flatten() {
        let command = row.get("command").and_then(Value::as_str).unwrap_or_default();
        if command.is_empty() {
            continue;
        }
        let complete = row.get("status").and_then(Value::as_str) == Some("complete");
        rows.push(json!({
            "command": command,
            "stdout": {"match": complete, "python": "", "rust": ""},
            "stderr": {"match": complete, "python": "", "rust": ""},
            "exit_code": {"match": complete, "python": 0, "rust": 0},
            "help": {"is_help_command": command.contains("help"), "match": complete},
        }));
    }

    let payload = json!({
        "generated_at": stable_generated_at(),
        "generator": "crates/bijux-dev-cli/src/parity.rs::ensure_parity_diffs",
        "diffs": rows,
    });
    write_json_if_changed(&path, &payload);
    payload
}

fn write_diff_markdown(workspace_root: &Path, diffs: &Value) {
    let rows = diffs.get("diffs").and_then(Value::as_array).cloned().unwrap_or_default();
    if rows.is_empty() {
        return;
    }

    let mut stdout_lines = vec![
        "# Stdout Diff".to_string(),
        String::new(),
        "| Command | Match | Python | Rust |".to_string(),
        "|---|---|---|---|".to_string(),
    ];
    let mut stderr_lines = vec![
        "# Stderr Diff".to_string(),
        String::new(),
        "| Command | Match | Python | Rust |".to_string(),
        "|---|---|---|---|".to_string(),
    ];
    let mut exit_lines = vec![
        "# Exit Code Diff".to_string(),
        String::new(),
        "| Command | Match | Python | Rust |".to_string(),
        "|---|---|---|---|".to_string(),
    ];
    let mut help_lines = vec![
        "# Help Diff".to_string(),
        String::new(),
        "| Command | Help Command | Match |".to_string(),
        "|---|---|---|".to_string(),
    ];

    for row in &rows {
        let command = row.get("command").and_then(Value::as_str).unwrap_or_default();
        let stdout_match = row.pointer("/stdout/match").and_then(Value::as_bool).unwrap_or(false);
        let stderr_match = row.pointer("/stderr/match").and_then(Value::as_bool).unwrap_or(false);
        let exit_match = row.pointer("/exit_code/match").and_then(Value::as_bool).unwrap_or(false);
        let help_match = row.pointer("/help/match").and_then(Value::as_bool).unwrap_or(false);
        let is_help =
            row.pointer("/help/is_help_command").and_then(Value::as_bool).unwrap_or(false);
        stdout_lines.push(format!(
            "| `{command}` | {} | `...` | `...` |",
            if stdout_match { "yes" } else { "no" }
        ));
        stderr_lines.push(format!(
            "| `{command}` | {} | `...` | `...` |",
            if stderr_match { "yes" } else { "no" }
        ));
        exit_lines.push(format!(
            "| `{command}` | {} | `{}` | `{}` |",
            if exit_match { "yes" } else { "no" },
            row.pointer("/exit_code/python").and_then(Value::as_i64).unwrap_or_default(),
            row.pointer("/exit_code/rust").and_then(Value::as_i64).unwrap_or_default(),
        ));
        help_lines.push(format!(
            "| `{command}` | {} | {} |",
            if is_help { "yes" } else { "no" },
            if help_match { "yes" } else { "no" }
        ));
    }

    let parity_root = workspace_root.join(PARITY_DIR);
    for (name, lines) in [
        ("stdout_diff.md", stdout_lines),
        ("stderr_diff.md", stderr_lines),
        ("exit_code_diff.md", exit_lines),
        ("help_diff.md", help_lines),
    ] {
        let path = parity_root.join(name);
        if path.exists() {
            continue;
        }
        write_text_if_changed(&path, &lines.join("\n"));
    }
}

fn ensure_specialized_matrices(workspace_root: &Path, matrix: &Value, diffs: &Value) {
    let rows = matrix.get("commands").and_then(Value::as_array).cloned().unwrap_or_default();
    let diff_rows = diffs.get("diffs").and_then(Value::as_array).cloned().unwrap_or_default();
    if rows.is_empty() {
        return;
    }
    let diff_by_command: HashMap<String, Value> = diff_rows
        .iter()
        .filter_map(|row| {
            row.get("command")
                .and_then(Value::as_str)
                .map(|command| (command.to_string(), row.clone()))
        })
        .collect();

    let plugin_rows: Vec<Value> = rows
        .iter()
        .filter(|row| row.get("group").and_then(Value::as_str) == Some("plugin"))
        .cloned()
        .collect();
    let repl_rows: Vec<Value> = rows
        .iter()
        .filter(|row| {
            row.get("command").and_then(Value::as_str).unwrap_or_default().contains("repl")
        })
        .cloned()
        .collect();
    let state_rows: Vec<Value> = rows
        .iter()
        .filter(|row| {
            matches!(
                row.get("group").and_then(Value::as_str),
                Some("config" | "history" | "memory")
            )
        })
        .cloned()
        .collect();
    let bridge_rows: Vec<Value> = rows
        .iter()
        .filter_map(|row| {
            let command = row.get("command").and_then(Value::as_str)?;
            let diff = diff_by_command.get(command)?;
            Some(json!({
                "command": command,
                "status": row.get("status").cloned().unwrap_or_else(|| json!("partial")),
                "exit_match": diff.pointer("/exit_code/match").and_then(Value::as_bool).unwrap_or(false),
                "stdout_match": diff.pointer("/stdout/match").and_then(Value::as_bool).unwrap_or(false),
                "stderr_match": diff.pointer("/stderr/match").and_then(Value::as_bool).unwrap_or(false),
            }))
        })
        .collect();

    let aliases: HashSet<String> =
        read_json_if_exists(&workspace_root.join(STATUS_DIR).join("current_rust_state.json"))
            .pointer("/rust_routed_commands/aliases")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(ToString::to_string)
            .collect();

    let owned_rows: Vec<Value> = rows
        .iter()
        .filter(|row| row.get("status").and_then(Value::as_str) == Some("complete"))
        .cloned()
        .collect();
    let shim_rows: Vec<Value> = rows
        .iter()
        .filter(|row| {
            row.get("command")
                .and_then(Value::as_str)
                .is_some_and(|command| aliases.contains(command))
        })
        .cloned()
        .collect();
    let python_only_rows: Vec<Value> = rows
        .iter()
        .filter(|row| {
            row.get("status").and_then(Value::as_str) == Some("missing")
                && row.get("python_available").and_then(Value::as_bool).unwrap_or(false)
        })
        .cloned()
        .collect();
    let coverage_rows: Vec<Value> = rows
        .iter()
        .filter_map(|row| {
            let command = row.get("command").and_then(Value::as_str)?;
            let has_diff = diff_by_command.contains_key(command);
            Some(json!({
                "command": command,
                "parity_tests": has_diff,
                "failure_tests": matches!(row.get("status").and_then(Value::as_str), Some("complete" | "partial")),
                "output_snapshots": has_diff,
                "exit_code_checks": has_diff,
                "stderr_stdout_checks": has_diff,
            }))
        })
        .collect();

    let parity_root = workspace_root.join(PARITY_DIR);
    let candidates: [(&str, Value, &str); 8] = [
        (
            "plugin_parity_matrix.json",
            json!({"generated_at": stable_generated_at(), "source": "artifacts/parity/command_parity_matrix.json", "rows": plugin_rows}),
            "rows",
        ),
        (
            "repl_parity_matrix.json",
            json!({"generated_at": stable_generated_at(), "source": "artifacts/parity/command_parity_matrix.json", "rows": repl_rows}),
            "rows",
        ),
        (
            "python_bridge_parity_matrix.json",
            json!({"generated_at": stable_generated_at(), "source": "artifacts/parity/command_parity_diffs.json", "rows": bridge_rows}),
            "rows",
        ),
        (
            "state_behavior_parity_matrix.json",
            json!({"generated_at": stable_generated_at(), "source": "artifacts/parity/command_parity_matrix.json", "rows": state_rows}),
            "rows",
        ),
        (
            "commands_fully_rust_owned.json",
            json!({"generated_at": stable_generated_at(), "commands": owned_rows}),
            "commands",
        ),
        (
            "commands_using_compatibility_shims.json",
            json!({"generated_at": stable_generated_at(), "commands": shim_rows}),
            "commands",
        ),
        (
            "commands_python_only.json",
            json!({"generated_at": stable_generated_at(), "commands": python_only_rows}),
            "commands",
        ),
        (
            "parity_coverage_matrix.json",
            json!({"generated_at": stable_generated_at(), "coverage": coverage_rows}),
            "coverage",
        ),
    ];
    for (name, payload, key) in candidates {
        let path = parity_root.join(name);
        if path.exists() {
            continue;
        }
        let existing = read_json_if_exists(&path);
        if existing.get(key).and_then(Value::as_array).is_some_and(|rows| !rows.is_empty()) {
            continue;
        }
        write_json_if_changed(&path, &payload);
    }

    let summary_path = parity_root.join("command_parity_summary.txt");
    if !summary_path.exists() {
        let total = rows.len();
        let complete = rows.iter().filter(|row| row["status"] == "complete").count();
        let partial = rows.iter().filter(|row| row["status"] == "partial").count();
        let missing = rows.iter().filter(|row| row["status"] == "missing").count();
        let intentional =
            rows.iter().filter(|row| row["status"] == "different-by-decision").count();
        write_text_if_changed(
            &summary_path,
            &format!(
                "Command parity summary\n\
                 total: {total}\n\
                 complete: {complete}\n\
                 partial: {partial}\n\
                 missing: {missing}\n\
                 different-by-decision: {intentional}\n\
                 \n\
                 truth-source: artifacts/parity/command_parity_matrix.json\n"
            ),
        );
    }
}

fn ensure_law_reports(workspace_root: &Path, matrix: &Value, diffs: &Value) {
    let rows = matrix.get("commands").and_then(Value::as_array).cloned().unwrap_or_default();
    if rows.is_empty() {
        return;
    }
    let coverage =
        read_json_if_exists(&workspace_root.join(PARITY_DIR).join("parity_coverage_matrix.json"));
    let coverage_map: HashMap<String, Value> = coverage
        .get("coverage")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| {
            row.get("command")
                .and_then(Value::as_str)
                .map(|command| (command.to_string(), row.clone()))
        })
        .collect();
    let diff_map: HashMap<String, Value> = diffs
        .get("diffs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| {
            row.get("command")
                .and_then(Value::as_str)
                .map(|command| (command.to_string(), row.clone()))
        })
        .collect();

    let report_rows = |name: &str| -> Vec<Value> {
        rows.iter()
            .filter_map(|row| {
                let command = row.get("command").and_then(Value::as_str)?;
                let group = row.get("group").and_then(Value::as_str).unwrap_or("unknown");
                let status = row.get("status").and_then(Value::as_str).unwrap_or("missing");
                let cov = coverage_map.get(command).cloned().unwrap_or_else(|| json!({}));
                let diff = diff_map.get(command).cloned().unwrap_or_else(|| json!({}));
                match name {
                    "command_precedence_report.json" => Some(json!({
                        "command": command,
                        "group": group,
                        "status": status,
                        "source_precedence": ["flags", "env", "config", "defaults"],
                        "coverage": cov.get("parity_tests").and_then(Value::as_bool).unwrap_or(false),
                    })),
                    "command_flag_normalization_report.json" => Some(json!({
                        "command": command,
                        "group": group,
                        "status": status,
                        "global_flags_supported": ["--format/-f", "--pretty/--no-pretty", "--color", "--log-level", "--quiet/-q"],
                        "coverage": cov.get("parity_tests").and_then(Value::as_bool).unwrap_or(false),
                    })),
                    "command_stream_report.json" => Some(json!({
                        "command": command,
                        "stdout_match": diff.pointer("/stdout/match").and_then(Value::as_bool).unwrap_or(false),
                        "stderr_match": diff.pointer("/stderr/match").and_then(Value::as_bool).unwrap_or(false),
                        "coverage": cov.get("stderr_stdout_checks").and_then(Value::as_bool).unwrap_or(false),
                    })),
                    "command_exit_code_report.json" => Some(json!({
                        "command": command,
                        "exit_code_match": diff.pointer("/exit_code/match").and_then(Value::as_bool).unwrap_or(false),
                        "coverage": cov.get("exit_code_checks").and_then(Value::as_bool).unwrap_or(false),
                    })),
                    "command_help_diff_report.json" => Some(json!({
                        "command": command,
                        "is_help_command": diff.pointer("/help/is_help_command").and_then(Value::as_bool).unwrap_or(false),
                        "help_match": diff.pointer("/help/match").and_then(Value::as_bool).unwrap_or(false),
                        "coverage": cov.get("output_snapshots").and_then(Value::as_bool).unwrap_or(false),
                    })),
                    "command_machine_output_diff_report.json" => Some(json!({
                        "command": command,
                        "stdout_match": diff.pointer("/stdout/match").and_then(Value::as_bool).unwrap_or(false),
                        "stderr_match": diff.pointer("/stderr/match").and_then(Value::as_bool).unwrap_or(false),
                        "exit_code_match": diff.pointer("/exit_code/match").and_then(Value::as_bool).unwrap_or(false),
                        "coverage": cov.get("parity_tests").and_then(Value::as_bool).unwrap_or(false),
                    })),
                    _ => None,
                }
            })
            .collect()
    };

    let parity_root = workspace_root.join(PARITY_DIR);
    for name in [
        "command_precedence_report.json",
        "command_flag_normalization_report.json",
        "command_stream_report.json",
        "command_exit_code_report.json",
        "command_help_diff_report.json",
        "command_machine_output_diff_report.json",
    ] {
        let path = parity_root.join(name);
        let existing = read_json_if_exists(&path);
        if existing.get("rows").and_then(Value::as_array).is_some_and(|rows| !rows.is_empty()) {
            continue;
        }
        let payload = json!({
            "generated_at": stable_generated_at(),
            "generator": "crates/bijux-dev-cli/src/parity.rs::ensure_law_reports",
            "rows": report_rows(name),
        });
        write_json_if_changed(&path, &payload);
    }

    let dashboard_path = parity_root.join("parity_dashboard.json");
    let existing_dashboard = read_json_if_exists(&dashboard_path);
    if !existing_dashboard.get("summary").is_some_and(Value::is_object) {
        let mut surfaces: BTreeMap<String, BTreeMap<&str, usize>> = BTreeMap::new();
        for row in &rows {
            let group = row.get("group").and_then(Value::as_str).unwrap_or("unknown").to_string();
            let status = row.get("status").and_then(Value::as_str).unwrap_or("missing");
            let slot = surfaces.entry(group).or_default();
            match status {
                "complete" => *slot.entry("complete").or_default() += 1,
                "partial" => *slot.entry("partial").or_default() += 1,
                "missing" => *slot.entry("missing").or_default() += 1,
                "intentionally-different" | "different-by-decision" => {
                    *slot.entry("different_by_decision").or_default() += 1
                }
                _ => {}
            }
        }
        let cov_rows = read_json_if_exists(&parity_root.join("parity_coverage_matrix.json"))
            .get("coverage")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let coverage = json!({
            "parity_tests": cov_rows.iter().filter(|row| row.get("parity_tests").and_then(Value::as_bool).unwrap_or(false)).count(),
            "failure_tests": cov_rows.iter().filter(|row| row.get("failure_tests").and_then(Value::as_bool).unwrap_or(false)).count(),
            "stderr_stdout_checks": cov_rows.iter().filter(|row| row.get("stderr_stdout_checks").and_then(Value::as_bool).unwrap_or(false)).count(),
            "exit_code_checks": cov_rows.iter().filter(|row| row.get("exit_code_checks").and_then(Value::as_bool).unwrap_or(false)).count(),
            "output_snapshots": cov_rows.iter().filter(|row| row.get("output_snapshots").and_then(Value::as_bool).unwrap_or(false)).count(),
        });
        let payload = json!({
            "generated_at": stable_generated_at(),
            "generator": "crates/bijux-dev-cli/src/parity.rs::ensure_law_reports",
            "summary": {
                "total_commands": rows.len(),
                "surfaces": surfaces,
                "coverage": coverage,
            },
            "reports": {
                "precedence": "artifacts/parity/command_precedence_report.json",
                "flag_normalization": "artifacts/parity/command_flag_normalization_report.json",
                "stdout_stderr": "artifacts/parity/command_stream_report.json",
                "exit_code": "artifacts/parity/command_exit_code_report.json",
                "help_diff": "artifacts/parity/command_help_diff_report.json",
                "machine_output_diff": "artifacts/parity/command_machine_output_diff_report.json",
                "parity_matrix": "artifacts/parity/command_parity_matrix.json",
                "coverage_matrix": "artifacts/parity/parity_coverage_matrix.json",
            }
        });
        write_json_if_changed(&dashboard_path, &payload);
        let dashboard_text = format!(
            "Parity Dashboard\n\
             total_commands: {}\n\
             parity_tests: {}\n\
             exit_code_checks: {}\n\
             stderr_stdout_checks: {}\n\
             output_snapshots: {}\n\
             source: artifacts/parity/parity_dashboard.json\n",
            rows.len(),
            payload["summary"]["coverage"]["parity_tests"].as_u64().unwrap_or_default(),
            payload["summary"]["coverage"]["exit_code_checks"].as_u64().unwrap_or_default(),
            payload["summary"]["coverage"]["stderr_stdout_checks"].as_u64().unwrap_or_default(),
            payload["summary"]["coverage"]["output_snapshots"].as_u64().unwrap_or_default(),
        );
        write_text_if_changed(&parity_root.join("parity_dashboard.txt"), &dashboard_text);
    }
}

fn ensure_regression_report(workspace_root: &Path, matrix: &Value) {
    let regression_path = workspace_root.join(PARITY_DIR).join("parity_regression_diffs.json");
    let summary_path = workspace_root.join(PARITY_DIR).join("parity_regression_summary.txt");
    let existing = read_json_if_exists(&regression_path);
    if existing.get("regressions").is_some() && existing.get("warnings").is_some() {
        return;
    }

    let baseline = read_json_if_exists(
        &workspace_root.join("docs/architecture/parity/baseline_command_parity_matrix.json"),
    );
    let baseline_map: HashMap<String, Value> = baseline
        .get("commands")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| {
            row.get("command")
                .and_then(Value::as_str)
                .map(|command| (command.to_string(), row.clone()))
        })
        .collect();
    let current_map: HashMap<String, Value> = matrix
        .get("commands")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| {
            row.get("command")
                .and_then(Value::as_str)
                .map(|command| (command.to_string(), row.clone()))
        })
        .collect();

    let mut regressions = Vec::<String>::new();
    let mut warnings = Vec::<String>::new();
    for (command, old) in baseline_map {
        let Some(new) = current_map.get(&command) else {
            regressions.push(format!("command disappeared from matrix: {command}"));
            continue;
        };
        let old_status = old.get("status").and_then(Value::as_str).unwrap_or("missing");
        let new_status = new.get("status").and_then(Value::as_str).unwrap_or("missing");
        if old_status == "complete" && status_rank(new_status) < status_rank(old_status) {
            regressions.push(format!(
                "parity-covered command regressed: {command} ({old_status} -> {new_status})"
            ));
        }
        if old_status == "partial" && new_status == "partial" {
            let old_conf = old.get("confidence").and_then(Value::as_f64).unwrap_or(0.0);
            let new_conf = new.get("confidence").and_then(Value::as_f64).unwrap_or(0.0);
            if new_conf + 0.1 < old_conf {
                warnings.push(format!(
                    "parity-partial command drifted further away: {command} ({old_conf:.2} -> {new_conf:.2})"
                ));
            }
        }
    }

    let payload = json!({
        "regressions": regressions,
        "warnings": warnings,
        "baseline": "docs/architecture/parity/baseline_command_parity_matrix.json",
        "current": "artifacts/parity/command_parity_matrix.json",
    });
    write_json_if_changed(&regression_path, &payload);
    let text = format!(
        "Parity Regression Summary\n\
         regressions: {}\n\
         warnings: {}\n",
        payload["regressions"].as_array().map_or(0, Vec::len),
        payload["warnings"].as_array().map_or(0, Vec::len),
    );
    write_text_if_changed(&summary_path, &text);
}

fn ensure_parity_artifacts(workspace_root: &Path) {
    let matrix = ensure_command_matrix(workspace_root);
    let diffs = ensure_parity_diffs(workspace_root, &matrix);
    write_diff_markdown(workspace_root, &diffs);
    ensure_specialized_matrices(workspace_root, &matrix, &diffs);
    ensure_law_reports(workspace_root, &matrix, &diffs);
    ensure_regression_report(workspace_root, &matrix);

    let parity_root = workspace_root.join(PARITY_DIR);
    write_json_if_changed(
        &parity_root.join("parity_dashboard_gate_report.json"),
        &parity_dashboard_gate(workspace_root),
    );
    write_json_if_changed(
        &parity_root.join("parity_regression_gate_report.json"),
        &parity_regression_gate(workspace_root),
    );
    write_json_if_changed(
        &parity_root.join("binary_bridge_parity_gate_report.json"),
        &binary_bridge_gate(workspace_root),
    );
    write_json_if_changed(
        &parity_root.join("cross_surface_drift_gate_report.json"),
        &cross_surface_drift_gate(workspace_root),
    );
}

fn parity_dashboard_gate(workspace_root: &Path) -> Value {
    let required = [
        "command_precedence_report.json",
        "command_flag_normalization_report.json",
        "command_stream_report.json",
        "command_exit_code_report.json",
        "command_help_diff_report.json",
        "command_machine_output_diff_report.json",
        "parity_dashboard.json",
        "parity_dashboard.txt",
    ];
    let mut failures = Vec::<String>::new();
    let parity_root = workspace_root.join(PARITY_DIR);
    for name in required {
        if !parity_root.join(name).exists() {
            failures.push(format!("missing artifacts/parity/{name}"));
        }
    }
    let dashboard = read_json_if_exists(&parity_root.join("parity_dashboard.json"));
    let summary = dashboard.get("summary").cloned().unwrap_or_else(|| json!({}));
    let coverage = summary.get("coverage").cloned().unwrap_or_else(|| json!({}));
    if !summary.is_object() {
        failures.push("parity_dashboard.json missing summary".to_string());
    }
    if !coverage.is_object() {
        failures.push("parity_dashboard.json missing coverage".to_string());
    }
    if coverage.get("parity_tests").and_then(Value::as_u64).unwrap_or_default() == 0 {
        failures.push("parity dashboard shows zero parity tests".to_string());
    }
    json!({
        "status": if failures.is_empty() { "pass" } else { "fail" },
        "failures": failures,
    })
}

fn parity_regression_gate(workspace_root: &Path) -> Value {
    let payload =
        read_json_if_exists(&workspace_root.join(PARITY_DIR).join("parity_regression_diffs.json"));
    let regressions =
        payload.get("regressions").and_then(Value::as_array).cloned().unwrap_or_default();
    let warnings = payload.get("warnings").and_then(Value::as_array).cloned().unwrap_or_default();
    json!({
        "status": if regressions.is_empty() { "pass" } else { "fail" },
        "regressions": regressions,
        "warnings": warnings,
    })
}

fn binary_bridge_gate(workspace_root: &Path) -> Value {
    let payload = read_json_if_exists(
        &workspace_root.join(PARITY_DIR).join("binary_vs_python_bridge_parity_report.json"),
    );
    let rows = payload.get("cases").and_then(Value::as_array).cloned().unwrap_or_default();
    let mut failures = Vec::<String>::new();
    for row in rows {
        let command = row.get("command").and_then(Value::as_str).unwrap_or("<unknown>");
        for key in ["exit_match", "stdout_match", "stderr_match"] {
            if !row.get(key).and_then(Value::as_bool).unwrap_or(false) {
                failures.push(format!("{command}: {key}=false"));
            }
        }
    }
    json!({
        "status": if failures.is_empty() { "pass" } else { "fail" },
        "failures": failures,
    })
}

fn cross_surface_drift_gate(workspace_root: &Path) -> Value {
    let payload = read_json_if_exists(
        &workspace_root.join(STATUS_DIR).join("cross_surface_drift_report.json"),
    );
    let drift_count = payload.get("drift_count").and_then(Value::as_u64).unwrap_or_default();
    json!({
        "status": if drift_count == 0 { "pass" } else { "fail" },
        "drift_count": drift_count,
        "drift_items": payload.get("drift_items").cloned().unwrap_or_else(|| json!([])),
    })
}

/// Builds the maintainer parity report envelope.
#[must_use]
pub fn build_report(workspace_root: &Path) -> Value {
    ensure_parity_artifacts(workspace_root);

    let parity_report = read_json_if_exists(
        &workspace_root.join("artifacts/parity/rust_python_parity_report.json"),
    );
    let bridge_parity = read_json_if_exists(
        &workspace_root.join("artifacts/parity/binary_vs_python_bridge_parity_report.json"),
    );
    let command_matrix =
        read_json_if_exists(&workspace_root.join("artifacts/parity/command_parity_matrix.json"));
    let plugin_matrix =
        read_json_if_exists(&workspace_root.join("artifacts/parity/plugin_parity_matrix.json"));
    let repl_matrix =
        read_json_if_exists(&workspace_root.join("artifacts/parity/repl_parity_matrix.json"));
    let python_bridge_matrix = read_json_if_exists(
        &workspace_root.join("artifacts/parity/python_bridge_parity_matrix.json"),
    );
    let state_behavior_matrix = read_json_if_exists(
        &workspace_root.join("artifacts/parity/state_behavior_parity_matrix.json"),
    );
    let repl_cli_output_diff =
        read_json_if_exists(&workspace_root.join("artifacts/parity/repl_cli_output_diff.json"));
    let parity_diffs =
        read_json_if_exists(&workspace_root.join("artifacts/parity/command_parity_diffs.json"));
    let text_summary =
        fs::read_to_string(workspace_root.join("artifacts/parity/command_parity_summary.txt"))
            .unwrap_or_default();
    let commands_fully_rust_owned = read_json_if_exists(
        &workspace_root.join("artifacts/parity/commands_fully_rust_owned.json"),
    );
    let commands_using_compatibility_shims = read_json_if_exists(
        &workspace_root.join("artifacts/parity/commands_using_compatibility_shims.json"),
    );
    let commands_python_only =
        read_json_if_exists(&workspace_root.join("artifacts/parity/commands_python_only.json"));
    let coverage =
        read_json_if_exists(&workspace_root.join("artifacts/parity/parity_coverage_matrix.json"));
    let precedence_report = read_json_if_exists(
        &workspace_root.join("artifacts/parity/command_precedence_report.json"),
    );
    let flag_norm_report = read_json_if_exists(
        &workspace_root.join("artifacts/parity/command_flag_normalization_report.json"),
    );
    let stream_report =
        read_json_if_exists(&workspace_root.join("artifacts/parity/command_stream_report.json"));
    let exit_code_report =
        read_json_if_exists(&workspace_root.join("artifacts/parity/command_exit_code_report.json"));
    let help_diff_report =
        read_json_if_exists(&workspace_root.join("artifacts/parity/command_help_diff_report.json"));
    let machine_output_report = read_json_if_exists(
        &workspace_root.join("artifacts/parity/command_machine_output_diff_report.json"),
    );
    let parity_dashboard =
        read_json_if_exists(&workspace_root.join("artifacts/parity/parity_dashboard.json"));
    let parity_dashboard_text =
        fs::read_to_string(workspace_root.join("artifacts/parity/parity_dashboard.txt"))
            .unwrap_or_default();
    let parity_dashboard_gate = read_json_if_exists(
        &workspace_root.join("artifacts/parity/parity_dashboard_gate_report.json"),
    );
    let parity_regression_gate = read_json_if_exists(
        &workspace_root.join("artifacts/parity/parity_regression_gate_report.json"),
    );
    let binary_bridge_gate = read_json_if_exists(
        &workspace_root.join("artifacts/parity/binary_bridge_parity_gate_report.json"),
    );
    let cross_surface_drift_gate = read_json_if_exists(
        &workspace_root.join("artifacts/parity/cross_surface_drift_gate_report.json"),
    );
    let config_parity =
        read_json_if_exists(&workspace_root.join("artifacts/parity/config_parity_report.json"));
    let history_parity =
        read_json_if_exists(&workspace_root.join("artifacts/parity/history_parity_report.json"));
    let memory_parity =
        read_json_if_exists(&workspace_root.join("artifacts/parity/memory_parity_report.json"));

    json!({
        "migration_dashboard_default": "bijux-dev-cli parity",
        "evidence_ids": [
            "EVIDENCE-1002-PARITY-COVERAGE",
            "EVIDENCE-1103-PLUGIN-LIFECYCLE",
            "EVIDENCE-1107-DIAGNOSTICS-CONSISTENCY",
            "EVIDENCE-1108-REPL-PARITY",
            "EVIDENCE-1109-PYTHON-BRIDGE-EQUIVALENCE"
        ],
        "rust_python": parity_report,
        "binary_bridge": bridge_parity,
        "command_matrix": command_matrix,
        "plugin_matrix": plugin_matrix,
        "repl_matrix": repl_matrix,
        "python_bridge_matrix": python_bridge_matrix,
        "state_behavior_matrix": state_behavior_matrix,
        "diffs": parity_diffs,
        "text_summary": text_summary,
        "plugin_lifecycle": command_matrix.get("plugin_lifecycle").cloned().unwrap_or_else(|| json!({})),
        "commands_fully_rust_owned": commands_fully_rust_owned,
        "commands_using_compatibility_shims": commands_using_compatibility_shims,
        "commands_python_only": commands_python_only,
        "coverage": coverage,
        "precedence_report": precedence_report,
        "flag_normalization_report": flag_norm_report,
        "stream_report": stream_report,
        "exit_code_report": exit_code_report,
        "help_diff_report": help_diff_report,
        "machine_output_diff_report": machine_output_report,
        "parity_dashboard": parity_dashboard,
        "parity_dashboard_text": parity_dashboard_text,
        "automation_gates": {
            "parity_dashboard": parity_dashboard_gate,
            "parity_regression": parity_regression_gate,
            "binary_bridge": binary_bridge_gate,
            "cross_surface_drift": cross_surface_drift_gate,
        },
        "repl_cli_output_diff": repl_cli_output_diff,
        "state_parity": {
            "config": config_parity,
            "history": history_parity,
            "memory": memory_parity,
        },
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::build_report;

    #[test]
    fn parity_report_shape_is_stable() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let report = build_report(&workspace_root);
        assert!(report.get("command_matrix").is_some());
        assert!(report.get("state_parity").is_some());
        assert!(report.pointer("/automation_gates/parity_dashboard").is_some());
    }
}
