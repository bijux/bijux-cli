#![forbid(unsafe_code)]
//! Contracts and snapshot heads for dev-cli audit command outputs.

use std::process::Command;

use serde_json::Value;

fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .output()
        .expect("binary should run");
    (
        out.status.code().unwrap_or(1),
        String::from_utf8(out.stdout).expect("stdout utf-8"),
        String::from_utf8(out.stderr).expect("stderr utf-8"),
    )
}

fn head(text: &str, lines: usize) -> String {
    text.lines().take(lines).collect::<Vec<_>>().join("\n") + "\n"
}

#[test]
fn script_docs_crate_health_json_contracts_are_stable() {
    let cases = [
        (
            vec![
                "dev",
                "cli",
                "script-audit",
                "--format",
                "json",
                "--no-pretty",
            ],
            "scripts",
        ),
        (
            vec![
                "dev",
                "cli",
                "docs-audit",
                "--format",
                "json",
                "--no-pretty",
            ],
            "docs",
        ),
        (
            vec![
                "dev",
                "cli",
                "crate-health",
                "--format",
                "json",
                "--no-pretty",
            ],
            "crate_metrics",
        ),
    ];

    for (args, key) in cases {
        let (code, stdout, stderr) = run(&args);
        assert_eq!(code, 0, "command failed: {args:?}");
        assert!(
            stderr.is_empty(),
            "stderr must be empty for success: {args:?}"
        );

        let payload: Value = serde_json::from_str(&stdout).expect("json parse");
        assert!(payload.get(key).is_some(), "missing key {key} for {args:?}");

        let (code2, stdout2, stderr2) = run(&args);
        assert_eq!(code2, 0);
        assert_eq!(stdout, stdout2, "json output drift for {args:?}");
        assert_eq!(stderr, stderr2, "stderr output drift for {args:?}");
    }
}

#[test]
fn script_docs_crate_health_text_snapshot_heads_match() {
    let cases = [
        (
            vec!["dev", "cli", "script-audit", "--format", "text"],
            include_str!("../../../data/golden/cli_surface/dev_cli_script_audit_text_head.txt"),
        ),
        (
            vec!["dev", "cli", "docs-audit", "--format", "text"],
            include_str!("../../../data/golden/cli_surface/dev_cli_docs_audit_text_head.txt"),
        ),
        (
            vec!["dev", "cli", "crate-health", "--format", "text"],
            include_str!("../../../data/golden/cli_surface/dev_cli_crate_health_text_head.txt"),
        ),
    ];

    for (args, expected_head) in cases {
        let (code, stdout, stderr) = run(&args);
        assert_eq!(code, 0, "command failed: {args:?}");
        assert!(
            stderr.is_empty(),
            "stderr must be empty for success: {args:?}"
        );
        assert_eq!(
            head(&stdout, 24),
            expected_head,
            "text snapshot head drift for {args:?}"
        );
    }
}
