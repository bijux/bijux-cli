use std::io::Write;
use std::process::Command;
use tempfile::{NamedTempFile, tempdir};

fn dag_binary() -> String {
    env!("CARGO_BIN_EXE_bijux").to_string()
}

fn write_temp_dag() -> String {
    let mut file = NamedTempFile::new().expect("temp dag");
    let content = r#"{
  "spec": "bijux-dag/v0.1",
  "nodes": [
    {
      "id": "const1",
      "kind": "const",
      "inputs": [],
      "outputs": [
        {
          "name": "value",
          "path": "value.txt"
        }
      ],
      "params": {
        "value": "hello"
      }
    }
  ],
  "edges": []
}
"#;
    file.write_all(content.as_bytes()).unwrap();
    file.into_temp_path().to_str().unwrap().to_string()
}

#[test]
fn dag_validate_help_is_stable_enough() {
    let output = Command::new(dag_binary())
        .args(["dag", "validate", "--help"])
        .output()
        .expect("validate help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("Usage:"));
    assert!(text.contains("dag validate [OPTIONS] <DAG>"));
}

#[test]
fn dag_unknown_subcommand_fails_with_code() {
    let output = Command::new(dag_binary())
        .args(["foo"])
        .output()
        .expect("unknown subcommand");

    assert!(!output.status.success());
}

#[test]
fn dag_validate_json_schema_contract() {
    let dag = write_temp_dag();
    let output = Command::new(dag_binary())
        .args(["dag", "validate", &dag, "--json"])
        .output()
        .expect("json validate");

    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("validate json");
    assert_eq!(payload["command"], "dag.validate");
    assert!(payload["status"].as_str().is_some());
    assert!(payload["data"].is_object());
}

#[test]
fn dag_validate_invalid_argument_fails() {
    let output = Command::new(dag_binary())
        .args(["dag", "validate", "non-existent-dag.json"])
        .output()
        .expect("invalid validate arg");

    assert!(!output.status.success());
}

#[test]
fn dag_run_json_output_contract_and_exit_code() {
    let dag = write_temp_dag();
    let out_dir = tempdir().expect("temp out");

    let output = Command::new(dag_binary())
        .args([
            "dag",
            "run",
            "--json",
            &dag,
            "--out",
            out_dir.path().to_str().unwrap(),
        ])
        .output()
        .expect("run with json");

    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("run json parse");
    assert_eq!(payload["command"], "dag.run");
    assert_eq!(payload["status"], "ok");
    assert!(payload["data"].get("run_dir").and_then(|v| v.as_str()).is_some());
}

#[test]
fn dag_help_surfaces_all_top_level_entries() {
    let output = Command::new(dag_binary())
        .arg("--help")
        .output()
        .expect("global help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    for command in ["dag", "rag", "rar"] {
        assert!(text.contains(command));
    }
}
