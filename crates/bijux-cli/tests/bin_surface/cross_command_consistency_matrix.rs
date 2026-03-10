#![forbid(unsafe_code)]
//! Cross-command consistency matrix across binary/core/bridge/repl surfaces.
//! test_type: cross-command-consistency

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use bijux_cli::app::run_app;
use bijux_cli_python as _;
use bijux_cli_python::{command_tree_introspection_api, execution_outcome_api};
use bijux_cli::repl::{execute_repl_line, startup_repl};
use libc as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run_bin(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("binary should execute")
}

fn run_bin_with_env(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("binary should execute")
}

fn parse_json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("json")
}

fn bridge_outcome(args: &[&str]) -> Value {
    let argv = std::iter::once("bijux".to_string())
        .chain(args.iter().map(|s| s.to_string()))
        .collect::<Vec<_>>();
    serde_json::from_str(&execution_outcome_api(&argv).expect("bridge outcome"))
        .expect("bridge json")
}

fn temp_dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir()
        .join(format!("bijux-cross-command-{name}-{}-{nanos}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir");
    root
}

#[test]
fn inspect_and_dev_routes_agree_on_route_ownership() {
    let inspect = parse_json(&run_bin(&["inspect", "--format", "json", "--no-pretty"]).stdout);
    let routes =
        parse_json(&run_bin(&["dev", "cli", "routes", "--format", "json", "--no-pretty"]).stdout);

    let inspect_routes: BTreeSet<String> = inspect["route_sources"]
        .as_array()
        .expect("route_sources")
        .iter()
        .map(|row| {
            row["segments"]
                .as_array()
                .expect("segments")
                .iter()
                .map(|s| s.as_str().expect("str"))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();

    let dev_routes: BTreeSet<String> = routes["routes"]
        .as_array()
        .expect("routes")
        .iter()
        .map(|row| {
            row["segments"]
                .as_array()
                .expect("segments")
                .iter()
                .map(|s| s.as_str().expect("str"))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();

    assert_eq!(inspect_routes, dev_routes);
}

#[test]
fn inspect_and_dev_registry_agree_on_plugin_ownership_model() {
    let inspect = parse_json(&run_bin(&["inspect", "--format", "json", "--no-pretty"]).stdout);
    let registry =
        parse_json(&run_bin(&["dev", "cli", "registry", "--format", "json", "--no-pretty"]).stdout);

    let inspect_reserved: BTreeSet<String> = inspect["reserved_namespaces"]
        .as_array()
        .expect("reserved")
        .iter()
        .filter(|row| row["reserved"] == true)
        .filter_map(|row| row["name"].as_str().map(ToString::to_string))
        .collect();

    let registry_reserved: BTreeSet<String> = registry["registry"]
        .as_array()
        .expect("registry")
        .iter()
        .filter(|row| row["reserved"] == true)
        .filter_map(|row| row["name"].as_str().map(ToString::to_string))
        .collect();

    assert_eq!(inspect_reserved, registry_reserved);
}

#[test]
fn config_get_and_dev_env_agree_on_source_precedence() {
    let root = temp_dir("config-precedence");
    let config = root.join("config.env");
    fs::write(&config, "BIJUXCLI_ALPHA=from-file\n").expect("write");
    let path = config.to_str().expect("utf-8");

    let get = parse_json(
        &run_bin_with_env(
            &["cli", "config", "get", "alpha", "--format", "json", "--no-pretty"],
            &[("BIJUXCLI_CONFIG", path)],
        )
        .stdout,
    );
    let env = parse_json(
        &run_bin_with_env(
            &["dev", "cli", "env", "--format", "json", "--no-pretty"],
            &[("BIJUXCLI_CONFIG", path)],
        )
        .stdout,
    );

    assert_eq!(get["source_path"], env["active"]["config_file"]);
    assert_eq!(env["source_precedence"], serde_json::json!(["flags", "env", "config", "defaults"]));
}

#[test]
fn doctor_and_state_audit_agree_on_corruption_detection_when_applicable() {
    let temp = temp_dir("doctor-audit");
    let config = temp.join("corrupt.env");
    fs::write(&config, "BIJUXCLI_ALPHA=1\nBROKEN\n").expect("write config");

    let doctor = parse_json(
        &run_bin_with_env(
            &["dev", "cli", "state-doctor", "--format", "json", "--no-pretty"],
            &[("BIJUXCLI_CONFIG", config.to_str().expect("utf-8"))],
        )
        .stdout,
    );
    let audit = parse_json(
        &run_bin_with_env(
            &["dev", "cli", "state-audit", "--format", "json", "--no-pretty"],
            &[("BIJUXCLI_CONFIG", config.to_str().expect("utf-8"))],
        )
        .stdout,
    );

    assert_eq!(doctor["doctor"]["status"], "degraded");
    assert!(audit["paths"]["config"]["path"].is_string());
}

#[test]
fn plugins_list_and_dev_registry_agree_on_installed_plugin_namespace_rules() {
    let list = parse_json(&run_bin(&["plugins", "list", "--format", "json", "--no-pretty"]).stdout);
    let registry =
        parse_json(&run_bin(&["dev", "cli", "registry", "--format", "json", "--no-pretty"]).stdout);
    let reserved: BTreeSet<String> = registry["registry"]
        .as_array()
        .expect("registry")
        .iter()
        .filter(|row| row["reserved"] == true)
        .filter_map(|row| row["name"].as_str().map(ToString::to_string))
        .collect();

    for plugin in list["plugins"].as_array().expect("plugins") {
        if let Some(namespace) = plugin["manifest"]["namespace"].as_str() {
            assert!(
                !reserved.contains(namespace),
                "installed plugin namespace should not overlap reserved set"
            );
        }
    }
}

#[test]
fn repl_execution_matches_non_interactive_for_config_get_plugins_list_and_status() {
    let root = temp_dir("repl-match");
    let config = root.join("config.env");
    fs::write(&config, "BIJUXCLI_ALPHA=from-file\n").expect("config");
    let config_arg = config.to_str().expect("utf-8");

    let mut session = startup_repl("", None).0;

    let _repl_config = execute_repl_line(
        &mut session,
        &format!("config get alpha --format json --no-pretty --config-path {config_arg}"),
    )
    .expect("repl config");
    let bin_config = run_bin(&[
        "config",
        "get",
        "alpha",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        config_arg,
    ]);
    assert_eq!(session.last_exit_code, bin_config.status.code().unwrap_or(-1));

    let _repl_plugins = execute_repl_line(&mut session, "plugins list --format json --no-pretty")
        .expect("repl plugins");
    let bin_plugins = run_bin(&["plugins", "list", "--format", "json", "--no-pretty"]);
    assert_eq!(session.last_exit_code, bin_plugins.status.code().unwrap_or(-1));

    let _repl_status =
        execute_repl_line(&mut session, "status --format json --no-pretty").expect("repl status");
    let bin_status = run_bin(&["status", "--format", "json", "--no-pretty"]);
    assert_eq!(session.last_exit_code, bin_status.status.code().unwrap_or(-1));
}

#[test]
fn binary_and_python_bridge_agree_on_config_history_memory_and_diagnostics_outputs() {
    let bin_config = run_bin(&["config", "--format", "json", "--no-pretty"]);
    let bridge_config = bridge_outcome(&["config", "--format", "json", "--no-pretty"]);
    assert_eq!(
        bridge_config["exit_code"].as_i64(),
        Some(i64::from(bin_config.status.code().unwrap_or(-1)))
    );

    let bin_history = run_bin(&["history", "--format", "json", "--no-pretty"]);
    let bridge_history = bridge_outcome(&["history", "--format", "json", "--no-pretty"]);
    assert_eq!(
        bridge_history["exit_code"].as_i64(),
        Some(i64::from(bin_history.status.code().unwrap_or(-1)))
    );

    let bin_memory = run_bin(&["memory", "list", "--format", "json", "--no-pretty"]);
    let bridge_memory = bridge_outcome(&["memory", "list", "--format", "json", "--no-pretty"]);
    assert_eq!(
        bridge_memory["exit_code"].as_i64(),
        Some(i64::from(bin_memory.status.code().unwrap_or(-1)))
    );

    let bin_diagnostics = run_bin(&["doctor", "--format", "json", "--no-pretty"]);
    let bridge_diagnostics = bridge_outcome(&["doctor", "--format", "json", "--no-pretty"]);
    assert_eq!(
        bridge_diagnostics["exit_code"].as_i64(),
        Some(i64::from(bin_diagnostics.status.code().unwrap_or(-1)))
    );
}

#[test]
fn binary_and_direct_core_agree_on_same_command_results() {
    let argv = vec![
        "bijux".to_string(),
        "status".to_string(),
        "--format".to_string(),
        "json".to_string(),
        "--no-pretty".to_string(),
    ];
    let core = run_app(&argv).expect("core run");
    let bin = run_bin(&["status", "--format", "json", "--no-pretty"]);

    assert_eq!(core.exit_code, bin.status.code().unwrap_or(-1));
    assert_eq!(core.stdout, String::from_utf8_lossy(&bin.stdout));
    assert_eq!(core.stderr, String::from_utf8_lossy(&bin.stderr));
}

#[test]
fn plugin_command_help_integrates_into_root_help_tree_deterministically() {
    let root_help_a = run_bin(&["--help"]);
    let root_help_b = run_bin(&["--help"]);
    assert_eq!(root_help_a.status.code(), Some(0));
    assert_eq!(root_help_a.stdout, root_help_b.stdout);
    let text = String::from_utf8(root_help_a.stdout).expect("utf-8");
    assert!(text.contains("plugins"), "root help should include plugin command group");

    let plugin_help = run_bin(&["plugins", "--help"]);
    assert_eq!(plugin_help.status.code(), Some(0));
    let plugin_text = String::from_utf8(plugin_help.stdout).expect("utf-8");
    assert!(plugin_text.contains("list"));
    assert!(plugin_text.contains("inspect"));
}

#[test]
fn command_tree_export_is_identical_across_binary_and_bridge() {
    let bin = run_bin(&["inspect", "--format", "json", "--no-pretty"]);
    assert_eq!(bin.status.code(), Some(0));
    let bin_payload = parse_json(&bin.stdout);
    let bridge_outcome = bridge_outcome(&["inspect", "--format", "json", "--no-pretty"]);
    let bridge_payload =
        parse_json(bridge_outcome["stdout"].as_str().unwrap_or_default().as_bytes());
    assert_eq!(bin_payload, bridge_payload);

    let tree = parse_json(command_tree_introspection_api().as_bytes());
    assert_eq!(tree["root"], "bijux");
    assert!(tree["namespaces"].is_array());
}

#[test]
fn route_ownership_is_stable_across_repeated_runs() {
    let first =
        parse_json(&run_bin(&["dev", "cli", "routes", "--format", "json", "--no-pretty"]).stdout);
    let second =
        parse_json(&run_bin(&["dev", "cli", "routes", "--format", "json", "--no-pretty"]).stdout);
    assert_eq!(first["routes"], second["routes"]);
    assert_eq!(first["aliases"], second["aliases"]);
}

#[test]
fn command_metadata_is_stable_across_repeated_runs() {
    let first = parse_json(&run_bin(&["inspect", "--format", "json", "--no-pretty"]).stdout);
    let second = parse_json(&run_bin(&["inspect", "--format", "json", "--no-pretty"]).stdout);
    assert_eq!(first["commands"], second["commands"]);
    assert_eq!(first["builtins"], second["builtins"]);
}

#[test]
fn diagnostics_payloads_do_not_drift_across_surfaces() {
    let bin = parse_json(&run_bin(&["doctor", "--format", "json", "--no-pretty"]).stdout);
    let bridge = bridge_outcome(&["doctor", "--format", "json", "--no-pretty"]);
    let bridge_payload = parse_json(bridge["stdout"].as_str().unwrap_or_default().as_bytes());
    assert_eq!(bin, bridge_payload);
}

#[test]
fn output_envelopes_do_not_drift_across_surfaces() {
    let bin = run_bin(&["unknown-command"]);
    let core = run_app(&["bijux".to_string(), "unknown-command".to_string()]).expect("core run");
    let bridge = bridge_outcome(&["unknown-command"]);

    let bin_error = parse_json(&bin.stderr);
    let core_error = parse_json(core.stderr.as_bytes());
    let bridge_error = parse_json(bridge["stderr"].as_str().unwrap_or_default().as_bytes());

    let mut bin_keys =
        bin_error.as_object().expect("bin error").keys().cloned().collect::<Vec<_>>();
    let mut core_keys =
        core_error.as_object().expect("core error").keys().cloned().collect::<Vec<_>>();
    let mut bridge_keys =
        bridge_error.as_object().expect("bridge error").keys().cloned().collect::<Vec<_>>();
    bin_keys.sort();
    core_keys.sort();
    bridge_keys.sort();
    assert_eq!(bin_keys, core_keys);
    assert_eq!(bin_keys, bridge_keys);
}

#[test]
fn exit_code_classes_do_not_drift_across_surfaces() {
    let success_bin = run_bin(&["status", "--format", "json", "--no-pretty"]);
    let success_core = run_app(&[
        "bijux".to_string(),
        "status".to_string(),
        "--format".to_string(),
        "json".to_string(),
        "--no-pretty".to_string(),
    ])
    .expect("core success");
    let success_bridge = bridge_outcome(&["status", "--format", "json", "--no-pretty"]);
    assert_eq!(success_bin.status.code().unwrap_or(-1), success_core.exit_code);
    assert_eq!(
        success_bin.status.code().unwrap_or(-1),
        success_bridge["exit_code"].as_i64().unwrap_or(-1) as i32
    );

    let usage_bin = run_bin(&["unknown-command"]);
    let usage_core =
        run_app(&["bijux".to_string(), "unknown-command".to_string()]).expect("core usage");
    let usage_bridge = bridge_outcome(&["unknown-command"]);
    assert_eq!(usage_bin.status.code().unwrap_or(-1), usage_core.exit_code);
    assert_eq!(
        usage_bin.status.code().unwrap_or(-1),
        usage_bridge["exit_code"].as_i64().unwrap_or(-1) as i32
    );
}
