use bijux_dag_app as _;
use clap as _;
use clap_complete as _;
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

fn dag_command() -> Command {
    if let Some(path) = option_env!("CARGO_BIN_EXE_bijux") {
        if std::path::Path::new(path).exists() {
            return Command::new(path);
        }
    }

    let mut command = Command::new("cargo");
    command.env("CARGO_TARGET_DIR", "artifacts/target");
    command.args([
        "run",
        "--quiet",
        "-p",
        "bijux-dag-cli",
        "--bin",
        "bijux",
        "--",
    ]);
    command
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn write_temp_dag() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "bijux-dag-cli-taxonomy-{}.json",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    let content = r#"{
  "spec": "bijux-dag/v0.1",
  "nodes": [
    {
      "id": "const1",
      "kind": "const",
      "inputs": [],
      "outputs": [{"name": "value", "path": "value.txt"}],
      "params": {"value": "hello"}
    }
  ],
  "edges": []
}
"#;
    std::fs::write(&path, content).expect("write dag");
    path
}

fn newest_run_dir(base: &std::path::Path) -> PathBuf {
    let mut entries: Vec<_> = std::fs::read_dir(base)
        .expect("read run output directory")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .collect();
    entries.sort_by_key(|entry| entry.file_name());
    entries.last().expect("at least one run directory").path()
}

#[test]
fn dag_help_stays_aligned_with_taxonomy_doc() {
    let taxonomy = std::fs::read_to_string(repo_root().join("docs/CLI_COMMAND_TAXONOMY.md"))
        .expect("read taxonomy");
    let help = dag_command()
        .args(["dag", "--help"])
        .output()
        .expect("dag help output");
    assert!(help.status.success(), "dag --help should succeed");
    let help_text = String::from_utf8_lossy(&help.stdout);

    for token in [
        "validate",
        "run",
        "replay",
        "diff",
        "fsck",
        "capabilities",
        "export",
        "import",
        "version",
    ] {
        assert!(taxonomy.contains(token), "taxonomy must include {token}");
        assert!(help_text.contains(token), "dag help must include {token}");
    }
}

#[test]
fn exit_code_table_stays_backed_by_error_code_policy() {
    let cli_doc = std::fs::read_to_string(repo_root().join("docs/CLI.md")).expect("read CLI doc");
    assert!(
        cli_doc.contains("## Exit code matrix"),
        "CLI doc must publish exit code matrix"
    );

    let policy: Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root().join("configs/policy/error_codes.json"))
            .expect("read error code policy"),
    )
    .expect("parse error code policy");
    let categories: Vec<String> = policy["categories"]
        .as_array()
        .expect("categories")
        .iter()
        .filter_map(|v| v.as_str().map(ToOwned::to_owned))
        .collect();
    for category in ["parse", "validation", "execution", "io"] {
        assert!(
            categories.iter().any(|it| it == category),
            "error code categories must include {category}"
        );
    }

    let missing = dag_command()
        .args(["dag", "validate", "/definitely/missing/file.json"])
        .output()
        .expect("validate missing");
    assert_eq!(
        missing.status.code(),
        Some(3),
        "CLI failure code must match documented matrix for dag validate"
    );
}

#[test]
fn alias_and_deprecation_surfaces_remain_callable() {
    let dag = write_temp_dag();
    let run_out = tempfile::tempdir().expect("run output dir");
    let run = dag_command()
        .args([
            "dag",
            "run",
            dag.to_str().expect("dag path"),
            "--out",
            run_out.path().to_str().expect("run out path"),
        ])
        .output()
        .expect("run output");
    assert!(run.status.success(), "run should succeed");
    let run_dir = newest_run_dir(run_out.path());
    let run_id = run_dir
        .file_name()
        .and_then(|name| name.to_str())
        .expect("run id");

    let verify_alias = dag_command()
        .args([
            "dag",
            "fsck",
            run_dir.to_str().unwrap(),
            "--strict",
            "--json",
        ])
        .output()
        .expect("fsck alias");
    assert!(
        verify_alias.status.success(),
        "fsck alias must stay supported"
    );

    let legacy_status = dag_command()
        .args(["dag", "status", run_dir.to_str().unwrap(), "--json"])
        .output()
        .expect("legacy status");
    assert!(
        legacy_status.status.success(),
        "status alias must remain callable"
    );
    let legacy_payload: Value =
        serde_json::from_slice(&legacy_status.stdout).expect("status payload");
    assert_eq!(legacy_payload["command"], "dag.status");

    let canonical_show = dag_command()
        .args([
            "dag",
            "runs",
            "show",
            "--root",
            run_out.path().to_str().unwrap(),
            run_id,
            "--json",
        ])
        .output()
        .expect("runs show");
    assert!(canonical_show.status.success(), "runs show must succeed");
}
