#![forbid(unsafe_code)]
//! Contracts for dev-cli evidence commands and integrity surfaces.

use std::process::Command;

use serde_json::Value;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("binary should execute")
}

fn run_ok_json(command: &[&str]) -> Value {
    let mut args = command.to_vec();
    args.push("--format");
    args.push("json");
    args.push("--no-pretty");
    let out = run(&args);
    assert!(
        out.status.success(),
        "command failed: {:?}\nstdout={}\nstderr={}",
        args,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("json payload")
}

fn run_ok_text_non_empty(command: &[&str]) {
    let mut args = command.to_vec();
    args.push("--format");
    args.push("text");
    let out = run(&args);
    assert!(
        out.status.success(),
        "command failed: {:?}\nstdout={}\nstderr={}",
        args,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8(out.stdout).expect("utf8");
    assert!(!text.trim().is_empty(), "text output must be non-empty");
}

#[test]
fn evidence_commands_json_contracts_are_available() {
    let list = run_ok_json(&["dev", "cli", "evidence", "list"]);
    let first_id = list["records"]
        .as_array()
        .and_then(|rows| rows.first())
        .and_then(|row| row.get("id"))
        .and_then(Value::as_str)
        .unwrap_or("EVIDENCE-1001-RELEASE-TRUTH")
        .to_string();
    let show = run_ok_json(&["dev", "cli", "evidence", "show", "--id", &first_id]);
    let audit = run_ok_json(&["dev", "cli", "evidence", "audit"]);
    let stale = run_ok_json(&["dev", "cli", "evidence", "stale"]);
    let matrix = run_ok_json(&["dev", "cli", "evidence", "matrix"]);
    let website_export = run_ok_json(&["dev", "cli", "evidence", "website-export"]);
    let ci_export = run_ok_json(&["dev", "cli", "evidence", "ci-export"]);
    let release_export = run_ok_json(&["dev", "cli", "evidence", "release-export"]);

    assert!(list.get("records").is_some());
    assert!(show.get("record").is_some());
    assert!(audit.get("status").is_some());
    assert!(stale.get("stale").is_some());
    assert!(matrix.get("status_matrix").is_some());
    assert!(website_export.get("website_export").is_some());
    assert!(ci_export.get("ci_export").is_some());
    assert!(release_export.get("release_export").is_some());
}

#[test]
fn evidence_commands_text_contracts_are_available() {
    run_ok_text_non_empty(&["dev", "cli", "evidence", "list"]);
    run_ok_text_non_empty(&["dev", "cli", "evidence", "show", "--id", "EVIDENCE-1001-RELEASE-TRUTH"]);
    run_ok_text_non_empty(&["dev", "cli", "evidence", "audit"]);
    run_ok_text_non_empty(&["dev", "cli", "evidence", "stale"]);
    run_ok_text_non_empty(&["dev", "cli", "evidence", "matrix"]);
    run_ok_text_non_empty(&["dev", "cli", "evidence", "website-export"]);
    run_ok_text_non_empty(&["dev", "cli", "evidence", "ci-export"]);
    run_ok_text_non_empty(&["dev", "cli", "evidence", "release-export"]);
}

#[test]
fn evidence_records_have_valid_ids_status_source_and_links() {
    let list = run_ok_json(&["dev", "cli", "evidence", "list"]);
    let records = list["records"].as_array().expect("records");
    let allowed_statuses = std::collections::BTreeSet::from(["proven", "partial", "stale", "blocked"]);
    for row in records {
        let id = row["id"].as_str().unwrap_or_default();
        let status = row["status"].as_str().unwrap_or_default();
        let source = row["source"].as_str().unwrap_or_default();
        let id_parts: Vec<&str> = id.split('-').collect();
        let id_has_valid_prefix = id.starts_with("EVIDENCE-");
        let id_has_numeric_segment =
            id_parts.get(1).is_some_and(|part| part.len() >= 4 && part.chars().all(|ch| ch.is_ascii_digit()));
        let id_has_suffix = id_parts.len() >= 3 && id_parts[2..].iter().all(|part| !part.is_empty());
        assert!(
            id_has_valid_prefix && id_has_numeric_segment && id_has_suffix,
            "invalid evidence id format: {id}"
        );
        assert!(allowed_statuses.contains(status), "invalid evidence status: {status}");
        assert!(!source.trim().is_empty(), "evidence source must be non-empty for id {id}");
        assert!(
            row["artifact_links"].as_array().is_some_and(|links| !links.is_empty()),
            "evidence record must include artifact links for id {id}"
        );
    }
}

#[test]
fn evidence_audit_surfaces_stale_missing_and_orphan_claims() {
    let audit = run_ok_json(&["dev", "cli", "evidence", "audit"]);
    assert!(audit.get("invalid_ids").is_some(), "audit must expose invalid id list");
    assert!(
        audit.get("missing_artifact_links").is_some(),
        "audit must expose missing artifact links"
    );
    assert!(audit.get("orphan_report").is_some(), "audit must expose orphan evidence report");
    assert!(
        audit.get("claims_without_evidence_report").is_some(),
        "audit must expose claims-without-evidence report"
    );
}

#[test]
fn evidence_stale_report_is_honest_and_parseable() {
    let stale = run_ok_json(&["dev", "cli", "evidence", "stale"]);
    let count = stale["count"].as_u64().unwrap_or_default();
    let rows = stale["stale"].as_array().cloned().unwrap_or_default();
    assert_eq!(count as usize, rows.len(), "stale evidence count must match row count");
}

#[test]
fn evidence_exports_reference_known_evidence_ids() {
    let list = run_ok_json(&["dev", "cli", "evidence", "list"]);
    let known_ids: std::collections::BTreeSet<String> = list["records"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("id").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect();
    let exports = [
        run_ok_json(&["dev", "cli", "evidence", "website-export"]),
        run_ok_json(&["dev", "cli", "evidence", "ci-export"]),
        run_ok_json(&["dev", "cli", "evidence", "release-export"]),
    ];
    for payload in exports {
        let ids: Vec<String> = payload
            .to_string()
            .split('"')
            .filter(|token| token.starts_with("EVIDENCE-"))
            .map(ToString::to_string)
            .collect();
        for id in ids {
            assert!(known_ids.contains(&id), "export references unknown evidence id: {id}");
        }
    }
}
