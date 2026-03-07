use bijux_dag_app as _;
use clap as _;
use clap_complete as _;
use serde_json as _;
use tempfile as _;

use std::process::Command;
use tempfile::{tempdir, NamedTempFile};

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

fn write_temp_dag() -> String {
    let path = std::env::temp_dir().join(format!(
        "bijux-dag-cli-contract-{}.json",
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
    std::fs::write(&path, content).expect("write dag");
    path.to_string_lossy().into_owned()
}

#[test]
fn dag_validate_help_is_stable_enough() {
    let output = dag_command()
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
    let output = dag_command()
        .args(["foo"])
        .output()
        .expect("unknown subcommand");

    assert!(!output.status.success());
}

#[test]
fn dag_validate_json_schema_contract() {
    let dag = write_temp_dag();
    let output = dag_command()
        .args(["dag", "validate", &dag, "--json"])
        .output()
        .expect("json validate");

    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("validate json");
    assert_eq!(payload["command"], "dag.validate");
    assert!(payload["status"].as_str().is_some());
    assert!(payload["data"].is_object());
}

#[test]
fn dag_root_help_lists_umbrella_commands() {
    let output = dag_command().arg("--help").output().expect("global help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("dag"));
    assert!(text.contains("completions"));
    assert!(text.contains("Git for computation graphs"));
}

#[test]
fn dag_command_help_surface_contract() {
    let output = dag_command().args(["dag"]).output().expect("dag help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    for token in [
        "validate", "run", "replay", "diff", "explain", "status", "cache", "adapters",
    ] {
        assert!(text.contains(token));
    }
}

#[test]
fn dag_run_help_surface_contract() {
    let output = dag_command()
        .args(["dag", "run", "--help"])
        .output()
        .expect("run help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    for token in [
        "--out",
        "--hermetic",
        "--deny-network",
        "--clean-env",
        "run",
    ] {
        assert!(text.contains(token));
    }
}

#[test]
fn dag_replay_help_surface_contract() {
    let output = dag_command()
        .args(["dag", "replay", "--help"])
        .output()
        .expect("replay help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    for token in ["--out", "--run-id", "--reuse-cache", "replay"] {
        assert!(text.contains(token));
    }
}

#[test]
fn dag_diff_help_surface_contract() {
    let output = dag_command()
        .args(["dag", "diff", "--help"])
        .output()
        .expect("diff help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("dag diff"));
    assert!(text.contains("--json"));
}

#[test]
fn dag_explain_help_surface_contract() {
    let output = dag_command()
        .args(["dag", "explain", "--help"])
        .output()
        .expect("explain help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("dag explain"));
    assert!(text.contains("--node"));
}

#[test]
fn dag_cache_help_surface_contract() {
    let output = dag_command()
        .args(["dag", "cache", "--help"])
        .output()
        .expect("cache help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    for token in ["cache", "verify", "pack", "explain"] {
        assert!(text.contains(token));
    }
}

#[test]
fn dag_adapters_help_surface_contract() {
    let output = dag_command()
        .args(["dag", "adapters", "--help"])
        .output()
        .expect("adapters help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("adapters"));
    assert!(text.contains("ls"));
    assert!(text.contains("doctor"));
}

#[test]
fn dag_validate_text_output_contract() {
    let dag = write_temp_dag();
    let output = dag_command()
        .args(["dag", "validate", &dag])
        .output()
        .expect("validate text");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("status:"));
}

#[test]
fn dag_validate_invalid_argument_fails() {
    let output = dag_command()
        .args(["dag", "validate", "non-existent-dag.json"])
        .output()
        .expect("invalid validate arg");

    assert!(!output.status.success());
}

#[test]
fn dag_validate_rejects_invalid_spec_with_validation_exit_code() {
    let invalid = NamedTempFile::new().expect("temp invalid");
    let invalid_path = invalid.path().to_path_buf();
    std::fs::write(
        &invalid_path,
        r#"{"spec":"bijux-dag/v9.9","nodes":[],"edges":[]}"#,
    )
    .expect("write invalid spec");

    let output = dag_command()
        .args(["dag", "validate", invalid_path.to_str().unwrap()])
        .output()
        .expect("invalid validate");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn dag_run_exit_code_success() {
    let dag = write_temp_dag();
    let out_dir = tempfile::tempdir().expect("run out");

    let output = dag_command()
        .args([
            "dag",
            "run",
            &dag,
            "--out",
            out_dir.path().to_str().unwrap(),
        ])
        .output()
        .expect("run success");

    assert!(output.status.success());
}

#[test]
fn dag_run_runtime_failure_returns_nonzero_exit() {
    let dag = {
        let path = std::env::temp_dir().join(format!(
            "bijux-dag-cli-contract-failing-{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        let content = r#"{
          "spec": "bijux-dag/v0.1",
          "nodes": [{
            "id": "fail",
            "kind": "shell",
            "inputs": [],
            "outputs": [{ "name": "value", "path": "value.txt" }],
            "params": {
              "argv": ["/bin/sh","-c","exit 7"]
            }
          }],
          "edges": []
        }"#;
        std::fs::write(&path, content).expect("write dag");
        path.to_string_lossy().into_owned()
    };
    let out_dir = tempfile::tempdir().expect("run out");

    let output = dag_command()
        .args([
            "dag",
            "run",
            &dag,
            "--out",
            out_dir.path().to_str().unwrap(),
        ])
        .output()
        .expect("run fail");

    assert!(!output.status.success());
    assert!(output.status.code().is_some_and(|code| code != 0));
}

#[test]
fn completions_generation_supports_all_supported_shells() {
    for shell in ["bash", "zsh", "fish", "elvish", "powershell"] {
        let output = dag_command()
            .args(["completions", "--shell", shell])
            .output()
            .expect("completion command");
        assert!(output.status.success(), "shell {shell} failed");
        assert!(
            !output.stdout.is_empty(),
            "shell {shell} emitted empty completion"
        );
    }
}

#[test]
fn fsck_alias_surface_runs_on_valid_run_dir() {
    let dag = write_temp_dag();
    let out_dir = tempdir().expect("run out");
    let run_output = dag_command()
        .args([
            "dag",
            "run",
            &dag,
            "--out",
            out_dir.path().to_str().expect("run out path"),
        ])
        .output()
        .expect("run");
    assert!(
        run_output.status.success(),
        "run must succeed for fsck setup"
    );

    let mut entries: Vec<_> = std::fs::read_dir(out_dir.path())
        .expect("read out dir")
        .filter_map(Result::ok)
        .collect();
    entries.sort_by_key(|entry| entry.file_name());
    let run_dir = entries
        .last()
        .expect("expected run directory")
        .path()
        .to_string_lossy()
        .into_owned();

    let fsck_output = dag_command()
        .args(["dag", "fsck", &run_dir, "--strict", "--json"])
        .output()
        .expect("fsck");
    assert!(
        fsck_output.status.success(),
        "fsck on valid run directory should succeed"
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&fsck_output.stdout).expect("fsck json payload");
    assert_eq!(payload["command"], "dag.fsck");
    assert_eq!(payload["ok"], true);
}

#[test]
fn capabilities_backend_query_supports_kubernetes() {
    let output = dag_command()
        .args(["dag", "capabilities", "--backend", "kubernetes", "--json"])
        .output()
        .expect("capabilities backend");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("capabilities backend json");
    assert_eq!(payload["command"], "dag.capabilities");
    assert_eq!(payload["data"]["backend"], "kubernetes");
    assert_eq!(payload["data"]["status"], "simulated");
}

#[test]
fn capabilities_backend_query_supports_hpc() {
    let output = dag_command()
        .args(["dag", "capabilities", "--backend", "hpc", "--json"])
        .output()
        .expect("capabilities hpc backend");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("capabilities hpc json");
    assert_eq!(payload["command"], "dag.capabilities");
    assert_eq!(payload["data"]["backend"], "hpc");
    assert_eq!(payload["data"]["status"], "simulated");
}

#[test]
fn dag_status_json_schema_contract() {
    let dag = write_temp_dag();
    let run_dir = tempfile::tempdir().expect("run out");
    let run = dag_command()
        .args([
            "dag",
            "run",
            "--json",
            &dag,
            "--out",
            run_dir.path().to_str().unwrap(),
        ])
        .output()
        .expect("run json");
    let run_payload: serde_json::Value =
        serde_json::from_slice(&run.stdout).expect("parse run payload");
    let run_path = run_payload["data"]["run_dir"].as_str().unwrap();

    let output = dag_command()
        .args(["dag", "status", "--json", run_path])
        .output()
        .expect("status json");

    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse status payload");
    assert_eq!(payload["command"], "dag.status");
    assert_eq!(payload["ok"], true);
    assert!(payload["data"]["manifest"].is_object());
    assert!(payload["data"]["traces"].is_array());
}

#[test]
fn dag_diff_json_schema_contract() {
    let dag = write_temp_dag();
    let first_run_dir = tempfile::tempdir().expect("first run out");
    let second_run_dir = tempfile::tempdir().expect("second run out");

    let run_a = dag_command()
        .args([
            "dag",
            "run",
            "--json",
            &dag,
            "--out",
            first_run_dir.path().to_str().unwrap(),
        ])
        .output()
        .expect("run a");
    let run_b = dag_command()
        .args([
            "dag",
            "run",
            "--json",
            &dag,
            "--out",
            second_run_dir.path().to_str().unwrap(),
        ])
        .output()
        .expect("run b");
    assert!(
        run_a.status.success(),
        "run a failed: {}",
        String::from_utf8_lossy(&run_a.stderr)
    );
    assert!(
        run_b.status.success(),
        "run b failed: {}",
        String::from_utf8_lossy(&run_b.stderr)
    );

    let payload_a: serde_json::Value =
        serde_json::from_slice(&run_a.stdout).expect("parse run a payload");
    let payload_b: serde_json::Value =
        serde_json::from_slice(&run_b.stdout).expect("parse run b payload");
    let run_a_path = payload_a["data"]["run_dir"].as_str().unwrap();
    let run_b_path = payload_b["data"]["run_dir"].as_str().unwrap();

    let output = dag_command()
        .args(["dag", "diff", "--json", run_a_path, run_b_path])
        .output()
        .expect("diff json");

    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse diff payload");
    assert_eq!(payload["command"], "dag.diff");
    assert!(payload["data"]["manifest"].is_object());
    assert!(payload["data"]["nodes"].is_object());
    assert!(payload["data"]["outputs"].is_object());
}

#[test]
fn dag_validate_json_exists_with_human_and_machine_contracts() {
    let dag = write_temp_dag();
    let output = dag_command()
        .args(["dag", "validate", &dag])
        .output()
        .expect("validate text");

    assert!(output.status.success());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("{\"ok\""));

    let output_json = dag_command()
        .args(["dag", "validate", "--json", &dag])
        .output()
        .expect("validate json");

    assert!(output_json.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output_json.stdout).expect("validate json parse");
    assert_eq!(payload["command"], "dag.validate");
    assert_eq!(payload["ok"], true);
    assert!(payload["data"].is_object());
}

#[test]
fn dag_run_json_output_contract_and_exit_code() {
    let dag = write_temp_dag();
    let out_dir = tempdir().expect("temp out");

    let output = dag_command()
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
    assert!(payload["data"]
        .get("run_dir")
        .and_then(|v| v.as_str())
        .is_some());
}
