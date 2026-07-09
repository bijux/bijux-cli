#![forbid(unsafe_code)]
//! Root command coverage and explicit root-surface law tests.

use std::process::{Command, Output};

use bijux_cli::api::runtime::run_app;
use bijux_cli::contracts::known_bijux_tool_by_query;
use libc as _;
use serde_json::Value;
use shlex as _;
use tempfile::TempDir;
use thiserror as _;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux")).args(args).output().expect("binary should execute")
}

fn run_ok_json(args: &[&str]) -> Value {
    let out = run(args);
    assert!(out.status.success(), "expected success for {args:?}");
    serde_json::from_slice(&out.stdout).expect("stdout should be valid json")
}

fn latest_version_tag() -> Option<String> {
    let output = Command::new("git")
        .args(["tag", "--list", "v[0-9]*", "--sort=-version:refname"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}

#[test]
fn parity_version_against_current_expected_behavior() {
    let out = run(&["version"]);
    assert!(out.status.success());
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(payload.get("version").is_some());

    let core = run_app(&["bijux".to_string(), "version".to_string()]).expect("core");
    assert_eq!(out.status.code(), Some(core.exit_code));
    assert_eq!(String::from_utf8_lossy(&out.stdout), core.stdout);
    assert_eq!(String::from_utf8_lossy(&out.stderr), core.stderr);
}

#[test]
fn parity_version_flag_matches_version_command() {
    let flag = run(&["--version"]);
    assert!(flag.status.success());
    let flagged_payload: Value = serde_json::from_slice(&flag.stdout).expect("json");
    assert!(flagged_payload.get("version").is_some());

    let command = run(&["version"]);
    assert_eq!(flag.status.code(), command.status.code());
    assert_eq!(flag.stdout, command.stdout);
    assert_eq!(flag.stderr, command.stderr);
}

#[test]
fn version_json_contract_exposes_provenance_fields() {
    let payload = run_ok_json(&["version", "--format", "json", "--no-pretty"]);
    assert_eq!(payload["name"], "bijux");
    assert!(payload["version"].is_string());
    assert!(payload["semver"].is_string());
    assert!(payload["source"].is_string());
    assert!(payload["build_profile"].is_string());
    assert!(payload["git_commit"].is_null() || payload["git_commit"].is_string());
    assert!(payload["git_dirty"].is_null() || payload["git_dirty"].is_boolean());
}

#[test]
fn version_json_tracks_the_latest_release_tag_in_git_checkouts() {
    let Some(tag) = latest_version_tag() else {
        return;
    };

    let payload = run_ok_json(&["version", "--format", "json", "--no-pretty"]);
    let tagged_semver = semver::Version::parse(tag.trim_start_matches('v')).expect("tag semver");
    let actual_semver = semver::Version::parse(payload["semver"].as_str().expect("runtime semver"))
        .expect("runtime semver");
    let source = payload["source"].as_str().expect("source");
    if source == "git-tag" {
        assert_eq!(actual_semver, tagged_semver);
    } else {
        assert_ne!(source, "package-fallback");
    }

    let version = payload["version"].as_str().expect("display version");
    assert!(
        version.starts_with(&tag),
        "display version should start with the latest real tag {tag}, got {version}"
    );
}

#[test]
fn version_text_mode_is_docker_style_one_line() {
    let out = run(&["version", "--format", "text"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stderr.is_empty());
    let text = String::from_utf8(out.stdout).expect("utf-8");
    assert!(text.starts_with("bijux version "));
    assert!(!text.contains("version: "));
    assert_eq!(text.lines().count(), 1);
}

#[test]
fn parity_status_against_current_expected_behavior() {
    let out = run(&["status"]);
    assert!(out.status.success());
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(payload.get("status").is_some());
    assert!(payload.get("runtime").is_some());

    let core = run_app(&["bijux".to_string(), "status".to_string()]).expect("core");
    assert_eq!(out.status.code(), Some(core.exit_code));
    assert_eq!(String::from_utf8_lossy(&out.stdout), core.stdout);
    assert_eq!(String::from_utf8_lossy(&out.stderr), core.stderr);
}

#[test]
fn parity_doctor_against_current_expected_behavior() {
    let out = run(&["doctor"]);
    assert!(out.status.success());
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(payload.get("status").is_some());
    assert!(payload.get("severity").is_some());
    assert!(payload.get("checks").and_then(Value::as_array).is_some());
    let install = payload.get("install").expect("install report");
    assert!(install.get("legacy_installer_conflicts").is_some());
    assert!(install.get("legacy_installer_conflict_paths").and_then(Value::as_array).is_some());

    let core = run_app(&["bijux".to_string(), "doctor".to_string()]).expect("core");
    assert_eq!(out.status.code(), Some(core.exit_code));
    assert_eq!(String::from_utf8_lossy(&out.stdout), core.stdout);
    assert_eq!(String::from_utf8_lossy(&out.stderr), core.stderr);
}

#[test]
fn doctor_paths_reports_state_locations_and_permissions() {
    let payload = run_ok_json(&["doctor", "paths", "--format", "json", "--no-pretty"]);
    assert!(payload["paths"]["config"]["path"].is_string());
    assert!(payload["paths"]["history"]["writable"].is_boolean());
    assert!(payload["diagnostics"]["issues"].is_array());
}

#[test]
fn doctor_routing_reports_routes_aliases_and_registry() {
    let payload = run_ok_json(&["doctor", "routing", "--format", "json", "--no-pretty"]);
    assert_eq!(payload["status"], "ok");
    assert!(payload["summary"]["route_count"].as_u64().unwrap_or_default() > 0);
    assert!(payload["aliases"].as_array().is_some_and(|items| !items.is_empty()));
    assert!(payload["namespaces"].as_array().is_some_and(|items| !items.is_empty()));
}

#[test]
fn doctor_python_reports_bridge_inventory() {
    let payload = run_ok_json(&["doctor", "python", "--format", "json", "--no-pretty"]);
    assert!(payload["interpreters"].is_array());
    assert!(payload["bridge"].is_object());
    assert!(payload["environment"].is_object());
}

#[test]
fn doctor_bundle_writes_artifacts_under_current_directory() {
    let temp = TempDir::new().expect("tempdir");
    let out = Command::new(env!("CARGO_BIN_EXE_bijux"))
        .current_dir(temp.path())
        .args(["doctor", "--bundle", "--format", "json", "--no-pretty"])
        .output()
        .expect("binary should execute");
    assert!(out.status.success(), "stderr:\n{}", String::from_utf8_lossy(&out.stderr));
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    let bundle_path = payload["bundle"]["path"].as_str().expect("bundle path");
    assert!(bundle_path.ends_with("artifacts/bijux-cli/doctor-bundle"));
    assert!(temp.path().join("artifacts/bijux-cli/doctor-bundle/doctor.json").exists());
    assert!(temp
        .path()
        .join("artifacts/bijux-cli/doctor-bundle/generated-config-reference.md")
        .exists());
}

#[test]
fn doctor_app_subject_delegates_to_official_app_doctor() {
    let canonical = known_bijux_tool_by_query("dag").expect("known official app");
    let root = run_ok_json(&["doctor", "dag", "--format", "json", "--no-pretty"]);
    let apps = run_ok_json(&["apps", "doctor", "dag", "--format", "json", "--no-pretty"]);
    assert_eq!(root["namespace"], canonical.namespace.to_string());
    assert_eq!(root["namespace"], apps["namespace"]);
    assert_eq!(root["matched_via"], apps["matched_via"]);
    assert_eq!(root["app"]["namespace"], apps["app"]["namespace"]);
    assert_eq!(root["app"]["entrypoint"], apps["app"]["entrypoint"]);
}

#[test]
fn doctor_shims_reports_legacy_app_shims_on_path() {
    let temp = TempDir::new().expect("tempdir");
    let shim_bin = temp.path().join("shim-bin");
    std::fs::create_dir_all(&shim_bin).expect("shim bin");
    let shim = shim_bin.join("bijux-workflow");
    std::fs::write(&shim, "#!/bin/sh\n").expect("shim");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    }

    let old_path = std::env::var_os("PATH");
    let joined_path = std::env::join_paths([&shim_bin]).expect("join path");
    std::env::set_var("PATH", joined_path);
    let payload = run_ok_json(&["doctor", "shims", "--format", "json", "--no-pretty"]);
    if let Some(value) = old_path {
        std::env::set_var("PATH", value);
    } else {
        std::env::remove_var("PATH");
    }

    assert_eq!(payload["severity"], "warning");
    assert!(payload["legacy_app_shims"].as_array().is_some_and(|items| !items.is_empty()));
}

#[test]
fn parity_inspect_against_current_expected_behavior() {
    let out = run(&["inspect"]);
    assert!(out.status.success());
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(payload.get("route_sources").is_some());

    let core = run_app(&["bijux".to_string(), "inspect".to_string()]).expect("core");
    assert_eq!(out.status.code(), Some(core.exit_code));
    assert_eq!(String::from_utf8_lossy(&out.stdout), core.stdout);
    assert_eq!(String::from_utf8_lossy(&out.stderr), core.stderr);
}

#[test]
fn parity_docs_against_current_expected_behavior() {
    let out = run(&["docs"]);
    assert!(out.status.success());
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(payload.get("references").is_some());
    assert!(payload.get("site_url").is_some());

    let core = run_app(&["bijux".to_string(), "docs".to_string()]).expect("core");
    assert_eq!(out.status.code(), Some(core.exit_code));
    assert_eq!(String::from_utf8_lossy(&out.stdout), core.stdout);
    assert_eq!(String::from_utf8_lossy(&out.stderr), core.stderr);
}

#[test]
fn parity_audit_against_current_expected_behavior() {
    let out = run(&["audit"]);
    assert!(out.status.success());
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(payload.get("checks").is_some());

    let core = run_app(&["bijux".to_string(), "audit".to_string()]).expect("core");
    assert_eq!(out.status.code(), Some(core.exit_code));
    assert_eq!(String::from_utf8_lossy(&out.stdout), core.stdout);
    assert_eq!(String::from_utf8_lossy(&out.stderr), core.stderr);
}

#[test]
fn help_snapshot_exists_for_every_root_command() {
    let roots = [
        "status",
        "version",
        "doctor",
        "inspect",
        "docs",
        "audit",
        "apps",
        "config",
        "plugins",
        "history",
        "install",
        "memory",
        "repl",
        "completion",
    ];
    for cmd in roots {
        let out = run(&[cmd, "--help"]);
        assert!(out.status.success(), "help must succeed for root command {cmd}");
        let stdout = String::from_utf8(out.stdout).expect("utf-8");
        assert!(stdout.contains("Usage:"), "help for {cmd} should include Usage");
    }
}

#[test]
fn exit_code_and_stream_discipline_for_root_commands() {
    let success_cases: &[&[&str]] = &[
        &["version"],
        &["status"],
        &["doctor"],
        &["inspect"],
        &["docs"],
        &["audit"],
        &["install", "cli", "--dry-run"],
    ];
    for args in success_cases {
        let out = run(args);
        assert_eq!(out.status.code(), Some(0), "expected success for {args:?}");
        assert!(!out.stdout.is_empty(), "stdout should contain payload for {args:?}");
        assert!(out.stderr.is_empty(), "stderr should be empty for {args:?}");
    }

    let fail = run(&["config", "get"]);
    assert_ne!(fail.status.code(), Some(0));
    assert!(fail.stdout.is_empty());
    assert!(!fail.stderr.is_empty());
}

#[test]
fn machine_readable_root_commands_support_json_and_yaml() {
    let machine_cases: &[&[&str]] = &[
        &["status"],
        &["doctor"],
        &["inspect"],
        &["docs"],
        &["audit"],
        &["apps", "list"],
        &["apps", "doctor"],
        &["history"],
        &["install", "cli", "--dry-run"],
        &["memory"],
        &["plugins", "list"],
    ];

    for base in machine_cases {
        let mut json_args = base.to_vec();
        json_args.extend(["--format", "json", "--no-pretty"]);
        let json_out = run(&json_args);
        assert!(json_out.status.success(), "json mode failed for {base:?}");
        let _: Value = serde_json::from_slice(&json_out.stdout).expect("json parse");

        let mut yaml_args = base.to_vec();
        yaml_args.extend(["--format", "yaml", "--pretty"]);
        let yaml_out = run(&yaml_args);
        assert!(yaml_out.status.success(), "yaml mode failed for {base:?}");
        let yaml = String::from_utf8(yaml_out.stdout).expect("utf-8");
        assert!(!yaml.trim().is_empty(), "yaml output should not be empty for {base:?}");
    }
}

#[test]
fn quiet_mode_is_supported_for_relevant_root_commands() {
    let relevant: &[&[&str]] = &[
        &["status"],
        &["doctor"],
        &["inspect"],
        &["docs"],
        &["audit"],
        &["install", "cli", "--dry-run"],
    ];
    for args in relevant {
        let mut quiet_args = args.to_vec();
        quiet_args.insert(0, "--quiet");
        let out = run(&quiet_args);
        assert!(out.status.success(), "quiet mode failed for {args:?}");
        assert!(out.stdout.is_empty(), "quiet should suppress stdout for {args:?}");
        assert!(out.stderr.is_empty(), "quiet should suppress stderr for {args:?}");
    }
}

#[test]
fn completion_supports_explicit_shell_selection_in_text_and_json_modes() {
    let bash = run(&["completion", "--shell", "bash"]);
    assert_eq!(bash.status.code(), Some(0));
    assert!(bash.stderr.is_empty());
    let bash_text = String::from_utf8(bash.stdout).expect("utf-8");
    assert!(bash_text.contains("complete -W "));

    let zsh_payload =
        run_ok_json(&["completion", "--shell", "zsh", "--format", "json", "--no-pretty"]);
    assert_eq!(zsh_payload["active_shell"], "zsh");
    assert_eq!(zsh_payload["selection_source"], "explicit");
    assert!(zsh_payload["script"].as_str().expect("script").contains("#compdef bijux"));
}

#[test]
fn canonical_cli_alias_routes_are_executable() {
    let cases: &[&[&str]] = &[
        &["cli", "doctor"],
        &["cli", "version"],
        &["cli", "repl"],
        &["cli", "completion", "--shell", "fish"],
    ];
    for args in cases {
        let out = run(&args);
        assert_eq!(out.status.code(), Some(0), "expected success for {args:?}");
    }
}

#[test]
fn no_color_is_supported_for_text_root_commands() {
    for args in [vec!["help"], vec!["help", "status"], vec!["help", "plugins"]] {
        let mut argv = vec!["--color", "never"];
        argv.extend(args);
        let out = run(&argv);
        assert!(out.status.success());
        let text = String::from_utf8(out.stdout).expect("utf-8");
        assert!(!text.contains("\u{1b}["));
    }
}

#[test]
fn help_command_matches_explicit_help_flag_output() {
    let command_help = run(&["help"]);
    let flag_help = run(&["--help"]);
    assert_eq!(command_help.status.code(), Some(0));
    assert_eq!(flag_help.status.code(), Some(0));
    assert_eq!(command_help.stdout, flag_help.stdout);
    assert_eq!(command_help.stderr, flag_help.stderr);

    let topic_help = run(&["help", "status"]);
    let topic_flag = run(&["status", "--help"]);
    assert_eq!(topic_help.status.code(), Some(0));
    assert_eq!(topic_flag.status.code(), Some(0));
    assert_eq!(topic_help.stdout, topic_flag.stdout);
    assert_eq!(topic_help.stderr, topic_flag.stderr);
}

#[test]
fn help_command_rejects_invalid_global_flag_values_with_usage_exit() {
    for args in [
        vec!["help", "--format", "toml"],
        vec!["help", "--color", "rainbow"],
        vec!["help", "--log-level", "verbose"],
    ] {
        let out = run(&args);
        assert_eq!(
            out.status.code(),
            Some(2),
            "invalid help flag should be usage error for {args:?}"
        );
        assert!(out.stdout.is_empty(), "invalid help flag should not print stdout for {args:?}");
        let stderr = String::from_utf8(out.stderr).expect("utf-8");
        assert!(stderr.contains("invalid "), "expected actionable validation error for {args:?}");
        assert!(stderr.contains("Run `bijux --help`"), "expected help guidance for {args:?}");
    }
}

#[test]
fn unknown_help_topics_include_suggestions_for_alias_typoes() {
    let alias_typo = run(&["help", "versoin"]);
    assert_eq!(alias_typo.status.code(), Some(2));
    let alias_stderr = String::from_utf8(alias_typo.stderr).expect("utf-8");
    assert!(alias_stderr.contains("Unknown help topic: versoin."));
    assert!(alias_stderr.contains("bijux help version"));
}

#[test]
fn root_help_surfaces_official_apps_and_plugin_inventory_sections() {
    let out = run(&["--help"]);
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).expect("utf-8");
    assert!(stdout.contains("Official apps:"));
    assert!(stdout.contains("Installed plugins:"));
    assert!(stdout.contains("Diagnostics:"));
    assert!(stdout.contains("bijux apps list"));
    assert!(stdout.contains("bijux-dag --help"));
    assert!(stdout.contains("bijux doctor"));
    assert!(stdout.contains("bijux apps doctor"));
}

#[test]
fn official_app_help_topics_explain_routed_namespaces_and_public_binaries() {
    let out = run(&["help", "dag"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stderr.is_empty());
    let stdout = String::from_utf8(out.stdout).expect("utf-8");
    assert!(stdout.contains("Official app help: Bijux DAG"));
    assert!(stdout.contains("root route: bijux dag <command> ..."));
    assert!(stdout.contains("product binary: bijux-dag"));
    assert!(stdout.contains("cargo install bijux-dag-cli"));
    assert!(stdout.contains("bijux-dag --help"));
}

#[test]
fn unknown_routes_suggest_canonical_official_app_names() {
    let out = run(&["workflw", "status"]);
    assert_eq!(out.status.code(), Some(2));
    let combined = format!(
        "{}{}",
        String::from_utf8(out.stdout).expect("utf-8"),
        String::from_utf8(out.stderr).expect("utf-8")
    );
    assert!(combined.contains("bijux dag"));
    assert!(combined.contains("bijux help dag"));
}

#[test]
fn malformed_input_is_rejected_for_argument_taking_root_commands() {
    let malformed: &[&[&str]] = &[
        &["config", "get"],
        &["plugins", "uninstall"],
        &["history", "--bad-flag"],
        &["memory", "set"],
    ];
    for args in malformed {
        let out = run(args);
        assert_ne!(out.status.code(), Some(0), "malformed input should fail for {args:?}");
        assert!(out.stdout.is_empty(), "malformed input should not print stdout for {args:?}");
        assert!(!out.stderr.is_empty(), "malformed input should print stderr for {args:?}");
    }
}

#[test]
fn repeated_run_determinism_for_machine_readable_root_commands() {
    let deterministic: &[&[&str]] = &[
        &["status", "--format", "json", "--no-pretty"],
        &["doctor", "--format", "json", "--no-pretty"],
        &["inspect", "--format", "json", "--no-pretty"],
        &["docs", "--format", "json", "--no-pretty"],
        &["audit", "--format", "json", "--no-pretty"],
    ];

    for args in deterministic {
        let first = run(args);
        let second = run(args);
        assert!(first.status.success());
        assert!(second.status.success());
        assert_eq!(first.stdout, second.stdout, "output drift for {args:?}");
        assert_eq!(first.stderr, second.stderr, "stderr drift for {args:?}");
    }
}

#[test]
fn root_command_coverage_artifact_smoke_uses_supported_commands() {
    // Smoke check for the matrix command list used by report generation.
    let matrix = [
        ["version"].as_slice(),
        ["status"].as_slice(),
        ["doctor"].as_slice(),
        ["inspect"].as_slice(),
        ["docs"].as_slice(),
        ["audit"].as_slice(),
    ];
    for args in matrix {
        let payload = run_ok_json(args);
        assert!(payload.is_object(), "matrix command should return object payload: {args:?}");
    }
}
