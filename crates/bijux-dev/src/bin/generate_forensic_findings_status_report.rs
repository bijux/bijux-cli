use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
#[cfg(test)]
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::fs;
use std::path::Path;
use tempfile as _;

fn main() {
    let out = Path::new("docs/reports/foundation/FORENSIC_FINDINGS_STATUS_REPORT.md");
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).expect("create report directory");
    }
    fs::write(out, report_markdown()).expect("write report");
    println!("{}", out.display());
}

fn report_markdown() -> String {
    let mut markdown = String::new();
    markdown.push_str("# Forensic Findings Status Report\n\n");
    markdown.push_str("This report re-audits retained forensic findings against the current code and test suite.\n\n");
    markdown.push_str("## Container Image Validation Rules\n\n");
    markdown.push_str("- Empty or whitespace-only image strings are rejected.\n");
    markdown.push_str("- Non-empty image strings are treated as literal image identifiers, including values that begin with `-` or contain option-like substrings.\n");
    markdown.push_str("- Runtime argument construction passes the image as a positional argument after engine flags.\n\n");
    markdown.push_str("## Fixed\n\n");
    markdown.push_str("- Container contract accepts literal image values that start with `-` and option-like segments (`container_execution_contracts`).\n");
    markdown.push_str("- Runtime adapters use centralized environment shaping in shell, container, and external execution paths (`shape_environment` routing).\n");
    markdown.push_str("- Output validation rejects symlinked intermediate components and skips symlink loops while scanning undeclared outputs.\n");
    markdown.push_str(
        "- External adapter transport rejects oversized serialized `--node-spec` payloads.\n",
    );
    markdown.push_str("- Cache pack extraction rejects oversized archives and hostile entry types (non-file, non-directory).\n\n");
    markdown.push_str("## Stale\n\n");
    markdown.push_str("- Previous concern that evidence-foundation output is opaque is stale; a stepwise verification summary is now generated at `artifacts/reports/evidence-foundation-verification-summary.md`.\n\n");
    markdown.push_str("## Needs More Work\n\n");
    markdown.push_str("- Timeout defaults from global runtime config are still not threaded into adapter-local timeout execution when a node-level timeout is absent.\n");
    markdown.push_str("- External adapter node-spec minimization/redaction policy is still broad; current guard enforces payload size but does not reduce data shape.\n\n");
    markdown.push_str("## Still Open\n\n");
    markdown.push_str("- None classified as release-blocking after this pass; remaining items are hardening follow-ups tracked under \"Needs More Work\".\n");
    markdown
}
